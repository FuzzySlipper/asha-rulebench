//! Local Rust authority incubation surface for ASHA Rulebench.
//!
//! This crate establishes the local authority lane: typed intents enter,
//! rejections fail closed, accepted facts are represented as DomainEvent-shaped
//! records, and trace/readout values explain what happened. It does not claim to
//! be upstream ASHA or a complete combat resolver.

#![forbid(unsafe_code)]

pub const AUTHORITY_SURFACE: &str = "asha-rulebench.local-authority.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioOutcomeClass {
    AcceptedHit,
    AcceptedMiss,
    RejectedTargetLegality,
}

impl ScenarioOutcomeClass {
    pub const fn code(self) -> &'static str {
        match self {
            ScenarioOutcomeClass::AcceptedHit => "acceptedHit",
            ScenarioOutcomeClass::AcceptedMiss => "acceptedMiss",
            ScenarioOutcomeClass::RejectedTargetLegality => "rejectedTargetLegality",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub outcome_class: ScenarioOutcomeClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogCase {
    pub summary: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub intent: UseActionIntent,
    pub roll_stream: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioCatalogResolution {
    pub case: ScenarioCatalogSummary,
    pub scenario: RulebenchScenario,
    pub receipt: RulebenchReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioCatalogError {
    UnknownScenarioId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioMetadata {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchScenario {
    pub metadata: ScenarioMetadata,
    pub grid: Grid,
    pub combatants: Vec<Combatant>,
    pub selected_action: ActionDefinition,
}

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
    pub range: u32,
    pub line_of_sight_required: bool,
    pub visible_target_ids: Vec<String>,
    pub attack: AttackSpec,
    pub hit: HitEffect,
    pub action_text: String,
    pub effect_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackSpec {
    pub modifier: i32,
    pub defense_id: String,
    pub defense_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitEffect {
    pub damage_bonus: i32,
    pub damage_type: String,
    pub modifier_id: String,
    pub modifier_label: String,
    pub modifier_duration: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulebenchRejection {
    EmptyActorId,
    EmptyActionId,
    EmptyTargetId,
    InvalidActor,
    InvalidAction,
    InvalidTarget,
    TargetLegalityFailed,
    TargetOutOfRange,
    TargetNotVisible,
    MissingAttackRoll,
    MissingDamageRoll,
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
            RulebenchRejection::TargetOutOfRange => "targetOutOfRange",
            RulebenchRejection::TargetNotVisible => "targetNotVisible",
            RulebenchRejection::MissingAttackRoll => "missingAttackRoll",
            RulebenchRejection::MissingDamageRoll => "missingDamageRoll",
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioProjection {
    pub summary: String,
    pub combatants: Vec<FinalCombatantState>,
}

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

/// Resolve the first local Hexing Bolt-shaped action path.
///
/// The resolver is intentionally narrow and deterministic. It consumes a
/// scenario, a typed intent, and an explicit roll stream. It returns accepted
/// DomainEvents plus final projection, or a typed rejection with no accepted
/// events and unchanged projection.
pub fn resolve_use_action(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    roll_stream: &[i32],
) -> RulebenchReceipt {
    let trace = vec![TraceEntry::new(
        1,
        TracePhase::Proposal,
        TraceStatus::Info,
        "UseActionIntent received.",
        format!(
            "Actor {} proposed action {} against {}.",
            intent.actor_id, intent.action_id, intent.target_id
        ),
    )];

    if intent.actor_id.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::EmptyActorId,
            None,
            trace,
        );
    }
    if intent.action_id.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::EmptyActionId,
            None,
            trace,
        );
    }
    if intent.target_id.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::EmptyTargetId,
            None,
            trace,
        );
    }

    let Some(actor) = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == intent.actor_id)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidActor,
            None,
            trace,
        );
    };

