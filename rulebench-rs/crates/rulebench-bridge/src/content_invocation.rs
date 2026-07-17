use rpg_ir::RulesetProviderCatalog;
use rulebench_combat::COMBAT_AUTOMATION_POLICY_REGISTRY;
use rulebench_content::{
    import_content_pack, AuthoredScenarioControlMode, CanonicalContentPack, ContentImportContext,
    ContentImportLimits, ImportedContentPack,
};
use rulebench_protocol::{
    AuthoredContentPackDocumentDto, ContentImportDiagnosticDto, ContentPackIdentityDto,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentInvocationError {
    pub code: String,
    pub message: String,
    pub pack: Option<ContentPackIdentityDto>,
    pub diagnostics: Vec<ContentImportDiagnosticDto>,
}

pub fn import_authored_content(
    document: &AuthoredContentPackDocumentDto,
    available_packs: &[CanonicalContentPack],
    provider_catalog: &RulesetProviderCatalog,
) -> Result<ImportedContentPack, ContentInvocationError> {
    let identity = ContentPackIdentityDto {
        id: document.pack.id.clone(),
        version: document.pack.version.clone(),
        fingerprint: None,
    };
    let authored = document
        .to_authority()
        .map_err(|error| ContentInvocationError {
            code: error.code.to_string(),
            message: error.message,
            pack: Some(identity.clone()),
            diagnostics: vec![ContentImportDiagnosticDto {
                severity: "error".to_string(),
                code: error.code.to_string(),
                path: error.path,
                reference_id: Some(document.pack.id.clone()),
                definition_kind: None,
                message: "The authored payload could not be converted to authority content."
                    .to_string(),
            }],
        })?;
    let rulesets = available_packs
        .iter()
        .flat_map(|pack| pack.catalogs.rulesets.iter().cloned())
        .collect::<Vec<_>>();
    let imported = import_content_pack(
        authored,
        ContentImportLimits::default(),
        ContentImportContext {
            available_packs,
            rulesets: &rulesets,
            provider_catalog: Some(provider_catalog),
        },
    )
    .map_err(|report| {
        let diagnostics = report
            .diagnostics
            .iter()
            .map(ContentImportDiagnosticDto::from)
            .collect::<Vec<_>>();
        let code = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == "error")
            .map(|diagnostic| diagnostic.code.clone())
            .unwrap_or_else(|| "contentImportRejected".to_string());
        ContentInvocationError {
            code,
            message: "Rust authority rejected the authored content pack.".to_string(),
            pack: Some(identity.clone()),
            diagnostics,
        }
    })?;
    for scenario in &imported.pack.catalogs.scenarios {
        if scenario.control.mode != AuthoredScenarioControlMode::Automatic {
            continue;
        }
        let policy_id = scenario
            .control
            .automation_policy_id
            .as_deref()
            .unwrap_or_default();
        let policy_version = scenario
            .control
            .automation_policy_version
            .unwrap_or_default();
        if !COMBAT_AUTOMATION_POLICY_REGISTRY
            .iter()
            .any(|registration| {
                registration.id == policy_id && registration.version == policy_version
            })
        {
            return Err(ContentInvocationError {
                code: "unsupportedAuthoredScenarioAutomationPolicy".to_string(),
                message: "Rust authority rejected an unknown authored scenario automation policy."
                    .to_string(),
                pack: Some(identity),
                diagnostics: vec![ContentImportDiagnosticDto {
                    severity: "error".to_string(),
                    code: "unsupportedAuthoredScenarioAutomationPolicy".to_string(),
                    path: format!("catalogs.scenarios[{}].control", scenario.id),
                    reference_id: Some(scenario.id.clone()),
                    definition_kind: Some("scenario".to_string()),
                    message: format!(
                        "Automation policy {policy_id} v{policy_version} is not registered by the Rust authority."
                    ),
                }],
            });
        }
    }
    Ok(imported)
}
