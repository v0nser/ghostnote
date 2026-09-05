//! Invoking the bundled `whisper-cli` sidecar.
//!
//! Whisper runs as a separate process rather than linked into the app. A
//! segfault or OOM inside a 141 MB GGML graph then kills a child process
//! instead of the window the user is taking notes in, and the OS reclaims the
//! model's memory between meetings.

use std::path::Path;

use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

use super::error::{TranscribeError, TranscribeResult};

/// Name of the external binary declared in `tauri.conf.json`.
const SIDECAR: &str = "whisper-cli";

/// Whisper emits these placeholders when handed audio it considers empty.
/// They are artefacts, not speech, and must never reach the transcript.
const ARTEFACTS: [&str; 6] = [
    "[BLANK_AUDIO]",
    "[SILENCE]",
    "(silence)",
    "[ Silence ]",
    "[INAUDIBLE]",
    "[NOISE]",
];

/// Transcribes one 16 kHz mono WAV file and returns the recognised text.
///
/// Returns `Ok(None)` when the clip contained no speech, which is common: the
/// segmenter cuts on energy, and a door slam has plenty of energy.
pub async fn transcribe(
    app: &AppHandle,
    model: &Path,
    wav: &Path,
) -> TranscribeResult<Option<String>> {
    // Leave headroom: saturating every core makes the UI and the audio
    // callbacks compete with transcription, which causes dropped blocks.
    let threads = std::thread::available_parallelism()
        .map(|n| (n.get().saturating_sub(2)).clamp(4, 8))
        .unwrap_or(4);

    let command = app
        .shell()
        .sidecar(SIDECAR)
        .map_err(|err| TranscribeError::SidecarSpawn(err.to_string()))?
        .args([
            "--model".into(),
            model.to_string_lossy().into_owned(),
            "--file".into(),
            wav.to_string_lossy().into_owned(),
            "--language".into(),
            "en".into(),
            "--threads".into(),
            threads.to_string(),
            // Greedy decode. Beam search 5× is accurate on long-form but
            // each extra hypothesis is latency we cannot spare on a live turn.
            "--best-of".into(),
            "1".into(),
            "--beam-size".into(),
            "1".into(),
            "--no-fallback".into(),
            // Metal for the clip that matters: the finished question.
            // Ollama is idle until that text exists, so they do not fight.
            "--suppress-nst".into(),
            "--no-speech-thold".into(),
            "0.7".into(),
            // Timestamps come from our segmenter, which knows where the clip
            // sat in the meeting; Whisper's are relative to the clip.
            "--no-timestamps".into(),
            // Keeps stdout to the transcript alone. Progress and model
            // diagnostics still go to stderr.
            "--no-prints".into(),
        ]);

    let output = command
        .output()
        .await
        .map_err(|err| TranscribeError::SidecarFailed(err.to_string()))?;

    if !output.status.success() {
        // stderr can contain file paths but never audio content, so it is safe
        // to surface. Truncated because GGML backtraces are enormous.
        let detail: String = String::from_utf8_lossy(&output.stderr)
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(TranscribeError::SidecarFailed(detail));
    }

    Ok(clean(&String::from_utf8_lossy(&output.stdout)))
}

/// Primes the sidecar so the first real transcription is not the slow one.
///
/// On macOS the very first execution of a freshly installed binary pays for
/// Gatekeeper verification, and the Metal backend compiles its shader library
/// on first use. Together that was ~26 s in testing versus ~0.4 s warm, which
/// would otherwise land on the user's first sentence of their first meeting.
pub async fn warm_up(app: &AppHandle) {
    let Ok(command) = app.shell().sidecar(SIDECAR) else {
        log::warn!("whisper sidecar is unavailable; skipping warm-up");
        return;
    };

    match command.args(["--help"]).output().await {
        Ok(_) => log::debug!("whisper sidecar binary is ready"),
        Err(err) => log::warn!("whisper sidecar warm-up failed: {err}"),
    }

    // Compiles the Metal graph now. `--help` does not load the model, so the
    // first real question would otherwise pay that cost on the live turn.
    let Ok(model) = super::model::resolve(app) else {
        return;
    };
    let Ok(cache) = app.path().app_cache_dir() else {
        return;
    };
    let tone: Vec<f32> = (0..3_200)
        .map(|i| ((i as f32) * 0.08).sin() * 0.2)
        .collect();
    let Ok(clip) = crate::audio::wav::write(&cache.join("utterances"), "warmup", &tone) else {
        return;
    };
    match transcribe(app, &model, clip.path()).await {
        Ok(_) => log::info!("whisper model is warm"),
        Err(err) => log::warn!("whisper model warm-up failed: {err}"),
    }
}

