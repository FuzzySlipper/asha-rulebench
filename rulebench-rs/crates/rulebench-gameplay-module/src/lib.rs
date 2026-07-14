//! Rulebench's real downstream gameplay module and public ASHA host adapter.

#![forbid(unsafe_code)]

use asha_gameplay_module_sdk::*;
use asha_gameplay_runtime_host::{
    BundleArtifacts, GameplayBindingEntityTargets, GameplayDecisionContinuation,
    GameplayDecisionMoment, GameplayDecisionReceipt, GameplayDecisionStatus,
    GameplayOperationWorkspace, GameplayRuntimeDecisionOwner, GameplayRuntimeDecisionOwnerOutput,
    GameplayRuntimeDeclaredReadPlan, GameplayRuntimeHost, GameplayRuntimeProjectInput,
    GameplayRuntimeSchedulerDefinition, LoadPlan, LoadStep, RuntimeSessionId, SceneId,
};
use serde::{Deserialize, Serialize};

const MODULE_ID: &str = "rulebench.pre-effect-reaction";
const PROVIDER_ID: &str = "provider.rulebench.pre-effect-reaction";
const OWNER_ID: &str = "authority.rulebench.combat";
const STATE_READ_ID: &str = "reaction-state";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreEffectWorkspace {
    pub decision_id: String,
    pub actor_id: String,
    pub target_id: String,
    pub action_id: String,
    pub damage_amount: u32,
    pub damage_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionFabricConfig {
    pub accepted_reaction_damage_reduction: u32,
}

impl Default for ReactionFabricConfig {
    fn default() -> Self {
        Self {
            accepted_reaction_damage_reduction: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionFabricState {
    pub revision: u64,
    pub opened_windows: u64,
    pub resolved_windows: u64,
    pub accepted_reactions: u64,
    pub last_decision_id: Option<String>,
    pub last_option_id: Option<String>,
    pub last_resolution_accepted: bool,
    pub accepted_reaction_damage_reduction: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum ReactionFabricFact {
    Opened {
        decision_id: String,
    },
    Resolved {
        decision_id: String,
        accepted: bool,
        option_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactionOpenedEvent {
    decision_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactionResolvedEvent {
    decision_id: String,
    accepted: bool,
    option_id: Option<String>,
}

struct PreEffectReactionBehavior;

impl GameplayModuleBehavior for PreEffectReactionBehavior {
    fn invoke(
        &self,
        context: &GameplayModuleContext<'_>,
    ) -> Result<GameplayModuleActions, GameplayModuleError> {
        if context.event_contract() == Some(&contract("reaction-opened")) {
            let event: ReactionOpenedEvent = context.event_payload()?;
            return record_fact(
                context,
                ReactionFabricFact::Opened {
                    decision_id: event.decision_id,
                },
            );
        }
        if context.event_contract() == Some(&contract("reaction-resolved")) {
            let event: ReactionResolvedEvent = context.event_payload()?;
            return record_fact(
                context,
                ReactionFabricFact::Resolved {
                    decision_id: event.decision_id,
                    accepted: event.accepted,
                    option_id: event.option_id,
                },
            );
        }

        let state: ReactionFabricState = context.named_view(STATE_READ_ID)?;
        let mut workspace: PreEffectWorkspace = context.decision_workspace()?;
        let mut actions = context.actions();
        match context.invocation_id() {
            "rulebench.pre-effect.transform" => {
                if state.last_decision_id.as_deref() == Some(workspace.decision_id.as_str())
                    && state.last_resolution_accepted
                {
                    workspace.damage_amount = workspace
                        .damage_amount
                        .saturating_sub(state.accepted_reaction_damage_reduction);
                }
                actions.transform_workspace_json(
                    contract("pre-effect-workspace"),
                    context
                        .decision_workspace_hash()
                        .expect("Transform has a Workspace hash"),
                    &workspace,
                )?;
            }
            "rulebench.pre-effect.react" if context.decision_resume_token().is_none() => {
                actions.react(
                    GameplayReactionDisposition::Suspend {
                        token: format!("rulebench-window:{}", workspace.decision_id),
                    },
                    None,
                );
            }
            "rulebench.pre-effect.react"
                if state.last_decision_id.as_deref() == Some(workspace.decision_id.as_str()) =>
            {
                actions.react(GameplayReactionDisposition::Continue, None);
            }
            "rulebench.pre-effect.react" => {
                actions.react(
                    GameplayReactionDisposition::Cancel {
                        reason: "reaction window has not recorded a matching resolution".to_owned(),
                    },
                    None,
                );
            }
            _ => {
                return Err(GameplayModuleError {
                    code: "unexpectedRulebenchInvocation".to_owned(),
                    message: context.invocation_id().to_owned(),
                });
            }
        }
        Ok(actions)
    }
}

fn record_fact(
    context: &GameplayModuleContext<'_>,
    fact: ReactionFabricFact,
) -> Result<GameplayModuleActions, GameplayModuleError> {
    let state: ReactionFabricState = context.named_view(STATE_READ_ID)?;
    let mut actions = context.actions();
    actions.record_local_fact_json(
        contract("reaction-fact"),
        contract("reaction-state"),
        GameplayModuleStateScope::Session,
        state.revision,
        &fact,
    )?;
    Ok(actions)
}

struct ReactionStateAdapter;

impl GameplayTypedModuleStateAdapter for ReactionStateAdapter {
    type Config = ReactionFabricConfig;
    type State = ReactionFabricState;
    type Fact = ReactionFabricFact;
    type View = ReactionFabricState;

    fn module_id(&self) -> &str {
        MODULE_ID
    }

    fn state_schema(&self) -> &GameplayContractRef {
        static VALUE: std::sync::OnceLock<GameplayContractRef> = std::sync::OnceLock::new();
        VALUE.get_or_init(|| contract("reaction-state"))
    }

    fn fact_schema(&self) -> &GameplayContractRef {
        static VALUE: std::sync::OnceLock<GameplayContractRef> = std::sync::OnceLock::new();
        VALUE.get_or_init(|| contract("reaction-fact"))
    }

    fn owner(&self) -> &GameplayOwnerRef {
        static VALUE: std::sync::OnceLock<GameplayOwnerRef> = std::sync::OnceLock::new();
        VALUE.get_or_init(state_owner)
    }

    fn decode_config(&self, value: &[u8]) -> Result<Self::Config, String> {
        serde_json::from_slice(value).map_err(|error| error.to_string())
    }

    fn initialize(&self, config: &Self::Config) -> Result<Self::State, String> {
        Ok(ReactionFabricState {
            revision: 0,
            opened_windows: 0,
            resolved_windows: 0,
            accepted_reactions: 0,
            last_decision_id: None,
            last_option_id: None,
            last_resolution_accepted: false,
            accepted_reaction_damage_reduction: config.accepted_reaction_damage_reduction,
        })
    }

    fn decode_state(&self, value: &[u8]) -> Result<Self::State, String> {
        serde_json::from_slice(value).map_err(|error| error.to_string())
    }

    fn encode_state(&self, state: &Self::State) -> Result<Vec<u8>, String> {
        serde_json::to_vec(state).map_err(|error| error.to_string())
    }

    fn decode_fact(&self, value: &[u8]) -> Result<Self::Fact, String> {
        serde_json::from_slice(value).map_err(|error| error.to_string())
    }

    fn apply_fact(&self, state: &Self::State, fact: &Self::Fact) -> Result<Self::State, String> {
        let mut next = state.clone();
        next.revision = next.revision.saturating_add(1);
        match fact {
            ReactionFabricFact::Opened { decision_id } => {
                next.opened_windows = next.opened_windows.saturating_add(1);
                next.last_decision_id = Some(decision_id.clone());
                next.last_option_id = None;
                next.last_resolution_accepted = false;
            }
            ReactionFabricFact::Resolved {
                decision_id,
                accepted,
                option_id,
            } => {
                next.resolved_windows = next.resolved_windows.saturating_add(1);
                next.accepted_reactions =
                    next.accepted_reactions.saturating_add(u64::from(*accepted));
                next.last_decision_id = Some(decision_id.clone());
                next.last_option_id.clone_from(option_id);
                next.last_resolution_accepted = *accepted;
            }
        }
        Ok(next)
    }

    fn migrate(&self, _from_version: u32, state: &Self::State) -> Result<Self::State, String> {
        Ok(state.clone())
    }

    fn view_schema(&self) -> Option<&GameplayContractRef> {
        static VALUE: std::sync::OnceLock<GameplayContractRef> = std::sync::OnceLock::new();
        Some(VALUE.get_or_init(|| contract("reaction-state-view")))
    }

    fn project_view(&self, state: &Self::State) -> Result<Self::View, String> {
        Ok(state.clone())
    }

    fn encode_view(&self, view: &Self::View) -> Result<Vec<u8>, String> {
        serde_json::to_vec(view).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchGameplayContinuation {
    pub decision_id: String,
    pub operation: GameplayProposalEnvelope,
    pub expected_owner_revision: String,
    pub continuation: GameplayDecisionContinuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchGameplayDecisionEvidence {
    pub decision_id: String,
    pub status: String,
    pub receipt_hash: String,
    pub initial_workspace_hash: String,
    pub final_workspace_hash: String,
    pub declared_read_hashes: Vec<String>,
    pub invocation_output_hashes: Vec<String>,
    pub routing_hash: Option<String>,
    pub diagnostic_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchGameplayFabricReadout {
    pub registry_digest: String,
    pub binding_registry_hash: String,
    pub module_state_hash: String,
    pub runtime_host_hash: String,
    pub reaction_frame_hashes: Vec<String>,
    pub decisions: Vec<RulebenchGameplayDecisionEvidence>,
    pub pending_decision_count: u32,
}

pub trait RulebenchPreEffectOwner {
    fn revision_hash(&self) -> String;
    fn commit(&mut self, workspace: &PreEffectWorkspace) -> Result<Vec<String>, Vec<String>>;
}

pub struct RulebenchGameplayFabric {
    host: GameplayRuntimeHost,
}

impl core::fmt::Debug for RulebenchGameplayFabric {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RulebenchGameplayFabric")
            .field("readout", &self.readout())
            .finish()
    }
}

impl RulebenchGameplayFabric {
    pub fn new() -> Self {
        Self {
            host: GameplayRuntimeHost::activate_project(project_input())
                .expect("static Rulebench gameplay composition is valid"),
        }
    }

    pub fn restore(snapshot: &str) -> Result<Self, String> {
        GameplayRuntimeHost::restore_project(project_input(), snapshot)
            .map(|host| Self { host })
            .map_err(|error| error.to_string())
    }

    pub fn snapshot(&self) -> String {
        self.host
            .compose_snapshot()
            .expect("Rulebench gameplay host snapshot")
            .text
    }

    pub fn begin_before_effect(
        &mut self,
        workspace: PreEffectWorkspace,
        expected_owner_revision: String,
    ) -> Result<RulebenchGameplayContinuation, String> {
        let moment = decision_moment(workspace, expected_owner_revision.clone());
        let operation = moment.operation.clone();
        let mut owner = SuspensionOnlyOwner {
            revision: expected_owner_revision.clone(),
        };
        let receipt = self.host.decide(moment, &mut owner);
        if receipt.status != GameplayDecisionStatus::Suspended {
            return Err(format!(
                "decision did not suspend: {:?}",
                receipt.diagnostics
            ));
        }
        let continuation = receipt
            .continuation
            .ok_or_else(|| "suspended decision omitted continuation".to_owned())?;
        self.observe_opened(&receipt.decision_id)?;
        Ok(RulebenchGameplayContinuation {
            decision_id: receipt.decision_id,
            operation,
            expected_owner_revision,
            continuation,
        })
    }

    pub fn resolve_before_effect(
        &mut self,
        pending: &RulebenchGameplayContinuation,
        accepted: bool,
        option_id: Option<String>,
        owner: &mut dyn RulebenchPreEffectOwner,
    ) -> Result<GameplayDecisionReceipt, String> {
        let actual_owner_revision = owner.revision_hash();
        if actual_owner_revision != pending.expected_owner_revision {
            return Err(format!(
                "stale Rulebench owner revision: expected {}, found {actual_owner_revision}",
                pending.expected_owner_revision
            ));
        }
        let snapshot = self
            .host
            .compose_snapshot()
            .map_err(|error| error.to_string())?;
        let mut staged_host = GameplayRuntimeHost::restore_project(project_input(), &snapshot.text)
            .map_err(|error| error.to_string())?;
        observe_resolved(&mut staged_host, &pending.decision_id, accepted, option_id)?;
        let moment = GameplayDecisionMoment {
            decision_id: pending.decision_id.clone(),
            operation: pending.operation.clone(),
            expected_owner_revision: pending.expected_owner_revision.clone(),
            workspace: pending.continuation.workspace.clone(),
            resume_token: Some(pending.continuation.token.clone()),
        };
        let mut adapter = RulebenchOwnerAdapter { owner };
        let receipt = staged_host.decide(moment, &mut adapter);
        if receipt.accepted() {
            self.host = staged_host;
        }
        Ok(receipt)
    }

    pub fn readout(&self) -> RulebenchGameplayFabricReadout {
        let host = self.host.readout();
        RulebenchGameplayFabricReadout {
            registry_digest: host.gameplay_registry_digest,
            binding_registry_hash: host.binding_registry_hash,
            module_state_hash: host.module_state_hash,
            runtime_host_hash: host.runtime_host_hash,
            reaction_frame_hashes: self
                .host
                .reaction_frames()
                .iter()
                .map(|frame| frame.frame_hash.clone())
                .collect(),
            decisions: self
                .host
                .decision_receipts()
                .iter()
                .map(decision_evidence)
                .collect(),
            pending_decision_count: host.pending_decision_count,
        }
    }

    fn observe_opened(&mut self, decision_id: &str) -> Result<(), String> {
        self.observe(
            contract("reaction-opened"),
            decision_id,
            &ReactionOpenedEvent {
                decision_id: decision_id.to_owned(),
            },
        )
    }

    fn observe<T: Serialize>(
        &mut self,
        event_contract: GameplayContractRef,
        decision_id: &str,
        payload: &T,
    ) -> Result<(), String> {
        observe(&mut self.host, event_contract, decision_id, payload)
    }
}

fn observe_resolved(
    host: &mut GameplayRuntimeHost,
    decision_id: &str,
    accepted: bool,
    option_id: Option<String>,
) -> Result<(), String> {
    observe(
        host,
        contract("reaction-resolved"),
        decision_id,
        &ReactionResolvedEvent {
            decision_id: decision_id.to_owned(),
            accepted,
            option_id,
        },
    )
}

fn observe<T: Serialize>(
    host: &mut GameplayRuntimeHost,
    event_contract: GameplayContractRef,
    decision_id: &str,
    payload: &T,
) -> Result<(), String> {
    let canonical_payload = serde_json::to_vec(payload).map_err(|error| error.to_string())?;
    let sequence = u64::from(host.readout().reaction_frame_count);
    let event_id = format!("{decision_id}/state/{sequence}");
    let event = GameplayEventEnvelope {
        event_id: event_id.clone(),
        event: event_contract,
        tick: sequence,
        root_sequence: sequence,
        wave: 0,
        event_sequence: 0,
        phase: GameplayEventPhase::PostCommit,
        emitter: GameplayEmitterRef::Owner {
            owner_id: OWNER_ID.to_owned(),
        },
        causation: GameplayCausationRef {
            root_id: event_id,
            parent_event_id: None,
            decision_id: Some(decision_id.to_owned()),
        },
        source: None,
        subjects: Vec::new(),
        targets: Vec::new(),
        scope: Some("rulebench.reaction-window".to_owned()),
        tags: vec!["pre-effect".to_owned()],
        payload_hash: gameplay_canonical_payload_hash(&canonical_payload),
        canonical_payload,
    };
    let receipt = host.observe(event).map_err(|error| error.to_string())?;
    if receipt.observe.accepted() {
        Ok(())
    } else {
        Err(format!(
            "reaction state event rejected: {:?}",
            receipt.observe.diagnostics
        ))
    }
}

impl Default for RulebenchGameplayFabric {
    fn default() -> Self {
        Self::new()
    }
}

struct SuspensionOnlyOwner {
    revision: String,
}

impl GameplayRuntimeDecisionOwner for SuspensionOnlyOwner {
    fn revision_hash(&self, _owner: &GameplayOwnerRef) -> String {
        self.revision.clone()
    }

    fn route_precommit(
        &mut self,
        _owner: &GameplayOwnerRef,
        _operation: &GameplayProposalEnvelope,
    ) -> GameplayRuntimeDecisionOwnerOutput {
        GameplayRuntimeDecisionOwnerOutput {
            accepted: false,
            diagnostic_codes: vec!["unexpectedCommitBeforeReactionResolution".to_owned()],
            ..GameplayRuntimeDecisionOwnerOutput::default()
        }
    }
}

struct RulebenchOwnerAdapter<'a> {
    owner: &'a mut dyn RulebenchPreEffectOwner,
}

impl GameplayRuntimeDecisionOwner for RulebenchOwnerAdapter<'_> {
    fn revision_hash(&self, _owner: &GameplayOwnerRef) -> String {
        self.owner.revision_hash()
    }

    fn route_precommit(
        &mut self,
        _owner: &GameplayOwnerRef,
        operation: &GameplayProposalEnvelope,
    ) -> GameplayRuntimeDecisionOwnerOutput {
        let workspace: PreEffectWorkspace =
            match serde_json::from_slice(&operation.canonical_payload) {
                Ok(workspace) => workspace,
                Err(_) => {
                    return GameplayRuntimeDecisionOwnerOutput {
                        accepted: false,
                        diagnostic_codes: vec!["rulebenchWorkspaceDecodeFailed".to_owned()],
                        ..GameplayRuntimeDecisionOwnerOutput::default()
                    };
                }
            };
        match self.owner.commit(&workspace) {
            Ok(fact_hashes) => GameplayRuntimeDecisionOwnerOutput {
                accepted: true,
                fact_hashes,
                ..GameplayRuntimeDecisionOwnerOutput::default()
            },
            Err(diagnostic_codes) => GameplayRuntimeDecisionOwnerOutput {
                accepted: false,
                diagnostic_codes,
                ..GameplayRuntimeDecisionOwnerOutput::default()
            },
        }
    }
}

fn provider() -> GameplayStaticModuleProvider {
    let owner = state_owner();
    let proposal = contract("pre-effect-operation");
    let state_view = contract("reaction-state-view");
    let configuration_metadata = GameplayConfigurationSchemaMetadata {
        module_id: MODULE_ID.to_owned(),
        configuration: contract("configuration"),
        codec_id: gameplay_canonical_codec_id(&contract("configuration").schema_hash),
        fields: vec![GameplayConfigurationFieldMetadata {
            name: "acceptedReactionDamageReduction".to_owned(),
            value_type: "u32".to_owned(),
            required: true,
        }],
    };
    let manifest = GameplayModuleManifest {
        module_ref: module_ref(),
        published_events: vec![
            event_schema("reaction-opened"),
            event_schema("reaction-resolved"),
        ],
        subscriptions: vec![
            subscription("reaction-opened"),
            subscription("reaction-resolved"),
        ],
        invocations: vec![
            observe_invocation("reaction-opened"),
            observe_invocation("reaction-resolved"),
            GameplayInvocationDescriptor {
                invocation_id: "rulebench.pre-effect.transform".to_owned(),
                family: GameplayInvocationFamily::Transform,
                input_contract: proposal.clone(),
                output_contract: contract("pre-effect-workspace"),
                read_requirements: state_read_requirements(),
                max_outputs: 1,
                max_payload_bytes: 8_192,
            },
            GameplayInvocationDescriptor {
                invocation_id: "rulebench.pre-effect.react".to_owned(),
                family: GameplayInvocationFamily::React,
                input_contract: proposal.clone(),
                output_contract: contract("pre-effect-workspace"),
                read_requirements: state_read_requirements(),
                max_outputs: 1,
                max_payload_bytes: 8_192,
            },
        ],
        read_views: vec![GameplayReadViewRequirement {
            view: state_view.clone(),
            provider_id: PROVIDER_ID.to_owned(),
            kind: GameplayReadViewKind::ModuleNamed,
            fields: vec![
                "acceptedReactionDamageReduction".to_owned(),
                "lastDecisionId".to_owned(),
                "lastResolutionAccepted".to_owned(),
                "revision".to_owned(),
            ],
            selector_capabilities: vec![GameplayReadSelectorCapability::ModuleStateScope],
            max_items: 1,
        }],
        proposal_kinds: vec![GameplayProposalDeclaration {
            proposal: proposal.clone(),
            owner: combat_owner(),
        }],
        state_schemas: vec![GameplayOwnedSchemaDeclaration {
            schema: contract("reaction-state"),
            owner: owner.clone(),
        }],
        fact_schemas: vec![GameplayOwnedSchemaDeclaration {
            schema: contract("reaction-fact"),
            owner: owner.clone(),
        }],
        ordering: Vec::new(),
        budget: GameplayExecutionBudget {
            max_waves: 4,
            max_events_per_root: 8,
            max_proposals_per_root: 2,
            max_invocations_per_root: 12,
            max_payload_bytes_per_root: 32_768,
        },
        deterministic_requirements: vec!["canonical-json".to_owned()],
        source_hash: "sha256:rulebench-pre-effect-source-v1".to_owned(),
    };
    let provenance = build_provenance();
    let mut manifest = manifest;
    provenance.apply_to_manifest::<PreEffectReactionBehavior>(&mut manifest);
    GameplayStaticModuleProvider::linked_from_manifest(
        manifest,
        &provenance,
        PreEffectReactionBehavior,
    )
    .event_codec(codec::<ReactionOpenedEvent>(contract("reaction-opened")))
    .event_codec(codec::<ReactionResolvedEvent>(contract(
        "reaction-resolved",
    )))
    .proposal_codec(codec::<PreEffectWorkspace>(proposal.clone()))
    .proposal_owner(GameplayProposalOwnerRegistration {
        proposal,
        owner: combat_owner(),
    })
    .read_view_provider(GameplayReadViewProviderRegistration {
        view: state_view,
        provider_id: PROVIDER_ID.to_owned(),
        kind: GameplayReadViewKind::ModuleNamed,
        fields: vec![
            "acceptedReactionDamageReduction".to_owned(),
            "lastDecisionId".to_owned(),
            "lastResolutionAccepted".to_owned(),
            "revision".to_owned(),
        ],
        selector_capabilities: vec![GameplayReadSelectorCapability::ModuleStateScope],
        max_items: 1,
        ordering: "singleton".to_owned(),
    })
    .state_owner(GameplayStateOwnerRegistration {
        schema: contract("reaction-state"),
        owner: owner.clone(),
    })
    .state_owner(GameplayStateOwnerRegistration {
        schema: contract("reaction-fact"),
        owner,
    })
    .state_adapter(GameplayModuleStateRegistration::typed(ReactionStateAdapter))
    .configuration_schema(configuration_metadata.clone())
    .configuration_codec(GameplayConfigurationCodecRegistration::typed::<
        ReactionFabricConfig,
    >(configuration_metadata))
}

fn project_input() -> GameplayRuntimeProjectInput {
    let provider = provider();
    let linked_module = provider.manifest.module_ref.clone();
    let mut composition = GameplayStaticCompositionBuilder::new();
    composition.add_provider(provider);
    GameplayRuntimeProjectInput {
        load_plan: LoadPlan {
            steps: vec![
                LoadStep::ValidateVersions {
                    bundle_schema_version: 1,
                    protocol_version: 1,
                },
                LoadStep::LoadAssetLock {
                    artifact: "assets/lock.json".to_owned(),
                    asset_count: 0,
                },
                LoadStep::LoadSceneDocument {
                    artifact: "scene/scene.json".to_owned(),
                    scene: SceneId::new(1),
                },
                LoadStep::BootstrapScene {
                    scene: SceneId::new(1),
                    runtime_session: RuntimeSessionId::new(1),
                },
                LoadStep::ValidateFinalState,
            ],
        },
        artifacts: BundleArtifacts::new()
            .with_artifact("assets/lock.json", "{ \"entries\": [] }\n")
            .with_artifact("scene/scene.json", SCENE_JSON),
        composition: composition.build().expect("Rulebench composition"),
        bindings: binding_registry(linked_module),
        entity_targets: GameplayBindingEntityTargets::new(),
        spatial_entities: Vec::new(),
        declared_reads: declared_reads(),
        triggers: Vec::new(),
        scheduler: GameplayRuntimeSchedulerDefinition::new(
            GameplayOwnerRef {
                owner_id: "authority.rulebench.scheduler".to_owned(),
                provider_id: PROVIDER_ID.to_owned(),
            },
            Vec::new(),
            Vec::new(),
        ),
    }
}

fn binding_registry(module: GameplayModuleRef) -> GameplayModuleBindingRegistry {
    let canonical_config = serde_json::to_vec(&ReactionFabricConfig::default())
        .expect("Rulebench reaction config serializes");
    let configuration = GameplayModuleConfiguration {
        configuration_id: "rulebench.pre-effect.default".to_owned(),
        module,
        configuration: contract("configuration"),
        codec_id: gameplay_canonical_codec_id(&contract("configuration").schema_hash),
        config_hash: gameplay_module_payload_hash(&canonical_config),
        canonical_config,
    };
    let binding = GameplayModuleBinding {
        binding_id: "rulebench.pre-effect.session".to_owned(),
        module_id: MODULE_ID.to_owned(),
        configuration_id: configuration.configuration_id.clone(),
        state_schema: contract("reaction-state"),
        target: GameplayModuleBindingTarget::Session,
        required_reads: vec![GameplayReadViewRequirement {
            view: contract("reaction-state-view"),
            provider_id: PROVIDER_ID.to_owned(),
            kind: GameplayReadViewKind::ModuleNamed,
            fields: vec![
                "acceptedReactionDamageReduction".to_owned(),
                "lastDecisionId".to_owned(),
                "lastResolutionAccepted".to_owned(),
                "revision".to_owned(),
            ],
            selector_capabilities: vec![GameplayReadSelectorCapability::ModuleStateScope],
            max_items: 1,
        }],
        output_contracts: vec![contract("reaction-opened"), contract("reaction-resolved")],
        enabled: true,
    };
    let mut builder = GameplayModuleBindingRegistryBuilder::new();
    builder.configuration(configuration).binding(binding);
    builder.build()
}

fn declared_reads() -> Vec<GameplayRuntimeDeclaredReadPlan> {
    [
        "rulebench.reaction-opened.observe",
        "rulebench.reaction-resolved.observe",
        "rulebench.pre-effect.transform",
        "rulebench.pre-effect.react",
    ]
    .into_iter()
    .map(|invocation_id| GameplayRuntimeDeclaredReadPlan {
        module_id: MODULE_ID.to_owned(),
        invocation_id: invocation_id.to_owned(),
        requests: vec![GameplayReadRequest {
            request_id: STATE_READ_ID.to_owned(),
            view: contract("reaction-state-view"),
            fields: vec![
                "acceptedReactionDamageReduction".to_owned(),
                "lastDecisionId".to_owned(),
                "lastResolutionAccepted".to_owned(),
                "revision".to_owned(),
            ],
            selector: GameplayReadSelector::ModuleNamed {
                scope: GameplayModuleStateScope::Session,
            },
        }],
    })
    .collect()
}

fn decision_moment(
    workspace: PreEffectWorkspace,
    expected_owner_revision: String,
) -> GameplayDecisionMoment {
    let decision_id = workspace.decision_id.clone();
    let canonical_payload = serde_json::to_vec(&workspace).expect("Workspace serializes");
    GameplayDecisionMoment {
        decision_id: decision_id.clone(),
        operation: GameplayProposalEnvelope {
            proposal_id: format!("{decision_id}/operation"),
            proposal: contract("pre-effect-operation"),
            tick: 0,
            root_sequence: 0,
            wave: 0,
            proposal_sequence: 0,
            emitter: GameplayEmitterRef::Owner {
                owner_id: OWNER_ID.to_owned(),
            },
            causation: GameplayCausationRef {
                root_id: decision_id.clone(),
                parent_event_id: None,
                decision_id: Some(decision_id),
            },
            originating_event_id: None,
            source: None,
            targets: Vec::new(),
            payload_hash: gameplay_canonical_payload_hash(&canonical_payload),
            canonical_payload: canonical_payload.clone(),
        },
        expected_owner_revision,
        workspace: GameplayOperationWorkspace::from_payload(
            contract("pre-effect-workspace"),
            canonical_payload,
        ),
        resume_token: None,
    }
}

fn decision_evidence(receipt: &GameplayDecisionReceipt) -> RulebenchGameplayDecisionEvidence {
    RulebenchGameplayDecisionEvidence {
        decision_id: receipt.decision_id.clone(),
        status: format!("{:?}", receipt.status),
        receipt_hash: receipt.receipt_hash.clone(),
        initial_workspace_hash: receipt.initial_workspace_hash.clone(),
        final_workspace_hash: receipt.final_workspace_hash.clone(),
        declared_read_hashes: receipt
            .invocations
            .iter()
            .filter_map(|invocation| invocation.declared_read_set_hash.clone())
            .collect(),
        invocation_output_hashes: receipt
            .invocations
            .iter()
            .map(|invocation| invocation.output_hash.clone())
            .collect(),
        routing_hash: receipt
            .routing
            .as_ref()
            .map(|routing| routing.routing_hash.clone()),
        diagnostic_codes: receipt
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{:?}", diagnostic.code))
            .collect(),
    }
}

fn contract(name: &str) -> GameplayContractRef {
    gameplay_contract("rulebench.pre-effect", name, 1, schema_descriptor(name))
}

fn schema_descriptor(name: &str) -> &'static str {
    match name {
        "configuration" => {
            "rulebench.pre-effect.configuration.v1:{acceptedReactionDamageReduction:u32};canonical-json-v1"
        }
        "pre-effect-operation" => {
            "rulebench.pre-effect.operation.v1:{decisionId:string,actorId:string,targetId:string,actionId:string,damageAmount:u32,damageType:string};canonical-json-v1"
        }
        "pre-effect-workspace" => {
            "rulebench.pre-effect.workspace.v1:{decisionId:string,actorId:string,targetId:string,actionId:string,damageAmount:u32,damageType:string};canonical-json-v1"
        }
        "reaction-opened" => {
            "rulebench.pre-effect.reaction-opened.v1:{decisionId:string};canonical-json-v1"
        }
        "reaction-resolved" => {
            "rulebench.pre-effect.reaction-resolved.v1:{decisionId:string,accepted:boolean,optionId:string?};canonical-json-v1"
        }
        "reaction-fact" => {
            "rulebench.pre-effect.reaction-fact.v1:opened{decisionId:string}|resolved{decisionId:string,accepted:boolean,optionId:string?};canonical-json-v1"
        }
        "reaction-state" | "reaction-state-view" => {
            "rulebench.pre-effect.reaction-state.v1:{revision:u64,openedWindows:u64,resolvedWindows:u64,acceptedReactions:u64,lastDecisionId:string?,lastOptionId:string?,lastResolutionAccepted:boolean,acceptedReactionDamageReduction:u32};canonical-json-v1"
        }
        _ => panic!("unknown Rulebench gameplay contract: {name}"),
    }
}

fn build_provenance() -> GameplayModuleBuildProvenance {
    GameplayModuleBuildProvenance::from_build_inputs(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        &[include_bytes!("lib.rs")],
        include_bytes!("../../../Cargo.lock"),
        &[],
    )
}

fn module_ref() -> GameplayModuleRef {
    GameplayModuleRef {
        module_id: MODULE_ID.to_owned(),
        namespace: "rulebench.pre-effect".to_owned(),
        version: "1.0.0".to_owned(),
        sdk_hash: "sha256:gameplay-sdk-v1".to_owned(),
        contract_hash: "sha256:rulebench-pre-effect-contract-v1".to_owned(),
        artifact_hash: "sha256:rulebench-pre-effect-artifact-v1".to_owned(),
        provider_id: PROVIDER_ID.to_owned(),
    }
}

fn state_owner() -> GameplayOwnerRef {
    GameplayOwnerRef {
        owner_id: "authority.rulebench.pre-effect-state".to_owned(),
        provider_id: PROVIDER_ID.to_owned(),
    }
}

fn combat_owner() -> GameplayOwnerRef {
    GameplayOwnerRef {
        owner_id: OWNER_ID.to_owned(),
        provider_id: PROVIDER_ID.to_owned(),
    }
}

fn event_schema(name: &str) -> GameplayEventSchemaDeclaration {
    let event = contract(name);
    GameplayEventSchemaDeclaration {
        codec_id: gameplay_canonical_codec_id(&event.schema_hash),
        event,
    }
}

fn subscription(name: &str) -> GameplaySubscriptionDeclaration {
    GameplaySubscriptionDeclaration {
        subscription_id: format!("rulebench.{name}.subscribe"),
        event: contract(name),
        invocation_id: format!("rulebench.{name}.observe"),
        selector: GameplayHeaderSelector {
            source: None,
            target: None,
            scope: None,
            required_tags: Vec::new(),
        },
        max_deliveries_per_root: 2,
    }
}

fn observe_invocation(name: &str) -> GameplayInvocationDescriptor {
    GameplayInvocationDescriptor {
        invocation_id: format!("rulebench.{name}.observe"),
        family: GameplayInvocationFamily::Observe,
        input_contract: contract(name),
        output_contract: contract(name),
        read_requirements: state_read_requirements(),
        max_outputs: 1,
        max_payload_bytes: 4_096,
    }
}

fn state_read_requirements() -> Vec<GameplayInvocationReadRequirement> {
    vec![GameplayInvocationReadRequirement {
        request_id: STATE_READ_ID.to_owned(),
        view: contract("reaction-state-view"),
    }]
}

fn codec<T>(event: GameplayContractRef) -> GameplayEventCodecRegistration
where
    T: Serialize + for<'de> Deserialize<'de> + 'static,
{
    let descriptor = schema_descriptor(&event.name);
    gameplay_serde_json_codec_registration::<T>(event, descriptor)
}

const SCENE_JSON: &str = r#"{
  "schemaVersion": 1,
  "id": 1,
  "metadata": { "name": "rulebench-gameplay-host", "authoringFormatVersion": 1 },
  "dependencies": [],
  "nodes": [
    { "id": 1, "parent": null, "childOrder": 0, "label": null, "tags": [], "transform": { "translation": [0, 0, 0], "rotation": [0, 0, 0, 1], "scale": [1, 1, 1] }, "kind": { "kind": "emptyGroup" } }
  ]
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Owner {
        revision: u64,
        commits: Vec<PreEffectWorkspace>,
    }

    impl RulebenchPreEffectOwner for Owner {
        fn revision_hash(&self) -> String {
            format!("rulebench:{:016x}", self.revision)
        }

        fn commit(&mut self, workspace: &PreEffectWorkspace) -> Result<Vec<String>, Vec<String>> {
            self.commits.push(workspace.clone());
            self.revision = self.revision.saturating_add(1);
            Ok(vec![gameplay_module_payload_hash(
                &serde_json::to_vec(workspace).unwrap(),
            )])
        }
    }

    #[test]
    fn real_module_suspends_records_state_transforms_and_restores() {
        let mut fabric = RulebenchGameplayFabric::new();
        let mut owner = Owner::default();
        let pending = fabric
            .begin_before_effect(
                PreEffectWorkspace {
                    decision_id: "step-1".to_owned(),
                    actor_id: "adept".to_owned(),
                    target_id: "raider".to_owned(),
                    action_id: "hexing-bolt".to_owned(),
                    damage_amount: 9,
                    damage_type: "psychic".to_owned(),
                },
                owner.revision_hash(),
            )
            .unwrap();
        assert_eq!(fabric.readout().pending_decision_count, 1);
        assert_eq!(fabric.readout().reaction_frame_hashes.len(), 1);

        let snapshot = fabric.snapshot();
        let mut restored = RulebenchGameplayFabric::restore(&snapshot).unwrap();
        let receipt = restored
            .resolve_before_effect(&pending, true, Some("raider.ward".to_owned()), &mut owner)
            .unwrap();
        assert_eq!(receipt.status, GameplayDecisionStatus::Accepted);
        assert_eq!(owner.commits.len(), 1);
        assert_eq!(owner.commits[0].damage_amount, 7);
        assert_eq!(restored.readout().pending_decision_count, 0);
        assert_eq!(restored.readout().reaction_frame_hashes.len(), 2);
        assert!(restored.readout().decisions[1].routing_hash.is_some());

        let final_snapshot = restored.snapshot();
        let final_restored = RulebenchGameplayFabric::restore(&final_snapshot).unwrap();
        assert_eq!(final_restored.readout(), restored.readout());
    }

    #[test]
    fn stale_owner_rejects_before_recording_resolution_state() {
        let mut fabric = RulebenchGameplayFabric::new();
        let mut owner = Owner::default();
        let pending = fabric
            .begin_before_effect(
                PreEffectWorkspace {
                    decision_id: "stale-step".to_owned(),
                    actor_id: "adept".to_owned(),
                    target_id: "raider".to_owned(),
                    action_id: "hexing-bolt".to_owned(),
                    damage_amount: 9,
                    damage_type: "psychic".to_owned(),
                },
                owner.revision_hash(),
            )
            .unwrap();
        owner.revision = 1;
        let before = fabric.readout();

        let error = fabric
            .resolve_before_effect(&pending, true, Some("raider.ward".to_owned()), &mut owner)
            .expect_err("stale owner must fail before observing resolution");

        assert!(error.contains("stale Rulebench owner revision"));
        assert_eq!(fabric.readout(), before);
        assert!(owner.commits.is_empty());
    }

    #[test]
    fn consumed_resume_token_cannot_mutate_state_frames_or_readout() {
        let mut fabric = RulebenchGameplayFabric::new();
        let mut owner = Owner::default();
        let pending = fabric
            .begin_before_effect(
                PreEffectWorkspace {
                    decision_id: "replayed-step".to_owned(),
                    actor_id: "adept".to_owned(),
                    target_id: "raider".to_owned(),
                    action_id: "hexing-bolt".to_owned(),
                    damage_amount: 9,
                    damage_type: "psychic".to_owned(),
                },
                owner.revision_hash(),
            )
            .unwrap();
        let first = fabric
            .resolve_before_effect(&pending, true, Some("raider.ward".to_owned()), &mut owner)
            .unwrap();
        assert!(first.accepted());
        assert_eq!(owner.commits.len(), 1);

        owner.revision = 0;
        let before_readout = fabric.readout();
        let before_snapshot = fabric.snapshot();
        let replay = fabric
            .resolve_before_effect(&pending, true, Some("raider.ward".to_owned()), &mut owner)
            .unwrap();

        assert!(!replay.accepted());
        assert_eq!(owner.commits.len(), 1);
        assert_eq!(fabric.readout(), before_readout);
        assert_eq!(fabric.snapshot(), before_snapshot);
    }
}
