//! Everyday agent skills. These run inside Coda — the model only names
//! the skill and arguments. Results are verified before the user is told.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use super::computer;
use super::error::{AgentError, AgentResult};
use super::tools::ToolOutput;

pub fn execute(tool: &str, args: &Value) -> AgentResult<ToolOutput> {
    match tool {
        "SET_REMINDER" | "SET_ALARM" => set_reminder(args),
        "PLAY_MEDIA" => play_media(str_arg(args, "query")?),
        "FIND_FILE" => find_file(str_arg(args, "query")?),
        "CREATE_DOCUMENT" => create_document(
            args.get("title").and_then(Value::as_str).unwrap_or("Note"),
            args.get("content").and_then(Value::as_str).unwrap_or(""),
        ),
        "DRAFT_EMAIL" => draft_email(args),
        "SEND_EMAIL" => send_email(args),
        "CALENDAR_EVENT" => calendar_event(args),
        "SUMMARIZE_FILE" => summarize_file(args),
        _ => Err(AgentError::msg(format!("{tool} is not a skill"))),
    }
}

fn set_reminder(args: &Value) -> AgentResult<ToolOutput> {
    let title = args
        .get("title")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or("Coda reminder");
    let when = parse_when(
        args.get("when")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim(),
    )?;
    let apple = apple_date(when)?;
    let created = if cfg!(target_os = "macos") {
        remind_macos(title, &apple)?
    } else if cfg!(target_os = "windows") {
        remind_windows(title, when)?
    } else {
        false
    };
    if !created {
        return Err(AgentError::msg(
            "I could not create that reminder. Grant Reminders access in System Settings if you are on a Mac.",
        ));
    }
    Ok(ToolOutput {
        summary: format!("Reminder set: {title} at {apple}."),
        value: json!({
            "success": true,
            "tool": "set_reminder",
            "data": { "title": title, "when": apple },
            "error": null
        }),
    })
}

fn play_media(query: &str) -> AgentResult<ToolOutput> {
    let query = query.trim();
    if query.is_empty() {
        return Err(AgentError::msg("Say what to play."));
    }
    let url = format!(
        "https://www.youtube.com/results?search_query={}",
        url_encode(query)
    );
    computer::execute("OPEN_URL", &json!({ "url": url }))?;
    Ok(ToolOutput {
        summary: format!("Playing search for “{query}” on YouTube."),
        value: json!({
            "success": true,
            "tool": "play_media",
            "data": { "query": query, "url": url },
            "error": null
        }),
    })
}

fn find_file(query: &str) -> AgentResult<ToolOutput> {
    let query = query.trim().trim_matches('"');
    if query.is_empty() {
        return Err(AgentError::msg("Say which file to open."));
    }
    let hits = search_files(query);
    if hits.is_empty() {
        return Err(AgentError::msg(format!(
            "I could not find a file matching “{query}” in Downloads, Documents, Desktop, or Home."
        )));
    }
    let path = hits[0].clone();
    computer::execute("OPEN_FILE", &json!({ "path": path.to_string_lossy() }))?;
    Ok(ToolOutput {
        summary: if hits.len() == 1 {
            format!("Opened {}.", path.display())
        } else {
            format!(
                "Opened {}. {} other matches.",
                path.display(),
                hits.len() - 1
            )
        },
        value: json!({
            "success": true,
            "tool": "find_file",
            "data": {
                "opened": path,
                "matches": hits.iter().take(8).map(|p| p.display().to_string()).collect::<Vec<_>>()
            },
            "error": null
        }),
    })
}

fn create_document(title: &str, content: &str) -> AgentResult<ToolOutput> {
    let dir = documents_dir().join("Coda");
    std::fs::create_dir_all(&dir)
        .map_err(|err| AgentError::msg(format!("Could not create Documents/Coda: {err}")))?;
    let stem = sanitize_name(title);
    let path = dir.join(format!("{stem}.md"));
    let body = if content.trim().is_empty() {
        format!("# {title}\n\n")
    } else {
        content.to_string()
    };
    std::fs::write(&path, body)
        .map_err(|err| AgentError::msg(format!("Could not write {}: {err}", path.display())))?;
    computer::execute("OPEN_FILE", &json!({ "path": path.to_string_lossy() }))?;
    Ok(ToolOutput {
        summary: format!("Created and opened {}.", path.display()),
        value: json!({
            "success": true,
            "tool": "create_document",
            "data": { "path": path },
            "error": null
        }),
    })
}

