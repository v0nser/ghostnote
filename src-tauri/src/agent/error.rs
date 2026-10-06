use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("{0}")]
    Message(String),

    #[error("task not found")]
    TaskNotFound,

    #[error("approval not found")]
    ApprovalNotFound,

    #[error("this action is blocked by Sentinel")]
    Denied,

    #[error("this action needs your approval")]
    NeedsApproval,

    #[error("a meeting is already being recorded")]
    CaptureBusy,

    #[error("voice listening is already active")]
    VoiceBusy,

    #[error("the speech model is not installed — open Setup and download it")]
    ModelMissing,
}

impl AgentError {
    pub fn msg(text: impl Into<String>) -> Self {
        Self::Message(text.into())
    }
}

impl Serialize for AgentError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AgentResult<T> = Result<T, AgentError>;