    let action = &scenario.selected_action;
    if action.id != intent.action_id || action.actor_id != intent.actor_id {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidAction,
            None,
            trace,
        );
    }

    let Some(target) = scenario
        .combatants
        .iter()
        .find(|combatant| combatant.id == intent.target_id)
    else {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::InvalidTarget,
            None,
            trace,
        );
    };

    let target_legality = validate_target_legality(actor, target, action);
    if !target_legality.accepted {
        let rejection = target_legality_rejection(&target_legality);
        return rejected_with_projection(scenario, intent, rejection, Some(target_legality), trace);
    }

    if roll_stream.is_empty() {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::MissingAttackRoll,
            Some(target_legality),
            trace,
        );
    }
    if roll_stream.len() < 2 {
        return rejected_with_projection(
            scenario,
            intent,
            RulebenchRejection::MissingDamageRoll,
            Some(target_legality),
            trace,
        );
    }

    resolve_accepted_action(
        scenario,
        intent,
        actor,
        target,
        target_legality,
        roll_stream,
    )
}

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
            range: 10,
            line_of_sight_required: true,
            visible_target_ids: vec!["entity-raider".to_string()],
            attack: AttackSpec {
                modifier: 4,
                defense_id: "nerve".to_string(),
                defense_label: "Nerve".to_string(),
            },
            hit: HitEffect {
                damage_bonus: 4,
                damage_type: "psychic".to_string(),
                modifier_id: "rattled".to_string(),
                modifier_label: "rattled".to_string(),
                modifier_duration: "until end of next turn".to_string(),
            },
            action_text: "Mind vs Nerve at range 10".to_string(),
            effect_text: "1d8 + Mind psychic damage and rattled until end of next turn on hit"
                .to_string(),
        },
    }
}

pub fn accepted_hexing_bolt_fixture_receipt() -> RulebenchReceipt {
    resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        &[17, 5],
    )
}

pub fn rejected_target_fixture_receipt() -> RulebenchReceipt {
    resolve_use_action(
        &hexing_bolt_fixture_scenario(),
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        &[17, 5],
    )
}

pub fn scenario_catalog_summaries() -> Vec<ScenarioCatalogSummary> {
    scenario_catalog_cases()
        .into_iter()
        .map(|case| case.summary)
        .collect()
}

pub fn scenario_catalog_cases() -> Vec<ScenarioCatalogCase> {
    vec![
        accepted_hit_catalog_case(),
        accepted_miss_catalog_case(),
        rejected_target_legality_catalog_case(),
    ]
}

pub fn resolve_catalog_scenario(
    id: &str,
) -> Result<ScenarioCatalogResolution, ScenarioCatalogError> {
    let Some(case) = scenario_catalog_cases()
        .into_iter()
        .find(|case| case.summary.id == id)
    else {
        return Err(ScenarioCatalogError::UnknownScenarioId);
    };
    let receipt = resolve_use_action(&case.scenario, case.intent.clone(), &case.roll_stream);
    Ok(ScenarioCatalogResolution {
        case: case.summary,
        scenario: case.scenario,
        receipt,
    })
}

fn accepted_hit_catalog_case() -> ScenarioCatalogCase {
    catalog_case(
        "hexing-bolt-hit",
        "Hexing Bolt Hit",
        "Adept hits Raider, applying psychic damage and rattled.",
        "roll-stream:17,5",
        ScenarioOutcomeClass::AcceptedHit,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![17, 5],
    )
}

fn accepted_miss_catalog_case() -> ScenarioCatalogCase {
    catalog_case(
        "hexing-bolt-miss",
        "Hexing Bolt Miss",
        "Adept targets Raider but the attack misses, leaving state unchanged.",
        "roll-stream:2,5",
        ScenarioOutcomeClass::AcceptedMiss,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
        vec![2, 5],
    )
}

fn rejected_target_legality_catalog_case() -> ScenarioCatalogCase {
    catalog_case(
        "hexing-bolt-self-target-rejected",
        "Hexing Bolt Self Target Rejected",
        "Adept attempts to target themself and target legality rejects the intent.",
        "roll-stream:17,5",
        ScenarioOutcomeClass::RejectedTargetLegality,
        UseActionIntent::new("entity-adept", "hexing_bolt", "entity-adept"),
        vec![17, 5],
    )
}

fn catalog_case(
    id: &str,
    title: &str,
    summary: &str,
    seed_label: &str,
    outcome_class: ScenarioOutcomeClass,
    intent: UseActionIntent,
    roll_stream: Vec<i32>,
) -> ScenarioCatalogCase {
    let mut scenario = hexing_bolt_fixture_scenario();
    scenario.metadata = ScenarioMetadata {
        id: id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        seed_label: seed_label.to_string(),
    };
    ScenarioCatalogCase {
        summary: ScenarioCatalogSummary {
            id: id.to_string(),
            title: title.to_string(),
            summary: summary.to_string(),
            seed_label: seed_label.to_string(),
            outcome_class,
        },
        scenario,
        intent,
        roll_stream,
    }
}

