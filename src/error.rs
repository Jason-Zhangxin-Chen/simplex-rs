use thiserror::Error;

/// Errors that can arise during consensus operation.
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// The message could not be verified (bad signature, wrong author, etc.).
    #[error("message verification failed: {0}")]
    VerificationFailed(String),

    /// The message carries an unsupported or unknown type tag.
    #[error("unknown message type")]
    UnknownMessageType,

    /// The view number is too far behind and the message cannot be processed.
    #[error("stale view: message from view {msg_view}, current view {current_view}")]
    StaleView {
        msg_view: crate::types::View,
        current_view: crate::types::View,
    },

    /// Network / IO error (wraps the underlying transport error).
    #[error("network error: {0}")]
    Network(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Crypto error (signing, hashing, etc.).
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Application state-machine rejected a command.
    #[error("state machine error: {0}")]
    StateMachine(String),
}
