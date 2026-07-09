use super::NamedNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatDefinitionKind {
    Base,
    Derived,
}

impl StatDefinitionKind {
    pub const fn code(self) -> &'static str {
        match self {
            StatDefinitionKind::Base => "base",
            StatDefinitionKind::Derived => "derived",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatDefinition {
    pub id: String,
    pub label: String,
    pub kind: StatDefinitionKind,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatBlock {
    pub base_stats: Vec<NamedNumber>,
    pub derived_stats: Vec<NamedNumber>,
}

impl StatBlock {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.base_stats
            .iter()
            .chain(self.derived_stats.iter())
            .find(|stat| stat.id == stat_id)
    }
}
