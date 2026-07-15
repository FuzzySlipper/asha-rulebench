//! Deterministic state fingerprints used by combat audit readbacks.

use crate::model::{ActionResourceLedgerReadout, ScenarioProjection, StateFingerprint};

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
pub const PROJECTION_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-projection.v0";
pub const STATE_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-state.v0";
pub const ACTION_RESOURCE_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-action-resources.v0";

pub fn fingerprint_projection(projection: &ScenarioProjection) -> StateFingerprint {
    let mut builder = FingerprintBuilder::new();

    builder.feed_str("summary");
    builder.feed_str(&projection.summary);
    feed_projected_combatants(&mut builder, projection);

    StateFingerprint {
        algorithm: PROJECTION_FINGERPRINT_ALGORITHM.to_string(),
        value: format!("{:016x}", builder.finish()),
    }
}

pub fn fingerprint_projected_state(projection: &ScenarioProjection) -> StateFingerprint {
    let mut builder = FingerprintBuilder::new();

    feed_projected_combatants(&mut builder, projection);

    StateFingerprint {
        algorithm: STATE_FINGERPRINT_ALGORITHM.to_string(),
        value: format!("{:016x}", builder.finish()),
    }
}

pub fn fingerprint_action_resource_ledger(
    ledger: &ActionResourceLedgerReadout,
) -> StateFingerprint {
    let mut builder = FingerprintBuilder::new();
    let mut combatants = ledger.combatants.iter().collect::<Vec<_>>();
    combatants.sort_by(|left, right| left.combatant_id.cmp(&right.combatant_id));
    builder.feed_u32(combatants.len() as u32);
    for combatant in combatants {
        builder.feed_str(&combatant.combatant_id);
        let mut resources = combatant.resources.iter().collect::<Vec<_>>();
        resources.sort_by(|left, right| left.resource_id.cmp(&right.resource_id));
        builder.feed_u32(resources.len() as u32);
        for resource in resources {
            builder.feed_str(&resource.resource_id);
            builder.feed_str(resource.kind.code());
            builder.feed_i32(resource.current);
            builder.feed_i32(resource.max);
            builder.feed_u32(resource.available as u32);
        }
    }
    StateFingerprint {
        algorithm: ACTION_RESOURCE_FINGERPRINT_ALGORITHM.to_string(),
        value: format!("{:016x}", builder.finish()),
    }
}

fn feed_projected_combatants(builder: &mut FingerprintBuilder, projection: &ScenarioProjection) {
    builder.feed_str("board");
    builder.feed_str(&projection.board.id);
    builder.feed_u32(projection.board.width);
    builder.feed_u32(projection.board.height);
    let mut cells = projection.board.cells.iter().collect::<Vec<_>>();
    cells.sort_by_key(|cell| (cell.position.y, cell.position.x));
    builder.feed_u32(cells.len() as u32);
    for cell in cells {
        builder.feed_u32(cell.position.x);
        builder.feed_u32(cell.position.y);
        builder.feed_u32(cell.blocks_movement as u32);
        let mut terrain_tags = cell.terrain_tags.iter().collect::<Vec<_>>();
        terrain_tags.sort();
        builder.feed_u32(terrain_tags.len() as u32);
        for tag in terrain_tags {
            builder.feed_str(tag);
        }
        let mut occupants = cell.occupant_ids.iter().collect::<Vec<_>>();
        occupants.sort();
        builder.feed_u32(occupants.len() as u32);
        for occupant in occupants {
            builder.feed_str(occupant);
        }
    }

    let mut combatants = projection.combatants.iter().collect::<Vec<_>>();
    combatants.sort_by(|left, right| left.id.cmp(&right.id));
    builder.feed_u32(combatants.len() as u32);
    for combatant in combatants {
        builder.feed_str("combatant");
        builder.feed_str(&combatant.id);
        builder.feed_str(&combatant.name);
        builder.feed_i32(combatant.hit_points.current);
        builder.feed_i32(combatant.hit_points.max);
        builder.feed_u32(combatant.position.x);
        builder.feed_u32(combatant.position.y);
        builder.feed_u32(combatant.movement_remaining);
        builder.feed_u32(combatant.movement_maximum);

        builder.feed_u32(combatant.conditions.len() as u32);
        for condition in &combatant.conditions {
            builder.feed_str(condition);
        }
    }
}

struct FingerprintBuilder {
    hash: u64,
}

impl FingerprintBuilder {
    const fn new() -> Self {
        Self { hash: FNV_OFFSET }
    }

    fn feed_str(&mut self, value: &str) {
        self.feed_u32(value.len() as u32);
        for byte in value.as_bytes() {
            self.feed_byte(*byte);
        }
    }

    fn feed_i32(&mut self, value: i32) {
        for byte in value.to_le_bytes() {
            self.feed_byte(byte);
        }
    }

    fn feed_u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.feed_byte(byte);
        }
    }

    fn feed_byte(&mut self, byte: u8) {
        self.hash ^= u64::from(byte);
        self.hash = self.hash.wrapping_mul(FNV_PRIME);
    }

    const fn finish(self) -> u64 {
        self.hash
    }
}
