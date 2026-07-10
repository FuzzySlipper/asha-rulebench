//! Stable transport protocol surfaces for Rust authority.
//!
//! This crate owns wire DTO contracts, protocol metadata, and explicit mapping
//! from portable authority values. It does not own transport implementations
//! or rule semantics.

#![forbid(unsafe_code)]

mod authoring;
mod content;
mod replay;
mod session;
mod typescript;

pub use authoring::{
    validate_ruleset_definition, RuleModuleConfigurationDto, RuleModuleDeclarationDto,
    RulesetAuthoringError, RulesetDefinitionDto,
};
pub use content::{
    ContentFingerprintDto, ContentImportDiagnosticDto, ContentImportReadoutDto,
    ContentPackIdentityDto,
};
pub use replay::{
    ReplayArchiveErrorDto, ReplayArchiveMetadataDto, ReplayComparisonDifferenceDto,
    ReplayComparisonReadoutDto, ReplayMismatchDto, ReplayPackageReviewDto,
    ReplayStateFingerprintDto, ReplayVerificationReadoutDto,
};
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

    #[test]
    fn typescript_contract_exposes_combat_sides_and_immutable_finalization() {
        let contract = render_api_types();

        assert!(contract.contains("export type RulebenchCombatSideIdDto = string;"));
        assert!(contract.contains("readonly sideId: RulebenchCombatSideIdDto;"));
        assert!(contract.contains("readonly policy: RulebenchCombatEndPolicyDto;"));
        assert!(contract.contains("readonly outcomeKind: RulebenchCombatOutcomeKindDto;"));
        assert!(contract.contains("readonly winningSides: readonly RulebenchCombatSideIdDto[];"));
        assert!(contract.contains("readonly finalization: RulebenchCombatFinalizationDto | null;"));
    }
}
