/// Bounded reaction-window state, commands, audit, and lifecycle evidence.
use super::{ReactionWindow, TraceEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionWindowStatus {
    Open,
    Resolved,
}

impl ReactionWindowStatus {
    pub const fn code(self) -> &'static str {
        match self {
            ReactionWindowStatus::Open => "open",
            ReactionWindowStatus::Resolved => "resolved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionResponseKind {
    Pass,
    Accept,
}

impl ReactionResponseKind {
    pub const fn code(self) -> &'static str {
        match self {
            ReactionResponseKind::Pass => "pass",
            ReactionResponseKind::Accept => "accept",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionOptionReadout {
    pub option_id: String,
    pub reactor_id: String,
    pub opens_nested_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionResponseEntry {
    pub sequence: u32,
    pub reactor_id: String,
    pub response_kind: ReactionResponseKind,
    pub option_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionWindowReadout {
    pub id: String,
    pub hook_id: String,
    pub timing: ReactionWindow,
    pub depth: u32,
    pub maximum_nested_depth: u32,
    pub parent_window_id: Option<String>,
    pub trigger_step_id: String,
    pub trigger_action_id: String,
    pub eligible_reactor_ids: Vec<String>,
    pub current_reactor_id: Option<String>,
    pub options: Vec<ReactionOptionReadout>,
    pub responses: Vec<ReactionResponseEntry>,
    pub status: ReactionWindowStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionCommandSpec {
    pub window_id: String,
    pub reactor_id: String,
    pub response_kind: ReactionResponseKind,
    pub option_id: Option<String>,
}

impl ReactionCommandSpec {
    pub fn pass(window_id: impl Into<String>, reactor_id: impl Into<String>) -> Self {
        Self {
            window_id: window_id.into(),
            reactor_id: reactor_id.into(),
            response_kind: ReactionResponseKind::Pass,
            option_id: None,
        }
    }

    pub fn accept(
        window_id: impl Into<String>,
        reactor_id: impl Into<String>,
        option_id: impl Into<String>,
    ) -> Self {
        Self {
            window_id: window_id.into(),
            reactor_id: reactor_id.into(),
            response_kind: ReactionResponseKind::Accept,
            option_id: Some(option_id.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionDecisionKind {
    Accepted,
    RejectedNoOpenWindow,
    RejectedStaleWindow,
    RejectedOutOfOrder,
    RejectedInvalidOption,
    RejectedNestedLimit,
}

impl ReactionDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            ReactionDecisionKind::Accepted => "accepted",
            ReactionDecisionKind::RejectedNoOpenWindow => "rejectedNoOpenWindow",
            ReactionDecisionKind::RejectedStaleWindow => "rejectedStaleWindow",
            ReactionDecisionKind::RejectedOutOfOrder => "rejectedOutOfOrder",
            ReactionDecisionKind::RejectedInvalidOption => "rejectedInvalidOption",
            ReactionDecisionKind::RejectedNestedLimit => "rejectedNestedLimit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionCommandReadout {
    pub command: ReactionCommandSpec,
    pub accepted: bool,
    pub decision_kind: ReactionDecisionKind,
    pub previous_window: Option<ReactionWindowReadout>,
    pub next_window: Option<ReactionWindowReadout>,
    pub opened_nested_window: Option<ReactionWindowReadout>,
    pub resumed_pending_resolution: bool,
    pub trace: Vec<TraceEntry>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionWindowLifecycleKind {
    Opened,
    NestedOpened,
    ResponseAccepted,
    Resolved,
    ResolutionResumed,
}

impl ReactionWindowLifecycleKind {
    pub const fn code(self) -> &'static str {
        match self {
            ReactionWindowLifecycleKind::Opened => "opened",
            ReactionWindowLifecycleKind::NestedOpened => "nestedOpened",
            ReactionWindowLifecycleKind::ResponseAccepted => "responseAccepted",
            ReactionWindowLifecycleKind::Resolved => "resolved",
            ReactionWindowLifecycleKind::ResolutionResumed => "resolutionResumed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionWindowLifecycleEntry {
    pub sequence: u32,
    pub lifecycle_kind: ReactionWindowLifecycleKind,
    pub window_id: String,
    pub parent_window_id: Option<String>,
    pub depth: u32,
    pub reactor_id: Option<String>,
    pub option_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionAuditEntry {
    pub sequence: u32,
    pub window_id: String,
    pub reactor_id: String,
    pub response_kind: ReactionResponseKind,
    pub option_id: Option<String>,
    pub accepted: bool,
    pub decision_kind: ReactionDecisionKind,
    pub trace: Vec<TraceEntry>,
    pub reason: String,
}
