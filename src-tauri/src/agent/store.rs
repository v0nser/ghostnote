use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::config::AgentConfig;
use super::error::{AgentError, AgentResult};
use super::ids;
use super::types::{
    AgentProfile, AgentTask, Approval, Conversation, MemoryItem, MemoryScope, PermissionGrant,
    ScheduledTask,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentSnapshot {
    pub config: Option<AgentConfig>,
    #[serde(default)]
    pub tasks: Vec<AgentTask>,
    #[serde(default)]
    pub memories: Vec<MemoryItem>,
    #[serde(default)]
    pub approvals: Vec<Approval>,
    #[serde(default)]
    pub profiles: Vec<AgentProfile>,
    #[serde(default)]
    pub schedules: Vec<ScheduledTask>,
    #[serde(default)]
    pub conversations: Vec<Conversation>,
    #[serde(default)]
    pub permissions: Vec<PermissionGrant>,
}

pub fn root(_app: &AppHandle) -> AgentResult<PathBuf> {
    let dir = crate::coda::agent_dir();
    fs::create_dir_all(dir.join("tasks")).ok();
    fs::create_dir_all(dir.join("workspace")).ok();
    fs::create_dir_all(dir.join("artifacts")).ok();
    fs::create_dir_all(dir.join("sandboxes")).ok();
    Ok(dir)
}

pub fn workspace(app: &AppHandle) -> AgentResult<PathBuf> {
    Ok(root(app)?.join("workspace"))
}

pub fn load(app: &AppHandle) -> AgentResult<AgentSnapshot> {
    let path = root(app)?.join("state.json");
    if !path.is_file() {
        let mut snap = AgentSnapshot::default();
        snap.profiles = default_profiles();
        return Ok(snap);
    }
    let raw = fs::read_to_string(&path).map_err(|err| AgentError::msg(err.to_string()))?;
    let mut snap: AgentSnapshot =
        serde_json::from_str(&raw).map_err(|err| AgentError::msg(err.to_string()))?;
    if snap.profiles.is_empty() {
        snap.profiles = default_profiles();
    }
    Ok(snap)
}

pub fn save(app: &AppHandle, snap: &AgentSnapshot) -> AgentResult<()> {
    let path = root(app)?.join("state.json");
    let raw = serde_json::to_string_pretty(snap).map_err(|err| AgentError::msg(err.to_string()))?;
    fs::write(path, raw).map_err(|err| AgentError::msg(err.to_string()))
}

pub fn append_audit(app: &AppHandle, event: &str, body: &serde_json::Value) -> AgentResult<()> {
    let ts = ids::now_ms();
    let line = serde_json::json!({
        "timestamp": ts,
        "event": event,
        "data": body,
    });
    let path = root(app)?.join("audit.jsonl");
    append_line(&path, &line.to_string())?;
    let mirror = crate::coda::home().join("coda_audit.log");
    let human = format!(
        "{ts} {event} {}\n",
        body.to_string().chars().take(400).collect::<String>()
    );
    append_line(&mirror, &human)?;
    Ok(())
}

fn append_line(path: &std::path::Path, line: &str) -> AgentResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| AgentError::msg(err.to_string()))?;
    use std::io::Write;
    writeln!(file, "{}", line.trim_end()).map_err(|err| AgentError::msg(err.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRow {
    pub timestamp: u64,
    pub event: String,
    pub data: serde_json::Value,
}

pub fn read_audit(_app: &AppHandle, query: Option<&str>, limit: usize) -> AgentResult<Vec<AuditRow>> {
    let dir = crate::coda::agent_dir();
    let mut rows = Vec::new();
    read_jsonl(&dir.join("audit.jsonl"), &mut rows);
    if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
        let needle = q.to_ascii_lowercase();
        rows.retain(|row| {
            row.event.to_ascii_lowercase().contains(&needle)
                || row.data.to_string().to_ascii_lowercase().contains(&needle)
        });
    }
    rows.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    rows.truncate(limit.max(1).min(2000));
    Ok(rows)
}

pub fn export_audit_text() -> AgentResult<String> {
    let path = crate::coda::home().join("coda_audit.log");
    if !path.is_file() {
        return Ok(String::new());
    }
    fs::read_to_string(path).map_err(|err| AgentError::msg(err.to_string()))
}

fn read_jsonl(path: &std::path::Path, rows: &mut Vec<AuditRow>) {
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(row) = serde_json::from_str::<AuditRow>(line) {
            rows.push(row);
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            rows.push(AuditRow {
                timestamp: value.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0),
                event: value
                    .get("event")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                data: value.get("data").cloned().unwrap_or(value),
            });
        }
    }
}

