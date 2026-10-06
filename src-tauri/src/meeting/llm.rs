//! Local Ollama call for meeting extraction. Always `llama3.1:8b`.

use crate::ollama::client::{Client, EXTRACT_MODEL};
use crate::ollama::error::{OllamaError, OllamaResult};

use super::extract::{
    chunk_transcript, merge_batches, parse_extracted, ExtractedBatch, EXTRACT_RETRY, EXTRACT_SYSTEM,
};

/// Extract structured items from a transcript. Empty input returns an empty
/// batch without calling the model. Malformed JSON is retried once per chunk.
pub async fn extract_actions(client: &Client, transcript: &str) -> OllamaResult<ExtractedBatch> {
    let chunks = chunk_transcript(transcript);
    if chunks.is_empty() {
        return Ok(ExtractedBatch::default());
    }

    let model = client.resolve_extract_model().await?;
    let mut batches = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let user = format!(
            "Transcript chunk {} of {}:\n{chunk}",
            index + 1,
            chunks.len()
        );
        let batch = extract_chunk(client, &model, &user).await?;
        batches.push(batch);
    }
    Ok(merge_batches(batches))
}

async fn extract_chunk(client: &Client, model: &str, user: &str) -> OllamaResult<ExtractedBatch> {
    let first = client
        .complete_json(model, EXTRACT_SYSTEM, user, 360, 2048)
        .await?;
    match parse_extracted(&first) {
        Ok(batch) => Ok(batch),
        Err(first_err) => {
            log::warn!("meeting extract JSON was unusable; retrying once: {first_err}");
            let retry_user = format!("{user}\n\n{EXTRACT_RETRY}");
            let second = client
                .complete_json(model, EXTRACT_SYSTEM, &retry_user, 360, 2048)
                .await?;
            parse_extracted(&second).map_err(|err| {
                OllamaError::Request(format!(
                    "llama3.1:8b did not return usable action-item JSON ({err})"
                ))
            })
        }
    }
}

pub fn extract_model_name() -> &'static str {
    EXTRACT_MODEL
}
