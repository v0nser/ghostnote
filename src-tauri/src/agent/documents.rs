//! Local document reading. PDFs stay on this machine.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::error::{AgentError, AgentResult};
use super::skills;

const MAX_CHARS: usize = 24_000;

/// Find a user document and return extractable text.
pub fn load_for_summary(query: &str) -> AgentResult<(PathBuf, String)> {
    let path = resolve_path(query)?;
    let text = extract_text(&path)?;
    if text.chars().count() < 40 {
        return Err(AgentError::msg(format!(
            "I opened {} but it has almost no extractable text. Scanned image PDFs need a vision model, which is not wired yet.",
            path.display()
        )));
    }
    Ok((path, truncate(&text, MAX_CHARS)))
}

fn resolve_path(query: &str) -> AgentResult<PathBuf> {
    let trimmed = query.trim().trim_matches('"');
    let as_path = PathBuf::from(trimmed);
    if as_path.is_file() {
        return Ok(as_path);
    }
    if trimmed.starts_with("~/") {
        if let Some(home) = home_dir() {
            let expanded = home.join(trimmed.trim_start_matches("~/"));
            if expanded.is_file() {
                return Ok(expanded);
            }
        }
    }

    let needle = if trimmed.is_empty()
        || matches!(trimmed, "pdf" | "the pdf" | "this pdf" | "my pdf" | "document")
    {
        "pdf"
    } else {
        trimmed.trim_end_matches(".pdf")
    };

    let mut hits = skills::search_files(needle);
    if needle == "pdf" {
        hits = latest_pdfs();
    } else {
        hits.retain(|path| is_readable_doc(path));
        if hits.is_empty() {
            hits = skills::search_files(&format!("{needle}.pdf"));
        }
    }
    hits.into_iter().next().ok_or_else(|| {
        AgentError::msg(
            "I could not find that PDF in Downloads, Documents, Desktop, or Home. Name the file.",
        )
    })
}

pub fn extract_text(path: &Path) -> AgentResult<String> {
    if super::paths::is_sensitive(path) {
        return Err(AgentError::msg("that file is not readable by the agent"));
    }
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "txt" | "md" | "markdown" | "csv" | "json" | "rs" | "py" | "ts" | "tsx" | "js" => {
            let bytes = std::fs::read(path).map_err(|err| AgentError::msg(err.to_string()))?;
            if bytes.len() > 400_000 {
                return Err(AgentError::msg("file too large to summarize"));
            }
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
        "pdf" => extract_pdf(path),
        _ => Err(AgentError::msg(
            "I can summarize PDF, text, and markdown files on this computer.",
        )),
    }
}

fn extract_pdf(path: &Path) -> AgentResult<String> {
    if let Some(text) = pdftotext(path) {
        if usable(&text) {
            return Ok(text);
        }
    }
    if let Ok(text) = pdf_extract::extract_text(path) {
        if usable(&text) {
            return Ok(text);
        }
    }
    let fallback = parenthesis_strings(path);
    if usable(&fallback) {
        return Ok(fallback);
    }
    Err(AgentError::msg(format!(
        "I could not extract text from {}. If this is a scan, I cannot read it yet. For text PDFs, install poppler (`pdftotext`) and try again.",
        path.display()
    )))
}

fn pdftotext(path: &Path) -> Option<String> {
    let output = Command::new("pdftotext")
        .args(["-layout", "-enc", "UTF-8", "-q"])
        .arg(path)
        .arg("-")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    Some(text)
}

fn parenthesis_strings(path: &Path) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    let raw = String::from_utf8_lossy(&bytes);
    let mut out = String::new();
    let mut rest = raw.as_ref();
    while let Some(start) = rest.find('(') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find(')') else {
            break;
        };
        let piece = rest[..end].trim();
        rest = &rest[end + 1..];
        if piece.len() >= 3 && piece.bytes().filter(|b| b.is_ascii_alphabetic()).count() >= 3 {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(piece);
        }
    }
    out
}

fn latest_pdfs() -> Vec<PathBuf> {
    let mut hits = Vec::new();
    for root in skills::search_roots() {
        collect_pdfs(&root, &mut hits, 0);
    }
    hits.sort_by_key(|path| {
        std::cmp::Reverse(
            path.metadata()
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0),
        )
    });
    hits.truncate(8);
    hits
}

fn collect_pdfs(dir: &Path, hits: &mut Vec<PathBuf>, depth: u8) {
    if depth > 3 || hits.len() >= 40 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_pdfs(&path, hits, depth + 1);
        } else if is_readable_doc(&path) && path.extension().and_then(|e| e.to_str()) == Some("pdf")
        {
            hits.push(path);
        }
    }
}

fn is_readable_doc(path: &Path) -> bool {
    if super::paths::is_sensitive(path) || !path.is_file() {
        return false;
    }
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "pdf" | "txt" | "md" | "markdown" | "csv"
    )
}

fn usable(text: &str) -> bool {
    text.chars().filter(|ch| ch.is_alphanumeric()).count() >= 40
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    text.chars().take(max).collect::<String>() + "\n\n[truncated]"
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_simple_pdf_literals() {
        let dir = std::env::temp_dir().join("coda-pdf-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("hello.pdf");
        let mut file = std::fs::File::create(&path).expect("create");
        file.write_all(
            b"%PDF-1.1\n1 0 obj<<>>endobj\nstream\nBT (Quarterly hiring plan for Asha) Tj ET\nendstream\n",
        )
        .expect("write");
        drop(file);
        let text = parenthesis_strings(&path);
        assert!(text.contains("hiring"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn truncates_long_text() {
        let long = "word ".repeat(10_000);
        let cut = truncate(&long, 80);
        assert!(cut.contains("[truncated]"));
        assert!(cut.chars().count() < 120);
    }
}
