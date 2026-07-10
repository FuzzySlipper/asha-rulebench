//! Deterministic state fingerprints used by combat audit readbacks.

use crate::model::{ScenarioProjection, StateFingerprint};

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
pub const PROJECTION_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-projection.v0";
pub const STATE_FINGERPRINT_ALGORITHM: &str = "fnv1a64.rulebench-state.v0";

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

fn feed_projected_combatants(builder: &mut FingerprintBuilder, projection: &ScenarioProjection) {
    builder.feed_u32(projection.combatants.len() as u32);
    for combatant in &projection.combatants {
        builder.feed_str("combatant");
        builder.feed_str(&combatant.id);
        builder.feed_str(&combatant.name);
        builder.feed_i32(combatant.hit_points.current);
        builder.feed_i32(combatant.hit_points.max);

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
