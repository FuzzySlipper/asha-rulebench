use super::*;

pub const FIRST_ACCEPTED_CANDIDATE_POLICY_ID: &str = "firstAcceptedCandidate";
pub const FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION: u32 = 1;
pub const LOWEST_VITALITY_TARGET_POLICY_ID: &str = "lowestVitalityTarget";
pub const LOWEST_VITALITY_TARGET_POLICY_VERSION: u32 = 1;
pub const OBJECTIVE_SIDE_PRESSURE_POLICY_ID: &str = "objectiveSidePressure";
pub const OBJECTIVE_SIDE_PRESSURE_POLICY_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAutomationPolicySelector {
    FirstAcceptedCandidate,
    LowestVitalityTarget,
    ObjectiveSidePressure,
}

impl CombatAutomationPolicySelector {
    pub const fn code(self) -> &'static str {
        match self {
            Self::FirstAcceptedCandidate => "firstAcceptedCandidate",
            Self::LowestVitalityTarget => "lowestVitalityTarget",
            Self::ObjectiveSidePressure => "objectiveSidePressure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAutomationPolicyRequirement {
    AnyCombatRuleset,
    ObjectiveSidePolicy,
}

impl CombatAutomationPolicyRequirement {
    pub const fn code(self) -> &'static str {
        match self {
            Self::AnyCombatRuleset => "anyCombatRuleset",
            Self::ObjectiveSidePolicy => "objectiveSidePolicy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatAutomationPolicyRegistration {
    pub id: &'static str,
    pub version: u32,
    pub title: &'static str,
    pub summary: &'static str,
    pub selector: CombatAutomationPolicySelector,
    pub requirement: CombatAutomationPolicyRequirement,
}

pub const COMBAT_AUTOMATION_POLICY_REGISTRY: &[CombatAutomationPolicyRegistration] = &[
    CombatAutomationPolicyRegistration {
        id: FIRST_ACCEPTED_CANDIDATE_POLICY_ID,
        version: FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION,
        title: "First accepted candidate",
        summary: "Select the first accepted Rust-projected command candidate.",
        selector: CombatAutomationPolicySelector::FirstAcceptedCandidate,
        requirement: CombatAutomationPolicyRequirement::AnyCombatRuleset,
    },
    CombatAutomationPolicyRegistration {
        id: LOWEST_VITALITY_TARGET_POLICY_ID,
        version: LOWEST_VITALITY_TARGET_POLICY_VERSION,
        title: "Lowest vitality target",
        summary: "Prefer the accepted target with the lowest remaining vitality ratio.",
        selector: CombatAutomationPolicySelector::LowestVitalityTarget,
        requirement: CombatAutomationPolicyRequirement::AnyCombatRuleset,
    },
    CombatAutomationPolicyRegistration {
        id: OBJECTIVE_SIDE_PRESSURE_POLICY_ID,
        version: OBJECTIVE_SIDE_PRESSURE_POLICY_VERSION,
        title: "Objective-side pressure",
        summary:
            "Prefer accepted targets on the ruleset-declared objective side, then lowest vitality.",
        selector: CombatAutomationPolicySelector::ObjectiveSidePressure,
        requirement: CombatAutomationPolicyRequirement::ObjectiveSidePolicy,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAutomationNoCandidateBehavior {
    AdvanceTurn,
    StopRun,
}

impl CombatAutomationNoCandidateBehavior {
    pub const fn code(self) -> &'static str {
        match self {
            CombatAutomationNoCandidateBehavior::AdvanceTurn => "advanceTurn",
            CombatAutomationNoCandidateBehavior::StopRun => "stopRun",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationPolicySpec {
    pub id: String,
    pub version: u32,
    pub no_candidate_behavior: CombatAutomationNoCandidateBehavior,
}

impl CombatAutomationPolicySpec {
    pub fn first_accepted_candidate() -> Self {
        Self {
            id: FIRST_ACCEPTED_CANDIDATE_POLICY_ID.to_string(),
            version: FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION,
            no_candidate_behavior: CombatAutomationNoCandidateBehavior::AdvanceTurn,
        }
    }

    pub fn lowest_vitality_target() -> Self {
        Self {
            id: LOWEST_VITALITY_TARGET_POLICY_ID.to_string(),
            version: LOWEST_VITALITY_TARGET_POLICY_VERSION,
            no_candidate_behavior: CombatAutomationNoCandidateBehavior::AdvanceTurn,
        }
    }

    pub fn objective_side_pressure() -> Self {
        Self {
            id: OBJECTIVE_SIDE_PRESSURE_POLICY_ID.to_string(),
            version: OBJECTIVE_SIDE_PRESSURE_POLICY_VERSION,
            no_candidate_behavior: CombatAutomationNoCandidateBehavior::AdvanceTurn,
        }
    }

    pub fn with_no_candidate_behavior(
        mut self,
        no_candidate_behavior: CombatAutomationNoCandidateBehavior,
    ) -> Self {
        self.no_candidate_behavior = no_candidate_behavior;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAutomationPolicyValidationCode {
    Accepted,
    UnsupportedPolicyId,
    UnsupportedPolicyVersion,
    IncompatibleRulesetCapability,
}

impl CombatAutomationPolicyValidationCode {
    pub const fn code(self) -> &'static str {
        match self {
            CombatAutomationPolicyValidationCode::Accepted => "accepted",
            CombatAutomationPolicyValidationCode::UnsupportedPolicyId => "unsupportedPolicyId",
            CombatAutomationPolicyValidationCode::UnsupportedPolicyVersion => {
                "unsupportedPolicyVersion"
            }
            CombatAutomationPolicyValidationCode::IncompatibleRulesetCapability => {
                "incompatibleRulesetCapability"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationPolicyValidationReadout {
    pub accepted: bool,
    pub code: CombatAutomationPolicyValidationCode,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationPolicyContext {
    pub ruleset_id: String,
    pub objective_side_id: Option<String>,
}

pub fn validate_combat_automation_policy(
    policy: &CombatAutomationPolicySpec,
) -> CombatAutomationPolicyValidationReadout {
    let registration = COMBAT_AUTOMATION_POLICY_REGISTRY
        .iter()
        .find(|registration| registration.id == policy.id);
    let Some(registration) = registration else {
        return CombatAutomationPolicyValidationReadout {
            accepted: false,
            code: CombatAutomationPolicyValidationCode::UnsupportedPolicyId,
            reason: format!("Unsupported combat automation policy id {}.", policy.id),
        };
    };
    if policy.version != registration.version {
        return CombatAutomationPolicyValidationReadout {
            accepted: false,
            code: CombatAutomationPolicyValidationCode::UnsupportedPolicyVersion,
            reason: format!(
                "Unsupported version {} for combat automation policy {}.",
                policy.version, policy.id
            ),
        };
    }

    CombatAutomationPolicyValidationReadout {
        accepted: true,
        code: CombatAutomationPolicyValidationCode::Accepted,
        reason: "Combat automation policy is supported by Rust authority.".to_string(),
    }
}

pub fn validate_combat_automation_policy_for_context(
    policy: &CombatAutomationPolicySpec,
    context: &CombatAutomationPolicyContext,
) -> CombatAutomationPolicyValidationReadout {
    let base = validate_combat_automation_policy(policy);
    if !base.accepted {
        return base;
    }
    let Some(registration) = COMBAT_AUTOMATION_POLICY_REGISTRY
        .iter()
        .find(|registration| registration.id == policy.id)
    else {
        return CombatAutomationPolicyValidationReadout {
            accepted: false,
            code: CombatAutomationPolicyValidationCode::UnsupportedPolicyId,
            reason: format!("Unsupported combat automation policy id {}.", policy.id),
        };
    };
    if registration.requirement == CombatAutomationPolicyRequirement::ObjectiveSidePolicy
        && context.objective_side_id.is_none()
    {
        return CombatAutomationPolicyValidationReadout {
            accepted: false,
            code: CombatAutomationPolicyValidationCode::IncompatibleRulesetCapability,
            reason: format!(
                "Combat automation policy {} requires an objective-side ruleset policy; ruleset {} does not declare one.",
                policy.id, context.ruleset_id
            ),
        };
    }
    base
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationCandidateEvidence {
    pub index: usize,
    pub action_id: String,
    pub target_id: String,
    pub target_side_id: String,
    pub target_current_hit_points: i32,
    pub target_max_hit_points: i32,
    pub accepted: bool,
    pub decision_kind: CommandPreflightDecisionKind,
    pub policy_score: i64,
    pub policy_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationPolicyDecisionEvidence {
    pub policy: CombatAutomationPolicySpec,
    pub state_before_fingerprint: StateFingerprint,
    pub operation_kind: Option<CombatSessionAutomaticStepOperationKind>,
    pub selected_action_id: Option<String>,
    pub selected_target_id: Option<String>,
    pub selected_candidate_index: Option<usize>,
    pub candidate_count: usize,
    pub accepted_candidate_count: usize,
    pub candidates: Vec<CombatAutomationCandidateEvidence>,
    pub reason: String,
}
