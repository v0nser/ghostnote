//! Deterministic command router. Simple computer-use requests never wait on Ollama.

use serde_json::{json, Value};

/// A command we can run without a planning model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FastIntent {
    OpenApplication { name: String },
    OpenUrl { url: String, app: Option<String> },
    OpenFolder { path: String },
    #[allow(dead_code)]
    OpenFile { path: String },
    CreateFolder { path: String },
    SetReminder { title: String, when: String },
    PlayMedia { query: String },
    PlayPlaylist { name: String },
    FindFile { query: String },
    CreateDocument { title: String, content: String },
    DraftEmail { to: String, subject: String, body: String },
    SendEmail { to: String, subject: String, body: String },
    CalendarEvent { title: String, when: String },
    SummarizeFile { query: String },
    TakeScreenshot,
    AskScreen { question: String },
}

impl FastIntent {
    pub fn tool_name(&self) -> &'static str {
        match self {
            Self::OpenApplication { .. } => "OPEN_APPLICATION",
            Self::OpenUrl { .. } => "OPEN_URL",
            Self::OpenFolder { .. } => "OPEN_FOLDER",
            Self::OpenFile { .. } => "OPEN_FILE",
            Self::CreateFolder { .. } => "CREATE_FOLDER",
            Self::SetReminder { .. } => "SET_REMINDER",
            Self::PlayMedia { .. } => "PLAY_MEDIA",
            Self::PlayPlaylist { .. } => "PLAY_PLAYLIST",
            Self::FindFile { .. } => "FIND_FILE",
            Self::CreateDocument { .. } => "CREATE_DOCUMENT",
            Self::DraftEmail { .. } => "DRAFT_EMAIL",
            Self::SendEmail { .. } => "SEND_EMAIL",
            Self::CalendarEvent { .. } => "CALENDAR_EVENT",
            Self::SummarizeFile { .. } => "SUMMARIZE_FILE",
            Self::TakeScreenshot => "TAKE_SCREENSHOT",
            Self::AskScreen { .. } => "ASK_SCREEN",
        }
    }

    pub fn args(&self) -> Value {
        match self {
            Self::OpenApplication { name } => json!({ "name": name }),
            Self::OpenUrl { url, app } => {
                let mut value = json!({ "url": url });
                if let Some(app) = app {
                    value["app"] = json!(app);
                }
                value
            }
            Self::OpenFolder { path } | Self::OpenFile { path } | Self::CreateFolder { path } => {
                json!({ "path": path })
            }
            Self::SetReminder { title, when } => json!({ "title": title, "when": when }),
            Self::PlayMedia { query } => json!({ "query": query }),
            Self::PlayPlaylist { name } => json!({ "name": name }),
            Self::FindFile { query } => json!({ "query": query }),
            Self::CreateDocument { title, content } => json!({ "title": title, "content": content }),
            Self::DraftEmail { to, subject, body } | Self::SendEmail { to, subject, body } => {
                json!({ "to": to, "subject": subject, "body": body })
            }
            Self::CalendarEvent { title, when } => json!({ "title": title, "when": when }),
            Self::SummarizeFile { query } => json!({ "query": query }),
            Self::TakeScreenshot => json!({}),
            Self::AskScreen { question } => json!({ "question": question }),
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::OpenApplication { name } => format!("Opening {name}"),
            Self::OpenUrl { url, app: Some(app) } => format!("Opening {url} in {app}"),
            Self::OpenUrl { url, app: None } => format!("Opening {url}"),
            Self::OpenFolder { path } => format!("Opening folder {path}"),
            Self::OpenFile { path } => format!("Opening {path}"),
            Self::CreateFolder { path } => format!("Creating folder {path}"),
            Self::SetReminder { title, when } => format!("Setting reminder “{title}” for {when}"),
            Self::PlayMedia { query } => format!("Playing {query}"),
            Self::PlayPlaylist { name } => format!("Playing playlist {name}"),
            Self::FindFile { query } => format!("Finding {query}"),
            Self::CreateDocument { title, .. } => format!("Creating {title}"),
            Self::DraftEmail { subject, .. } => format!("Drafting email: {subject}"),
            Self::SendEmail { to, .. } => format!("Sending email to {to} after you approve"),
            Self::CalendarEvent { title, when } => format!("Adding calendar event “{title}” at {when}"),
            Self::SummarizeFile { query } => {
                if query.is_empty() {
                    "Reading the latest PDF to summarize it".into()
                } else {
                    format!("Reading {query} to summarize it")
                }
            }
            Self::TakeScreenshot => "Taking a screenshot".into(),
            Self::AskScreen { question } => format!("Looking at the screen: {question}"),
        }
    }
}

