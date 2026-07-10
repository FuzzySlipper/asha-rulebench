//! Deterministic combat-session scripts.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub steps: Vec<CombatSessionScriptStepSpec>,
}

impl CombatSessionScriptSpec {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        steps: Vec<CombatSessionScriptStepSpec>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            steps,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptStepSpec {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub command: CombatSessionScriptCommandSpec,
}

impl CombatSessionScriptStepSpec {
    pub fn intent(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        command: CombatSessionIntentCommandSpec,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            command: CombatSessionScriptCommandSpec::Intent(command),
        }
    }

    pub fn control(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        command: CombatControlCommandSpec,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            command: CombatSessionScriptCommandSpec::Control(command),
        }
    }

    pub fn selected_candidate(
        id: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        command: CombatSessionCandidateSelectionSpec,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: summary.into(),
            command: CombatSessionScriptCommandSpec::SelectedCandidate(command),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatSessionScriptCommandSpec {
    Intent(CombatSessionIntentCommandSpec),
    Control(CombatControlCommandSpec),
    SelectedCandidate(CombatSessionCandidateSelectionSpec),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionScriptCommandKind {
    Intent,
    Control,
    SelectedCandidate,
}

impl CombatSessionScriptCommandKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionScriptCommandKind::Intent => "intent",
            CombatSessionScriptCommandKind::Control => "control",
            CombatSessionScriptCommandKind::SelectedCandidate => "selectedCandidate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSessionScriptDecisionKind {
    Intent(CommandDecisionKind),
    Control(CombatControlDecisionKind),
    SelectedCandidateSubmitted(CommandDecisionKind),
    SelectedCandidateSelection(CombatSessionCandidateSelectionDecisionKind),
}

impl CombatSessionScriptDecisionKind {
    pub const fn code(self) -> &'static str {
        match self {
            CombatSessionScriptDecisionKind::Intent(decision_kind) => decision_kind.code(),
            CombatSessionScriptDecisionKind::Control(decision_kind) => decision_kind.code(),
            CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(decision_kind) => {
                decision_kind.code()
            }
            CombatSessionScriptDecisionKind::SelectedCandidateSelection(decision_kind) => {
                decision_kind.code()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptStepReadout {
    pub sequence: u32,
    pub id: String,
    pub title: String,
    pub summary: String,
    pub command_kind: CombatSessionScriptCommandKind,
    pub accepted: bool,
    pub decision_kind: CombatSessionScriptDecisionKind,
    pub reason: String,
    pub state_before_fingerprint: StateFingerprint,
    pub state_after_fingerprint: StateFingerprint,
    pub runtime_step_id: Option<String>,
    pub command_audit_sequence: Option<u32>,
    pub control_history_sequence: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSessionScriptReadout {
    pub session_id: String,
    pub script_id: String,
    pub title: String,
    pub summary: String,
    pub steps: Vec<CombatSessionScriptStepReadout>,
    pub final_snapshot: CombatSessionSnapshot,
}

impl CombatSessionState {
    pub fn run_script(&mut self, spec: CombatSessionScriptSpec) -> CombatSessionScriptReadout {
        let mut steps = Vec::with_capacity(spec.steps.len());
        for (index, step) in spec.steps.into_iter().enumerate() {
            steps.push(self.run_script_step(index as u32, step));
        }

        CombatSessionScriptReadout {
            session_id: self.session_id.clone(),
            script_id: spec.id,
            title: spec.title,
            summary: spec.summary,
            steps,
            final_snapshot: self.snapshot(),
        }
    }

    fn run_script_step(
        &mut self,
        sequence: u32,
        spec: CombatSessionScriptStepSpec,
    ) -> CombatSessionScriptStepReadout {
        match spec.command.clone() {
            CombatSessionScriptCommandSpec::Intent(command) => {
                let readout = self.submit_intent_command(command);
                combat_session_script_intent_step_readout(sequence, spec, &readout)
            }
            CombatSessionScriptCommandSpec::Control(command) => {
                let previous_control_history_len = self.control_history.len();
                let readout = self.submit_control_command(command);
                let control_history_sequence = self
                    .control_history
                    .get(previous_control_history_len)
                    .map(|entry| entry.sequence);
                combat_session_script_control_step_readout(
                    sequence,
                    spec,
                    &readout,
                    control_history_sequence,
                )
            }
            CombatSessionScriptCommandSpec::SelectedCandidate(command) => {
                let state_before_fingerprint = self.snapshot().current_state_fingerprint;
                let execution = self.submit_candidate_command(command);
                let state_after_fingerprint = self.snapshot().current_state_fingerprint;
                combat_session_script_selected_candidate_step_readout(
                    sequence,
                    spec,
                    &execution,
                    state_before_fingerprint,
                    state_after_fingerprint,
                )
            }
        }
    }
}

fn combat_session_script_intent_step_readout(
    sequence: u32,
    spec: CombatSessionScriptStepSpec,
    readout: &CombatSessionStepReadout,
) -> CombatSessionScriptStepReadout {
    CombatSessionScriptStepReadout {
        sequence,
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        command_kind: CombatSessionScriptCommandKind::Intent,
        accepted: readout.audit_entry.accepted,
        decision_kind: CombatSessionScriptDecisionKind::Intent(readout.audit_entry.decision_kind),
        reason: intent_script_step_reason(&readout.audit_entry),
        state_before_fingerprint: readout.audit_entry.state_before_fingerprint.clone(),
        state_after_fingerprint: readout.audit_entry.state_after_fingerprint.clone(),
        runtime_step_id: Some(readout.step.id.clone()),
        command_audit_sequence: Some(readout.audit_entry.sequence),
        control_history_sequence: None,
    }
}

fn combat_session_script_control_step_readout(
    sequence: u32,
    spec: CombatSessionScriptStepSpec,
    readout: &CombatControlReadout,
    control_history_sequence: Option<u32>,
) -> CombatSessionScriptStepReadout {
    CombatSessionScriptStepReadout {
        sequence,
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        command_kind: CombatSessionScriptCommandKind::Control,
        accepted: readout.accepted,
        decision_kind: CombatSessionScriptDecisionKind::Control(readout.decision_kind),
        reason: readout.reason.clone(),
        state_before_fingerprint: readout.state_before_fingerprint.clone(),
        state_after_fingerprint: readout.state_after_fingerprint.clone(),
        runtime_step_id: None,
        command_audit_sequence: None,
        control_history_sequence,
    }
}

fn combat_session_script_selected_candidate_step_readout(
    sequence: u32,
    spec: CombatSessionScriptStepSpec,
    readout: &CombatSessionCandidateExecutionReadout,
    state_before_fingerprint: StateFingerprint,
    state_after_fingerprint: StateFingerprint,
) -> CombatSessionScriptStepReadout {
    if let Some(submitted_step) = &readout.submitted_step {
        return CombatSessionScriptStepReadout {
            sequence,
            id: spec.id,
            title: spec.title,
            summary: spec.summary,
            command_kind: CombatSessionScriptCommandKind::SelectedCandidate,
            accepted: submitted_step.audit_entry.accepted,
            decision_kind: CombatSessionScriptDecisionKind::SelectedCandidateSubmitted(
                submitted_step.audit_entry.decision_kind,
            ),
            reason: selected_candidate_submitted_script_step_reason(&submitted_step.audit_entry),
            state_before_fingerprint: submitted_step.audit_entry.state_before_fingerprint.clone(),
            state_after_fingerprint: submitted_step.audit_entry.state_after_fingerprint.clone(),
            runtime_step_id: Some(submitted_step.step.id.clone()),
            command_audit_sequence: Some(submitted_step.audit_entry.sequence),
            control_history_sequence: None,
        };
    }

    CombatSessionScriptStepReadout {
        sequence,
        id: spec.id,
        title: spec.title,
        summary: spec.summary,
        command_kind: CombatSessionScriptCommandKind::SelectedCandidate,
        accepted: false,
        decision_kind: CombatSessionScriptDecisionKind::SelectedCandidateSelection(
            readout.selection.decision_kind,
        ),
        reason: readout.selection.reason.clone(),
        state_before_fingerprint,
        state_after_fingerprint,
        runtime_step_id: None,
        command_audit_sequence: None,
        control_history_sequence: None,
    }
}

fn intent_script_step_reason(audit_entry: &CommandAuditEntry) -> String {
    match audit_entry.decision_kind {
        CommandDecisionKind::AcceptedByResolver => "Intent command accepted by resolver.",
        CommandDecisionKind::RejectedByResolver => "Intent command rejected by resolver.",
        CommandDecisionKind::RejectedByPreflight => "Intent command rejected by preflight.",
        CommandDecisionKind::RejectedByLifecycle => "Intent command rejected by lifecycle.",
        CommandDecisionKind::RejectedByTurnOrder => "Intent command rejected by turn order.",
    }
    .to_string()
}

fn selected_candidate_submitted_script_step_reason(audit_entry: &CommandAuditEntry) -> String {
    match audit_entry.decision_kind {
        CommandDecisionKind::AcceptedByResolver => {
            "Selected candidate command accepted by resolver."
        }
        CommandDecisionKind::RejectedByResolver => {
            "Selected candidate command rejected by resolver."
        }
        CommandDecisionKind::RejectedByPreflight => {
            "Selected candidate command rejected by preflight."
        }
        CommandDecisionKind::RejectedByLifecycle => {
            "Selected candidate command rejected by lifecycle."
        }
        CommandDecisionKind::RejectedByTurnOrder => {
            "Selected candidate command rejected by turn order."
        }
    }
    .to_string()
}
