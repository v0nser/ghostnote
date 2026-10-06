use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::error::{AgentError, AgentResult};
use super::net::parse_public_http_url;
use super::paths::jail;
use super::types::{RiskLevel, ToolDefinition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTool {
    WebSearch,
    WebFetch,
    BrowserOpen,
    BrowserClick,
    BrowserType,
    BrowserScroll,
    BrowserScreenshot,
    FilesList,
    FilesRead,
    FilesWrite,
    FilesSearch,
    FilesDelete,
    CodeExecute,
    ShellExecute,
    PythonExecute,
    Calculator,
    DateTime,
    MemorySearch,
    MemoryWrite,
    MemoryDelete,
    Notification,
    TaskSchedule,
    TaskCancel,
    OpenApplication,
    OpenUrl,
    OpenFolder,
    OpenFile,
    CreateFolder,
    ListApps,
    SetReminder,
    PlayMedia,
    FindFile,
    CreateDocument,
    DraftEmail,
    SendEmail,
    CalendarEvent,
    SummarizeFile,
    PlayPlaylist,
    CloseApplication,
    FocusApplication,
    TakeScreenshot,
}

impl BuiltinTool {
    pub fn all() -> &'static [BuiltinTool] {
        &[
            Self::WebSearch,
            Self::WebFetch,
            Self::BrowserOpen,
            Self::BrowserClick,
            Self::BrowserType,
            Self::BrowserScroll,
            Self::BrowserScreenshot,
            Self::FilesList,
            Self::FilesRead,
            Self::FilesWrite,
            Self::FilesSearch,
            Self::FilesDelete,
            Self::CodeExecute,
            Self::ShellExecute,
            Self::PythonExecute,
            Self::Calculator,
            Self::DateTime,
            Self::MemorySearch,
            Self::MemoryWrite,
            Self::MemoryDelete,
            Self::Notification,
            Self::TaskSchedule,
            Self::TaskCancel,
            Self::OpenApplication,
            Self::OpenUrl,
            Self::OpenFolder,
            Self::OpenFile,
            Self::CreateFolder,
            Self::ListApps,
            Self::SetReminder,
            Self::PlayMedia,
            Self::FindFile,
            Self::CreateDocument,
            Self::DraftEmail,
            Self::SendEmail,
            Self::CalendarEvent,
            Self::SummarizeFile,
            Self::PlayPlaylist,
            Self::CloseApplication,
            Self::FocusApplication,
            Self::TakeScreenshot,
        ]
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|tool| tool.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::WebSearch => "WEB_SEARCH",
            Self::WebFetch => "WEB_FETCH",
            Self::BrowserOpen => "BROWSER_OPEN",
            Self::BrowserClick => "BROWSER_CLICK",
            Self::BrowserType => "BROWSER_TYPE",
            Self::BrowserScroll => "BROWSER_SCROLL",
            Self::BrowserScreenshot => "BROWSER_SCREENSHOT",
            Self::FilesList => "FILES_LIST",
            Self::FilesRead => "FILES_READ",
            Self::FilesWrite => "FILES_WRITE",
            Self::FilesSearch => "FILES_SEARCH",
            Self::FilesDelete => "FILES_DELETE",
            Self::CodeExecute => "CODE_EXECUTE",
            Self::ShellExecute => "SHELL_EXECUTE",
            Self::PythonExecute => "PYTHON_EXECUTE",
            Self::Calculator => "CALCULATOR",
            Self::DateTime => "DATETIME",
            Self::MemorySearch => "MEMORY_SEARCH",
            Self::MemoryWrite => "MEMORY_WRITE",
            Self::MemoryDelete => "MEMORY_DELETE",
            Self::Notification => "NOTIFICATION",
            Self::TaskSchedule => "TASK_SCHEDULE",
            Self::TaskCancel => "TASK_CANCEL",
            Self::OpenApplication => "OPEN_APPLICATION",
            Self::OpenUrl => "OPEN_URL",
            Self::OpenFolder => "OPEN_FOLDER",
            Self::OpenFile => "OPEN_FILE",
            Self::CreateFolder => "CREATE_FOLDER",
            Self::ListApps => "LIST_APPS",
            Self::SetReminder => "SET_REMINDER",
            Self::PlayMedia => "PLAY_MEDIA",
            Self::FindFile => "FIND_FILE",
            Self::CreateDocument => "CREATE_DOCUMENT",
            Self::DraftEmail => "DRAFT_EMAIL",
            Self::SendEmail => "SEND_EMAIL",
            Self::CalendarEvent => "CALENDAR_EVENT",
            Self::SummarizeFile => "SUMMARIZE_FILE",
            Self::PlayPlaylist => "PLAY_PLAYLIST",
            Self::CloseApplication => "CLOSE_APPLICATION",
            Self::FocusApplication => "FOCUS_APPLICATION",
            Self::TakeScreenshot => "TAKE_SCREENSHOT",
        }
    }

    pub fn risk(self) -> RiskLevel {
        super::policy::risk_for_tool(self.name())
    }

    pub fn definition(self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().into(),
            description: self.description().into(),
            input_schema: json!({"type": "object"}),
            output_schema: json!({"type": "object"}),
            risk_level: self.risk(),
            permissions_required: vec![self.name().into()],
            requires_confirmation: matches!(
                self.risk(),
                RiskLevel::High | RiskLevel::Critical
            ),
            timeout_ms: 30_000,
            supports_cancellation: true,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::WebSearch => "Search the public web. Returns titles, URLs, and snippets.",
            Self::WebFetch => "Fetch a public https page and extract readable text.",
            Self::BrowserOpen => "Open a URL in the isolated browser context and extract content.",
            Self::BrowserClick => "Click in the isolated browser (Phase 6).",
            Self::BrowserType => "Type in the isolated browser (Phase 6).",
            Self::BrowserScroll => "Scroll the isolated browser (Phase 6).",
            Self::BrowserScreenshot => "Screenshot the isolated browser (Phase 6).",
            Self::FilesList => "List files in the agent workspace.",
            Self::FilesRead => "Read a text file in the agent workspace.",
            Self::FilesWrite => "Write a text file in the agent workspace.",
            Self::FilesSearch => "Search file contents in the agent workspace.",
            Self::FilesDelete => "Delete a workspace file (always needs approval).",
            Self::CodeExecute => "Run a short Node snippet in the sandbox.",
            Self::ShellExecute => "Run an allowlisted workspace command. Not a raw shell.",
            Self::PythonExecute => "Run a short Python snippet in the sandbox.",
            Self::Calculator => "Evaluate a basic arithmetic expression.",
            Self::DateTime => "Current UTC unix time and ISO-ish stamp.",
            Self::MemorySearch => "Search local memory.",
            Self::MemoryWrite => "Store a memory.",
            Self::MemoryDelete => "Forget a memory.",
            Self::Notification => "Show a local notification in the app.",
            Self::TaskSchedule => "Schedule a follow-up goal.",
            Self::TaskCancel => "Cancel a scheduled task.",
            Self::OpenApplication => "Launch an installed desktop application and verify it is running.",
            Self::OpenUrl => "Open a URL in the default browser, or a named browser.",
            Self::OpenFolder => "Reveal a folder in Finder or Explorer.",
            Self::OpenFile => "Open a file with the system default app.",
            Self::CreateFolder => "Create a folder and open it.",
            Self::ListApps => "List visible running applications.",
            Self::SetReminder => "Create a Reminders alarm or reminder at a time.",
            Self::PlayMedia => "Search YouTube and open it to play a song or artist.",
            Self::FindFile => "Find a file in Downloads, Documents, or Desktop and open it.",
            Self::CreateDocument => "Create a markdown document and open it.",
            Self::DraftEmail => "Open a draft email for the user to review and send.",
            Self::SendEmail => "Send an email after the user approves. Uses Mail on Mac, Outlook on Windows.",
            Self::CalendarEvent => "Create a Calendar event after approval.",
            Self::SummarizeFile => "Read a local PDF or text file and prepare it for a local summary.",
            Self::PlayPlaylist => "Open Apple Music and play a named playlist.",
            Self::CloseApplication => "Quit an application.",
            Self::FocusApplication => "Bring an application to the front.",
            Self::TakeScreenshot => "Capture the screen to a PNG file.",
        }
    }
}

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn definitions() -> Vec<ToolDefinition> {
        BuiltinTool::all().iter().map(|tool| tool.definition()).collect()
    }
}

