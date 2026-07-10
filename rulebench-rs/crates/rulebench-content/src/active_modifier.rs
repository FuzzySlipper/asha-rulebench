use rulebench_ruleset::ModifierTenure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveModifier {
    pub modifier_id: String,
    pub label: String,
    pub duration: String,
    pub tenure: ModifierTenure,
}

impl ActiveModifier {
    pub fn temporary(
        modifier_id: impl Into<String>,
        label: impl Into<String>,
        duration: impl Into<String>,
    ) -> Self {
        Self {
            modifier_id: modifier_id.into(),
            label: label.into(),
            duration: duration.into(),
            tenure: ModifierTenure::Temporary,
        }
    }

    pub fn permanent(modifier_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            modifier_id: modifier_id.into(),
            label: label.into(),
            duration: "permanent".to_string(),
            tenure: ModifierTenure::Permanent,
        }
    }
}
