use rulebench_rules::CombatSessionApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeErrorKind {
    ProtocolVersionMismatch,
    InvalidRequest,
    UnknownScenario,
    DuplicateSession,
    UnknownSession,
    InvalidScenario,
    InvalidLifecycle,
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
            BridgeErrorKind::InvalidLifecycle => "invalidLifecycle",
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
}
