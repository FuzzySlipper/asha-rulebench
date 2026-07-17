use std::collections::BTreeSet;

use rpg_compiler::{compile_normalized_rpg_ir, CompiledRpgRuleset, RpgDiagnostic};
use rpg_ir::NormalizedRpgIr;
use serde::Deserialize;

pub const RULEBENCH_RPG_CONTENT_SCHEMA: &str = "asha-rulebench.rpg-content@1";

const REPRESENTATIVE_RPG_CONTENT: &[u8] =
    include_bytes!("generated/representative-rpg-content.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulebenchRpgContent {
    pub normalized_ir: NormalizedRpgIr,
    pub bindings: Vec<RpgActionBindingMetadata>,
}

impl RulebenchRpgContent {
    pub fn compile(&self) -> Result<CompiledRpgRuleset, RulebenchRpgContentError> {
        compile_normalized_rpg_ir(self.normalized_ir.clone()).map_err(|failure| {
            RulebenchRpgContentError::CompileRejected {
                diagnostics: failure.diagnostics,
            }
        })
    }

    pub fn binding(&self, action_id: &str) -> Option<&RpgActionBindingMetadata> {
        self.bindings
            .iter()
            .find(|binding| binding.action_id == action_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RpgActionBindingMetadata {
    pub action_id: String,
    pub ability_id: String,
    pub action_text: String,
    pub effect_text: String,
    pub ruleset_ids: Vec<String>,
    pub actor_ids: Vec<String>,
    pub reaction: Option<RulebenchReactionOrchestration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RulebenchReactionOrchestration {
    pub window: RulebenchReactionWindow,
    pub eligible_reactors: RulebenchEligibleReactors,
    pub option_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RulebenchReactionWindow {
    BeforeEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RulebenchEligibleReactors {
    DeclaredTargets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulebenchRpgContentError {
    Decode { message: String },
    SchemaMismatch { actual: String },
    DuplicateBinding { action_id: String },
    UnknownBindingAction { action_id: String },
    MissingBinding { action_id: String },
    InvalidReaction { action_id: String, message: String },
    CompileRejected { diagnostics: Vec<RpgDiagnostic> },
}

pub fn representative_rpg_content() -> Result<RulebenchRpgContent, RulebenchRpgContentError> {
    let generated = serde_json::from_slice::<GeneratedRpgContent>(REPRESENTATIVE_RPG_CONTENT)
        .map_err(|error| RulebenchRpgContentError::Decode {
            message: error.to_string(),
        })?;
    if generated.generated.schema != RULEBENCH_RPG_CONTENT_SCHEMA {
        return Err(RulebenchRpgContentError::SchemaMismatch {
            actual: generated.generated.schema,
        });
    }

    let action_ids = generated
        .normalized_ir
        .actions
        .iter()
        .map(|action| action.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut binding_ids = BTreeSet::new();
    for binding in &generated.bindings {
        if !binding_ids.insert(binding.action_id.as_str()) {
            return Err(RulebenchRpgContentError::DuplicateBinding {
                action_id: binding.action_id.clone(),
            });
        }
        if !action_ids.contains(binding.action_id.as_str()) {
            return Err(RulebenchRpgContentError::UnknownBindingAction {
                action_id: binding.action_id.clone(),
            });
        }
        if let Some(reaction) = &binding.reaction {
            if reaction.option_id.trim().is_empty() {
                return Err(RulebenchRpgContentError::InvalidReaction {
                    action_id: binding.action_id.clone(),
                    message: "reaction option identity is required".to_string(),
                });
            }
        }
    }
    for action_id in action_ids {
        if !binding_ids.contains(action_id) {
            return Err(RulebenchRpgContentError::MissingBinding {
                action_id: action_id.to_string(),
            });
        }
    }

    let content = RulebenchRpgContent {
        normalized_ir: generated.normalized_ir,
        bindings: generated.bindings,
    };
    content.compile()?;
    Ok(content)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GeneratedRpgContent {
    #[serde(rename = "_generated")]
    generated: GeneratedRpgContentProvenance,
    normalized_ir: NormalizedRpgIr,
    bindings: Vec<RpgActionBindingMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedRpgContentProvenance {
    #[allow(dead_code)]
    emitter: String,
    schema: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_downstream_content_is_complete_and_rust_compilable() {
        let content = representative_rpg_content().expect("generated RPG content");
        let compiled = content.compile().expect("Rust compiler accepts content");

        assert_eq!(content.bindings.len(), 9);
        assert_eq!(compiled.action_ids().count(), 9);
        assert!(compiled
            .action_ids()
            .any(|id| id == "action.shatterline-burst"));
        assert_eq!(
            content
                .binding("hexing_bolt")
                .and_then(|binding| binding.reaction.as_ref())
                .map(|reaction| reaction.option_id.as_str()),
            Some("reaction.brace")
        );
    }
}
