//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate is deliberately small. It establishes the local authority lane:
//! typed intents enter, structural rejections fail closed, accepted facts are
//! represented as DomainEvent-shaped records, and trace/readout values explain
//! what happened. It does not claim to be upstream ASHA or a complete combat
//! resolver.

#![forbid(unsafe_code)]

/// Current local authority surface identifier.
pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

/// A proposed player, policy, or harness action.
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

/// A typed authority rejection. Rejections do not mutate state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
}

impl RulebenchRejection {
    pub const fn code(self) -> &'static str {
        match self {
            RulebenchRejection::EmptyActorId => "emptyActorId",
            RulebenchRejection::EmptyActionId => "emptyActionId",
            RulebenchRejection::EmptyTargetId => "emptyTargetId",
        }
    }
}

/// A diagnostic trace entry. Trace explains resolution; it is not authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEntry {
    pub step: u32,
    pub code: String,
    pub message: String,
}

impl TraceEntry {
    pub fn new(step: u32, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            step,
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Accepted facts emitted by local Rulebench authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    IntentShapeAccepted {
        actor_id: String,
        action_id: String,
        target_id: String,
    },
}

/// A minimal readout receipt for the authority boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchReceipt {
    pub accepted: bool,
    pub authority_surface: &'static str,
    pub rejection: Option<RulebenchRejection>,
    pub events: Vec<DomainEvent>,
    pub trace: Vec<TraceEntry>,
}

/// Validate only the structural shape of a `UseActionIntent`.
///
/// Later tasks add scenario state, target legality, deterministic rolls,
/// effects, modifiers, and final-state projection. This function exists so the
/// Rust workspace has a compiled fail-closed authority boundary before those
/// semantics arrive.
pub fn validate_intent_shape(intent: &UseActionIntent) -> RulebenchReceipt {
    let mut trace = vec![TraceEntry::new(
        1,
        "intent.received",
        "received use-action intent",
    )];

    if intent.actor_id.is_empty() {
        return rejected(RulebenchRejection::EmptyActorId, trace);
    }
    if intent.action_id.is_empty() {
        return rejected(RulebenchRejection::EmptyActionId, trace);
    }
    if intent.target_id.is_empty() {
        return rejected(RulebenchRejection::EmptyTargetId, trace);
    }

    trace.push(TraceEntry::new(
        2,
        "intent.shapeAccepted",
        "use-action intent shape accepted",
    ));

    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        rejection: None,
        events: vec![DomainEvent::IntentShapeAccepted {
            actor_id: intent.actor_id.clone(),
            action_id: intent.action_id.clone(),
            target_id: intent.target_id.clone(),
        }],
        trace,
    }
}

fn rejected(rejection: RulebenchRejection, trace: Vec<TraceEntry>) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        rejection: Some(rejection),
        events: Vec::new(),
        trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_intent_shape_emits_one_domain_event() {
        let intent = UseActionIntent::new(
            "combatant.hexwright",
            "action.hexing_bolt",
            "combatant.marauder",
        );

        let receipt = validate_intent_shape(&intent);

        assert!(receipt.accepted);
        assert_eq!(receipt.authority_surface, AUTHORITY_SURFACE);
        assert_eq!(receipt.rejection, None);
        assert_eq!(receipt.events.len(), 1);
        assert_eq!(receipt.trace.len(), 2);
        assert_eq!(receipt.trace[1].code, "intent.shapeAccepted");
    }

    #[test]
    fn empty_actor_rejects_without_events() {
        let intent = UseActionIntent::new("", "action.hexing_bolt", "combatant.marauder");

        let receipt = validate_intent_shape(&intent);

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::EmptyActorId));
        assert!(receipt.events.is_empty());
        assert_eq!(RulebenchRejection::EmptyActorId.code(), "emptyActorId");
    }
}
