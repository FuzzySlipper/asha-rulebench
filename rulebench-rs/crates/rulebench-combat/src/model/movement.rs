use super::{GridPosition, MovementActionDeclaration, ScenarioProjection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDecisionKind {
    Accepted,
    MissingDestination,
    MissingActor,
    DefeatedActor,
    OutOfBounds,
    Occupied,
    BlockedTerrain,
    OutOfRange,
    InsufficientBudget,
}

impl MovementDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::MissingDestination => "movementDestinationMissing",
            Self::MissingActor => "movementActorMissing",
            Self::DefeatedActor => "movementActorDefeated",
            Self::OutOfBounds => "movementOutOfBounds",
            Self::Occupied => "movementDestinationOccupied",
            Self::BlockedTerrain => "movementDestinationBlocked",
            Self::OutOfRange => "movementOutOfRange",
            Self::InsufficientBudget => "movementBudgetExhausted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementDecision {
    pub accepted: bool,
    pub kind: MovementDecisionKind,
    pub origin: Option<GridPosition>,
    pub destination: GridPosition,
    pub cost: u32,
    pub remaining_after: u32,
    pub reason: String,
}

pub fn evaluate_movement(
    projection: &ScenarioProjection,
    actor_id: &str,
    movement: &MovementActionDeclaration,
    destination: GridPosition,
) -> MovementDecision {
    let Some(actor) = projection
        .combatants
        .iter()
        .find(|item| item.id == actor_id)
    else {
        return rejected(
            MovementDecisionKind::MissingActor,
            None,
            destination,
            0,
            0,
            "Movement actor is not present.",
        );
    };
    if actor.hit_points.current <= 0 {
        return rejected(
            MovementDecisionKind::DefeatedActor,
            Some(actor.position),
            destination,
            0,
            actor.movement_remaining,
            "Defeated actors cannot move.",
        );
    }
    let Some(cell) = projection
        .board
        .cells
        .iter()
        .find(|cell| cell.position == destination)
    else {
        return rejected(
            MovementDecisionKind::OutOfBounds,
            Some(actor.position),
            destination,
            0,
            actor.movement_remaining,
            "Destination is outside the authoritative board.",
        );
    };
    if !cell.occupant_ids.is_empty() {
        return rejected(
            MovementDecisionKind::Occupied,
            Some(actor.position),
            destination,
            0,
            actor.movement_remaining,
            "Destination is occupied in the current authoritative state.",
        );
    }
    if cell.blocks_movement
        || cell
            .terrain_tags
            .iter()
            .any(|tag| movement.blocking_terrain_tags.contains(tag))
    {
        return rejected(
            MovementDecisionKind::BlockedTerrain,
            Some(actor.position),
            destination,
            0,
            actor.movement_remaining,
            "Destination terrain blocks movement.",
        );
    }
    let distance =
        actor.position.x.abs_diff(destination.x) + actor.position.y.abs_diff(destination.y);
    let terrain_cost = u32::from(
        cell.terrain_tags
            .iter()
            .any(|tag| movement.difficult_terrain_tags.contains(tag)),
    );
    let cost = distance + terrain_cost;
    if distance > movement.allowance {
        return rejected(
            MovementDecisionKind::OutOfRange,
            Some(actor.position),
            destination,
            cost,
            actor.movement_remaining,
            "Destination exceeds the action movement allowance.",
        );
    }
    if cost > actor.movement_remaining {
        return rejected(
            MovementDecisionKind::InsufficientBudget,
            Some(actor.position),
            destination,
            cost,
            actor.movement_remaining,
            "Destination cost exceeds remaining movement.",
        );
    }
    MovementDecision {
        accepted: true,
        kind: MovementDecisionKind::Accepted,
        origin: Some(actor.position),
        destination,
        cost,
        remaining_after: actor.movement_remaining - cost,
        reason: "Destination is legal under direct orthogonal Manhattan movement.".to_string(),
    }
}

fn rejected(
    kind: MovementDecisionKind,
    origin: Option<GridPosition>,
    destination: GridPosition,
    cost: u32,
    remaining_after: u32,
    reason: &str,
) -> MovementDecision {
    MovementDecision {
        accepted: false,
        kind,
        origin,
        destination,
        cost,
        remaining_after,
        reason: reason.to_string(),
    }
}
