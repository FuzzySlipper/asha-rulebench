//! Stable transport protocol surfaces for Rust authority.
//!
//! This crate owns wire DTO contracts, protocol metadata, and explicit mapping
//! from portable authority values. It does not own transport implementations
//! or rule semantics.

#![forbid(unsafe_code)]

mod session;
mod typescript;

pub use session::CombatSessionHandleDto;
pub use typescript::{render_api_types, ProtocolAlias, ProtocolField, ProtocolInterface};

#[cfg(test)]
mod tests {
    use rulebench_rules::CombatSessionHandle;

    use super::{render_api_types, CombatSessionHandleDto};

    const COMMITTED_API_TYPES: &str =
        include_str!("../../../../libs/protocol/src/generated/api-types.ts");

    #[test]
    fn committed_typescript_contract_matches_protocol_metadata() {
        assert_eq!(render_api_types(), COMMITTED_API_TYPES);
    }

    #[test]
    fn session_handle_mapping_preserves_the_opaque_authority_identity() {
        let handle = CombatSessionHandle::new("test-session");

        let dto = CombatSessionHandleDto::from(&handle);

        assert_eq!(dto.id, "test-session");
        assert_eq!(dto.to_combat_session_handle(), handle);
    }
}
