use rulebench_rules::{
    ReactionAuditEntry, ReactionCommandReadout, ReactionCommandSpec, ReactionDecisionKind,
    ReactionOptionReadout, ReactionResponseEntry, ReactionResponseKind,
    ReactionWindowLifecycleEntry, ReactionWindowLifecycleKind, ReactionWindowReadout,
};
use serde::{Deserialize, Serialize};

use crate::LiveTraceEntryDto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReactionResponseKindDto {
    Pass,
    Accept,
}

impl ReactionResponseKindDto {
    const fn to_authority(self) -> ReactionResponseKind {
        match self {
            Self::Pass => ReactionResponseKind::Pass,
            Self::Accept => ReactionResponseKind::Accept,
        }
    }
}

impl From<ReactionResponseKind> for ReactionResponseKindDto {
    fn from(value: ReactionResponseKind) -> Self {
        match value {
            ReactionResponseKind::Pass => Self::Pass,
            ReactionResponseKind::Accept => Self::Accept,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionCommandSpecDto {
    pub window_id: String,
    pub reactor_id: String,
    pub response_kind: ReactionResponseKindDto,
    pub option_id: Option<String>,
}

impl ReactionCommandSpecDto {
    pub fn to_authority(&self) -> ReactionCommandSpec {
        ReactionCommandSpec {
            window_id: self.window_id.clone(),
            reactor_id: self.reactor_id.clone(),
            response_kind: self.response_kind.to_authority(),
            option_id: self.option_id.clone(),
        }
    }
}

impl From<&ReactionCommandSpec> for ReactionCommandSpecDto {
    fn from(value: &ReactionCommandSpec) -> Self {
        Self {
            window_id: value.window_id.clone(),
            reactor_id: value.reactor_id.clone(),
            response_kind: value.response_kind.into(),
            option_id: value.option_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionOptionDto {
    pub option_id: String,
    pub reactor_id: String,
    pub opens_nested_window: bool,
}

impl From<&ReactionOptionReadout> for ReactionOptionDto {
    fn from(value: &ReactionOptionReadout) -> Self {
        Self {
            option_id: value.option_id.clone(),
            reactor_id: value.reactor_id.clone(),
            opens_nested_window: value.opens_nested_window,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionResponseEntryDto {
    pub sequence: u32,
    pub reactor_id: String,
    pub response_kind: ReactionResponseKindDto,
    pub option_id: Option<String>,
}

impl From<&ReactionResponseEntry> for ReactionResponseEntryDto {
    fn from(value: &ReactionResponseEntry) -> Self {
        Self {
            sequence: value.sequence,
            reactor_id: value.reactor_id.clone(),
            response_kind: value.response_kind.into(),
            option_id: value.option_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionWindowDto {
    pub id: String,
    pub hook_id: String,
    pub timing: String,
    pub depth: u32,
    pub maximum_nested_depth: u32,
    pub parent_window_id: Option<String>,
    pub trigger_step_id: String,
    pub trigger_action_id: String,
    pub eligible_reactor_ids: Vec<String>,
    pub current_reactor_id: Option<String>,
    pub options: Vec<ReactionOptionDto>,
    pub responses: Vec<ReactionResponseEntryDto>,
    pub status: String,
}

impl From<&ReactionWindowReadout> for ReactionWindowDto {
    fn from(value: &ReactionWindowReadout) -> Self {
        Self {
            id: value.id.clone(),
            hook_id: value.hook_id.clone(),
            timing: value.timing.code().to_owned(),
            depth: value.depth,
            maximum_nested_depth: value.maximum_nested_depth,
            parent_window_id: value.parent_window_id.clone(),
            trigger_step_id: value.trigger_step_id.clone(),
            trigger_action_id: value.trigger_action_id.clone(),
            eligible_reactor_ids: value.eligible_reactor_ids.clone(),
            current_reactor_id: value.current_reactor_id.clone(),
            options: value.options.iter().map(ReactionOptionDto::from).collect(),
            responses: value
                .responses
                .iter()
                .map(ReactionResponseEntryDto::from)
                .collect(),
            status: value.status.code().to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionCommandReadoutDto {
    pub command: ReactionCommandSpecDto,
    pub accepted: bool,
    pub decision_kind: String,
    pub previous_window: Option<ReactionWindowDto>,
    pub next_window: Option<ReactionWindowDto>,
    pub opened_nested_window: Option<ReactionWindowDto>,
    pub resumed_pending_resolution: bool,
    pub trace: Vec<LiveTraceEntryDto>,
    pub reason: String,
}

impl From<&ReactionCommandReadout> for ReactionCommandReadoutDto {
    fn from(value: &ReactionCommandReadout) -> Self {
        Self {
            command: ReactionCommandSpecDto::from(&value.command),
            accepted: value.accepted,
            decision_kind: value.decision_kind.code().to_owned(),
            previous_window: value.previous_window.as_ref().map(ReactionWindowDto::from),
            next_window: value.next_window.as_ref().map(ReactionWindowDto::from),
            opened_nested_window: value
                .opened_nested_window
                .as_ref()
                .map(ReactionWindowDto::from),
            resumed_pending_resolution: value.resumed_pending_resolution,
            trace: value.trace.iter().map(LiveTraceEntryDto::from).collect(),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionWindowLifecycleEntryDto {
    pub sequence: u32,
    pub lifecycle_kind: String,
    pub window_id: String,
    pub parent_window_id: Option<String>,
    pub depth: u32,
    pub reactor_id: Option<String>,
    pub option_id: Option<String>,
    pub reason: String,
}

impl From<&ReactionWindowLifecycleEntry> for ReactionWindowLifecycleEntryDto {
    fn from(value: &ReactionWindowLifecycleEntry) -> Self {
        Self {
            sequence: value.sequence,
            lifecycle_kind: lifecycle_code(value.lifecycle_kind).to_owned(),
            window_id: value.window_id.clone(),
            parent_window_id: value.parent_window_id.clone(),
            depth: value.depth,
            reactor_id: value.reactor_id.clone(),
            option_id: value.option_id.clone(),
            reason: value.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReactionAuditEntryDto {
    pub sequence: u32,
    pub window_id: String,
    pub reactor_id: String,
    pub response_kind: ReactionResponseKindDto,
    pub option_id: Option<String>,
    pub accepted: bool,
    pub decision_kind: String,
    pub trace: Vec<LiveTraceEntryDto>,
    pub reason: String,
}

impl From<&ReactionAuditEntry> for ReactionAuditEntryDto {
    fn from(value: &ReactionAuditEntry) -> Self {
        Self {
            sequence: value.sequence,
            window_id: value.window_id.clone(),
            reactor_id: value.reactor_id.clone(),
            response_kind: value.response_kind.into(),
            option_id: value.option_id.clone(),
            accepted: value.accepted,
            decision_kind: decision_code(value.decision_kind).to_owned(),
            trace: value.trace.iter().map(LiveTraceEntryDto::from).collect(),
            reason: value.reason.clone(),
        }
    }
}

const fn decision_code(value: ReactionDecisionKind) -> &'static str {
    value.code()
}

const fn lifecycle_code(value: ReactionWindowLifecycleKind) -> &'static str {
    value.code()
}
