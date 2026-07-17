//! Bounded automatic-combat replay specifications and evidence verification.

use rpg_core::StateFingerprint;
use rulebench_combat::RulebenchScenario;
use rulebench_combat::{
    ActionResourceTransitionEntry, ClassBuildLedgerReadout, CombatAutomationPolicyDecisionEvidence,
    CombatFinalizationReadout, CombatSessionAutomaticRunDecisionKind,
    CombatSessionAutomaticRunReadout, CombatSessionAutomaticRunSpec, CombatSessionState,
    ContentPackSetReference, EquipmentLedgerReadout, EquipmentTransitionEntry,
    ModifierDurationExpirationEntry, ReactionAuditEntry, ReactionWindowLifecycleEntry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticRunReplaySpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub initial_session_id: String,
    pub initial_scenario: RulebenchScenario,
    pub run: CombatSessionAutomaticRunSpec,
    pub expected_final_state_fingerprint: StateFingerprint,
    pub expected_finalization: Option<CombatFinalizationReadout>,
    pub expected_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub expected_executed_step_count: u32,
    pub expected_policy_decisions: Vec<CombatAutomationPolicyDecisionEvidence>,
    pub expected_action_resource_transition_log: Vec<ActionResourceTransitionEntry>,
    pub expected_equipment_ledger: EquipmentLedgerReadout,
    pub expected_class_build_ledger: ClassBuildLedgerReadout,
    pub expected_equipment_transition_log: Vec<EquipmentTransitionEntry>,
    pub expected_reaction_window_lifecycle_log: Vec<ReactionWindowLifecycleEntry>,
    pub expected_reaction_audit_log: Vec<ReactionAuditEntry>,
    pub expected_modifier_duration_expiration_log: Vec<ModifierDurationExpirationEntry>,
}

impl CombatSessionAutomaticRunReplaySpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        initial_session_id: impl Into<String>,
        initial_scenario: RulebenchScenario,
        run: CombatSessionAutomaticRunSpec,
        expected_final_state_fingerprint: StateFingerprint,
        expected_finalization: Option<CombatFinalizationReadout>,
        expected_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
        expected_executed_step_count: u32,
        expected_policy_decisions: Vec<CombatAutomationPolicyDecisionEvidence>,
        expected_action_resource_transition_log: Vec<ActionResourceTransitionEntry>,
        expected_equipment_ledger: EquipmentLedgerReadout,
        expected_class_build_ledger: ClassBuildLedgerReadout,
        expected_equipment_transition_log: Vec<EquipmentTransitionEntry>,
        expected_reaction_window_lifecycle_log: Vec<ReactionWindowLifecycleEntry>,
        expected_reaction_audit_log: Vec<ReactionAuditEntry>,
        expected_modifier_duration_expiration_log: Vec<ModifierDurationExpirationEntry>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            initial_session_id: initial_session_id.into(),
            initial_scenario,
            run,
            expected_final_state_fingerprint,
            expected_finalization,
            expected_run_decision_kind,
            expected_executed_step_count,
            expected_policy_decisions,
            expected_action_resource_transition_log,
            expected_equipment_ledger,
            expected_class_build_ledger,
            expected_equipment_transition_log,
            expected_reaction_window_lifecycle_log,
            expected_reaction_audit_log,
            expected_modifier_duration_expiration_log,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionAutomaticRunReplayDecisionKind {
    Verified,
    MismatchedEvidence,
}

impl CombatSessionAutomaticRunReplayDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionAutomaticRunReplayDecisionKind::Verified => "verified",
            CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence => "mismatchedEvidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionAutomaticRunReplayReadout {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub accepted: bool,
    pub decision_kind: CombatSessionAutomaticRunReplayDecisionKind,
    pub expected_final_state_fingerprint: StateFingerprint,
    pub actual_final_state_fingerprint: StateFingerprint,
    pub final_state_fingerprint_matches: bool,
    pub finalization_matches: bool,
    pub expected_content_pack_set: Option<ContentPackSetReference>,
    pub actual_content_pack_set: Option<ContentPackSetReference>,
    pub content_pack_set_matches: bool,
    pub expected_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub actual_run_decision_kind: CombatSessionAutomaticRunDecisionKind,
    pub run_decision_kind_matches: bool,
    pub expected_executed_step_count: u32,
    pub actual_executed_step_count: u32,
    pub executed_step_count_matches: bool,
    pub policy_decisions_match: bool,
    pub action_resource_transition_log_matches: bool,
    pub equipment_ledger_matches: bool,
    pub class_build_ledger_matches: bool,
    pub equipment_transition_log_matches: bool,
    pub reaction_window_lifecycle_log_matches: bool,
    pub reaction_audit_log_matches: bool,
    pub modifier_duration_expiration_log_matches: bool,
    pub replayed_run: CombatSessionAutomaticRunReadout,
    pub reason: String,
}

