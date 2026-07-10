use rulebench_combat::{
    CombatControlCommandSpec, CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec,
    CombatSessionCandidateSelectionSpec, CombatSessionCreateRequest,
    CombatSessionIntentCommandSpec, CommandAuditEntry, DomainEvent, EquipmentCommandSpec,
    ReactionCommandSpec, RollConsumptionEntry, RulesetArtifactProvenance, StateFingerprint,
    TraceEntry,
};

/// The only package version currently accepted by this crate.
pub const REPLAY_PACKAGE_VERSION: &str = "1.0.0";

/// Replay fingerprints are deterministic comparison keys, not cryptographic proofs.
pub const REPLAY_PACKAGE_FINGERPRINT_KIND: &str = "deterministicNonCryptographic";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPackage {
    pub package_version: String,
    pub id: String,
    pub initial_session: CombatSessionCreateRequest,
    pub ruleset: RulesetArtifactProvenance,
    pub commands: Vec<ReplayCommandRecord>,
    pub evidence: ReplayEvidence,
    pub final_state_fingerprint: StateFingerprint,
    pub fingerprint_kind: String,
    pub narration: Option<ReplayNarration>,
}

impl ReplayPackage {
    pub fn new(
        id: impl Into<String>,
        initial_session: CombatSessionCreateRequest,
        ruleset: RulesetArtifactProvenance,
        commands: Vec<ReplayCommandRecord>,
        evidence: ReplayEvidence,
        final_state_fingerprint: StateFingerprint,
    ) -> Self {
        Self {
            package_version: REPLAY_PACKAGE_VERSION.to_string(),
            id: id.into(),
            initial_session,
            ruleset,
            commands,
            evidence,
            final_state_fingerprint,
            fingerprint_kind: REPLAY_PACKAGE_FINGERPRINT_KIND.to_string(),
            narration: None,
        }
    }

    pub fn with_narration(mut self, narration: ReplayNarration) -> Self {
        self.narration = Some(narration);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCommandRecord {
    pub sequence: u32,
    pub id: String,
    pub command: ReplayCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayCommand {
    Intent(CombatSessionIntentCommandSpec),
    Control(CombatControlCommandSpec),
    SelectedCandidate(CombatSessionCandidateSelectionSpec),
    AutomaticStep(CombatSessionAutomaticStepSpec),
    AutomaticRun(CombatSessionAutomaticRunSpec),
    Equipment(EquipmentCommandSpec),
    Reaction(ReactionCommandSpec),
}

impl ReplayCommand {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Intent(_) => "intent",
            Self::Control(_) => "control",
            Self::SelectedCandidate(_) => "selectedCandidate",
            Self::AutomaticStep(_) => "automaticStep",
            Self::AutomaticRun(_) => "automaticRun",
            Self::Equipment(_) => "equipment",
            Self::Reaction(_) => "reaction",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayEvidence {
    pub accepted_events: Vec<ReplayAcceptedEvents>,
    pub command_audit: Vec<CommandAuditEntry>,
    pub rolls: Vec<ReplayRollEvidence>,
    pub trace: Vec<ReplayTraceEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayAcceptedEvents {
    pub command_sequence: u32,
    pub events: Vec<DomainEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRollEvidence {
    pub command_sequence: u32,
    pub consumption: Vec<RollConsumptionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayTraceEvidence {
    pub command_sequence: u32,
    pub entries: Vec<TraceEntry>,
}

/// Optional presentation copy. None of these fields are replay inputs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayNarration {
    pub title: String,
    pub summary: String,
    pub command_summaries: Vec<String>,
}
