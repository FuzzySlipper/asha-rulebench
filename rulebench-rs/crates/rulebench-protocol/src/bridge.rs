use rulebench_rules::{
    CombatAutomationNoCandidateBehavior, CombatAutomationPolicySpec, CombatControlCommandSpec,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec, CombatSessionIntentCommandSpec,
    GridPosition, UseActionIntent,
};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: &str = "asha-rulebench.protocol";
pub const PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProtocolHandshakeDto {
    pub protocol_id: String,
    pub protocol_version: u32,
    pub authority_surface: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScenarioOptionDto {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub ruleset_id: String,
    pub ruleset_version: String,
    pub content_pack_id: Option<String>,
    pub content_pack_version: Option<String>,
    pub participants: Vec<ScenarioParticipantOptionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScenarioParticipantOptionDto {
    pub id: String,
    pub name: String,
    pub side_id: String,
    pub initiative: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatSessionCreateRequestDto {
    pub session_id: String,
    pub scenario_id: String,
    #[serde(default)]
    pub participant_order: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UseActionIntentDto {
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
    #[serde(default)]
    pub destination_cell: Option<crate::LiveGridPositionDto>,
}

impl UseActionIntentDto {
    pub fn to_authority(&self) -> UseActionIntent {
        match &self.destination_cell {
            Some(cell) => UseActionIntent::for_cell(
                &self.actor_id,
                &self.action_id,
                GridPosition {
                    x: cell.x,
                    y: cell.y,
                },
            ),
            None => UseActionIntent::new(&self.actor_id, &self.action_id, &self.target_id),
        }
    }
}

impl From<&UseActionIntent> for UseActionIntentDto {
    fn from(value: &UseActionIntent) -> Self {
        Self {
            actor_id: value.actor_id.clone(),
            action_id: value.action_id.clone(),
            target_id: value.target_id.clone(),
            destination_cell: value
                .destination_cell
                .map(|cell| crate::LiveGridPositionDto {
                    x: cell.x,
                    y: cell.y,
                }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CombatControlCommandKindDto {
    ExplicitStart,
    ExplicitEnd,
    AdvanceTurn,
    EndIfConditionMet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CombatAutomationNoCandidateBehaviorDto {
    AdvanceTurn,
    StopRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
                destination_cell: None,
            },
            roll_stream: vec![17, 5],
        };

        let authority = command.to_authority();

        assert_eq!(authority.id, "step-1");
        assert_eq!(authority.intent.actor_id, "actor");
        assert_eq!(authority.roll_stream, vec![17, 5]);
    }

    #[test]
    fn cell_destination_round_trips_as_a_distinct_protocol_target() {
        let dto = UseActionIntentDto {
            actor_id: "actor".to_string(),
            action_id: "move".to_string(),
            target_id: String::new(),
            destination_cell: Some(crate::LiveGridPositionDto { x: 3, y: 4 }),
        };

        let json = serde_json::to_string(&dto).expect("cell intent serializes");
        let decoded: UseActionIntentDto =
            serde_json::from_str(&json).expect("cell intent deserializes");
        let authority = decoded.to_authority();

        assert_eq!(authority.target_id, "");
        assert_eq!(
            authority.destination_cell,
            Some(GridPosition { x: 3, y: 4 })
        );
    }

    #[test]
    fn legacy_entity_intent_without_destination_remains_compatible() {
        let decoded: UseActionIntentDto =
            serde_json::from_str(r#"{"actorId":"actor","actionId":"attack","targetId":"target"}"#)
                .expect("version one entity intent remains accepted");

        assert_eq!(decoded.destination_cell, None);
        assert_eq!(
            decoded.to_authority(),
            UseActionIntent::new("actor", "attack", "target")
        );
    }
}
