use rulebench_rpg_adapter::CombatSessionHandle;
use serde::{Deserialize, Serialize};

/// Stable wire representation of an opaque combat-session identity.
///
/// Hosts can retain this value and pass it back to a bridge without learning
/// anything about the authority-owned session state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatSessionHandleDto {
    pub id: String,
}

impl CombatSessionHandleDto {
    pub fn to_combat_session_handle(&self) -> CombatSessionHandle {
        CombatSessionHandle::new(self.id.clone())
    }
}

impl From<&CombatSessionHandle> for CombatSessionHandleDto {
    fn from(handle: &CombatSessionHandle) -> Self {
        Self {
            id: handle.id.clone(),
        }
    }
}