pub struct ToolOutput {
    pub value: Value,
    pub summary: String,
}

pub struct ToolExec<'a> {
    pub workspace: &'a Path,
    pub sandbox: &'a Path,
    pub http: &'a reqwest::Client,
}

pub async fn execute(tool: BuiltinTool, args: &Value, ctx: &ToolExec<'_>) -> AgentResult<ToolOutput> {
    match tool {
        BuiltinTool::WebSearch => web_search(ctx.http, str_arg(args, "query")?).await,
        BuiltinTool::WebFetch | BuiltinTool::BrowserOpen => {
            web_fetch(ctx.http, str_arg(args, "url")?).await
        }
        BuiltinTool::BrowserClick
        | BuiltinTool::BrowserType
        | BuiltinTool::BrowserScroll
        | BuiltinTool::BrowserScreenshot => Err(AgentError::msg(
            "interactive browser control ships in Phase 6 (Playwright sidecar)",
        )),
        BuiltinTool::FilesList => files_list(ctx.workspace, args.get("path").and_then(Value::as_str).unwrap_or("")),
        BuiltinTool::FilesRead => files_read(ctx.workspace, str_arg(args, "path")?),
        BuiltinTool::FilesWrite => files_write(
            ctx.workspace,
            str_arg(args, "path")?,
            str_arg(args, "content")?,
        ),
        BuiltinTool::FilesSearch => files_search(ctx.workspace, str_arg(args, "query")?),
        BuiltinTool::FilesDelete => files_delete(ctx.workspace, str_arg(args, "path")?),
        BuiltinTool::Calculator => calculator(str_arg(args, "expression")?),
        BuiltinTool::DateTime => Ok(ToolOutput {
            value: json!({"unixMs": super::ids::now_ms()}),
            summary: format!("Current time ms {}", super::ids::now_ms()),
        }),
        BuiltinTool::CodeExecute => sandbox_run(ctx.sandbox, "node", str_arg(args, "code")?),
        BuiltinTool::PythonExecute => sandbox_run(ctx.sandbox, "python", str_arg(args, "code")?),
        BuiltinTool::ShellExecute => {
            Err(AgentError::msg("raw shell is disabled; use PYTHON_EXECUTE or CODE_EXECUTE"))
        }
        BuiltinTool::MemorySearch
        | BuiltinTool::MemoryWrite
        | BuiltinTool::MemoryDelete
        | BuiltinTool::Notification
        | BuiltinTool::TaskSchedule
        | BuiltinTool::TaskCancel => Ok(ToolOutput {
            value: json!({"deferred": true, "tool": tool.name()}),
            summary: format!("{} handled by the orchestrator", tool.name()),
        }),
        BuiltinTool::OpenApplication
        | BuiltinTool::OpenUrl
        | BuiltinTool::OpenFolder
        | BuiltinTool::OpenFile
        | BuiltinTool::CreateFolder
        | BuiltinTool::ListApps => {
            tokio::task::spawn_blocking({
                let tool_name = tool.name().to_string();
                let args = args.clone();
                move || super::computer::execute(&tool_name, &args)
            })
            .await
            .map_err(|err| AgentError::msg(format!("computer tool worker failed: {err}")))?
        }
        BuiltinTool::PlayPlaylist
        | BuiltinTool::CloseApplication
        | BuiltinTool::FocusApplication
        | BuiltinTool::TakeScreenshot => {
            tokio::task::spawn_blocking({
                let tool_name = tool.name().to_string();
                let args = args.clone();
                move || super::computer::execute(&tool_name, &args)
            })
            .await
            .map_err(|err| AgentError::msg(format!("computer tool worker failed: {err}")))?
        }
        BuiltinTool::SetReminder
        | BuiltinTool::PlayMedia
        | BuiltinTool::FindFile
        | BuiltinTool::CreateDocument
        |         BuiltinTool::DraftEmail
        | BuiltinTool::SendEmail
        | BuiltinTool::CalendarEvent
        | BuiltinTool::SummarizeFile => {
            tokio::task::spawn_blocking({
                let tool_name = tool.name().to_string();
                let args = args.clone();
                move || super::skills::execute(&tool_name, &args)
            })
            .await
            .map_err(|err| AgentError::msg(format!("skill worker failed: {err}")))?
        }
    }
}