fn resolve_accepted_action(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    _actor: &Combatant,
    target: &Combatant,
    target_legality: TargetLegality,
    roll_stream: &[i32],
) -> RulebenchReceipt {
    let action = &scenario.selected_action;
    let defense_value = defense_value(target, &action.attack.defense_id);
    let total = roll_stream[0] + action.attack.modifier;
    let attack_roll = AttackRollResult {
        roll: roll_stream[0],
        modifier: action.attack.modifier,
        total,
        defense_id: action.attack.defense_id.clone(),
        defense_value,
        outcome: if total >= defense_value {
            AttackOutcome::Hit
        } else {
            AttackOutcome::Miss
        },
    };

    let mut trace = vec![
        TraceEntry::new(
            1,
            TracePhase::Proposal,
            TraceStatus::Info,
            "UseActionIntent received.",
            format!(
                "Actor {} proposed action {} against {}.",
                intent.actor_id, intent.action_id, intent.target_id
            ),
        ),
        TraceEntry::new(
            2,
            TracePhase::Validation,
            TraceStatus::Accepted,
            "Target legality accepted.",
            target_legality.reason.clone(),
        ),
    ];

    if attack_roll.outcome == AttackOutcome::Miss {
        trace.push(TraceEntry::new(
            3,
            TracePhase::Resolution,
            TraceStatus::Accepted,
            "Miss branch selected.",
            format!(
                "Roll stream supplied {}; total {} misses {} {}.",
                attack_roll.roll, attack_roll.total, action.attack.defense_label, defense_value
            ),
        ));
        trace.push(TraceEntry::new(
            4,
            TracePhase::Commit,
            TraceStatus::Accepted,
            "DomainEvents committed.",
            "ActionUsed and AttackRolled became accepted facts.",
        ));
        return accepted_miss_receipt(scenario, intent, target_legality, attack_roll, trace);
    }

    let damage = apply_damage(
        target,
        roll_stream[1] + action.hit.damage_bonus,
        &action.hit.damage_type,
    );
    let modifier = ModifierOutcome {
        target_id: target.id.clone(),
        modifier_id: action.hit.modifier_id.clone(),
        label: action.hit.modifier_label.clone(),
        duration: action.hit.modifier_duration.clone(),
    };

    trace.push(TraceEntry::new(
        3,
        TracePhase::Resolution,
        TraceStatus::Accepted,
        "Hit branch selected.",
        format!(
            "Roll stream supplied {}; total {} beats {} {}.",
            attack_roll.roll, attack_roll.total, action.attack.defense_label, defense_value
        ),
    ));
    trace.push(TraceEntry::new(
        4,
        TracePhase::Commit,
        TraceStatus::Accepted,
        "DomainEvents committed.",
        "ActionUsed, AttackRolled, DamageApplied, and ModifierApplied became accepted facts.",
    ));

    accepted_hit_receipt(
        scenario,
        intent,
        target_legality,
        attack_roll,
        damage,
        modifier,
        trace,
    )
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

fn accepted_hit_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target_legality: TargetLegality,
    attack_roll: AttackRollResult,
    damage: DamageOutcome,
    modifier: ModifierOutcome,
    trace: Vec<TraceEntry>,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: Some(attack_roll.clone()),
        damage: Some(damage.clone()),
        modifier: Some(modifier.clone()),
        events: accepted_hit_events(&intent, &attack_roll, &damage, &modifier),
        trace,
        projection: Some(project_final_state(
            scenario,
            "Raider is damaged and rattled; Adept is unchanged.",
            Some((&damage, &modifier)),
        )),
    }
}

fn accepted_miss_receipt(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    target_legality: TargetLegality,
    attack_roll: AttackRollResult,
    trace: Vec<TraceEntry>,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: Some(target_legality),
        attack_roll: Some(attack_roll.clone()),
        damage: None,
        modifier: None,
        events: vec![
            DomainEvent::ActionUsed {
                actor_id: intent.actor_id.clone(),
                action_id: intent.action_id.clone(),
                target_id: intent.target_id.clone(),
            },
            DomainEvent::AttackRolled {
                actor_id: intent.actor_id,
                target_id: intent.target_id,
                total: attack_roll.total,
                defense_id: attack_roll.defense_id,
                defense_value: attack_roll.defense_value,
                outcome: attack_roll.outcome,
            },
        ],
        trace,
        projection: Some(project_initial_state(
            scenario,
            "Attack missed; no authority state changed.",
        )),
    }
}

