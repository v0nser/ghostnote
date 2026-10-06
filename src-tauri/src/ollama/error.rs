use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum OllamaError {
    #[error(
        "Ollama is not reachable at {host}. Set OLLAMA_HOST (for example http://127.0.0.1:11434) and run `ollama serve`."
    )]
    Unavailable { host: String },

    #[error("no local language model is installed")]
    NoModel,

    #[error("llama3.1:8b is not installed. Run `ollama pull llama3.1:8b`.")]
    MissingExtractModel,

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
