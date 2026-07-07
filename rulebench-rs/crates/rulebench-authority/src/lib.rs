//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate establishes the local authority lane: typed intents enter,
//! structural rejections fail closed, accepted facts are represented as
//! DomainEvent-shaped records, and trace/readout values explain what happened.
//! It does not claim to be upstream ASHA or a complete combat resolver.

#![forbid(unsafe_code)]

/// Current local authority surface identifier.
pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

/// Stable scenario metadata used by readouts, traces, and fixture receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioMetadata {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
}

/// A tactical grid for the first Rulebench scenario model.
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

/// A bounded value such as hit points.
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
pub struct Combatant {
    pub id: String,
    pub name: String,
    pub team: Team,
    pub position: GridPosition,
    pub hit_points: BoundedValue,
    pub defenses: Vec<NamedNumber>,
    pub conditions: Vec<String>,
    pub is_actor: bool,
}

/// Scenario input/state before a proposed intent is resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchScenario {
    pub metadata: ScenarioMetadata,
    pub grid: Grid,
    pub combatants: Vec<Combatant>,
    pub selected_action: ActionDefinition,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDefinition {
    pub id: String,
    pub name: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub action_text: String,
    pub effect_text: String,
}

/// A typed authority rejection. Rejections do not mutate state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
    InvalidActor,
    InvalidAction,
    InvalidTarget,
    TargetLegalityFailed,
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
        }
    }
}

/// A diagnostic trace entry. Trace explains resolution; it is not authority.
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

/// Accepted facts emitted by local Rulebench authority.
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

/// Derived readout/projection for UI and review. It displays truth; it does not
/// own authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioProjection {
    pub summary: String,
    pub combatants: Vec<FinalCombatantState>,
}

/// A receipt for one authority pass over a proposed intent.
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

/// Validate only the structural shape of a `UseActionIntent`.
///
/// Later tasks add scenario state, target legality, deterministic rolls,
/// effects, modifiers, and final-state projection. This function exists so the
/// Rust workspace has a compiled fail-closed authority boundary before those
/// semantics arrive.
pub fn validate_intent_shape(intent: &UseActionIntent) -> RulebenchReceipt {
    let trace = vec![TraceEntry::new(
        1,
        TracePhase::Proposal,
        TraceStatus::Info,
        "UseActionIntent received.",
        "Structural intent validation started.",
    )];

    if intent.actor_id.is_empty() {
        return rejected(intent.clone(), RulebenchRejection::EmptyActorId, trace);
    }
    if intent.action_id.is_empty() {
        return rejected(intent.clone(), RulebenchRejection::EmptyActionId, trace);
    }
    if intent.target_id.is_empty() {
        return rejected(intent.clone(), RulebenchRejection::EmptyTargetId, trace);
    }

    accepted_shape(intent.clone(), trace)
}

fn accepted_shape(intent: UseActionIntent, mut trace: Vec<TraceEntry>) -> RulebenchReceipt {
    trace.push(TraceEntry::new(
        2,
        TracePhase::Validation,
        TraceStatus::Accepted,
        "Intent shape accepted.",
        "Actor, action, and target ids are present.",
    ));
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: None,
        attack_roll: None,
        damage: None,
        modifier: None,
        events: vec![DomainEvent::IntentShapeAccepted {
            actor_id: intent.actor_id,
            action_id: intent.action_id,
            target_id: intent.target_id,
        }],
        trace,
        projection: None,
    }
}

fn rejected(
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    mut trace: Vec<TraceEntry>,
) -> RulebenchReceipt {
    trace.push(TraceEntry::new(
        2,
        TracePhase::Validation,
        TraceStatus::Rejected,
        "Intent shape rejected.",
        rejection.code(),
    ));
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(rejection),
        target_legality: None,
        attack_roll: None,
        damage: None,
        modifier: None,
        events: Vec::new(),
        trace,
        projection: None,
    }
}

