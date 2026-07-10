use rulebench_rules::{
    CombatAutomationNoCandidateBehavior, CombatAutomationPolicySpec, CombatControlCommandSpec,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec, CombatSessionIntentCommandSpec,
    UseActionIntent,
};

pub const PROTOCOL_ID: &str = "asha-rulebench.protocol";
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolRequestContextDto {
    pub protocol_version: u32,
}

impl ProtocolRequestContextDto {
    pub const fn current() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolHandshakeDto {
    pub protocol_id: String,
    pub protocol_version: u32,
    pub authority_surface: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioOptionDto {
    pub id: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionCreateRequestDto {
    pub session_id: String,
    pub scenario_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseActionIntentDto {
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
}

impl UseActionIntentDto {
    pub fn to_authority(&self) -> UseActionIntent {
        UseActionIntent::new(&self.actor_id, &self.action_id, &self.target_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionIntentCommandDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub intent: UseActionIntentDto,
    pub roll_stream: Vec<i32>,
}

impl CombatSessionIntentCommandDto {
    pub fn to_authority(&self) -> CombatSessionIntentCommandSpec {
        CombatSessionIntentCommandSpec::new(
            &self.id,
            &self.title,
            &self.summary,
            self.intent.to_authority(),
            self.roll_stream.clone(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatControlCommandKindDto {
    ExplicitStart,
    ExplicitEnd,
    AdvanceTurn,
    EndIfConditionMet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatControlCommandDto {
    pub kind: CombatControlCommandKindDto,
}

impl CombatControlCommandDto {
    pub const fn to_authority(&self) -> CombatControlCommandSpec {
        match self.kind {
            CombatControlCommandKindDto::ExplicitStart => {
                CombatControlCommandSpec::explicit_start()
            }
            CombatControlCommandKindDto::ExplicitEnd => CombatControlCommandSpec::explicit_end(),
            CombatControlCommandKindDto::AdvanceTurn => CombatControlCommandSpec::advance_turn(),
            CombatControlCommandKindDto::EndIfConditionMet => {
                CombatControlCommandSpec::end_if_condition_met()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatAutomationNoCandidateBehaviorDto {
    AdvanceTurn,
    StopRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatAutomationPolicyDto {
    pub id: String,
    pub version: u32,
    pub no_candidate_behavior: CombatAutomationNoCandidateBehaviorDto,
}

impl CombatAutomationPolicyDto {
    pub fn to_authority(&self) -> CombatAutomationPolicySpec {
        CombatAutomationPolicySpec {
            id: self.id.clone(),
            version: self.version,
            no_candidate_behavior: match self.no_candidate_behavior {
                CombatAutomationNoCandidateBehaviorDto::AdvanceTurn => {
                    CombatAutomationNoCandidateBehavior::AdvanceTurn
                }
                CombatAutomationNoCandidateBehaviorDto::StopRun => {
                    CombatAutomationNoCandidateBehavior::StopRun
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomaticStepRequestDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub roll_stream: Vec<i32>,
    pub policy: CombatAutomationPolicyDto,
}

impl AutomaticStepRequestDto {
    pub fn to_authority(&self) -> CombatSessionAutomaticStepSpec {
        CombatSessionAutomaticStepSpec::new(
            &self.id,
            &self.title,
            &self.summary,
            self.roll_stream.clone(),
        )
        .with_policy(self.policy.to_authority())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomaticRunRequestDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub max_steps: u32,
    pub roll_stream: Vec<i32>,
    pub policy: CombatAutomationPolicyDto,
}

impl AutomaticRunRequestDto {
    pub fn to_authority(&self) -> CombatSessionAutomaticRunSpec {
        CombatSessionAutomaticRunSpec::new(
            &self.id,
            &self.title,
            &self.summary,
            self.max_steps,
            self.roll_stream.clone(),
        )
        .with_policy(self.policy.to_authority())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_commands_map_to_authority_without_rule_decisions() {
        let command = CombatSessionIntentCommandDto {
            id: "step-1".to_string(),
            title: "Use action".to_string(),
            summary: "Host-submitted command.".to_string(),
            intent: UseActionIntentDto {
                actor_id: "actor".to_string(),
                action_id: "action".to_string(),
                target_id: "target".to_string(),
            },
            roll_stream: vec![17, 5],
        };

        let authority = command.to_authority();

        assert_eq!(authority.id, "step-1");
        assert_eq!(authority.intent.actor_id, "actor");
        assert_eq!(authority.roll_stream, vec![17, 5]);
    }
}
