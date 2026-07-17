use rpg_core::NamedNumber;

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
    pub formula: Option<DerivedStatFormula>,
    pub summary: String,
}

/// Closed numeric formula vocabulary for derived stats.
///
/// Authored content can combine only typed numeric operations and stable stat
/// references. It cannot inject executable behavior into rule resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivedStatFormula {
    Constant {
        value: i32,
    },
    StatReference {
        stat_id: String,
    },
    Sum {
        operands: Vec<DerivedStatFormula>,
    },
    Product {
        operands: Vec<DerivedStatFormula>,
    },
    Difference {
        minuend: Box<DerivedStatFormula>,
        subtrahend: Box<DerivedStatFormula>,
    },
}

impl DerivedStatFormula {
    pub const fn code(&self) -> &'static str {
        match self {
            DerivedStatFormula::Constant { .. } => "constant",
            DerivedStatFormula::StatReference { .. } => "statReference",
            DerivedStatFormula::Sum { .. } => "sum",
            DerivedStatFormula::Product { .. } => "product",
            DerivedStatFormula::Difference { .. } => "difference",
        }
    }

    pub fn referenced_stat_ids(&self) -> Vec<&str> {
        let mut stat_ids = Vec::new();
        self.collect_referenced_stat_ids(&mut stat_ids);
        stat_ids
    }

    pub fn shape_is_valid(&self) -> bool {
        match self {
            DerivedStatFormula::Constant { .. } | DerivedStatFormula::StatReference { .. } => true,
            DerivedStatFormula::Sum { operands } | DerivedStatFormula::Product { operands } => {
                operands.len() >= 2 && operands.iter().all(DerivedStatFormula::shape_is_valid)
            }
            DerivedStatFormula::Difference {
                minuend,
                subtrahend,
            } => minuend.shape_is_valid() && subtrahend.shape_is_valid(),
        }
    }

    fn collect_referenced_stat_ids<'a>(&'a self, stat_ids: &mut Vec<&'a str>) {
        match self {
            DerivedStatFormula::Constant { .. } => {}
            DerivedStatFormula::StatReference { stat_id } => stat_ids.push(stat_id),
            DerivedStatFormula::Sum { operands } | DerivedStatFormula::Product { operands } => {
                for operand in operands {
                    operand.collect_referenced_stat_ids(stat_ids);
                }
            }
            DerivedStatFormula::Difference {
                minuend,
                subtrahend,
            } => {
                minuend.collect_referenced_stat_ids(stat_ids);
                subtrahend.collect_referenced_stat_ids(stat_ids);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatBlock {
    pub base_stats: Vec<NamedNumber>,
    /// Deprecated authored values retained only so older callers receive a
    /// validation diagnostic instead of silently becoming authority inputs.
    pub derived_stats: Vec<NamedNumber>,
}

impl StatBlock {
    pub fn stat_by_id(&self, stat_id: &str) -> Option<&NamedNumber> {
        self.base_stats.iter().find(|stat| stat.id == stat_id)
    }
}