/// A Rust-owned representation of the current Hexing Bolt fixture input.
pub fn hexing_bolt_fixture_scenario() -> RulebenchScenario {
    RulebenchScenario {
        metadata: ScenarioMetadata {
            id: "two-combatant-hexing-bolt".to_string(),
            title: "Hexing Bolt Opening".to_string(),
            summary: "A focused two-combatant fixture for proving board, event, trace, and final-state readouts.".to_string(),
            seed_label: "roll-stream:17,5".to_string(),
        },
        grid: Grid {
            width: 6,
            height: 4,
            cells: vec![
                GridCell {
                    position: GridPosition { x: 1, y: 1 },
                    terrain_tags: vec!["clear".to_string()],
                },
                GridCell {
                    position: GridPosition { x: 4, y: 1 },
                    terrain_tags: vec!["clear".to_string()],
                },
                GridCell {
                    position: GridPosition { x: 2, y: 2 },
                    terrain_tags: vec!["cover".to_string()],
                },
            ],
        },
        combatants: vec![adept_initial(), raider_initial()],
        selected_action: ActionDefinition {
            id: "hexing_bolt".to_string(),
            name: "Hexing Bolt".to_string(),
            actor_id: "entity-adept".to_string(),
            target_ids: vec!["entity-raider".to_string()],
            action_text: "Mind vs Nerve at range 10".to_string(),
            effect_text: "1d8 + Mind psychic damage and rattled until end of next turn on hit"
                .to_string(),
        },
    }
}

/// Accepted fixture receipt. This is hand-shaped model evidence, not the
/// resolver; task #4652 owns computing it from scenario + roll stream.
pub fn accepted_hexing_bolt_fixture_receipt() -> RulebenchReceipt {
    let intent = UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider");
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: None,
        target_legality: Some(TargetLegality {
            target_id: "entity-raider".to_string(),
            accepted: true,
            reason: "Target is hostile, within range, and line of sight is clear.".to_string(),
        }),
        attack_roll: Some(AttackRollResult {
            roll: 17,
            modifier: 4,
            total: 21,
            defense_id: "nerve".to_string(),
            defense_value: 13,
            outcome: AttackOutcome::Hit,
        }),
        damage: Some(DamageOutcome {
            target_id: "entity-raider".to_string(),
            damage_type: "psychic".to_string(),
            amount: 9,
            before: BoundedValue {
                current: 18,
                max: 18,
            },
            after: BoundedValue {
                current: 9,
                max: 18,
            },
        }),
        modifier: Some(ModifierOutcome {
            target_id: "entity-raider".to_string(),
            modifier_id: "rattled".to_string(),
            label: "rattled".to_string(),
            duration: "until end of next turn".to_string(),
        }),
        events: vec![
            DomainEvent::ActionUsed {
                actor_id: "entity-adept".to_string(),
                action_id: "hexing_bolt".to_string(),
                target_id: "entity-raider".to_string(),
            },
            DomainEvent::AttackRolled {
                actor_id: "entity-adept".to_string(),
                target_id: "entity-raider".to_string(),
                total: 21,
                defense_id: "nerve".to_string(),
                defense_value: 13,
                outcome: AttackOutcome::Hit,
            },
            DomainEvent::DamageApplied {
                target_id: "entity-raider".to_string(),
                amount: 9,
                damage_type: "psychic".to_string(),
            },
            DomainEvent::ModifierApplied {
                target_id: "entity-raider".to_string(),
                modifier_id: "rattled".to_string(),
                duration: "until end of next turn".to_string(),
            },
        ],
        trace: vec![
            TraceEntry::new(
                1,
                TracePhase::Proposal,
                TraceStatus::Info,
                "UseActionIntent received.",
                "Actor entity-adept proposed action hexing_bolt against entity-raider.",
            ),
            TraceEntry::new(
                2,
                TracePhase::Validation,
                TraceStatus::Accepted,
                "Target legality accepted.",
                "The target is hostile, in range, and visible.",
            ),
            TraceEntry::new(
                3,
                TracePhase::Resolution,
                TraceStatus::Accepted,
                "Hit branch selected.",
                "Roll stream supplied 17; total 21 beats Nerve 13.",
            ),
            TraceEntry::new(
                4,
                TracePhase::Commit,
                TraceStatus::Accepted,
                "DomainEvents committed.",
                "ActionUsed, AttackRolled, DamageApplied, and ModifierApplied became accepted facts.",
            ),
        ],
        projection: Some(ScenarioProjection {
            summary: "Raider is damaged and rattled; Adept is unchanged.".to_string(),
            combatants: vec![
                final_adept(),
                FinalCombatantState {
                    id: "entity-raider".to_string(),
                    name: "Raider".to_string(),
                    hit_points: BoundedValue {
                        current: 9,
                        max: 18,
                    },
                    conditions: vec!["rattled".to_string()],
                },
            ],
        }),
    }
}

