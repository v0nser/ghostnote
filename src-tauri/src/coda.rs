//! User-visible data root (`~/.coda`) and one-time Ghost Note → Coda copy.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LEGACY_BUNDLE: &str = "com.ghostnote.app";
const MARKER: &str = ".migrated-from-ghostnote";

/// `CODA_HOME` if set, otherwise `~/.coda`.
pub fn home() -> PathBuf {
    if let Ok(forced) = std::env::var("CODA_HOME") {
        let trimmed = forced.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    user_home().join(".coda")
}

pub fn agent_dir() -> PathBuf {
    home().join("agent")
}

pub fn models_dir() -> PathBuf {
    home().join("models")
}

pub fn logs_dir() -> PathBuf {
    home().join("logs")
}

/// Copy Ghost Note app-data into `~/.coda` without deleting the source or
/// overwriting files that Coda already wrote.
pub fn migrate_from_ghostnote() -> io::Result<MigrateReport> {
    let dest = home();
    fs::create_dir_all(&dest)?;
    let marker = dest.join(MARKER);
    if marker.is_file() {
        return Ok(MigrateReport {
            copied: false,
            from: None,
            reason: "already migrated",
        });
    }

    let Some(from) = legacy_app_data() else {
        write_marker(&marker, None)?;
        return Ok(MigrateReport {
            copied: false,
            from: None,
            reason: "no Ghost Note data directory",
        });
    };
    if !from.is_dir() {
        write_marker(&marker, Some(&from))?;
        return Ok(MigrateReport {
            copied: false,
            from: Some(from),
            reason: "legacy directory missing",
        });
    }

    let copied = copy_tree(&from, &dest)?;
    write_marker(&marker, Some(&from))?;
    Ok(MigrateReport {
        copied,
        from: Some(from),
        reason: "copied",
    })
}

#[derive(Debug, Clone)]
pub struct MigrateReport {
    pub copied: bool,
    pub from: Option<PathBuf>,
    pub reason: &'static str,
}

fn write_marker(path: &Path, from: Option<&Path>) -> io::Result<()> {
    let body = format!(
        "from={}\n",
        from.map(|p| p.display().to_string()).unwrap_or_default()
    );
    fs::write(path, body)
}

fn legacy_app_data() -> Option<PathBuf> {
    if let Ok(forced) = std::env::var("CODA_LEGACY_DATA") {
        if !forced.trim().is_empty() {
            return Some(PathBuf::from(forced));
        }
    }
    let home = user_home();
    if cfg!(target_os = "macos") {
        Some(
            home.join("Library")
                .join("Application Support")
                .join(LEGACY_BUNDLE),
        )
    } else if cfg!(target_os = "windows") {
        Some(
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("AppData").join("Roaming"))
                .join(LEGACY_BUNDLE),
        )
    } else {
        Some(
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local").join("share"))
                .join(LEGACY_BUNDLE),
        )
    }
}

fn user_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Copy `from` into `to`. Existing dest files are left untouched.
fn copy_tree(from: &Path, to: &Path) -> io::Result<bool> {
    let mut copied_any = false;
    let mut stack = vec![from.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let src = entry.path();
            let rel = src.strip_prefix(from).unwrap_or(&src);
            let dest = to.join(rel);
            if src.is_dir() {
                fs::create_dir_all(&dest)?;
                stack.push(src);
            } else if !dest.is_file() {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&src, &dest)?;
                copied_any = true;
            }
        }
    }
    Ok(copied_any)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());

    fn unique_temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "coda-mig-{}-{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp");
        dir
    }

    #[test]
    fn copies_legacy_without_deleting_source_or_overwriting() {
        let _guard = LOCK.lock().expect("lock");
        let root = unique_temp("copy");
        let legacy = root.join("legacy");
        let dest = root.join("coda");
        fs::create_dir_all(legacy.join("agent")).unwrap();
        fs::write(legacy.join("agent/audit.jsonl"), b"old-audit\n").unwrap();
        fs::write(legacy.join("agent/state.json"), b"{\"tasks\":[]}").unwrap();
        fs::create_dir_all(dest.join("agent")).unwrap();
        fs::write(dest.join("agent/state.json"), b"{\"keep\":true}").unwrap();

        std::env::set_var("CODA_HOME", dest.to_string_lossy().as_ref());
        std::env::set_var("CODA_LEGACY_DATA", legacy.to_string_lossy().as_ref());
        let report = migrate_from_ghostnote().expect("migrate");
        assert!(report.copied);
        assert_eq!(
            fs::read_to_string(dest.join("agent/state.json")).unwrap(),
            "{\"keep\":true}"
        );
        assert_eq!(
            fs::read_to_string(dest.join("agent/audit.jsonl")).unwrap(),
            "old-audit\n"
        );
        assert!(legacy.join("agent/audit.jsonl").is_file());
        assert!(dest.join(MARKER).is_file());

        let again = migrate_from_ghostnote().expect("second");
        assert!(!again.copied);
        assert_eq!(again.reason, "already migrated");

        std::env::remove_var("CODA_HOME");
        std::env::remove_var("CODA_LEGACY_DATA");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn skip_when_legacy_missing() {
        let _guard = LOCK.lock().expect("lock");
        let dest = unique_temp("nolegacy");
        std::env::set_var("CODA_HOME", dest.to_string_lossy().as_ref());
        std::env::set_var("CODA_LEGACY_DATA", dest.join("does-not-exist").to_string_lossy().as_ref());
        let report = migrate_from_ghostnote().expect("migrate");
        assert!(!report.copied);
        assert!(dest.join(MARKER).is_file());
        std::env::remove_var("CODA_HOME");
        std::env::remove_var("CODA_LEGACY_DATA");
        let _ = fs::remove_dir_all(dest);
    }
}