fn draft_email(args: &Value) -> AgentResult<ToolOutput> {
    let to = args.get("to").and_then(Value::as_str).unwrap_or("");
    let subject = args.get("subject").and_then(Value::as_str).unwrap_or("");
    let body = args.get("body").and_then(Value::as_str).unwrap_or("");
    let mut mailto = format!("mailto:{to}?subject={}&body={}", url_encode(subject), url_encode(body));
    if to.is_empty() {
        mailto = format!("mailto:?subject={}&body={}", url_encode(subject), url_encode(body));
    }
    computer::execute("OPEN_URL", &json!({ "url": mailto }))?;
    Ok(ToolOutput {
        summary: "Opened a draft email for you to review and send.".into(),
        value: json!({
            "success": true,
            "tool": "draft_email",
            "data": { "to": to, "subject": subject },
            "error": null
        }),
    })
}

fn send_email(args: &Value) -> AgentResult<ToolOutput> {
    let to = args.get("to").and_then(Value::as_str).unwrap_or("").trim();
    let subject = args
        .get("subject")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or("Coda");
    let body = args.get("body").and_then(Value::as_str).unwrap_or("");
    if to.is_empty() || !to.contains('@') {
        return Err(AgentError::msg(
            "Say who to email, including an address. Example: send email to name@mail.com about the recap.",
        ));
    }
    if cfg!(target_os = "macos") {
        let script = format!(
            r#"tell application "Mail"
                set theMessage to make new outgoing message with properties {{subject:"{}", content:"{}", visible:false}}
                tell theMessage
                    make new to recipient at end of to recipients with properties {{address:"{}"}}
                    send
                end tell
            end tell"#,
            escape_as(subject),
            escape_as(body),
            escape_as(to)
        );
        let ok = osascript(&script)?;
        if !ok {
            return Err(AgentError::msg(
                "Mail did not send. Grant Coda control of Mail in System Settings → Privacy & Security → Automation, then approve again.",
            ));
        }
        return Ok(ToolOutput {
            summary: format!("Sent email to {to} via Mail."),
            value: json!({
                "success": true,
                "tool": "send_email",
                "data": { "to": to, "subject": subject },
                "error": null
            }),
        });
    }
    if cfg!(target_os = "windows") {
        if outlook_send(to, subject, body) {
            return Ok(ToolOutput {
                summary: format!("Sent email to {to} via Outlook."),
                value: json!({
                    "success": true,
                    "tool": "send_email",
                    "data": { "to": to, "subject": subject },
                    "error": null
                }),
            });
        }
    }
    let _ = draft_email(args);
    Ok(ToolOutput {
        summary: format!(
            "I could not send from Mail or Outlook. I opened a compose window to {to} — press Send there."
        ),
        value: json!({
            "success": false,
            "tool": "send_email",
            "data": { "to": to, "subject": subject },
            "error": "no local mail sender"
        }),
    })
}

