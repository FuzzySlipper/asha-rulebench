use std::collections::BTreeMap;

use rulebench_combat::{
    CombatSessionCreateRequest, CombatSessionSnapshot, CombatSessionState,
    RulesetArtifactProvenance, StateFingerprint,
};

use crate::{
    record_replay_package, verification::execute_command, ReplayCommandRecord,
    ReplayCommandRecordingSpec,
};

pub const SESSION_RECOVERY_PACKAGE_VERSION: &str = "1.0.0";

/// Durable identity for the last fully verified authority frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecoveryFrame {
    pub generation: u64,
    pub command_sequence: Option<u32>,
    pub command_id: Option<String>,
    pub state_fingerprint: StateFingerprint,
    pub gameplay_module_state_hash: String,
    pub pending_reaction_window_id: Option<String>,
}

/// Rust-owned durable input for reconstructing one active authority session.
///
/// The package deliberately stores public scenario/provenance and typed command
/// evidence. It never stores `CombatSessionState`, ASHA continuation tokens, or
/// opaque composed-runtime checkpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecoveryPackage {
    pub package_version: String,
    pub initial_session: CombatSessionCreateRequest,
    pub ruleset: RulesetArtifactProvenance,
    pub commands: Vec<ReplayCommandRecord>,
    pub frame: SessionRecoveryFrame,
}

impl SessionRecoveryPackage {
    pub fn record(
        generation: u64,
        initial_session: CombatSessionCreateRequest,
        ruleset: RulesetArtifactProvenance,
        commands: Vec<ReplayCommandRecordingSpec>,
    ) -> Result<Self, SessionRecoveryError> {
        verify_ruleset(&initial_session, &ruleset)?;
        if generation != commands.len() as u64 {
            return Err(SessionRecoveryError::GenerationMismatch {
                expected: commands.len() as u64,
                actual: generation,
            });
        }
        let package = record_replay_package(
            format!("recovery:{}", initial_session.session.id),
            initial_session.clone(),
            ruleset.clone(),
            commands,
        );
        let restored = replay_commands(&initial_session, &package.commands)?;
        let snapshot = restored.snapshot();
        let command_sequence = package.commands.last().map(|command| command.sequence);
        let command_id = package.commands.last().map(|command| command.id.clone());
        Ok(Self {
            package_version: SESSION_RECOVERY_PACKAGE_VERSION.to_string(),
            initial_session,
            ruleset,
            commands: package.commands,
            frame: SessionRecoveryFrame {
                generation,
                command_sequence,
                command_id,
                state_fingerprint: snapshot.current_state_fingerprint.clone(),
                gameplay_module_state_hash: snapshot.gameplay_fabric.module_state_hash.clone(),
                pending_reaction_window_id: snapshot
                    .current_reaction_window
                    .as_ref()
                    .map(|window| window.id.clone()),
            },
        })
    }

    pub fn restore(&self) -> Result<RecoveredSession, SessionRecoveryError> {
        if self.package_version != SESSION_RECOVERY_PACKAGE_VERSION {
            return Err(SessionRecoveryError::UnsupportedVersion {
                version: self.package_version.clone(),
            });
        }
        if self.frame.generation != self.commands.len() as u64 {
            return Err(SessionRecoveryError::GenerationMismatch {
                expected: self.commands.len() as u64,
                actual: self.frame.generation,
            });
        }
        verify_ruleset(&self.initial_session, &self.ruleset)?;
        let state = replay_commands(&self.initial_session, &self.commands)?;
        let snapshot = state.snapshot();
        let actual_command_sequence = self.commands.last().map(|command| command.sequence);
        let actual_command_id = self.commands.last().map(|command| command.id.clone());
        let actual_reaction_window_id = snapshot
            .current_reaction_window
            .as_ref()
            .map(|window| window.id.clone());
        if self.frame.command_sequence != actual_command_sequence
            || self.frame.command_id != actual_command_id
            || self.frame.state_fingerprint != snapshot.current_state_fingerprint
            || self.frame.gameplay_module_state_hash != snapshot.gameplay_fabric.module_state_hash
            || self.frame.pending_reaction_window_id != actual_reaction_window_id
        {
            return Err(SessionRecoveryError::FrameMismatch);
        }
        Ok(RecoveredSession { state, snapshot })
    }

    pub fn session_id(&self) -> &str {
        &self.initial_session.session.id
    }
}

