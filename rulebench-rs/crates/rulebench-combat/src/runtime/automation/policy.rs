use super::*;

pub const FIRST_ACCEPTED_CANDIDATE_POLICY_ID: &str = "firstAcceptedCandidate";
pub const FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatAutomationPolicyRegistration {
    pub id: &'static str,
    pub version: u32,
}

pub const COMBAT_AUTOMATION_POLICY_REGISTRY: &[CombatAutomationPolicyRegistration] =
    &[CombatAutomationPolicyRegistration {
        id: FIRST_ACCEPTED_CANDIDATE_POLICY_ID,
        version: FIRST_ACCEPTED_CANDIDATE_POLICY_VERSION,
    }];

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
}

impl CombatAutomationPolicyValidationCode {
    pub const fn code(self) -> &'static str {
        match self {
            CombatAutomationPolicyValidationCode::Accepted => "accepted",
            CombatAutomationPolicyValidationCode::UnsupportedPolicyId => "unsupportedPolicyId",
            CombatAutomationPolicyValidationCode::UnsupportedPolicyVersion => {
                "unsupportedPolicyVersion"
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationCandidateEvidence {
    pub index: usize,
    pub action_id: String,
    pub target_id: String,
    pub accepted: bool,
    pub decision_kind: CommandPreflightDecisionKind,
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
