use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum OllamaError {
    #[error("Ollama is not running")]
    Unavailable,

    #[error("no local language model is installed")]
    NoModel,

    #[error("the language model returned nothing usable")]
    UnusableOutput,

    #[error("the language model request failed: {0}")]
    Request(String),
}

impl Serialize for OllamaError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type OllamaResult<T> = Result<T, OllamaError>;