fn accepted_hit_events(
    intent: &UseActionIntent,
    attack_roll: &AttackRollResult,
    damage: &DamageOutcome,
    modifier: &ModifierOutcome,
) -> Vec<DomainEvent> {
    vec![
        DomainEvent::ActionUsed {
            actor_id: intent.actor_id.clone(),
            action_id: intent.action_id.clone(),
            target_id: intent.target_id.clone(),
        },
        DomainEvent::AttackRolled {
            actor_id: intent.actor_id.clone(),
            target_id: intent.target_id.clone(),
            total: attack_roll.total,
            defense_id: attack_roll.defense_id.clone(),
            defense_value: attack_roll.defense_value,
            outcome: attack_roll.outcome,
        },
        DomainEvent::DamageApplied {
            target_id: damage.target_id.clone(),
            amount: damage.amount,
            damage_type: damage.damage_type.clone(),
        },
        DomainEvent::ModifierApplied {
            target_id: modifier.target_id.clone(),
            modifier_id: modifier.modifier_id.clone(),
            duration: modifier.duration.clone(),
        },
    ]
}

fn rejected_with_projection(
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    target_legality: Option<TargetLegality>,
    mut trace: Vec<TraceEntry>,
) -> RulebenchReceipt {
    let detail = target_legality.as_ref().map_or_else(
        || rejection.code().to_string(),
        |legality| legality.reason.clone(),
    );
    trace.push(TraceEntry::new(
        2,
        TracePhase::Validation,
        TraceStatus::Rejected,
        "Intent rejected.",
        detail,
    ));
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(rejection),
        target_legality,
        attack_roll: None,
        damage: None,
        modifier: None,
        events: Vec::new(),
        trace,
        projection: Some(project_initial_state(
            scenario,
            "No authority state changed; intent rejected.",
        )),
    }
}

fn validate_target_legality(
    actor: &Combatant,
    target: &Combatant,
    action: &ActionDefinition,
) -> TargetLegality {
    if actor.team == target.team {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is not hostile.".to_string(),
        };
    }
    if range_between(actor.position, target.position) > action.range {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Target is outside range.".to_string(),
        };
    }
    if action.line_of_sight_required && !action.visible_target_ids.contains(&target.id) {
        return TargetLegality {
            target_id: target.id.clone(),
            accepted: false,
            reason: "Line of sight is blocked.".to_string(),
        };
    }
    TargetLegality {
        target_id: target.id.clone(),
        accepted: true,
        reason: "Target is hostile, within range, and line of sight is clear.".to_string(),
    }
}

fn target_legality_rejection(target_legality: &TargetLegality) -> RulebenchRejection {
    match target_legality.reason.as_str() {
        "Target is outside range." => RulebenchRejection::TargetOutOfRange,
        "Line of sight is blocked." => RulebenchRejection::TargetNotVisible,
        _ => RulebenchRejection::TargetLegalityFailed,
    }
}

fn range_between(from: GridPosition, to: GridPosition) -> u32 {
    from.x.abs_diff(to.x) + from.y.abs_diff(to.y)
}

fn defense_value(target: &Combatant, defense_id: &str) -> i32 {
    target
        .defenses
        .iter()
        .find(|defense| defense.id == defense_id)
        .map_or(0, |defense| defense.value)
}

fn apply_damage(target: &Combatant, amount: i32, damage_type: &str) -> DamageOutcome {
    let before = target.hit_points;
    let next = before.current.saturating_sub(amount).max(0);
    DamageOutcome {
        target_id: target.id.clone(),
        damage_type: damage_type.to_string(),
        amount: before.current - next,
        before,
        after: BoundedValue {
            current: next,
            max: before.max,
        },
    }
}

fn project_initial_state(scenario: &RulebenchScenario, summary: &str) -> ScenarioProjection {
    ScenarioProjection {
        summary: summary.to_string(),
        combatants: scenario
            .combatants
            .iter()
            .map(|combatant| FinalCombatantState {
                id: combatant.id.clone(),
                name: combatant.name.clone(),
                hit_points: combatant.hit_points,
                conditions: combatant.conditions.clone(),
            })
            .collect(),
    }
}

