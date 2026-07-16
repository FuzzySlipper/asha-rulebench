use rulebench_rules::{
    AuthoredActionBindingError, CombatSessionApiError, ReplayArchiveError, SessionRecoveryError,
    SessionRecoveryStorageError,
};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeErrorKind {
    ProtocolVersionMismatch,
    InvalidRequest,
    UnknownScenario,
    DuplicateSession,
    UnknownSession,
    InvalidScenario,
    AuthoredActionBinding,
    InvalidLifecycle,
    ReplayArchive,
    SessionRecovery,
}

impl BridgeErrorKind {
    pub const fn code(self) -> &'static str {
        match self {
            BridgeErrorKind::ProtocolVersionMismatch => "protocolVersionMismatch",
            BridgeErrorKind::InvalidRequest => "invalidRequest",
            BridgeErrorKind::UnknownScenario => "unknownScenario",
            BridgeErrorKind::DuplicateSession => "duplicateSession",
            BridgeErrorKind::UnknownSession => "unknownSession",
            BridgeErrorKind::InvalidScenario => "invalidScenario",
            BridgeErrorKind::AuthoredActionBinding => "authoredActionBinding",
            BridgeErrorKind::InvalidLifecycle => "invalidLifecycle",
            BridgeErrorKind::ReplayArchive => "replayArchive",
            BridgeErrorKind::SessionRecovery => "sessionRecovery",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeError {
    pub kind: BridgeErrorKind,
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl Display for BridgeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BridgeError {}

impl BridgeError {
    pub(crate) fn new(kind: BridgeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            code: kind.code().to_string(),
            message: message.into(),
            retryable: false,
        }
    }

    pub(crate) fn from_session_error(error: CombatSessionApiError) -> Self {
        match error {
            CombatSessionApiError::EmptySessionId => Self::new(
                BridgeErrorKind::InvalidRequest,
                "Session id must not be empty.",
            ),
            CombatSessionApiError::DuplicateSessionId { session_id } => Self::new(
                BridgeErrorKind::DuplicateSession,
                format!("Session already exists: {session_id}"),
            ),
            CombatSessionApiError::UnknownSessionId { session_id } => Self::new(
                BridgeErrorKind::UnknownSession,
                format!("Session does not exist: {session_id}"),
            ),
            CombatSessionApiError::SessionNotFinalized { session_id } => Self::new(
                BridgeErrorKind::InvalidLifecycle,
                format!("Session must be finalized before close: {session_id}"),
            ),
            CombatSessionApiError::InvalidScenario { .. } => Self::new(
                BridgeErrorKind::InvalidScenario,
                "Scenario content was rejected by Rust validation.",
            ),
        }
    }

    pub(crate) fn from_replay_error(error: ReplayArchiveError) -> Self {
        Self {
            kind: BridgeErrorKind::ReplayArchive,
            code: error.code().to_string(),
            message: format!("{error:?}"),
            retryable: matches!(error, ReplayArchiveError::Storage(_)),
        }
    }

    pub(crate) fn from_authored_action_binding_error(error: AuthoredActionBindingError) -> Self {
        let diagnostic_suffix = if error.diagnostic_codes.is_empty() {
            String::new()
        } else {
            format!(" Diagnostics: {}.", error.diagnostic_codes.join(", "))
        };
        Self {
            kind: BridgeErrorKind::AuthoredActionBinding,
            code: error.code.to_string(),
            message: format!("{}{}", error.message, diagnostic_suffix),
            retryable: false,
        }
    }

    pub(crate) fn from_recovery_error(error: SessionRecoveryError) -> Self {
        Self {
            kind: BridgeErrorKind::SessionRecovery,
            code: error.code().to_string(),
            message: format!("{error:?}"),
            retryable: false,
        }
    }

    pub(crate) fn from_recovery_storage_error(error: SessionRecoveryStorageError) -> Self {
        Self {
            kind: BridgeErrorKind::SessionRecovery,
            code: "sessionRecoveryStorageFailed".to_string(),
            message: format!("{error:?}"),
            retryable: true,
        }
    }
}