pub fn verify_automatic_run_replay(
    spec: CombatSessionAutomaticRunReplaySpec,
) -> CombatSessionAutomaticRunReplayReadout {
    let expected_content_pack_set = spec.initial_scenario.content_pack_set.clone();
    let mut replay_session =
        CombatSessionState::new(spec.initial_session_id.clone(), spec.initial_scenario);
    let replayed_run = replay_session.run_automatic_combat(spec.run);
    let actual_final_state_fingerprint = replayed_run
        .final_snapshot
        .current_state_fingerprint
        .clone();
    let actual_run_decision_kind = replayed_run.decision_kind;
    let actual_executed_step_count = replayed_run.executed_step_count;

    let final_state_fingerprint_matches =
        actual_final_state_fingerprint == spec.expected_final_state_fingerprint;
    let finalization_matches =
        replayed_run.final_snapshot.finalization == spec.expected_finalization;
    let actual_content_pack_set = replayed_run.final_snapshot.content_pack_set.clone();
    let content_pack_set_matches = actual_content_pack_set == expected_content_pack_set;
    let run_decision_kind_matches = actual_run_decision_kind == spec.expected_run_decision_kind;
    let executed_step_count_matches =
        actual_executed_step_count == spec.expected_executed_step_count;
    let policy_decisions_match = replayed_run.policy_decisions == spec.expected_policy_decisions;
    let action_resource_transition_log_matches =
        replayed_run.final_snapshot.action_resource_transition_log
            == spec.expected_action_resource_transition_log;
    let equipment_ledger_matches =
        replayed_run.final_snapshot.equipment_ledger == spec.expected_equipment_ledger;
    let class_build_ledger_matches =
        replayed_run.final_snapshot.class_build_ledger == spec.expected_class_build_ledger;
    let equipment_transition_log_matches = replayed_run.final_snapshot.equipment_transition_log
        == spec.expected_equipment_transition_log;
    let reaction_window_lifecycle_log_matches =
        replayed_run.final_snapshot.reaction_window_lifecycle_log
            == spec.expected_reaction_window_lifecycle_log;
    let reaction_audit_log_matches =
        replayed_run.final_snapshot.reaction_audit_log == spec.expected_reaction_audit_log;
    let modifier_duration_expiration_log_matches =
        replayed_run.final_snapshot.modifier_duration_expiration_log
            == spec.expected_modifier_duration_expiration_log;
    let accepted = final_state_fingerprint_matches
        && finalization_matches
        && content_pack_set_matches
        && run_decision_kind_matches
        && executed_step_count_matches
        && policy_decisions_match
        && action_resource_transition_log_matches
        && equipment_ledger_matches
        && class_build_ledger_matches
        && equipment_transition_log_matches
        && reaction_window_lifecycle_log_matches
        && reaction_audit_log_matches
        && modifier_duration_expiration_log_matches;
    let decision_kind = if accepted {
        CombatSessionAutomaticRunReplayDecisionKind::Verified
    } else {
        CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence
    };
    let reason = if accepted {
        "Automatic run replay verified expected final evidence.".to_string()
    } else {
        "Automatic run replay produced evidence that does not match expected final evidence."
            .to_string()
    };

    CombatSessionAutomaticRunReplayReadout {
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        accepted,
        decision_kind,
        expected_final_state_fingerprint: spec.expected_final_state_fingerprint,
        actual_final_state_fingerprint,
        final_state_fingerprint_matches,
        finalization_matches,
        expected_content_pack_set,
        actual_content_pack_set,
        content_pack_set_matches,
        expected_run_decision_kind: spec.expected_run_decision_kind,
        actual_run_decision_kind,
        run_decision_kind_matches,
        expected_executed_step_count: spec.expected_executed_step_count,
        actual_executed_step_count,
        executed_step_count_matches,
        policy_decisions_match,
        action_resource_transition_log_matches,
        equipment_ledger_matches,
        class_build_ledger_matches,
        equipment_transition_log_matches,
        reaction_window_lifecycle_log_matches,
        reaction_audit_log_matches,
        modifier_duration_expiration_log_matches,
        replayed_run,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rulebench_combat::ActionResourceCost;
    use rulebench_combat::{
        fingerprint_content_pack_set, ActionDefinition, AttackCheckDeclaration, CheckDeclaration,
        CombatSessionState, ContentFingerprint, ContentPackReference, ContentPackSetReference,
        DefenseReference, Grid, HitEffect, RulebenchScenario, ScenarioMetadata, TargetKind,
        TargetSelection, TargetTeamConstraint, TargetingDeclaration, VisibilityRequirement,
        CONTENT_PACK_FINGERPRINT_ALGORITHM,
    };

    #[test]
    fn automatic_run_replay_verifies_matching_evidence() {
        let mut scenario = minimal_replay_scenario();
        scenario.content_pack_set = Some(test_pack_set_reference());
        let run = zero_step_run();
        let expected = expected_run(scenario.clone(), run.clone());

        let readout = verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
            "matching",
            "Matching replay",
            "Replay matches expected evidence.",
            "matching-session",
            scenario,
            run,
            expected.final_snapshot.current_state_fingerprint.clone(),
            expected.final_snapshot.finalization.clone(),
            expected.decision_kind,
            expected.executed_step_count,
            expected.policy_decisions.clone(),
            expected
                .final_snapshot
                .action_resource_transition_log
                .clone(),
            expected.final_snapshot.equipment_ledger.clone(),
            expected.final_snapshot.class_build_ledger.clone(),
            expected.final_snapshot.equipment_transition_log.clone(),
            expected
                .final_snapshot
                .reaction_window_lifecycle_log
                .clone(),
            expected.final_snapshot.reaction_audit_log.clone(),
            expected
                .final_snapshot
                .modifier_duration_expiration_log
                .clone(),
        ));

        assert!(readout.accepted);
        assert_eq!(
            readout.decision_kind,
            CombatSessionAutomaticRunReplayDecisionKind::Verified
        );
        assert!(readout.final_state_fingerprint_matches);
        assert!(readout.finalization_matches);
        assert!(readout.content_pack_set_matches);
        assert_eq!(
            readout.actual_content_pack_set,
            readout.expected_content_pack_set
        );
        assert!(readout.run_decision_kind_matches);
        assert!(readout.executed_step_count_matches);
        assert!(readout.policy_decisions_match);
        assert!(readout.action_resource_transition_log_matches);
        assert!(readout.equipment_ledger_matches);
        assert!(readout.class_build_ledger_matches);
        assert!(readout.equipment_transition_log_matches);
        assert!(readout.reaction_window_lifecycle_log_matches);
        assert!(readout.reaction_audit_log_matches);
        assert!(readout.modifier_duration_expiration_log_matches);
    }

    #[test]
    fn automatic_run_replay_rejects_mismatched_finalization_evidence() {
        let scenario = minimal_replay_scenario();
        let run = one_step_run();
        let expected = expected_run(scenario.clone(), run.clone());

        let readout = verify_automatic_run_replay(CombatSessionAutomaticRunReplaySpec::new(
            "mismatch",
            "Mismatch replay",
            "Replay reports mismatched evidence.",
            "mismatch-session",
            scenario,
            run,
            expected.final_snapshot.current_state_fingerprint.clone(),
            None,
            expected.decision_kind,
            expected.executed_step_count,
            expected.policy_decisions.clone(),
            expected
                .final_snapshot
                .action_resource_transition_log
                .clone(),
            expected.final_snapshot.equipment_ledger.clone(),
            expected.final_snapshot.class_build_ledger.clone(),
            expected.final_snapshot.equipment_transition_log.clone(),
            expected
                .final_snapshot
                .reaction_window_lifecycle_log
                .clone(),
            expected.final_snapshot.reaction_audit_log.clone(),
            expected
                .final_snapshot
                .modifier_duration_expiration_log
                .clone(),
        ));

        assert!(!readout.accepted);
        assert_eq!(
            readout.decision_kind,
            CombatSessionAutomaticRunReplayDecisionKind::MismatchedEvidence
        );
        assert!(readout.final_state_fingerprint_matches);
        assert!(!readout.finalization_matches);
        assert!(readout.run_decision_kind_matches);
        assert!(readout.executed_step_count_matches);
        assert!(readout.policy_decisions_match);
    }

    fn expected_run(
        scenario: RulebenchScenario,
        run: CombatSessionAutomaticRunSpec,
    ) -> CombatSessionAutomaticRunReadout {
        let mut session = CombatSessionState::new("expected-session", scenario);
        session.run_automatic_combat(run)
    }

    fn zero_step_run() -> CombatSessionAutomaticRunSpec {
        CombatSessionAutomaticRunSpec::new(
            "zero-step",
            "Zero step",
            "Replay evidence without mutation.",
            0,
            Vec::new(),
        )
    }

    fn one_step_run() -> CombatSessionAutomaticRunSpec {
        CombatSessionAutomaticRunSpec::new(
            "one-step",
            "One step",
            "Replay finalization evidence.",
            1,
            Vec::new(),
        )
    }

    fn minimal_replay_scenario() -> RulebenchScenario {
        let selected_action = ActionDefinition {
            id: "placeholder".to_string(),
            ruleset_id: "placeholder-rules".to_string(),
            ability_id: "placeholder-ability".to_string(),
            name: "Placeholder".to_string(),
            actor_id: "placeholder-actor".to_string(),
            targeting: TargetingDeclaration {
                target_kind: TargetKind::Combatant,
                selection: TargetSelection::Single,
                team_constraint: TargetTeamConstraint::Hostile,
                maximum_range: 0,
                visibility_requirement: VisibilityRequirement::Ignored,
                target_ids: Vec::new(),
                visible_target_ids: Vec::new(),
                operation_pipeline: None,
            },
            check: CheckDeclaration::Attack(AttackCheckDeclaration {
                modifier: 0,
                modifier_stat_id: "placeholder-stat".to_string(),
                defense: DefenseReference {
                    id: "placeholder-defense".to_string(),
                    label: "Placeholder defense".to_string(),
                },
            }),
            hit: HitEffect {
                damage_bonus: 0,
                damage_type: "placeholder".to_string(),
                modifier_id: "placeholder-modifier".to_string(),
                modifier_label: "Placeholder modifier".to_string(),
                modifier_duration: "placeholder".to_string(),
                operations: Vec::new(),
            },
            resource_costs: vec![ActionResourceCost::standard_action()],
            movement: None,
            action_text: "Placeholder action.".to_string(),
            effect_text: "Placeholder effect.".to_string(),
        };

        RulebenchScenario {
            metadata: ScenarioMetadata {
                id: "replay-test".to_string(),
                title: "Replay test".to_string(),
                summary: "Minimal no-mutation replay scenario.".to_string(),
                seed_label: "replay-test".to_string(),
            },
            content_pack_set: None,
            authored_action_binding: None,
            authored_scenario_binding: None,
            rulesets: Vec::new(),
            selected_ruleset_id: "placeholder-rules".to_string(),
            grid: Grid {
                width: 0,
                height: 0,
                cells: Vec::new(),
            },
            combatants: Vec::new(),
            entities: Vec::new(),
            abilities: Vec::new(),
            selected_ability_id: None,
            classes: Vec::new(),
            selected_class_id: None,
            stat_definitions: Vec::new(),
            modifiers: Vec::new(),
            items: Vec::new(),
            selected_item_id: None,
            actions: Vec::new(),
            selected_action,
        }
    }

    fn test_pack_set_reference() -> ContentPackSetReference {
        let root = ContentPackReference {
            id: "replay.content".to_string(),
            version: "1.0.0".to_string(),
            fingerprint: ContentFingerprint {
                algorithm: CONTENT_PACK_FINGERPRINT_ALGORITHM.to_string(),
                value: "0123456789abcdef".to_string(),
            },
        };
        let packs = vec![root.clone()];
        ContentPackSetReference {
            fingerprint: fingerprint_content_pack_set(&root, &packs),
            root,
            packs,
        }
    }
}
