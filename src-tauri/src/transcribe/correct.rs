//! Repair Whisper misses on short voice commands.
//!
//! `base.en` often hears “play” as “plea” / “please” / “pliam” on accents
//! and clipped clips. We only rewrite when the rest of the phrase looks
//! like a computer command — we do not invent meeting speech.

const COMMAND_VERBS: &[&str] = &[
    "play", "open", "launch", "start", "set", "send", "summarize", "search",
    "find", "create", "schedule", "remind", "draft", "close", "focus",
];

/// Words Whisper invents for “play” on short or accented clips.
const PLAY_MISHEARS: &[&str] = &[
    "plea", "pleas", "plee", "pley", "pliam", "pliams", "plier", "ply",
    "pray", "prays", "clay", "blake", "blade", "place", "plain",
];

const PLEASE_KEEP: &[&str] = &[
    "help", "don't", "dont", "stop", "wait", "can", "could", "would",
    "send", "tell", "make", "do", "give", "let",
];

const MEDIA_HINTS: &[&str] = &[
    "music", "song", "songs", "playlist", "album", "artist", "track",
    "youtube", "spotify", "radio", "tune", "gaana", "audio",
];

/// Fix a spoken command after Whisper. Safe to run on the full utterance.
pub fn voice_command(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let tokens: Vec<String> = trimmed
        .split_whitespace()
        .map(|word| word.trim_matches(|ch: char| ".,!?;:\"'()[]".contains(ch)).to_string())
        .filter(|word| !word.is_empty())
        .collect();
    if tokens.is_empty() {
        return trimmed.to_string();
    }

    let first = tokens[0].to_ascii_lowercase();
    if COMMAND_VERBS.contains(&first.as_str()) {
        return trimmed.to_string();
    }

    if looks_like_play(&first, &tokens) {
        return rewrite_first(trimmed, &tokens[0], "play");
    }
    if looks_like_open(&first, &tokens) {
        return rewrite_first(trimmed, &tokens[0], "open");
    }
    if looks_like_set(&first) {
        return rewrite_first(trimmed, &tokens[0], "set");
    }
    if looks_like_summarize(&first) {
        return rewrite_first(trimmed, &tokens[0], "summarize");
    }

    trimmed.to_string()
}

fn looks_like_play(first: &str, tokens: &[String]) -> bool {
    let rest: Vec<String> = tokens.iter().skip(1).map(|w| w.to_ascii_lowercase()).collect();
    if first == "please" && rest.first().is_some_and(|next| PLEASE_KEEP.contains(&next.as_str())) {
        return false;
    }
    if PLAY_MISHEARS.contains(&first) || first == "please" {
        if rest.is_empty() {
            return first != "please";
        }
        if rest.iter().any(|word| MEDIA_HINTS.contains(&word.as_str())) {
            return true;
        }
        if rest.first().is_some_and(|next| PLEASE_KEEP.contains(&next.as_str())) {
            return false;
        }
        // “plea Kumar Sanu”, “please my workout playlist”
        return true;
    }
    false
}

fn looks_like_open(first: &str, tokens: &[String]) -> bool {
    matches!(first, "opon" | "opin" | "owen" | "hopin" | "hope" | "opened")
        && tokens.len() > 1
}

fn looks_like_set(first: &str) -> bool {
    matches!(first, "sat" | "said" | "sit")
}

fn looks_like_summarize(first: &str) -> bool {
    matches!(first, "summarise" | "sommarize" | "summerize" | "summaries")
}

fn rewrite_first(original: &str, first: &str, replacement: &str) -> String {
    let Some(pos) = original.find(first) else {
        return format!("{replacement} {}", original.trim());
    };
    let mut out = String::new();
    out.push_str(&original[..pos]);
    out.push_str(replacement);
    out.push_str(&original[pos + first.len()..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plea_alone_is_play() {
        assert_eq!(voice_command("plea"), "play");
        assert_eq!(voice_command("Plea."), "play.");
        assert_eq!(voice_command("pliam"), "play");
    }

    #[test]
    fn plea_artist_is_play() {
        assert_eq!(
            voice_command("plea Kumar Sanu"),
            "play Kumar Sanu"
        );
        assert_eq!(voice_command("please my workout playlist"), "play my workout playlist");
    }

    #[test]
    fn please_help_stays() {
        assert_eq!(voice_command("please help"), "please help");
        assert_eq!(voice_command("please don't"), "please don't");
    }

    #[test]
    fn already_play_stays() {
        assert_eq!(voice_command("play music"), "play music");
    }

    #[test]
    fn empty_stays_empty() {
        assert_eq!(voice_command("   "), "");
    }
}