fn str_arg<'a>(args: &'a Value, key: &'a str) -> AgentResult<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| AgentError::msg(format!("missing {key}")))
}

async fn web_search(http: &reqwest::Client, query: &str) -> AgentResult<ToolOutput> {
    let encoded = query_encode(query);
    let mut results = ddg_instant(http, &encoded).await.unwrap_or_default();
    if results.len() < 3 {
        let url = format!("https://html.duckduckgo.com/html/?q={encoded}");
        if let Ok(body) = fetch_text(http, &url).await {
            for item in extract_ddg(&body) {
                if !results.iter().any(|existing| existing["url"] == item["url"]) {
                    results.push(item);
                }
                if results.len() >= 8 {
                    break;
                }
            }
        }
    }
    if results.is_empty() {
        if let Ok(wiki) = wikipedia_search(http, &encoded).await {
            results.extend(wiki);
        }
    }
    let summary = if results.is_empty() {
        format!("No public results for “{query}”.")
    } else {
        let mut lines = format!("Found {} sources for “{query}”:", results.len());
        for item in results.iter().take(5) {
            let title = item.get("title").and_then(Value::as_str).unwrap_or("source");
            let url = item.get("url").and_then(Value::as_str).unwrap_or("");
            let snippet = item.get("snippet").and_then(Value::as_str).unwrap_or("");
            lines.push_str(&format!("\n- {title} ({url})"));
            if !snippet.is_empty() {
                lines.push_str(&format!(" — {snippet}"));
            }
        }
        lines
    };
    Ok(ToolOutput {
        value: json!({ "results": results }),
        summary,
    })
}