/// Trims whitespace and drops Whisper's no-speech placeholders.
fn clean(raw: &str) -> Option<String> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }

    let stripped: String = text
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty() && !ARTEFACTS.iter().any(|a| a.eq_ignore_ascii_case(line))
        })
        .collect::<Vec<_>>()
        .join(" ");

    let stripped = stripped.trim();
    if stripped.is_empty() {
        return None;
    }

    // Whisper invents new artefacts between versions, so matching the list
    // above is not enough on its own.
    if is_all_annotation(stripped) {
        return None;
    }

    if is_whisper_junk(stripped) {
        return None;
    }

    Some(stripped.to_string())
}

/// YouTube-style lines Whisper invents on noise. They are not interview speech
/// and must not be sent to the language model.
fn is_whisper_junk(text: &str) -> bool {
    let lower = text.to_lowercase();
    const JUNK: [&str; 8] = [
        "thank you for watching",
        "thanks for watching",
        "please subscribe",
        "like and subscribe",
        "subscribe to",
        "leave a comment",
        "see you in the next",
        "don't forget to smash",
    ];
    JUNK.iter().any(|phrase| lower.contains(phrase))
}

/// True when the text is nothing but bracketed annotations.
///
/// Handed a clip with no speech in it, Whisper does not return an empty
/// string — it narrates: "(dramatic music)", "(speaks in foreign language)",
/// "[laughing]", "[_TT_170]". Whatever the annotation claims, a clip that
/// transcribed to annotation alone contained no words, and a transcript is a
/// record of what was said.
///
/// Annotation mixed *with* speech is left alone: "[laughs] that is fair" is a
/// faithful record of a real sentence.
fn is_all_annotation(text: &str) -> bool {
    let mut chars = text.chars();
    let mut saw_annotation = false;

    while let Some(c) = chars.next() {
        match c {
            '(' | '[' => {
                let close = if c == '(' { ')' } else { ']' };
                // An unclosed bracket is far more likely to be speech that
                // happens to contain one than a truncated annotation.
                if !chars.any(|c| c == close) {
                    return false;
                }
                saw_annotation = true;
            }
            c if c.is_whitespace() => {}
            // Punctuation between or after spans is still annotation: Whisper
            // writes "(music)." and "[laughs], [sighs]".
            '.' | ',' | '!' | '?' | ';' | ':' | '-' | '…' => {}
            _ => return false,
        }
    }

    saw_annotation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_blank_audio_marker() {
        assert_eq!(clean("  [BLANK_AUDIO]  "), None);
        assert_eq!(clean("\n"), None);
        assert_eq!(clean("[_TT_170]"), None);
    }

    #[test]
    fn joins_multiline_output() {
        assert_eq!(
            clean(" Hello there.\n How are you? \n"),
            Some("Hello there. How are you?".to_string())
        );
    }

    #[test]
    fn keeps_bracketed_text_inside_speech() {
        assert_eq!(
            clean("[laughs] that is fair"),
            Some("[laughs] that is fair".to_string())
        );
    }

    /// What Whisper actually returned for clips of an empty room during
    /// end-to-end testing.
    #[test]
    fn drops_narrated_silence() {
        for hallucination in [
            "(dramatic music) (speaks in foreign language)",
            "(laughing) (speaking in foreign language)",
            "[ Silence ]",
            "(music).",
            "[laughs], [sighs]",
        ] {
            assert_eq!(clean(hallucination), None, "kept {hallucination:?}");
        }
    }

    #[test]
    fn keeps_speech_containing_an_unclosed_bracket() {
        assert_eq!(
            clean("the array index (zero based"),
            Some("the array index (zero based".to_string())
        );
    }

    #[test]
    fn drops_youtube_hallucinations() {
        assert_eq!(clean("Thank you for watching"), None);
        assert_eq!(
            clean("can you explain closures in javascript"),
            Some("can you explain closures in javascript".to_string())
        );
    }
}