pub fn is_vague(goal: &str) -> bool {
    let lower = normalize(goal);
    matches!(
        lower.as_str(),
        "what to do"
            | "what should i do"
            | "what can you do"
            | "help"
            | "hello"
            | "hi"
            | "hey"
            | "behave like muse"
            | "behave like a muse"
            | "behave like muse ai"
            | "act like muse"
            | "be muse"
            | "muse"
            | "muse ai"
    ) || lower.starts_with("behave like")
        || lower.starts_with("act like")
}

pub fn capabilities() -> &'static str {
    "I run on this computer with a local Ollama model. I will ask before email, calendar, or delete.\n\n\
     • Summarize a PDF (“summarize the pdf” or “summarize offer.pdf”)\n\
     • Set an alarm or reminder\n\
     • Play a song, artist, or Apple Music playlist\n\
     • Open apps, folders, websites, and files\n\
     • Send or draft email (I ask first, then Mail/Outlook does it)\n\
     • Add a calendar event (I ask first, then Calendar creates it)\n\
     • Search the web\n\n\
     Pull `llama3.1:8b` in Ollama. Grant Mail, Calendar, and Reminders when macOS asks."
}

/// First application named in a complex “open X and …” request.
pub fn leading_application(goal: &str) -> Option<String> {
    let lower = normalize(goal);
    let rest = after_any(&lower, &["open ", "launch ", "start ", "run "])?;
    let name = rest
        .split(" and ")
        .next()?
        .trim()
        .trim_start_matches("the ")
        .trim_start_matches("my ")
        .trim_end_matches(" app")
        .trim();
    if name.is_empty() || looks_like_folder_word(name) || (name.contains('.') && !name.contains(' ')) {
        return None;
    }
    Some(name.to_string())
}

