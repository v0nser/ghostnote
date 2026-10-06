//! Local crash / failure reports. Never uploaded.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use super::ids;

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info.to_string();
        write_report(
            None,
            "panic",
            &message,
            json!({
                "location": info.location().map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column())),
            }),
        );
        previous(info);
    }));
}

pub fn write_report(app: Option<&AppHandle>, component: &str, error: &str, extra: Value) {
    let payload = json!({
        "timestamp": ids::now_ms(),
        "component": component,
        "error": error,
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "extra": extra,
    });
    for dir in log_dirs(app) {
        if fs::create_dir_all(&dir).is_err() {
            continue;
        }
        let path = dir.join("failures.jsonl");
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{payload}");
            break;
        }
    }
    log::error!("[{component}] {error}");
}

fn log_dirs(app: Option<&AppHandle>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(app) = app {
        if let Ok(root) = app.path().app_data_dir() {
            dirs.push(root.join("logs"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("logs"));
        if cwd.file_name().is_some_and(|name| name == "src-tauri") {
            if let Some(parent) = cwd.parent() {
                dirs.push(parent.join("logs"));
            }
        }
    }
    dirs.push(crate::coda::logs_dir());
    dirs.push(std::env::temp_dir().join("coda-logs"));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_does_not_panic() {
        write_report(None, "test", "tool failed", json!({"tool": "OPEN_APPLICATION"}));
    }
}