fn outlook_send(to: &str, subject: &str, body: &str) -> bool {
    let script = format!(
        "$o = New-Object -ComObject Outlook.Application; $m = $o.CreateItem(0); $m.To = '{}'; $m.Subject = '{}'; $m.Body = '{}'; $m.Send()",
        powershell_escape(to),
        powershell_escape(subject),
        powershell_escape(body)
    );
    Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn powershell_escape(text: &str) -> String {
    text.replace('\'', "''")
}

fn summarize_file(args: &Value) -> AgentResult<ToolOutput> {
    let query = args
        .get("query")
        .or_else(|| args.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let (path, text) = super::documents::load_for_summary(query)?;
    Ok(ToolOutput {
        summary: format!("Read {} ({} characters).", path.display(), text.chars().count()),
        value: json!({
            "success": true,
            "tool": "summarize_file",
            "path": path,
            "text": text,
            "error": null
        }),
    })
}

fn calendar_event(args: &Value) -> AgentResult<ToolOutput> {
    let title = args
        .get("title")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or("Coda event");
    let when = parse_when(args.get("when").and_then(Value::as_str).unwrap_or(""))?;
    let apple = apple_date(when)?;
    if cfg!(target_os = "macos") {
        let script = format!(
            r#"tell application "Calendar"
                tell calendar "Calendar"
                    make new event with properties {{summary:"{}", start date:date "{}", end date:date "{}" + 1 * hours}}
                end tell
                activate
            end tell"#,
            escape_as(title),
            apple,
            apple
        );
        let ok = osascript(&script)?;
        if !ok {
            return Err(AgentError::msg(
                "Could not add the Calendar event. Grant Calendar access in System Settings.",
            ));
        }
    } else {
        return set_reminder(args);
    }
    Ok(ToolOutput {
        summary: format!("Calendar event created: {title} at {apple}."),
        value: json!({
            "success": true,
            "tool": "calendar_event",
            "data": { "title": title, "when": apple },
            "error": null
        }),
    })
}

fn remind_macos(title: &str, apple_date: &str) -> AgentResult<bool> {
    let script = format!(
        r#"tell application "Reminders"
            tell list "Reminders"
                make new reminder with properties {{name:"{}", remind me date:date "{}"}}
            end tell
            activate
        end tell"#,
        escape_as(title),
        apple_date
    );
    osascript(&script)
}

fn remind_windows(title: &str, when: SystemTime) -> AgentResult<bool> {
    let secs = when
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AgentError::msg(err.to_string()))?
        .as_secs();
    let stamp = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "[DateTimeOffset]::FromUnixTimeSeconds({secs}).LocalDateTime.ToString('HH:mm')"
            ),
        ])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|| "09:00".into());
    let status = Command::new("schtasks")
        .args([
            "/Create",
            "/SC",
            "ONCE",
            "/ST",
            &stamp,
            "/TN",
            &format!("Coda\\{title}"),
            "/TR",
            &format!("msg * {title}"),
            "/F",
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    Ok(status)
}

pub(crate) fn search_files(query: &str) -> Vec<PathBuf> {
    let mut hits = Vec::new();
    if cfg!(target_os = "macos") {
        let mut cmd = Command::new("mdfind");
        for root in search_roots() {
            cmd.arg("-onlyin").arg(root);
        }
        cmd.arg(format!("kMDItemDisplayName == '*{query}*'cd"));
        if let Ok(output) = cmd.output() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let path = PathBuf::from(line.trim());
                if path.is_file() && !super::paths::is_sensitive(&path) {
                    hits.push(path);
                }
                if hits.len() >= 12 {
                    break;
                }
            }
        }
    }
    if hits.is_empty() {
        for root in search_roots() {
            walk_named(&root, query, &mut hits, 0);
            if hits.len() >= 12 {
                break;
            }
        }
    }
    hits
}

fn walk_named(dir: &Path, query: &str, hits: &mut Vec<PathBuf>, depth: u8) {
    if depth > 3 || hits.len() >= 12 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let needle = query.to_ascii_lowercase();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            walk_named(&path, query, hits, depth + 1);
        } else if name.contains(&needle) && !super::paths::is_sensitive(&path) {
            hits.push(path);
        }
    }
}

pub(crate) fn search_roots() -> Vec<PathBuf> {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    vec![
        home.join("Downloads"),
        home.join("Documents"),
        home.join("Desktop"),
        home.clone(),
    ]
}

fn documents_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Documents")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn parse_when(text: &str) -> AgentResult<SystemTime> {
    let now = SystemTime::now();
    let lower = text.to_ascii_lowercase();
    if lower.is_empty() || lower.contains("later") && !lower.contains("minute") {
        return Ok(now + Duration::from_secs(60 * 60));
    }
    if let Some(mins) = minutes_from_now(&lower) {
        return Ok(now + Duration::from_secs(mins * 60));
    }
    if let Some((hour, minute, pm)) = clock_time(&lower) {
        return Ok(next_clock(now, hour, minute, pm));
    }
    Ok(now + Duration::from_secs(60 * 60))
}

