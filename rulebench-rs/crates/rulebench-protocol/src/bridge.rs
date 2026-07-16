use rulebench_rules::{
    CombatAutomationNoCandidateBehavior, CombatAutomationPolicySpec, CombatControlCommandSpec,
    CombatSessionAutomaticRunSpec, CombatSessionAutomaticStepSpec, CombatSessionIntentCommandSpec,
    GridPosition, UseActionIntent,
};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: &str = "asha-rulebench.protocol";
pub const PROTOCOL_VERSION: u32 = 5;

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
    #[serde(default)]
    pub content_pack: Option<crate::ContentPackReferenceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UseActionIntentDto {
    pub actor_id: String,
    pub action_id: String,
    pub target_id: String,
    #[serde(default)]
    pub target_ids: Vec<String>,
    #[serde(default)]
    pub target_cell: Option<crate::LiveGridPositionDto>,
    #[serde(default)]
    pub destination_cell: Option<crate::LiveGridPositionDto>,
    #[serde(default)]
    pub observed_origin: Option<crate::LiveGridPositionDto>,
}

impl UseActionIntentDto {
    pub fn to_authority(&self) -> UseActionIntent {
        let intent = match (
            &self.destination_cell,
            &self.target_cell,
            self.target_ids.is_empty(),
        ) {
            (Some(cell), _, _) => UseActionIntent::for_cell(
                &self.actor_id,
                &self.action_id,
                GridPosition {
                    x: cell.x,
                    y: cell.y,
                },
            ),
            (None, Some(cell), _) => UseActionIntent::for_area(
                &self.actor_id,
                &self.action_id,
                GridPosition {
                    x: cell.x,
                    y: cell.y,
                },
            ),
            (None, None, false) => UseActionIntent::for_targets(
                &self.actor_id,
                &self.action_id,
                self.target_ids.clone(),
            ),
            (None, None, true) => {
                UseActionIntent::new(&self.actor_id, &self.action_id, &self.target_id)
            }
        };
        match &self.observed_origin {
            Some(cell) => intent.with_observed_origin(GridPosition {
                x: cell.x,
                y: cell.y,
            }),
            None => intent,
        }
    }
}

impl From<&UseActionIntent> for UseActionIntentDto {
    fn from(value: &UseActionIntent) -> Self {
        Self {
            actor_id: value.actor_id.clone(),
            action_id: value.action_id.clone(),
            target_id: value.target_id.clone(),
            target_ids: value.target_ids.clone(),
            target_cell: value.target_cell.map(|cell| crate::LiveGridPositionDto {
                x: cell.x,
                y: cell.y,
            }),
            destination_cell: value
                .destination_cell
                .map(|cell| crate::LiveGridPositionDto {
                    x: cell.x,
                    y: cell.y,
                }),
            observed_origin: value
                .observed_origin
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
    #[serde(default)]
    pub roll_mode: CommandRollModeDto,
    #[serde(default)]
    pub generated_seed: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandRollModeDto {
    #[default]
    Supplied,
    AuthorityGenerated,
}

impl CombatSessionIntentCommandDto {
    pub fn to_authority(&self) -> CombatSessionIntentCommandSpec {
        let command = CombatSessionIntentCommandSpec::new(
            &self.id,
            &self.title,
            &self.summary,
            self.intent.to_authority(),
            self.roll_stream.clone(),
        );
        match self.roll_mode {
            CommandRollModeDto::Supplied => command,
            CommandRollModeDto::AuthorityGenerated => {
                command.with_generated_rolls(u64::from(self.generated_seed.unwrap_or_default()))
            }
        }
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
    #[serde(default)]
    pub roll_mode: CommandRollModeDto,
    #[serde(default)]
    pub generated_seed: Option<u32>,
}

impl AutomaticStepRequestDto {
    pub fn to_authority(&self) -> CombatSessionAutomaticStepSpec {
        let spec = CombatSessionAutomaticStepSpec::new(
            &self.id,
            &self.title,
            &self.summary,
            self.roll_stream.clone(),
        )
        .with_policy(self.policy.to_authority());
        match self.roll_mode {
            CommandRollModeDto::Supplied => spec,
            CommandRollModeDto::AuthorityGenerated => {
                spec.with_generated_rolls(u64::from(self.generated_seed.unwrap_or_default()))
            }
        }
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
    #[serde(default)]
    pub roll_mode: CommandRollModeDto,
    #[serde(default)]
    pub generated_seed: Option<u32>,
}

impl AutomaticRunRequestDto {
    pub fn to_authority(&self) -> CombatSessionAutomaticRunSpec {
        let spec = CombatSessionAutomaticRunSpec::new(
            &self.id,
            &self.title,
            &self.summary,
            self.max_steps,
            self.roll_stream.clone(),
        )
        .with_policy(self.policy.to_authority());
        match self.roll_mode {
            CommandRollModeDto::Supplied => spec,
            CommandRollModeDto::AuthorityGenerated => {
                spec.with_generated_rolls(u64::from(self.generated_seed.unwrap_or_default()))
            }
        }
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
                target_ids: Vec::new(),
                target_cell: None,
                destination_cell: None,
                observed_origin: None,
            },
            roll_stream: vec![17, 5],
            roll_mode: CommandRollModeDto::Supplied,
            generated_seed: None,
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
            target_ids: Vec::new(),
            target_cell: None,
            destination_cell: Some(crate::LiveGridPositionDto { x: 3, y: 4 }),
            observed_origin: None,
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