#[derive(Debug)]
pub struct RecoveredSession {
    pub state: CombatSessionState,
    pub snapshot: CombatSessionSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRecoveryError {
    UnsupportedVersion { version: String },
    GenerationMismatch { expected: u64, actual: u64 },
    RulesetProvenanceMismatch,
    CommandEvidenceMismatch { command_id: String },
    FrameMismatch,
}

impl SessionRecoveryError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion { .. } => "unsupportedSessionRecoveryVersion",
            Self::GenerationMismatch { .. } => "sessionRecoveryGenerationMismatch",
            Self::RulesetProvenanceMismatch => "sessionRecoveryRulesetProvenanceMismatch",
            Self::CommandEvidenceMismatch { .. } => "sessionRecoveryCommandEvidenceMismatch",
            Self::FrameMismatch => "sessionRecoveryFrameMismatch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionRecoveryStorageError {
    WriteFailed { session_id: String },
    ReadFailed,
    DeleteFailed { session_id: String },
}

pub trait SessionRecoveryStorage: core::fmt::Debug {
    fn write(&mut self, package: SessionRecoveryPackage)
        -> Result<(), SessionRecoveryStorageError>;
    fn list(&self) -> Result<Vec<SessionRecoveryPackage>, SessionRecoveryStorageError>;
    fn delete(&mut self, session_id: &str) -> Result<(), SessionRecoveryStorageError>;
}

#[derive(Debug, Default)]
pub struct InMemorySessionRecoveryStorage {
    packages: BTreeMap<String, SessionRecoveryPackage>,
}

impl InMemorySessionRecoveryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SessionRecoveryStorage for InMemorySessionRecoveryStorage {
    fn write(
        &mut self,
        package: SessionRecoveryPackage,
    ) -> Result<(), SessionRecoveryStorageError> {
        self.packages
            .insert(package.session_id().to_string(), package);
        Ok(())
    }

    fn list(&self) -> Result<Vec<SessionRecoveryPackage>, SessionRecoveryStorageError> {
        Ok(self.packages.values().cloned().collect())
    }

    fn delete(&mut self, session_id: &str) -> Result<(), SessionRecoveryStorageError> {
        self.packages.remove(session_id);
        Ok(())
    }
}

fn verify_ruleset(
    initial_session: &CombatSessionCreateRequest,
    expected: &RulesetArtifactProvenance,
) -> Result<(), SessionRecoveryError> {
    let Some(actual) = initial_session.scenario.selected_ruleset() else {
        return Err(SessionRecoveryError::RulesetProvenanceMismatch);
    };
    if actual.artifact_provenance() != *expected {
        return Err(SessionRecoveryError::RulesetProvenanceMismatch);
    }
    Ok(())
}

fn replay_commands(
    initial_session: &CombatSessionCreateRequest,
    commands: &[ReplayCommandRecord],
) -> Result<CombatSessionState, SessionRecoveryError> {
    let mut state = CombatSessionState::new(
        initial_session.session.id.clone(),
        initial_session.scenario.clone(),
    );
    for command in commands {
        let actual = execute_command(&mut state, &command.command);
        if actual != command.expected {
            return Err(SessionRecoveryError::CommandEvidenceMismatch {
                command_id: command.id.clone(),
            });
        }
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReplayCommand;
    use rulebench_combat::{CombatControlCommandSpec, CombatLifecyclePhase};

    fn initial_session() -> CombatSessionCreateRequest {
        CombatSessionCreateRequest::new(
            "recoverable",
            crate::verification::tests::recorded_control_package()
                .initial_session
                .scenario,
        )
    }

    #[test]
    fn package_reconstructs_exact_active_frame_and_rejects_tampering() {
        let initial = initial_session();
        let ruleset = initial
            .scenario
            .selected_ruleset()
            .expect("fixture ruleset")
            .artifact_provenance();
        let commands = vec![ReplayCommandRecordingSpec::new(
            "start",
            ReplayCommand::Control(CombatControlCommandSpec::explicit_start()),
        )];
        let package = SessionRecoveryPackage::record(1, initial, ruleset, commands)
            .expect("recovery package records");

        let restored = package.restore().expect("package restores");
        assert_eq!(
            restored.snapshot.lifecycle.phase,
            CombatLifecyclePhase::InProgress
        );

        let mut tampered = package;
        tampered
            .frame
            .gameplay_module_state_hash
            .push_str("-tampered");
        assert_eq!(
            tampered
                .restore()
                .expect_err("tampered frame rejects")
                .code(),
            "sessionRecoveryFrameMismatch"
        );
    }

    #[test]
    fn generation_and_ruleset_identity_are_strict() {
        let initial = initial_session();
        let mut ruleset = initial
            .scenario
            .selected_ruleset()
            .expect("fixture ruleset")
            .artifact_provenance();
        assert_eq!(
            SessionRecoveryPackage::record(1, initial.clone(), ruleset.clone(), Vec::new())
                .expect_err("generation mismatch")
                .code(),
            "sessionRecoveryGenerationMismatch"
        );
        ruleset.ruleset_version.push_str("-other");
        assert_eq!(
            SessionRecoveryPackage::record(0, initial, ruleset, Vec::new())
                .expect_err("provenance mismatch")
                .code(),
            "sessionRecoveryRulesetProvenanceMismatch"
        );
    }
}
