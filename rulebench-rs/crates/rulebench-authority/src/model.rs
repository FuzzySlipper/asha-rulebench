pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioOutcomeClass {
    AcceptedHit,
    AcceptedMiss,
    RejectedTargetLegality,
}

impl ScenarioOutcomeClass {
    pub const fn code(self) -> &'static str {
        match self {
            ScenarioOutcomeClass::AcceptedHit => "acceptedHit",
            ScenarioOutcomeClass::AcceptedMiss => "acceptedMiss",
            ScenarioOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub outcome_class: ScenarioOutcomeClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogCase {
    pub summary: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub intent: UseActionIntent,
    pub roll_stream: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogResolution {
    pub case: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub receipt: RulebenchReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioCatalogError {
    UnknownScenarioId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOutcomeClass {
    AcceptedHit,
    AcceptedMiss,
    RejectedTargetLegality,
}

impl CommandOutcomeClass {
    pub const fn code(self) -> &'static str {
        match self {
            CommandOutcomeClass::AcceptedHit => "acceptedHit",
            CommandOutcomeClass::AcceptedMiss => "acceptedMiss",
            CommandOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub steps: Vec<CombatSessionStepSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionStepSummary {
    pub id: String,
    pub index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: CommandOutcomeClass,
    pub log_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAttempt {
    pub step_id: String,
    pub step_index: u32,
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
    pub roll_stream: Vec<i32>,
    pub outcome_class: CommandOutcomeClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatLogEntry {
    pub id: String,
    pub step_id: String,
    pub log_index: u32,
    pub title: String,
    pub summary: String,
    pub outcome_class: CommandOutcomeClass,
    pub event_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionTranscript {
    pub summary: CombatSessionSummary,
    pub steps: Vec<CombatSessionStepReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionStepReadout {
    pub session_id: String,
    pub step: CombatSessionStepSummary,
    pub command: CommandAttempt,
    pub scenario: RulebenchScenario,
    pub receipt: RulebenchReceipt,
    pub combat_log: Vec<CombatLogEntry>,
    pub state_before: ScenarioProjection,
    pub state_after: ScenarioProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionError {
    UnknownSessionId,
    UnknownStepId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioMetadata {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<GridCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridCell {
    pub position: GridPosition,
    pub terrain_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Ally,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedValue {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedNumber {
    pub id: String,
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatBlock {
    pub base_stats: Vec<NamedNumber>,
    pub derived_stats: Vec<NamedNumber>,
}

impl StatBlock {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.base_stats
            .iter()
            .chain(self.derived_stats.iter())
            .find(|stat| stat.id == stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combatant {
    pub id: String,
    pub name: String,
    pub team: Team,
    pub position: GridPosition,
    pub hit_points: BoundedValue,
    pub stats: StatBlock,
    pub defenses: Vec<NamedNumber>,
    pub conditions: Vec<String>,
    pub is_actor: bool,
}

impl Combatant {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.stats.stat_by_id(stat_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchScenario {
    pub metadata: ScenarioMetadata,
    pub grid: Grid,
    pub combatants: Vec<Combatant>,
    pub actions: Vec<ActionDefinition>,
    pub selected_action: ActionDefinition,
}

impl RulebenchScenario {
    pub fn action_by_id(&self, action_id: &str) -> Option<&ActionDefinition> {
        self.actions.iter().find(|action| action.id == action_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseActionIntent {
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
}

impl UseActionIntent {
    pub fn new(
        actor_id: impl Into<String>,
        action_id: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_id: action_id.into(),
            target_id: target_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDefinition {
    pub id: String,
    pub name: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub range: u32,
    pub line_of_sight_required: bool,
    pub visible_target_ids: Vec<String>,
    pub attack: AttackSpec,
    pub hit: HitEffect,
    pub action_text: String,
    pub effect_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackSpec {
    pub modifier: i32,
    pub defense_id: String,
    pub defense_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitEffect {
    pub damage_bonus: i32,
    pub damage_type: String,
    pub modifier_id: String,
    pub modifier_label: String,
    pub modifier_duration: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
    InvalidActor,
    InvalidAction,
    InvalidTarget,
    TargetLegalityFailed,
    TargetOutOfRange,
    TargetNotVisible,
    MissingAttackRoll,
    MissingDamageRoll,
}

impl RulebenchRejection {
    pub const fn code(self) -> &'static str {
        match self {
            RulebenchRejection::EmptyActorId => "emptyActorId",
            RulebenchRejection::EmptyActionId => "emptyActionId",
            RulebenchRejection::EmptyTargetId => "emptyTargetId",
            RulebenchRejection::InvalidActor => "invalidActor",
            RulebenchRejection::InvalidAction => "invalidAction",
            RulebenchRejection::InvalidTarget => "invalidTarget",
            RulebenchRejection::TargetLegalityFailed => "targetLegalityFailed",
            RulebenchRejection::TargetOutOfRange => "targetOutOfRange",
            RulebenchRejection::TargetNotVisible => "targetNotVisible",
            RulebenchRejection::MissingAttackRoll => "missingAttackRoll",
            RulebenchRejection::MissingDamageRoll => "missingDamageRoll",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEntry {
    pub sequence: u32,
    pub phase: TracePhase,
    pub status: TraceStatus,
    pub message: String,
    pub detail: String,
}

impl TraceEntry {
    pub fn new(
        sequence: u32,
        phase: TracePhase,
        status: TraceStatus,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            sequence,
            phase,
            status,
            message: message.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracePhase {
    Proposal,
    Validation,
    Resolution,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceStatus {
    Accepted,
    Rejected,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetLegality {
    pub target_id: String,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackOutcome {
    Hit,
    Miss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackRollResult {
    pub roll: i32,
    pub modifier: i32,
    pub total: i32,
    pub defense_id: String,
    pub defense_value: i32,
    pub outcome: AttackOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageOutcome {
    pub target_id: String,
    pub damage_type: String,
    pub amount: i32,
    pub before: BoundedValue,
    pub after: BoundedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierOutcome {
    pub target_id: String,
    pub modifier_id: String,
    pub label: String,
    pub duration: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    IntentShapeAccepted {
        actor_id: String,
        action_id: String,
        target_id: String,
    },
    ActionUsed {
        actor_id: String,
        action_id: String,
        target_id: String,
    },
    AttackRolled {
        actor_id: String,
        target_id: String,
        total: i32,
        defense_id: String,
        defense_value: i32,
        outcome: AttackOutcome,
    },
    DamageApplied {
        target_id: String,
        amount: i32,
        damage_type: String,
    },
    ModifierApplied {
        target_id: String,
        modifier_id: String,
        duration: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalCombatantState {
    pub id: String,
    pub name: String,
    pub hit_points: BoundedValue,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioProjection {
    pub summary: String,
    pub combatants: Vec<FinalCombatantState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchReceipt {
    pub accepted: bool,
    pub authority_surface: &'static str,
    pub intent: UseActionIntent,
    pub rejection: Option<RulebenchRejection>,
    pub target_legality: Option<TargetLegality>,
    pub attack_roll: Option<AttackRollResult>,
    pub damage: Option<DamageOutcome>,
    pub modifier: Option<ModifierOutcome>,
    pub events: Vec<DomainEvent>,
    pub trace: Vec<TraceEntry>,
    pub projection: Option<ScenarioProjection>,
}
