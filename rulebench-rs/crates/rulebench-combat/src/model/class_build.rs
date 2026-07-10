/// Versioned class inputs and deterministic derived-grant provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassBuildInputReadout {
    pub class_id: String,
    pub version: String,
    pub level: u32,
    pub applied_grant_levels: Vec<u32>,
    pub source_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatantClassBuildReadout {
    pub combatant_id: String,
    pub class_inputs: Vec<ClassBuildInputReadout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassBuildLedgerReadout {
    pub combatants: Vec<CombatantClassBuildReadout>,
}
