use super::config::AgentConfig;
use super::types::{PolicyDecision, RiskLevel};

/// Deterministic policy engine. The model cannot change these outcomes.
pub fn decide(tool: &str, risk: RiskLevel, config: &AgentConfig, computer_use_allowed: bool) -> PolicyDecision {
    if tool.starts_with("COMPUTER_") && !computer_use_allowed {
        return PolicyDecision::Deny;
    }

    match tool {
        "EMAIL_SEND" | "MESSAGE_SEND" | "SEND_EMAIL" | "DRAFT_EMAIL" | "EMAIL_DRAFT"
            if config.require_approval_for_email =>
        {
            return PolicyDecision::RequireApproval;
        }
        "CALENDAR_EVENT" => return PolicyDecision::RequireApproval,
        "SUGGEST_FILE" => return PolicyDecision::RequireApproval,
        "PURCHASE" | "PAY" if config.require_approval_for_purchase => {
            return PolicyDecision::RequireApproval;
        }
        "FILES_DELETE" if config.require_approval_for_file_delete => {
            return PolicyDecision::RequireApproval;
        }
        "FILES_WRITE" if config.require_approval_for_file_write => {
            return PolicyDecision::RequireApproval;
        }
        "SHELL_EXECUTE" => return PolicyDecision::RequireApproval,
        "MEMORY_DELETE" => return PolicyDecision::RequireApproval,
        name if is_credential_tool(name) => return PolicyDecision::Deny,
        _ => {}
    }

    match risk {
        RiskLevel::Low => PolicyDecision::Allow,
        RiskLevel::Medium => PolicyDecision::Allow,
        RiskLevel::High => PolicyDecision::RequireApproval,
        RiskLevel::Critical => PolicyDecision::RequireApproval,
    }
}

pub fn risk_for_tool(tool: &str) -> RiskLevel {
    match tool {
        "WEB_SEARCH" | "CALCULATOR" | "DATETIME" | "MEMORY_SEARCH" | "NOTIFICATION"
        | "OPEN_APPLICATION" | "OPEN_URL" | "OPEN_FOLDER" | "LIST_APPS"         | "SET_REMINDER"
        | "PLAY_MEDIA"
        | "PLAY_PLAYLIST"
        | "CLOSE_APPLICATION"
        | "FOCUS_APPLICATION"
        | "TAKE_SCREENSHOT" => {
            RiskLevel::Low
        }
        "WEB_FETCH" | "BROWSER_OPEN" | "BROWSER_SCROLL" | "FILES_LIST" | "FILES_READ"
        | "FILES_SEARCH" | "MEMORY_WRITE" | "TASK_SCHEDULE" | "OPEN_FILE" | "CREATE_FOLDER"
        | "FIND_FILE" | "CREATE_DOCUMENT" | "SUMMARIZE_FILE" => {
            RiskLevel::Medium
        }
        "BROWSER_CLICK" | "BROWSER_TYPE" | "BROWSER_SCREENSHOT" | "FILES_WRITE"
        | "CODE_EXECUTE" | "PYTHON_EXECUTE" | "TASK_CANCEL" => RiskLevel::Medium,
        "FILES_DELETE" | "SHELL_EXECUTE" | "EMAIL_SEND" | "SEND_EMAIL" | "EMAIL_DRAFT"
        | "DRAFT_EMAIL" | "CALENDAR_EVENT" | "MEMORY_DELETE" | "SUGGEST_FILE" => {
            RiskLevel::High
        }
        "PURCHASE" | "CREDENTIAL_READ" | "PERMISSION_GRANT" | "COMPUTER_CLICK"
        | "COMPUTER_TYPE" | "COMPUTER_KEYPRESS" => RiskLevel::Critical,
        _ => RiskLevel::Medium,
    }
}

pub fn profile_allows_tool(allowed: &[String], tool: &str) -> bool {
    if allowed.iter().any(|name| name == "*") {
        return !matches!(tool, "PERMISSION_GRANT" | "CREDENTIAL_READ");
    }
    allowed.iter().any(|name| name == tool)
}

fn is_credential_tool(name: &str) -> bool {
    matches!(name, "CREDENTIAL_READ" | "PERMISSION_GRANT" | "PERMISSION_ESCALATE")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_always_asks() {
        let config = AgentConfig::default();
        assert_eq!(
            decide("EMAIL_SEND", RiskLevel::High, &config, false),
            PolicyDecision::RequireApproval
        );
    }

    #[test]
    fn search_is_allowed() {
        let config = AgentConfig::default();
        assert_eq!(
            decide("WEB_SEARCH", RiskLevel::Low, &config, false),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn workspace_write_is_allowed_by_default() {
        let config = AgentConfig::default();
        assert_eq!(
            decide("FILES_WRITE", RiskLevel::Medium, &config, false),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn computer_use_denied_by_default() {
        let config = AgentConfig::default();
        assert_eq!(
            decide("COMPUTER_CLICK", RiskLevel::Critical, &config, false),
            PolicyDecision::Deny
        );
    }

    #[test]
    fn credentials_are_denied() {
        let config = AgentConfig::default();
        assert_eq!(
            decide("CREDENTIAL_READ", RiskLevel::Critical, &config, true),
            PolicyDecision::Deny
        );
    }

    #[test]
    fn profile_cannot_include_escalation_via_wildcard() {
        assert!(!profile_allows_tool(&["*".into()], "PERMISSION_GRANT"));
        assert!(profile_allows_tool(&["*".into()], "WEB_SEARCH"));
    }

    #[test]
    fn open_application_is_allowed_without_computer_flag() {
        let config = AgentConfig::default();
        assert_eq!(
            decide("OPEN_APPLICATION", RiskLevel::Low, &config, false),
            PolicyDecision::Allow
        );
    }
}