fn query_encode(query: &str) -> String {
    query
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.' {
                vec![byte]
            } else if byte == b' ' {
                vec![b'+']
            } else {
                format!("%{byte:02X}").into_bytes()
            }
        })
        .map(|b| b as char)
        .collect()
}

async fn ddg_instant(http: &reqwest::Client, encoded: &str) -> AgentResult<Vec<Value>> {
    let url = format!(
        "https://api.duckduckgo.com/?q={encoded}&format=json&no_html=1&no_redirect=1&skip_disambig=1"
    );
    let body = fetch_text(http, &url).await?;
    let value: Value = serde_json::from_str(&body).map_err(|err| AgentError::msg(err.to_string()))?;
    let mut results = Vec::new();
    let abstract_text = value.get("AbstractText").and_then(Value::as_str).unwrap_or("");
    let abstract_url = value.get("AbstractURL").and_then(Value::as_str).unwrap_or("");
    let heading = value.get("Heading").and_then(Value::as_str).unwrap_or("DuckDuckGo");
    if !abstract_text.is_empty() {
        results.push(json!({
            "title": heading,
            "url": abstract_url,
            "snippet": abstract_text
        }));
    }
    let answer = value.get("Answer").and_then(Value::as_str).unwrap_or("");
    if !answer.is_empty() {
        results.push(json!({
            "title": value.get("AnswerType").and_then(Value::as_str).unwrap_or("Answer"),
            "url": abstract_url,
            "snippet": answer
        }));
    }
    push_related(&value, &mut results);
    Ok(results)
}

fn push_related(value: &Value, results: &mut Vec<Value>) {
    let Some(topics) = value.get("RelatedTopics").and_then(Value::as_array) else {
        return;
    };
    for topic in topics {
        if results.len() >= 8 {
            return;
        }
        if let Some(nested) = topic.get("Topics").and_then(Value::as_array) {
            for child in nested {
                push_topic(child, results);
                if results.len() >= 8 {
                    return;
                }
            }
            continue;
        }
        push_topic(topic, results);
    }
}

