use rpg_core::{BoundedValue, GridPosition, NamedNumber, Team};
use rpg_ir::ActionResourcePool;

use crate::{ClassLevelInput, ContentPackSetReference, Grid, StatBlock};

pub const AUTHORED_SCENARIO_BINDING_VERSION: u32 = 1;

/// Dependency-closed scenario content stored in a versioned content pack.
///
/// It references catalog definitions by stable id and contains only initial
/// authority state. Runtime state, target legality, derived stats, action
/// availability, automation decisions, and replay evidence remain Rust-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioDefinition {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub seed_label: String,
    pub ruleset_id: String,
    pub grid: Grid,
    pub participants: Vec<AuthoredScenarioParticipant>,
    pub selected_action_id: String,
    pub control: AuthoredScenarioControl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioControl {
    pub mode: AuthoredScenarioControlMode,
    pub automation_policy_id: Option<String>,
    pub automation_policy_version: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoredScenarioControlMode {
    Manual,
    Automatic,
}

impl AuthoredScenarioControlMode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Automatic => "automatic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioParticipant {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub team: Team,
    pub side_id: String,
    pub initiative: i32,
    pub position: GridPosition,
    pub hit_points: BoundedValue,
    pub temporary_vitality: i32,
    pub class_inputs: Vec<ClassLevelInput>,
    pub stats: StatBlock,
    pub defenses: Vec<NamedNumber>,
    pub resource_pools: Vec<ActionResourcePool>,
    pub inventory_item_ids: Vec<String>,
    pub equipped_item_ids: Vec<String>,
    pub base_ability_ids: Vec<String>,
    pub action_grants: Vec<AuthoredScenarioActionGrant>,
    pub visible_target_ids: Vec<String>,
    pub is_actor: bool,
}

/// Maps one reusable authored action to a scenario-local runtime identity.
///
/// Keeping the identities separate lets two participants use the same
/// definition without creating a global action-id collision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioActionGrant {
    pub action_id: String,
    pub runtime_action_id: String,
}

/// Exact composition receipt retained by sessions, recovery, and replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioBindingReceipt {
    pub binding_version: u32,
    pub content_pack_set: ContentPackSetReference,
    pub scenario_id: String,
    pub participants: Vec<AuthoredScenarioParticipantReceipt>,
    pub control: AuthoredScenarioControl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredScenarioParticipantReceipt {
    pub participant_id: String,
    pub archetypes: Vec<ClassLevelInput>,
    pub loadout_item_ids: Vec<String>,
    pub action_grants: Vec<AuthoredScenarioActionGrant>,
}