/// Rejected target fixture receipt for model coverage.
pub fn rejected_target_fixture_receipt() -> RulebenchReceipt {
    let intent = UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept");
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(RulebenchRejection::TargetLegalityFailed),
        target_legality: Some(TargetLegality {
            target_id: "entity-adept".to_string(),
            accepted: false,
            reason: "Target is not hostile.".to_string(),
        }),
        attack_roll: None,
        damage: None,
        modifier: None,
        events: Vec::new(),
        trace: vec![
            TraceEntry::new(
                1,
                TracePhase::Proposal,
                TraceStatus::Info,
                "UseActionIntent received.",
                "Actor entity-adept proposed action hexing_bolt against entity-adept.",
            ),
            TraceEntry::new(
                2,
                TracePhase::Validation,
                TraceStatus::Rejected,
                "Target legality rejected.",
                "Target is not hostile.",
            ),
        ],
        projection: Some(ScenarioProjection {
            summary: "No authority state changed; target legality rejected.".to_string(),
            combatants: vec![final_adept(), final_raider_initial()],
        }),
    }
}

fn adept_initial() -> Combatant {
    Combatant {
        id: "entity-adept".to_string(),
        name: "Adept".to_string(),
        team: Team::Ally,
        position: GridPosition { x: 1, y: 1 },
        hit_points: BoundedValue {
            current: 24,
            max: 24,
        },
        defenses: vec![
            NamedNumber {
                id: "guard".to_string(),
                label: "Guard".to_string(),
                value: 16,
            },
            NamedNumber {
                id: "nerve".to_string(),
                label: "Nerve".to_string(),
                value: 15,
            },
        ],
        conditions: Vec::new(),
        is_actor: true,
    }
}

fn raider_initial() -> Combatant {
    Combatant {
        id: "entity-raider".to_string(),
        name: "Raider".to_string(),
        team: Team::Enemy,
        position: GridPosition { x: 4, y: 1 },
        hit_points: BoundedValue {
            current: 18,
            max: 18,
        },
        defenses: vec![
            NamedNumber {
                id: "guard".to_string(),
                label: "Guard".to_string(),
                value: 14,
            },
            NamedNumber {
                id: "nerve".to_string(),
                label: "Nerve".to_string(),
                value: 13,
            },
        ],
        conditions: Vec::new(),
        is_actor: false,
    }
}

fn final_adept() -> FinalCombatantState {
    FinalCombatantState {
        id: "entity-adept".to_string(),
        name: "Adept".to_string(),
        hit_points: BoundedValue {
            current: 24,
            max: 24,
        },
        conditions: Vec::new(),
    }
}

fn final_raider_initial() -> FinalCombatantState {
    FinalCombatantState {
        id: "entity-raider".to_string(),
        name: "Raider".to_string(),
        hit_points: BoundedValue {
            current: 18,
            max: 18,
        },
        conditions: Vec::new(),
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
        assert_eq!(receipt.trace[1].phase, TracePhase::Validation);
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

    #[test]
    fn model_represents_current_accepted_hexing_bolt_fixture() {
        let scenario = hexing_bolt_fixture_scenario();
        let receipt = accepted_hexing_bolt_fixture_receipt();

        assert_eq!(scenario.metadata.id, "two-combatant-hexing-bolt");
        assert_eq!(scenario.grid.width, 6);
        assert_eq!(scenario.combatants.len(), 2);
        assert!(receipt.accepted);
        assert_eq!(receipt.events.len(), 4);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.total),
            Some(21)
        );
        assert_eq!(
            receipt.damage.as_ref().map(|damage| damage.after.current),
            Some(9)
        );
        assert_eq!(
            receipt
                .modifier
                .as_ref()
                .map(|modifier| modifier.modifier_id.as_str()),
            Some("rattled")
        );
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].conditions.as_slice()),
            Some(&["rattled".to_string()][..])
        );
    }

    #[test]
    fn model_represents_rejected_target_without_events_or_damage() {
        let receipt = rejected_target_fixture_receipt();

        assert!(!receipt.accepted);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
        assert!(receipt.damage.is_none());
        assert_eq!(
            receipt
                .target_legality
                .as_ref()
                .map(|target| target.accepted),
            Some(false)
        );
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].hit_points.current),
            Some(18)
        );
    }
}