async fn wikipedia_search(http: &reqwest::Client, encoded: &str) -> AgentResult<Vec<Value>> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=opensearch&search={encoded}&limit=5&namespace=0&format=json"
    );
    let body = fetch_text(http, &url).await?;
    let value: Value = serde_json::from_str(&body).map_err(|err| AgentError::msg(err.to_string()))?;
    let titles = value.get(1).and_then(Value::as_array).cloned().unwrap_or_default();
    let snippets = value.get(2).and_then(Value::as_array).cloned().unwrap_or_default();
    let urls = value.get(3).and_then(Value::as_array).cloned().unwrap_or_default();
    let mut results = Vec::new();
    for index in 0..titles.len() {
        let title = titles[index].as_str().unwrap_or("");
        let url = urls.get(index).and_then(Value::as_str).unwrap_or("");
        if title.is_empty() || url.is_empty() {
            continue;
        }
        results.push(json!({
            "title": title,
            "url": url,
            "snippet": snippets.get(index).and_then(Value::as_str).unwrap_or("")
        }));
    }
    Ok(results)
}

fn push_topic(topic: &Value, results: &mut Vec<Value>) {
    let text = topic.get("Text").and_then(Value::as_str).unwrap_or("");
    let url = topic.get("FirstURL").and_then(Value::as_str).unwrap_or("");
    if text.is_empty() || url.is_empty() {
        return;
    }
    results.push(json!({
        "title": text.chars().take(80).collect::<String>(),
        "url": url,
        "snippet": text
    }));
}

async fn web_fetch(http: &reqwest::Client, url: &str) -> AgentResult<ToolOutput> {
    let safe = parse_public_http_url(url)?;
    let body = fetch_text(http, &safe.raw).await?;
    let text = extract_text(&body);
    let clipped: String = text.chars().take(6_000).collect();
    Ok(ToolOutput {
        value: json!({ "url": safe.raw, "host": safe.host, "text": clipped }),
        summary: format!("Read {} ({} chars).", safe.host, clipped.len()),
    })
}

