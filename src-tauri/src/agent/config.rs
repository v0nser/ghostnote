use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub agent_enabled: bool,
    pub default_model: String,
    pub ollama_url: String,
    pub browser_enabled: bool,
    pub computer_use_enabled: bool,
    pub sandbox_enabled: bool,
    pub remote_models_enabled: bool,
    pub require_approval_for_email: bool,
    pub require_approval_for_purchase: bool,
    pub require_approval_for_file_delete: bool,
    pub require_approval_for_file_write: bool,
    pub prefer_fast_model: bool,
    #[serde(default)]
    pub agent_model: Option<String>,
    #[serde(default)]
    pub fast_router_model: Option<String>,
    #[serde(default)]
    pub vision_model: Option<String>,
    #[serde(default)]
    pub speech_model: Option<String>,
    #[serde(default)]
    pub save_screen_captures: bool,
    #[serde(default = "default_screen_hotkey")]
    pub screen_hotkey: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub watch_folder: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_enabled: env_flag_any(&["CODA_AGENT_ENABLED", "GHOSTNOTE_AGENT_ENABLED"], true),
            default_model: env_first(&[
                "CODA_AGENT_MODEL",
                "CODA_MODEL",
                "GHOSTNOTE_AGENT_MODEL",
                "GHOSTNOTE_MODEL",
                "DEFAULT_MODEL",
            ])
            .unwrap_or_else(|| "llama3.1:8b".into()),
            ollama_url: std::env::var("OLLAMA_HOST")
                .or_else(|_| std::env::var("OLLAMA_URL"))
                .map(|raw| {
                    let trimmed = raw.trim().trim_end_matches('/');
                    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                        trimmed.to_string()
                    } else {
                        format!("http://{trimmed}")
                    }
                })
                .unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
            browser_enabled: env_flag("BROWSER_ENABLED", true),
            computer_use_enabled: env_flag("COMPUTER_USE_ENABLED", false),
            sandbox_enabled: env_flag("SANDBOX_ENABLED", true),
            remote_models_enabled: env_flag("REMOTE_MODELS_ENABLED", false),
            require_approval_for_email: env_flag("REQUIRE_APPROVAL_FOR_EMAIL", true),
            require_approval_for_purchase: env_flag("REQUIRE_APPROVAL_FOR_PURCHASE", true),
            require_approval_for_file_delete: env_flag("REQUIRE_APPROVAL_FOR_FILE_DELETE", true),
            require_approval_for_file_write: env_flag("REQUIRE_APPROVAL_FOR_FILE_WRITE", false),
            prefer_fast_model: env_flag_any(&["CODA_FAST", "GHOSTNOTE_FAST"], true),
            agent_model: env_first(&["CODA_AGENT_MODEL", "GHOSTNOTE_AGENT_MODEL"]),
            fast_router_model: env_first(&["CODA_FAST_MODEL", "GHOSTNOTE_FAST_MODEL"]),
            vision_model: env_first(&["CODA_VISION_MODEL", "GHOSTNOTE_VISION_MODEL"]),
            speech_model: env_first(&["CODA_SPEECH_MODEL", "GHOSTNOTE_SPEECH_MODEL"]),
            save_screen_captures: env_flag_any(
                &["CODA_SAVE_SCREEN_CAPTURES", "GHOSTNOTE_SAVE_SCREEN_CAPTURES"],
                false,
            ),
            screen_hotkey: env_first(&["CODA_SCREEN_HOTKEY", "GHOSTNOTE_SCREEN_HOTKEY"])
                .unwrap_or_else(default_screen_hotkey),
            autostart: env_flag("CODA_AUTOSTART", false),
            watch_folder: env_first(&["CODA_WATCH_FOLDER"]).unwrap_or_default(),
        }
    }
}

fn env_flag(name: &str, default: bool) -> bool {
    env_flag_any(&[name], default)
}

fn env_flag_any(names: &[&str], default: bool) -> bool {
    for name in names {
        if let Ok(value) = std::env::var(name) {
            return matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on");
        }
    }
    default
}

fn env_first(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
}

fn default_screen_hotkey() -> String {
    "CommandOrControl+Shift+C".into()
}
