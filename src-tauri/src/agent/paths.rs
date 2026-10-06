use std::path::{Path, PathBuf};

use super::error::{AgentError, AgentResult};

/// Resolve `requested` inside `root`. Rejects traversal and sensitive names.
pub fn jail(root: &Path, requested: &str) -> AgentResult<PathBuf> {
    if requested.trim().is_empty() {
        return Ok(root.to_path_buf());
    }
    if requested.starts_with('~') || Path::new(requested).is_absolute() {
        return Err(AgentError::msg("absolute paths are not allowed"));
    }
    let joined = root.join(requested);
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let candidate = if joined.exists() {
        joined.canonicalize().map_err(|err| AgentError::msg(err.to_string()))?
    } else {
        let parent = joined.parent().unwrap_or(root);
        let file = joined.file_name().ok_or_else(|| AgentError::msg("invalid path"))?;
        let parent = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
        parent.join(file)
    };
    if !candidate.starts_with(&canonical_root) {
        return Err(AgentError::msg("path escapes the workspace"));
    }
    if is_sensitive(&candidate) {
        return Err(AgentError::msg("that file is not readable by the agent"));
    }
    Ok(candidate)
}

pub fn is_sensitive(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name == "id_rsa"
        || name == "id_ed25519"
        || name == ".env"
        || name.ends_with(".pem")
        || name == "credentials.json"
        || name == "secrets.json"
        || text.contains("/.ssh/")
        || text.contains("/.gnupg/")
        || text.contains("/appdata/roaming/microsoft/credentials")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_traversal_and_home() {
        let root = std::env::temp_dir();
        match jail(&root, "../../etc/passwd") {
            Err(_) => {}
            Ok(path) => {
                let canon = root.canonicalize().unwrap_or(root.clone());
                assert!(path.starts_with(&canon), "jail leaked to {}", path.display());
            }
        }
        assert!(jail(&root, "~/.ssh/id_rsa").is_err());
        assert!(jail(&root, "/etc/passwd").is_err());
    }

    #[test]
    fn blocks_ssh_keys() {
        assert!(is_sensitive(Path::new("/Users/me/.ssh/id_rsa")));
        assert!(!is_sensitive(Path::new("/tmp/workspace/notes.md")));
    }
}
