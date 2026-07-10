use crate::{ModifierDurationPolicy, ModifierStackingPolicy};
use rulebench_ruleset::ModifierTenure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveModifier {
    pub modifier_id: String,
    pub source_id: String,
    pub label: String,
    pub duration: String,
    pub tenure: ModifierTenure,
    pub stacking_group: String,
    pub stacking_policy: ModifierStackingPolicy,
    pub duration_policy: ModifierDurationPolicy,
    pub remaining_turns: Option<u32>,
    pub remaining_rounds: Option<u32>,
}

impl ActiveModifier {
    pub fn temporary(
        modifier_id: impl Into<String>,
        label: impl Into<String>,
        duration: impl Into<String>,
    ) -> Self {
        Self {
            modifier_id: modifier_id.into(),
            source_id: "legacy".to_string(),
            label: label.into(),
            duration: duration.into(),
            tenure: ModifierTenure::Temporary,
            stacking_group: "legacy".to_string(),
            stacking_policy: ModifierStackingPolicy::Replace,
            duration_policy: ModifierDurationPolicy::Turns(1),
            remaining_turns: Some(1),
            remaining_rounds: None,
        }
    }

    pub fn permanent(modifier_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            modifier_id: modifier_id.into(),
            source_id: "legacy".to_string(),
            label: label.into(),
            duration: "permanent".to_string(),
            tenure: ModifierTenure::Permanent,
            stacking_group: "legacy".to_string(),
            stacking_policy: ModifierStackingPolicy::Replace,
            duration_policy: ModifierDurationPolicy::Permanent,
            remaining_turns: None,
            remaining_rounds: None,
        }
    }
}
