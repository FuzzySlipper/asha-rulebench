use super::*;

pub(super) fn resolve_movement_command(
    state: &CombatState,
    scenario: &RulebenchScenario,
    intent: UseActionIntent,
) -> RulebenchReceipt {
    let current = state.project("Current state for movement resolution.");
    let Some(action) = scenario.action_by_id(&intent.action_id) else {
        return rejected_movement(
            intent,
            RulebenchRejection::InvalidAction,
            current,
            "Movement action is not declared.",
        );
    };
    let Some(movement) = action.movement.as_ref() else {
        return rejected_movement(
            intent,
            RulebenchRejection::InvalidAction,
            current,
            "Action does not declare movement behavior.",
        );
    };
    let Some(destination) = intent.destination_cell else {
        return rejected_movement(
            intent,
            RulebenchRejection::MovementDestinationMissing,
            current,
            "Movement destination is missing.",
        );
    };
    if let Some(observed_origin) = intent.observed_origin {
        let actual_origin = current
            .combatants
            .iter()
            .find(|combatant| combatant.id == intent.actor_id)
            .map(|combatant| combatant.position);
        if actual_origin != Some(observed_origin) {
            return rejected_movement(
                intent,
                RulebenchRejection::MovementStaleDestination,
                current,
                "Movement destination was selected from a stale actor position.",
            );
        }
    }
    let decision = evaluate_movement(&current, &intent.actor_id, movement, destination);
    if !decision.accepted {
        return rejected_movement(
            intent,
            rejection_for(decision.kind),
            current,
            &decision.reason,
        );
    }

    let mut next = state.clone();
    next.apply_movement(&intent.actor_id, destination, decision.cost);
    let projection = next.project("State after accepted movement.");
    let origin = decision.origin.expect("accepted movement has an origin");
    RulebenchReceipt {
        accepted: true,
        authority_surface: AUTHORITY_SURFACE,
        intent: intent.clone(),
        rejection: None,
        target_legality: None,
        attack_roll: None,
        damage: None,
        healing: None,
        temporary_vitality: None,
        modifier: None,
        roll_consumption: Vec::new(),
        events: vec![
            DomainEvent::PositionChanged {
                actor_id: intent.actor_id.clone(),
                from: origin,
                to: destination,
            },
            DomainEvent::MovementSpent {
                actor_id: intent.actor_id,
                amount: decision.cost,
                remaining: decision.remaining_after,
            },
        ],
        trace: vec![
            TraceEntry::new(
                1,
                TracePhase::Proposal,
                TraceStatus::Info,
                "Movement proposed.",
                format!("Destination {},{}.", destination.x, destination.y),
            ),
            TraceEntry::new(
                2,
                TracePhase::Validation,
                TraceStatus::Accepted,
                "Movement accepted.",
                decision.reason,
            ),
            TraceEntry::new(
                3,
                TracePhase::Commit,
                TraceStatus::Accepted,
                "Movement committed.",
                format!(
                    "Spent {} movement; {} remains.",
                    decision.cost, decision.remaining_after
                ),
            ),
        ],
        projection: Some(projection),
    }
}

fn rejected_movement(
    intent: UseActionIntent,
    rejection: RulebenchRejection,
    projection: ScenarioProjection,
    reason: &str,
) -> RulebenchReceipt {
    RulebenchReceipt {
        accepted: false,
        authority_surface: AUTHORITY_SURFACE,
        intent,
        rejection: Some(rejection),
        target_legality: None,
        attack_roll: None,
        damage: None,
        healing: None,
        temporary_vitality: None,
        modifier: None,
        roll_consumption: Vec::new(),
        events: Vec::new(),
        trace: vec![TraceEntry::new(
            1,
            TracePhase::Validation,
            TraceStatus::Rejected,
            "Movement rejected.",
            reason,
        )],
        projection: Some(projection),
    }
}

fn rejection_for(kind: MovementDecisionKind) -> RulebenchRejection {
    match kind {
        MovementDecisionKind::Accepted => RulebenchRejection::InvalidAction,
        MovementDecisionKind::MissingDestination => RulebenchRejection::MovementDestinationMissing,
        MovementDecisionKind::MissingActor => RulebenchRejection::InvalidActor,
        MovementDecisionKind::DefeatedActor => RulebenchRejection::MovementActorDefeated,
        MovementDecisionKind::OutOfBounds => RulebenchRejection::MovementOutOfBounds,
        MovementDecisionKind::Occupied => RulebenchRejection::MovementDestinationOccupied,
        MovementDecisionKind::BlockedTerrain => RulebenchRejection::MovementDestinationBlocked,
        MovementDecisionKind::OutOfRange => RulebenchRejection::MovementOutOfRange,
        MovementDecisionKind::InsufficientBudget => RulebenchRejection::MovementBudgetExhausted,
    }
}