fn minutes_from_now(lower: &str) -> Option<u64> {
    for (idx, word) in lower.split_whitespace().enumerate() {
        if word == "minute" || word == "minutes" {
            let prev = lower.split_whitespace().nth(idx.saturating_sub(1))?;
            return prev.parse().ok();
        }
    }
    if lower.contains("half an hour") {
        return Some(30);
    }
    None
}

fn clock_time(lower: &str) -> Option<(u32, u32, bool)> {
    let pm = lower.contains("p.m") || lower.contains("pm") || lower.contains("evening") || lower.contains("night");
    let am = lower.contains("a.m") || lower.contains("am") || lower.contains("morning");
    for token in lower.split_whitespace() {
        let clean = token.trim_matches(|ch: char| !ch.is_ascii_digit() && ch != ':');
        if let Some((h, m)) = clean.split_once(':') {
            let hour: u32 = h.parse().ok()?;
            let minute: u32 = m.parse().ok()?;
            return Some((hour, minute, pm && !am));
        }
        if let Ok(hour) = clean.parse::<u32>() {
            if (1..=12).contains(&hour) && (am || pm || lower.contains("o'clock") || lower.contains("alarm")) {
                return Some((hour, 0, pm && !am));
            }
        }
    }
    None
}

fn next_clock(now: SystemTime, hour: u32, minute: u32, pm: bool) -> SystemTime {
    let mut hour24 = hour % 12;
    if pm {
        hour24 += 12;
    }
    if hour == 12 && !pm {
        hour24 = 0;
    }
    if hour == 12 && pm {
        hour24 = 12;
    }
    let unix = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let local_offset = local_offset_secs();
    let local = unix as i64 + local_offset;
    let day = local.div_euclid(86_400);
    let tod = (hour24 as i64) * 3600 + (minute as i64) * 60;
    let mut target = day * 86_400 + tod - local_offset;
    if target <= unix as i64 {
        target += 86_400;
    }
    UNIX_EPOCH + Duration::from_secs(target.max(0) as u64)
}

fn local_offset_secs() -> i64 {
    Command::new("date")
        .arg("+%z")
        .output()
        .ok()
        .and_then(|out| {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let sign = if text.starts_with('-') { -1 } else { 1 };
            let digits: String = text.chars().filter(|ch| ch.is_ascii_digit()).collect();
            if digits.len() < 4 {
                return None;
            }
            let hours: i64 = digits[..2].parse().ok()?;
            let mins: i64 = digits[2..4].parse().ok()?;
            Some(sign * (hours * 3600 + mins * 60))
        })
        .unwrap_or(0)
}

fn apple_date(when: SystemTime) -> AgentResult<String> {
    let secs = when
        .duration_since(UNIX_EPOCH)
        .map_err(|err| AgentError::msg(err.to_string()))?
        .as_secs();
    let output = Command::new("date")
        .args(["-r", &secs.to_string(), "+%A, %B %d, %Y at %I:%M:%S %p"])
        .output()
        .map_err(|err| AgentError::msg(err.to_string()))?;
    if !output.status.success() {
        return Err(AgentError::msg("Could not format the reminder time."));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn osascript(script: &str) -> AgentResult<bool> {
    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map(|status| status.success())
        .map_err(|err| AgentError::msg(err.to_string()))
}

fn escape_as(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn sanitize_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == ' ' { ch } else { '-' })
        .collect();
    let stem = cleaned.split_whitespace().take(8).collect::<Vec<_>>().join("-");
    if stem.is_empty() {
        "note".into()
    } else {
        stem
    }
}

fn str_arg<'a>(args: &'a Value, key: &'a str) -> AgentResult<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| AgentError::msg(format!("missing {key}")))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_ten_minutes() {
        let when = parse_when("in 10 minutes").unwrap();
        let delta = when.duration_since(SystemTime::now()).unwrap_or_default().as_secs();
        assert!(delta >= 9 * 60 && delta <= 11 * 60);
    }

    #[test]
    fn seven_am_is_parsed() {
        assert!(parse_when("set an alarm for 7am").is_ok());
    }

    #[test]
    fn sanitize_title() {
        assert!(sanitize_name("Q3 report!").to_ascii_lowercase().contains("q3"));
    }
}