/// Classify a spoken or typed goal. Returns `None` for multi-step work the
/// agent loop should handle.
pub fn route(goal: &str) -> Option<FastIntent> {
    let trimmed = goal.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = normalize(trimmed);

    if let Some(intent) = route_screen_ask(&lower) {
        return Some(intent);
    }
    if lower.contains("screenshot") || lower.contains("screen shot") {
        return Some(FastIntent::TakeScreenshot);
    }
    if let Some(intent) = route_reminder(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_play(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_url(trimmed, &lower) {
        return Some(intent);
    }
    if let Some(intent) = route_summarize(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_file(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_document(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_email(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_calendar(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_search(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_folder(&lower) {
        return Some(intent);
    }
    if let Some(intent) = route_create_folder(&lower) {
        return Some(intent);
    }
    if is_complex(&lower) && !is_browser_search(&lower) {
        return None;
    }
    route_open_application(&lower)
}

fn route_screen_ask(lower: &str) -> Option<FastIntent> {
    if lower.contains("screenshot") || lower.contains("screen shot") {
        return None;
    }
    let hit = lower.contains("on my screen")
        || lower.contains("on the screen")
        || lower.contains("what's on screen")
        || lower.contains("whats on screen")
        || lower.contains("what is on screen")
        || lower.contains("describe my screen")
        || lower.contains("look at my screen")
        || lower.contains("read my screen");
    if !hit {
        return None;
    }
    Some(FastIntent::AskScreen {
        question: if lower.len() > 8 {
            lower.to_string()
        } else {
            "What's on my screen?".into()
        },
    })
}

fn route_reminder(lower: &str) -> Option<FastIntent> {
    if !(lower.contains("alarm")
        || lower.contains("remind me")
        || lower.contains("reminder")
        || lower.contains("wake me"))
    {
        return None;
    }
    let title = if lower.contains("alarm") {
        "Alarm"
    } else {
        "Reminder"
    };
    Some(FastIntent::SetReminder {
        title: title.into(),
        when: lower.to_string(),
    })
}

fn route_play(lower: &str) -> Option<FastIntent> {
    if looks_like_play_music(lower) {
        return Some(FastIntent::OpenApplication {
            name: "Apple Music".into(),
        });
    }
    if let Some(name) = playlist_name(lower) {
        return Some(FastIntent::PlayPlaylist { name });
    }
    let query = after_any(lower, &["play ", "play me ", "listen to "])?;
    let query = query
        .trim()
        .trim_start_matches("a song ")
        .trim_start_matches("songs by ");
    if query.is_empty() || looks_like_folder_word(query) {
        return None;
    }
    Some(FastIntent::PlayMedia {
        query: query.to_string(),
    })
}

fn playlist_name(lower: &str) -> Option<String> {
    if !lower.contains("playlist") {
        return None;
    }
    let rest = after_any(
        lower,
        &[
            "play my ",
            "play the ",
            "play playlist ",
            "play the playlist ",
            "play my playlist ",
        ],
    )
    .or_else(|| after_any(lower, &["play "]))?;
    let name = rest
        .replace(" playlist", "")
        .replace("playlist ", "")
        .replace("called ", "")
        .trim()
        .trim_start_matches("my ")
        .trim_start_matches("the ")
        .trim()
        .to_string();
    if name.is_empty() || name == "playlist" {
        return None;
    }
    Some(name)
}

fn route_file(lower: &str) -> Option<FastIntent> {
    if let Some(query) = after_any(
        lower,
        &[
            "open the file ",
            "open file ",
            "find the file ",
            "find file ",
            "open my file ",
        ],
    ) {
        let query = query.trim().trim_matches('"');
        if !query.is_empty() {
            return Some(FastIntent::FindFile {
                query: query.to_string(),
            });
        }
    }
    if lower.starts_with("open ") {
        let rest = lower.strip_prefix("open ")?;
        if rest.contains('.') && !rest.contains("http") && !rest.contains(' ') {
            return Some(FastIntent::FindFile {
                query: rest.to_string(),
            });
        }
    }
    None
}

fn route_document(lower: &str) -> Option<FastIntent> {
    let rest = after_any(
        lower,
        &[
            "create a document ",
            "create document ",
            "write a document ",
            "write a note ",
            "create a note ",
            "make a document ",
        ],
    )?;
    let title = rest.trim().trim_start_matches("called ").trim_start_matches("about ");
    if title.is_empty() {
        return None;
    }
    Some(FastIntent::CreateDocument {
        title: title.to_string(),
        content: String::new(),
    })
}

fn route_summarize(lower: &str) -> Option<FastIntent> {
    let wants = lower.contains("summarize")
        || lower.contains("summary of")
        || lower.contains("tldr")
        || lower.contains("tl;dr")
        || lower.contains("what's in")
        || lower.contains("what is in")
        || ((lower.contains("read") || lower.contains("recap")) && lower.contains("pdf"));
    if !wants {
        return None;
    }
    if !(lower.contains("pdf")
        || lower.contains(".pdf")
        || lower.contains("document")
        || lower.contains("file")
        || lower.contains("resume")
        || lower.contains("cv"))
    {
        return None;
    }
    let query = after_any(
        lower,
        &[
            "summarize the file ",
            "summarize file ",
            "summarize the pdf ",
            "summarize pdf ",
            "summarize this ",
            "summarize my ",
            "summarize ",
            "summary of ",
            "read the pdf ",
            "read this pdf ",
            "what's in ",
            "what is in ",
        ],
    )
    .unwrap_or("")
    .trim()
    .trim_start_matches("the ")
    .trim_start_matches("this ")
    .trim_start_matches("my ")
    .trim_start_matches("file ")
    .trim()
    .to_string();
    let query = if query.is_empty() || query == "pdf" || query == "document" {
        String::new()
    } else {
        query
    };
    Some(FastIntent::SummarizeFile { query })
}

fn route_email(lower: &str) -> Option<FastIntent> {
    if !(lower.contains("email") || lower.contains("mail ")) {
        return None;
    }
    if !(lower.contains("send") || lower.contains("draft") || lower.contains("write") || lower.starts_with("email ")) {
        return None;
    }
    let (to, subject, body) = parse_email_fields(lower);
    if lower.contains("send") && !lower.contains("draft") {
        return Some(FastIntent::SendEmail { to, subject, body });
    }
    Some(FastIntent::DraftEmail { to, subject, body })
}

fn parse_email_fields(lower: &str) -> (String, String, String) {
    let to = lower
        .split_whitespace()
        .find(|word| word.contains('@') && word.contains('.'))
        .map(|word| word.trim_matches(|ch: char| "<>\"',;".contains(ch)).to_string())
        .unwrap_or_default();
    let subject = after_any(lower, &["about ", "subject ", "regarding "])
        .map(|rest| {
            rest.split(" saying ")
                .next()
                .unwrap_or(rest)
                .split(" that ")
                .next()
                .unwrap_or(rest)
                .trim()
                .to_string()
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "Coda".into());
    let body = after_any(lower, &["saying ", "that says ", "body "])
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    (to, subject, body)
}

fn route_calendar(lower: &str) -> Option<FastIntent> {
    if !(lower.contains("calendar") || lower.contains("meeting") || lower.contains("appointment") || lower.contains("event"))
    {
        return None;
    }
    if !(lower.contains("add")
        || lower.contains("create")
        || lower.contains("schedule")
        || lower.contains("put")
        || lower.contains("book"))
    {
        return None;
    }
    let title = after_any(
        lower,
        &[
            "schedule a meeting ",
            "schedule meeting ",
            "schedule a ",
            "add a meeting ",
            "add meeting ",
            "create a calendar event ",
            "create calendar event ",
            "book a meeting ",
        ],
    )
    .map(|rest| {
        rest.trim()
            .trim_start_matches("called ")
            .trim_start_matches("for ")
            .to_string()
    })
    .filter(|text| !text.is_empty())
    .unwrap_or_else(|| "Coda event".into());
    Some(FastIntent::CalendarEvent {
        title,
        when: lower.to_string(),
    })
}

fn route_search(lower: &str) -> Option<FastIntent> {
    if let Some(query) = after_any(
        lower,
        &[
            "search youtube for ",
            "search on youtube for ",
            "youtube search ",
            "find on youtube ",
        ],
    ) {
        return Some(FastIntent::OpenUrl {
            url: youtube_search_url(query),
            app: browser_hint(lower),
        });
    }
    if lower == "open youtube" || lower == "youtube" {
        return Some(FastIntent::OpenUrl {
            url: "https://www.youtube.com".into(),
            app: browser_hint(lower),
        });
    }
    if let Some(query) = after_any(
        lower,
        &[
            "search google for ",
            "google search ",
            "search for ",
            "google ",
        ],
    ) {
        if query.is_empty() || query == "google" {
            return None;
        }
        return Some(FastIntent::OpenUrl {
            url: google_search_url(query),
            app: browser_hint(lower),
        });
    }
    None
}

fn route_folder(lower: &str) -> Option<FastIntent> {
    let special = [
        ("downloads", "~/Downloads"),
        ("download", "~/Downloads"),
        ("documents", "~/Documents"),
        ("desktop", "~/Desktop"),
        ("home", "~"),
        ("pictures", "~/Pictures"),
        ("music folder", "~/Music"),
        ("movies", "~/Movies"),
    ];
    for (needle, path) in special {
        if (lower.contains("open") || lower.contains("show") || lower.contains("go to"))
            && (lower.contains(needle) && (lower.contains("folder") || needle.ends_with("folder") || lower.contains("my ")))
        {
            return Some(FastIntent::OpenFolder { path: path.into() });
        }
        if lower == format!("open {needle}") || lower == format!("open my {needle}") {
            return Some(FastIntent::OpenFolder { path: path.into() });
        }
    }
    None
}

fn route_create_folder(lower: &str) -> Option<FastIntent> {
    let Some(rest) = after_any(lower, &["create a folder ", "create folder ", "make a folder ", "make folder "]) else {
        if lower == "create a folder" || lower == "create folder" {
            return Some(FastIntent::CreateFolder {
                path: "~/Downloads/New Folder".into(),
            });
        }
        return None;
    };
    let name = rest.trim().trim_matches('"');
    if name.is_empty() {
        return None;
    }
    Some(FastIntent::CreateFolder {
        path: format!("~/Downloads/{name}"),
    })
}

fn route_url(original: &str, lower: &str) -> Option<FastIntent> {
    if let Some(url) = extract_http_url(original) {
        return Some(FastIntent::OpenUrl {
            url,
            app: browser_hint(lower),
        });
    }
    if let Some(host) = bare_host(lower.strip_prefix("open ").unwrap_or(lower)) {
        return Some(FastIntent::OpenUrl {
            url: format!("https://{host}"),
            app: browser_hint(lower),
        });
    }
    None
}

fn route_open_application(lower: &str) -> Option<FastIntent> {
    let rest = after_any(lower, &["open ", "launch ", "start ", "run "])?;
    let name = rest
        .trim()
        .trim_start_matches("the ")
        .trim_start_matches("my ")
        .trim_start_matches("app ")
        .trim_end_matches(" app")
        .trim()
        .to_string();
    if name.is_empty() || looks_like_folder_word(&name) {
        return None;
    }
    if name.contains('.') && !name.contains(' ') {
        return None;
    }
    Some(FastIntent::OpenApplication { name })
}

fn is_complex(lower: &str) -> bool {
    let extra = [" then ", " after that", " click ", " type ", " first result"];
    extra.iter().any(|token| lower.contains(token))
        || (lower.contains(" and ")
            && contains_any(
                lower,
                &["play ", "search ", "find ", "click ", "type ", "scroll "],
            )
            && !is_browser_search(lower))
}

fn is_browser_search(lower: &str) -> bool {
    (lower.contains("safari") || lower.contains("chrome") || lower.contains("firefox") || lower.contains("browser"))
        && (lower.contains("search") || lower.contains("google") || lower.contains("youtube"))
}

fn browser_hint(lower: &str) -> Option<String> {
    if lower.contains("safari") {
        Some("Safari".into())
    } else if lower.contains("chrome") {
        Some("Google Chrome".into())
    } else if lower.contains("firefox") {
        Some("Firefox".into())
    } else if lower.contains("edge") {
        Some("Microsoft Edge".into())
    } else {
        None
    }
}

fn looks_like_play_music(lower: &str) -> bool {
    matches!(
        lower,
        "play music" | "play some music" | "start music" | "play apple music"
    )
}

fn looks_like_folder_word(name: &str) -> bool {
    matches!(
        name,
        "downloads" | "download" | "documents" | "desktop" | "folder" | "this file" | "this website"
    )
}

fn after_any<'a>(lower: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|prefix| lower.strip_prefix(prefix).or_else(|| {
        lower.find(prefix).map(|index| &lower[index + prefix.len()..])
    }))
}

fn extract_http_url(text: &str) -> Option<String> {
    text.split_whitespace().find_map(|word| {
        let clean = word.trim_matches(|ch: char| "()[],.\"'".contains(ch));
        if clean.starts_with("http://") || clean.starts_with("https://") {
            Some(clean.to_string())
        } else {
            None
        }
    })
}

fn bare_host(text: &str) -> Option<String> {
    let host = text.trim().trim_end_matches('/');
    if host.contains(' ') || !host.contains('.') {
        return None;
    }
    if host.starts_with("http") {
        return None;
    }
    Some(host.to_string())
}

fn google_search_url(query: &str) -> String {
    format!("https://www.google.com/search?q={}", url_encode(query.trim()))
}

fn youtube_search_url(query: &str) -> String {
    format!("https://www.youtube.com/results?search_query={}", url_encode(query.trim()))
}

fn url_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn normalize(text: &str) -> String {
    text.to_ascii_lowercase()
        .replace(['’', '`'], "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_apple_music() {
        match route("Open Apple Music").unwrap() {
            FastIntent::OpenApplication { name } => assert_eq!(name.to_ascii_lowercase(), "apple music"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn open_safari() {
        assert!(matches!(
            route("open safari"),
            Some(FastIntent::OpenApplication { .. })
        ));
    }

    #[test]
    fn open_youtube() {
        match route("Open YouTube").unwrap() {
            FastIntent::OpenUrl { url, .. } => assert!(url.contains("youtube.com")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn search_google_uses_url() {
        match route("Search Google for today's weather").unwrap() {
            FastIntent::OpenUrl { url, app } => {
                assert!(url.contains("google.com/search"));
                assert!(url.contains("weather"));
                assert!(app.is_none());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn safari_then_google() {
        match route("Open Safari and search Google for today's weather").unwrap() {
            FastIntent::OpenUrl { url, app } => {
                assert_eq!(app.as_deref(), Some("Safari"));
                assert!(url.contains("google.com/search"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn downloads_folder() {
        match route("Open my Downloads folder").unwrap() {
            FastIntent::OpenFolder { path } => assert_eq!(path, "~/Downloads"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn play_music() {
        assert!(matches!(
            route("Play music"),
            Some(FastIntent::OpenApplication { .. })
        ));
    }

    #[test]
    fn play_artist_is_media() {
        match route("Play Kumar Sanu").unwrap() {
            FastIntent::PlayMedia { query } => assert!(query.contains("kumar")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn play_workout_playlist() {
        match route("Play my workout playlist").unwrap() {
            FastIntent::PlayPlaylist { name } => assert!(name.contains("workout")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn open_music_and_play_playlist() {
        match route("Open Apple Music and play my workout playlist").unwrap() {
            FastIntent::PlayPlaylist { name } => assert!(name.contains("workout")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn alarm_is_reminder() {
        assert!(matches!(
            route("set an alarm for 7am"),
            Some(FastIntent::SetReminder { .. })
        ));
    }

    #[test]
    fn vague_help_is_detected() {
        assert!(is_vague("behave like a muse"));
        assert!(is_vague("what can you do"));
        assert!(!is_vague("set an alarm for 7am"));
    }

    #[test]
    fn complex_click_sequence_is_not_fast() {
        assert!(route("Open Safari then click the first result").is_none());
    }

    #[test]
    fn website_host() {
        match route("open google.com").unwrap() {
            FastIntent::OpenUrl { url, .. } => assert_eq!(url, "https://google.com"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn summarize_pdf_is_fast() {
        match route("summarize the pdf").unwrap() {
            FastIntent::SummarizeFile { query } => assert!(query.is_empty()),
            other => panic!("{other:?}"),
        }
        match route("summarize offer.pdf").unwrap() {
            FastIntent::SummarizeFile { query } => assert!(query.contains("offer")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn send_email_asks_then_sends() {
        match route("send email to ada@example.com about the recap").unwrap() {
            FastIntent::SendEmail { to, subject, .. } => {
                assert_eq!(to, "ada@example.com");
                assert!(subject.contains("recap"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn whats_on_my_screen_skips_planner() {
        match route("what's on my screen").unwrap() {
            FastIntent::AskScreen { question } => assert!(question.contains("screen")),
            other => panic!("{other:?}"),
        }
        match route("look at my screen").unwrap() {
            FastIntent::AskScreen { .. } => {}
            other => panic!("{other:?}"),
        }
        assert!(matches!(route("take a screenshot"), Some(FastIntent::TakeScreenshot)));
    }

    #[test]
    fn schedule_meeting_is_calendar() {
        match route("schedule a meeting standup tomorrow at 9am").unwrap() {
            FastIntent::CalendarEvent { title, .. } => assert!(title.contains("standup")),
            other => panic!("{other:?}"),
        }
    }
}