pub fn default_profiles() -> Vec<AgentProfile> {
    vec![
        AgentProfile {
            id: "meeting".into(),
            name: "Meeting Assistant".into(),
            system_prompt: "You turn meeting speech into one spoken answer.".into(),
            model: None,
            tools: vec!["MEMORY_WRITE".into(), "DATETIME".into(), "NOTIFICATION".into()],
            memory_scope: MemoryScope::Conversation,
            approval_policy: "strict".into(),
        },
        AgentProfile {
            id: "personal".into(),
            name: "Personal Assistant".into(),
            system_prompt: "Help with everyday goals. Ask before anything irreversible.".into(),
            model: None,
            tools: vec!["*".into()],
            memory_scope: MemoryScope::User,
            approval_policy: "strict".into(),
        },
        AgentProfile {
            id: "research".into(),
            name: "Research Agent".into(),
            system_prompt: "Research, compare, and report. Do not send or delete.".into(),
            model: None,
            tools: vec![
                "WEB_SEARCH".into(),
                "WEB_FETCH".into(),
                "BROWSER_OPEN".into(),
                "MEMORY_SEARCH".into(),
                "MEMORY_WRITE".into(),
                "FILES_WRITE".into(),
                "CALCULATOR".into(),
                "DATETIME".into(),
            ],
            memory_scope: MemoryScope::Task,
            approval_policy: "strict".into(),
        },
        AgentProfile {
            id: "coding".into(),
            name: "Coding Agent".into(),
            system_prompt: "Work only inside the agent workspace.".into(),
            model: None,
            tools: vec![
                "FILES_LIST".into(),
                "FILES_READ".into(),
                "FILES_WRITE".into(),
                "FILES_SEARCH".into(),
                "CODE_EXECUTE".into(),
                "PYTHON_EXECUTE".into(),
                "WEB_SEARCH".into(),
                "MEMORY_SEARCH".into(),
            ],
            memory_scope: MemoryScope::Agent,
            approval_policy: "strict".into(),
        },
        AgentProfile {
            id: "jobs".into(),
            name: "Job Search Agent".into(),
            system_prompt: "Find roles, compare them, draft applications. Never submit silently.".into(),
            model: None,
            tools: vec![
                "WEB_SEARCH".into(),
                "WEB_FETCH".into(),
                "MEMORY_SEARCH".into(),
                "MEMORY_WRITE".into(),
                "FILES_WRITE".into(),
                "NOTIFICATION".into(),
            ],
            memory_scope: MemoryScope::User,
            approval_policy: "strict".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_jsonl_parses_unified_and_meeting_shapes() {
        let dir = std::env::temp_dir().join(format!(
            "coda-audit-parse-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.jsonl");
        fs::write(
            &path,
            r#"{"timestamp":1,"event":"task.created","data":{"goal":"play music"}}
{"timestamp":2,"event":"meeting.extract","meetingId":"m1","model":"llama3.1:8b","input":{"n":1},"output":{"ok":true}}
"#,
        )
        .unwrap();
        let mut rows = Vec::new();
        read_jsonl(Path::new(&path), &mut rows);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].event, "task.created");
        assert_eq!(rows[0].data["goal"], "play music");
        assert_eq!(rows[1].event, "meeting.extract");
        assert_eq!(rows[1].data["meetingId"], "m1");
        let _ = fs::remove_dir_all(dir);
    }
}
