use rulebench_rules::CombatSessionHandle;

/// Stable wire representation of an opaque combat-session identity.
///
/// Hosts can retain this value and pass it back to a bridge without learning
/// anything about the authority-owned session state.
#[derive(Debug, Clone, PartialEq, Eq)]
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