fn project_final_state(
    scenario: &RulebenchScenario,
    summary: &str,
    target_update: Option<(&DamageOutcome, &ModifierOutcome)>,
) -> ScenarioProjection {
    let mut projection = project_initial_state(scenario, summary);
    if let Some((damage, modifier)) = target_update {
        for combatant in &mut projection.combatants {
            if combatant.id == damage.target_id {
                combatant.hit_points = damage.after;
            }
            if combatant.id == modifier.target_id && !combatant.conditions.contains(&modifier.label)
            {
                combatant.conditions.push(modifier.label.clone());
            }
        }
    }
    projection
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
    fn resolver_accepts_hexing_bolt_hit_from_deterministic_roll_stream() {
        let receipt = resolve_use_action(
            &hexing_bolt_fixture_scenario(),
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(receipt.accepted);
        assert_eq!(receipt.rejection, None);
        assert_eq!(receipt.events.len(), 4);
        assert_eq!(
            receipt.attack_roll.as_ref().map(|roll| roll.outcome),
            Some(AttackOutcome::Hit)
        );
        assert_eq!(receipt.damage.as_ref().map(|damage| damage.amount), Some(9));
        assert_eq!(
            receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].hit_points.current),
            Some(9)
        );
    }

    #[test]
    fn resolver_rejects_non_hostile_target_without_events_or_damage() {
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

    #[test]
    fn resolver_rejects_missing_attack_roll_without_events() {
        let receipt = resolve_use_action(
            &hexing_bolt_fixture_scenario(),
            UseActionIntent::new("entity-adept", "hexing_bolt", "entity-raider"),
            &[],
        );

        assert!(!receipt.accepted);
        assert_eq!(
            receipt.rejection,
            Some(RulebenchRejection::MissingAttackRoll)
        );
        assert!(receipt.events.is_empty());
        assert!(receipt.damage.is_none());
    }

    #[test]
    fn resolver_rejects_invalid_action_without_events() {
        let receipt = resolve_use_action(
            &hexing_bolt_fixture_scenario(),
            UseActionIntent::new("entity-adept", "not_hexing_bolt", "entity-raider"),
            &[17, 5],
        );

        assert!(!receipt.accepted);
        assert_eq!(receipt.rejection, Some(RulebenchRejection::InvalidAction));
        assert!(receipt.events.is_empty());
        assert!(receipt.attack_roll.is_none());
    }

    #[test]
    fn catalog_enumerates_stable_scenario_summaries() {
        let summaries = scenario_catalog_summaries();

        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "hexing-bolt-hit",
                "hexing-bolt-miss",
                "hexing-bolt-self-target-rejected"
            ]
        );
        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.outcome_class.code())
                .collect::<Vec<_>>(),
            vec!["acceptedHit", "acceptedMiss", "rejectedTargetLegality"]
        );
    }

    #[test]
    fn catalog_resolves_accepted_hit_case() {
        let resolution = resolve_catalog_scenario("hexing-bolt-hit").expect("case exists");

        assert_eq!(
            resolution.case.outcome_class,
            ScenarioOutcomeClass::AcceptedHit
        );
        assert_eq!(resolution.scenario.metadata.id, "hexing-bolt-hit");
        assert!(resolution.receipt.accepted);
        assert_eq!(
            resolution
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Hit)
        );
        assert_eq!(resolution.receipt.events.len(), 4);
    }

    #[test]
    fn catalog_resolves_accepted_miss_case() {
        let resolution = resolve_catalog_scenario("hexing-bolt-miss").expect("case exists");

        assert_eq!(
            resolution.case.outcome_class,
            ScenarioOutcomeClass::AcceptedMiss
        );
        assert!(resolution.receipt.accepted);
        assert_eq!(
            resolution
                .receipt
                .attack_roll
                .as_ref()
                .map(|roll| roll.outcome),
            Some(AttackOutcome::Miss)
        );
        assert!(resolution.receipt.damage.is_none());
        assert!(resolution.receipt.modifier.is_none());
        assert_eq!(resolution.receipt.events.len(), 2);
        assert_eq!(
            resolution
                .receipt
                .projection
                .as_ref()
                .map(|projection| projection.combatants[1].hit_points.current),
            Some(18)
        );
    }

    #[test]
    fn catalog_resolves_rejected_target_legality_case() {
        let resolution =
            resolve_catalog_scenario("hexing-bolt-self-target-rejected").expect("case exists");

        assert_eq!(
            resolution.case.outcome_class,
            ScenarioOutcomeClass::RejectedTargetLegality
        );
        assert!(!resolution.receipt.accepted);
        assert_eq!(
            resolution.receipt.rejection,
            Some(RulebenchRejection::TargetLegalityFailed)
        );
        assert!(resolution.receipt.events.is_empty());
        assert_eq!(
            resolution
                .receipt
                .target_legality
                .as_ref()
                .map(|target| target.reason.as_str()),
            Some("Target is not hostile.")
        );
    }

    #[test]
    fn catalog_rejects_unknown_scenario_id() {
        let error = resolve_catalog_scenario("not-a-scenario").expect_err("unknown id fails");

        assert_eq!(error, ScenarioCatalogError::UnknownScenarioId);
    }
}
