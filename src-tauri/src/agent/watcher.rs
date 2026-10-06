//! Watch a local folder for new files and queue a suggestion. Never auto-open.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Manager};

use super::error::AgentResult;
use super::runtime::AgentRuntime;

pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    std::thread::Builder::new()
        .name("coda-folder-watch".into())
        .spawn(move || {
            if let Err(err) = run(app) {
                log::warn!("folder watcher stopped: {err}");
            }
        })
        .ok();
}

fn run(app: AppHandle) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
        .map_err(|err| err.to_string())?;
    let mut watching = PathBuf::new();
    let mut last_path: Option<PathBuf> = None;
    let mut last_at = Instant::now() - Duration::from_secs(10);

    loop {
        let folder = watch_folder(&app);
        if folder != watching && folder.is_dir() {
            if watching.as_os_str().len() > 0 {
                let _ = watcher.unwatch(&watching);
            }
            if let Err(err) = watcher.watch(&folder, RecursiveMode::NonRecursive) {
                log::warn!("could not watch {}: {err}", folder.display());
            } else {
                log::info!("watching {}", folder.display());
                watching = folder;
            }
        }

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(event)) => {
                if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    continue;
                }
                for path in event.paths {
                    if path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with('.'))
                    {
                        continue;
                    }
                    if !path.is_file() {
                        continue;
                    }
                    let now = Instant::now();
                    if last_path.as_ref() == Some(&path) && now.duration_since(last_at) < Duration::from_millis(800)
                    {
                        continue;
                    }
                    last_path = Some(path.clone());
                    last_at = now;
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(err) = suggest_file(&app, &path).await {
                            log::warn!("folder suggestion: {err}");
                        }
                    });
                }
            }
            Ok(Err(err)) => log::warn!("watch error: {err}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn watch_folder(app: &AppHandle) -> PathBuf {
    let Some(runtime) = app.try_state::<AgentRuntime>() else {
        return default_downloads();
    };
    runtime.ensure_loaded(app);
    let configured = runtime.watch_folder();
    if configured.trim().is_empty() {
        default_downloads()
    } else {
        PathBuf::from(shellexpand(&configured))
    }
}

fn default_downloads() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Downloads")
}

fn shellexpand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        return default_downloads()
            .parent()
            .unwrap_or(Path::new("."))
            .join(rest)
            .to_string_lossy()
            .into_owned();
    }
    path.to_string()
}

async fn suggest_file(app: &AppHandle, path: &Path) -> AgentResult<()> {
    let Some(runtime) = app.try_state::<AgentRuntime>() else {
        return Ok(());
    };
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(name);
    let hits = runtime.recall(app, stem, 3).await;
    if hits.is_empty() {
        return Ok(());
    }
    let matched = hits
        .iter()
        .map(|hit| hit.text.clone())
        .collect::<Vec<_>>()
        .join("; ");
    let goal = format!(
        "New file `{name}` may match a stored memory ({matched}). Do not open or send it."
    );
    runtime.start_task(app, goal, "personal".into(), None).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_home_prefix() {
        let expanded = shellexpand("~/Downloads");
        assert!(expanded.ends_with("Downloads"));
        assert!(!expanded.starts_with('~'));
    }
}