async fn fetch_text(http: &reqwest::Client, url: &str) -> AgentResult<String> {
    let response = http
        .get(url)
        .timeout(Duration::from_secs(15))
        .header(
            "user-agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
        )
        .header("accept", "text/html,application/json;q=0.9,*/*;q=0.8")
        .send()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    if !response.status().is_success() {
        return Err(AgentError::msg(format!("http {}", response.status())));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    if bytes.len() > 1_500_000 {
        return Err(AgentError::msg("response too large"));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn extract_ddg(html: &str) -> Vec<Value> {
    let mut results = Vec::new();
    for chunk in html.split("result__a") {
        let href = attr_after(chunk, "href=\"").or_else(|| attr_after(chunk, "href='"));
        let title = between(chunk, ">", "</a>").map(strip_tags);
        let snippet = between(chunk, "result__snippet", "</")
            .map(strip_tags)
            .unwrap_or_default();
        if let (Some(href), Some(title)) = (href, title) {
            if let Some(url) = decode_result_url(&href) {
                if !title.is_empty() {
                    results.push(json!({ "title": title, "url": url, "snippet": snippet }));
                }
            }
        }
        if results.len() >= 8 {
            break;
        }
    }
    results
}

fn decode_result_url(raw: &str) -> Option<String> {
    let raw = html_unescape(raw);
    let raw = raw
        .strip_prefix("//")
        .map(|rest| format!("https://{rest}"))
        .unwrap_or(raw);
    if let Some(encoded) = raw.split("uddg=").nth(1) {
        let encoded = encoded.split('&').next().unwrap_or(encoded);
        let decoded = percent_decode(encoded);
        if decoded.starts_with("http://") || decoded.starts_with("https://") {
            return Some(decoded);
        }
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        return Some(raw);
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (from_hex(bytes[index + 1]), from_hex(bytes[index + 2])) {
                out.push(high * 16 + low);
                index += 3;
                continue;
            }
        }
        out.push(if bytes[index] == b'+' { b' ' } else { bytes[index] });
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn attr_after<'a>(hay: &'a str, prefix: &str) -> Option<String> {
    let start = hay.find(prefix)? + prefix.len();
    let rest = &hay[start..];
    let end = rest.find(['"', '\'', ' '])?;
    Some(rest[..end].to_string())
}

fn between<'a>(hay: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = hay.find(start)? + start.len();
    let rest = &hay[from..];
    let to = rest.find(end)?;
    Some(&rest[..to])
}

fn strip_tags(raw: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    collapse_ws(&html_unescape(&out))
}

fn extract_text(html: &str) -> String {
    let mut cut = html.to_string();
    for tag in ["script", "style", "noscript"] {
        while let Some(start) = cut.to_ascii_lowercase().find(&format!("<{tag}")) {
            let after = &cut[start..];
            if let Some(end_rel) = after.to_ascii_lowercase().find(&format!("</{tag}>")) {
                let end = start + end_rel + tag.len() + 3;
                cut.replace_range(start..end.min(cut.len()), " ");
            } else {
                break;
            }
        }
    }
    strip_tags(&cut)
}

fn html_unescape(raw: &str) -> String {
    raw.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn collapse_ws(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn files_list(root: &Path, rel: &str) -> AgentResult<ToolOutput> {
    let path = jail(root, rel)?;
    let mut names = Vec::new();
    if path.is_dir() {
        for entry in std::fs::read_dir(&path).map_err(|err| AgentError::msg(err.to_string()))? {
            let entry = entry.map_err(|err| AgentError::msg(err.to_string()))?;
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(ToolOutput {
        value: json!({ "entries": names }),
        summary: format!("Listed {} items.", names.len()),
    })
}

fn files_read(root: &Path, rel: &str) -> AgentResult<ToolOutput> {
    let path = jail(root, rel)?;
    let bytes = std::fs::read(&path).map_err(|err| AgentError::msg(err.to_string()))?;
    if bytes.len() > 200_000 {
        return Err(AgentError::msg("file too large"));
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(ToolOutput {
        value: json!({ "path": rel, "text": text }),
        summary: format!("Read {rel} ({} chars).", text.len()),
    })
}

fn files_write(root: &Path, rel: &str, content: &str) -> AgentResult<ToolOutput> {
    let path = jail(root, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| AgentError::msg(err.to_string()))?;
    }
    std::fs::write(&path, content).map_err(|err| AgentError::msg(err.to_string()))?;
    Ok(ToolOutput {
        value: json!({ "path": rel, "bytes": content.len() }),
        summary: format!("Wrote {rel}."),
    })
}

fn files_search(root: &Path, query: &str) -> AgentResult<ToolOutput> {
    let mut hits = Vec::new();
    visit(root, root, query, &mut hits);
    Ok(ToolOutput {
        value: json!({ "hits": hits }),
        summary: format!("Found {} matching files.", hits.len()),
    })
}

fn visit(root: &Path, dir: &Path, query: &str, hits: &mut Vec<Value>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let needle = query.to_ascii_lowercase();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(root, &path, query, hits);
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let name = rel.to_string_lossy().to_ascii_lowercase();
        let mut matched = name.contains(&needle);
        if !matched {
            if let Ok(bytes) = std::fs::read(&path) {
                if bytes.len() < 80_000 {
                    matched = String::from_utf8_lossy(&bytes)
                        .to_ascii_lowercase()
                        .contains(&needle);
                }
            }
        }
        if matched {
            hits.push(json!({ "path": rel.to_string_lossy() }));
        }
        if hits.len() >= 20 {
            return;
        }
    }
}

fn files_delete(root: &Path, rel: &str) -> AgentResult<ToolOutput> {
    let path = jail(root, rel)?;
    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|err| AgentError::msg(err.to_string()))?;
    } else {
        std::fs::remove_file(&path).map_err(|err| AgentError::msg(err.to_string()))?;
    }
    Ok(ToolOutput {
        value: json!({ "deleted": rel }),
        summary: format!("Deleted {rel}."),
    })
}

fn calculator(expr: &str) -> AgentResult<ToolOutput> {
    let value = eval_arith(expr).ok_or_else(|| AgentError::msg("could not evaluate expression"))?;
    Ok(ToolOutput {
        value: json!({ "value": value }),
        summary: format!("{expr} = {value}"),
    })
}

pub fn eval_arith_for_plan(expr: &str) -> Option<f64> {
    eval_arith(expr)
}

fn eval_arith(expr: &str) -> Option<f64> {
    let tokens = tokenize(expr)?;
    let (value, rest) = parse_expr(&tokens, 0)?;
    if rest != tokens.len() {
        return None;
    }
    Some(value)
}

fn tokenize(expr: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for ch in expr.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
            continue;
        }
        if !num.is_empty() {
            tokens.push(std::mem::take(&mut num));
        }
        if ch.is_whitespace() {
            continue;
        }
        if matches!(ch, '+' | '-' | '*' | '/' | '(' | ')') {
            tokens.push(ch.to_string());
        } else {
            return None;
        }
    }
    if !num.is_empty() {
        tokens.push(num);
    }
    Some(tokens)
}

fn parse_expr(tokens: &[String], mut i: usize) -> Option<(f64, usize)> {
    let (mut value, next) = parse_term(tokens, i)?;
    i = next;
    while i < tokens.len() && (tokens[i] == "+" || tokens[i] == "-") {
        let op = &tokens[i];
        let (rhs, next) = parse_term(tokens, i + 1)?;
        value = if op == "+" { value + rhs } else { value - rhs };
        i = next;
    }
    Some((value, i))
}

fn parse_term(tokens: &[String], mut i: usize) -> Option<(f64, usize)> {
    let (mut value, next) = parse_factor(tokens, i)?;
    i = next;
    while i < tokens.len() && (tokens[i] == "*" || tokens[i] == "/") {
        let op = &tokens[i];
        let (rhs, next) = parse_factor(tokens, i + 1)?;
        value = if op == "*" { value * rhs } else { value / rhs };
        i = next;
    }
    Some((value, i))
}

fn parse_factor(tokens: &[String], i: usize) -> Option<(f64, usize)> {
    let token = tokens.get(i)?;
    if token == "(" {
        let (value, next) = parse_expr(tokens, i + 1)?;
        if tokens.get(next)? != ")" {
            return None;
        }
        return Some((value, next + 1));
    }
    if token == "-" {
        let (value, next) = parse_factor(tokens, i + 1)?;
        return Some((-value, next));
    }
    Some((token.parse().ok()?, i + 1))
}

fn sandbox_run(sandbox: &Path, runtime: &str, code: &str) -> AgentResult<ToolOutput> {
    std::fs::create_dir_all(sandbox).map_err(|err| AgentError::msg(err.to_string()))?;
    let (binaries, ext) = match runtime {
        "node" => (&["node", "nodejs"][..], "js"),
        _ => (&["python3", "python"][..], "py"),
    };
    let file = sandbox.join(format!("main.{ext}"));
    std::fs::write(&file, code).map_err(|err| AgentError::msg(err.to_string()))?;

    let mut last_err = AgentError::msg("runtime not installed");
    for bin in binaries {
        let started = Instant::now();
        let output = Command::new(bin)
            .arg(&file)
            .current_dir(sandbox)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("SYSTEMROOT", std::env::var("SYSTEMROOT").unwrap_or_default())
            .env("TEMP", std::env::temp_dir())
            .output();
        match output {
            Ok(output) => {
                if started.elapsed() > Duration::from_secs(20) {
                    return Err(AgentError::msg("sandbox timeout"));
                }
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                if !output.stderr.is_empty() {
                    text.push('\n');
                    text.push_str(&String::from_utf8_lossy(&output.stderr));
                }
                if text.len() > 8_000 {
                    text.truncate(8_000);
                }
                let _ = std::fs::remove_dir_all(sandbox);
                return Ok(ToolOutput {
                    value: json!({ "stdout": text, "ok": output.status.success() }),
                    summary: format!("Ran {runtime} in the sandbox."),
                });
            }
            Err(err) => last_err = AgentError::msg(err.to_string()),
        }
    }
    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_order() {
        let out = calculator("2+3*4").unwrap();
        assert_eq!(out.value["value"], 14.0);
    }

    #[test]
    fn registry_lists_expected_tools() {
        let names: Vec<_> = ToolRegistry::definitions().into_iter().map(|d| d.name).collect();
        assert!(names.contains(&"WEB_SEARCH".into()));
        assert!(names.contains(&"SHELL_EXECUTE".into()));
    }

    #[test]
    fn decodes_duckduckgo_redirect() {
        let url = decode_result_url(
            "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fjobs&rut=abc",
        );
        assert_eq!(url.as_deref(), Some("https://example.com/jobs"));
    }
}
