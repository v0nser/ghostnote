use super::types::{PlanStep, RiskLevel};

/// Turns a goal into a structured plan. The planner never executes.
pub fn plan(goal: &str) -> Vec<PlanStep> {
    if let Some(intent) = super::intent::route(goal) {
        return vec![PlanStep::new(
            intent.summary(),
            &[intent.tool_name()],
            RiskLevel::Low,
        )];
    }
    if let Some(name) = super::intent::leading_application(goal) {
        return vec![
            PlanStep::new(
                format!("Opening {name}"),
                &["OPEN_APPLICATION"],
                RiskLevel::Low,
            ),
            PlanStep::new("Continue the rest of the request", &[], RiskLevel::Low),
        ];
    }

    let lower = goal.to_ascii_lowercase();

    if looks_like_remember_only(&lower) {
        return vec![PlanStep::new("Store this in memory", &["MEMORY_WRITE"], RiskLevel::Low)];
    }
    if looks_like_forget_only(&lower) {
        return vec![PlanStep::new(
            "Forget the requested memory",
            &["MEMORY_DELETE"],
            RiskLevel::High,
        )];
    }
    if looks_like_summarize(&lower) {
        return vec![
            PlanStep::new(
                "Read the local document",
                &["SUMMARIZE_FILE"],
                RiskLevel::Medium,
            ),
            PlanStep::new("Write the summary", &[], RiskLevel::Low),
        ];
    }
    if looks_like_time(&lower) {
        return vec![
            PlanStep::new("Read the current time", &["DATETIME"], RiskLevel::Low),
            PlanStep::new("Write the answer", &[], RiskLevel::Low),
        ];
    }
    if extract_math(&lower).is_some() {
        return vec![
            PlanStep::new("Run the calculation", &["CALCULATOR"], RiskLevel::Low),
            PlanStep::new("Write the answer", &[], RiskLevel::Low),
        ];
    }

    let mut steps = vec![PlanStep::new(
        "Understand the request",
        &[],
        RiskLevel::Low,
    )];

    if first_http_url(goal).is_some() {
        steps.push(PlanStep::new(
            "Open the link the user provided",
            &["WEB_FETCH"],
            RiskLevel::Medium,
        ));
    } else if needs_research(&lower) {
        steps.push(PlanStep::new(
            "Search the web for current information",
            &["WEB_SEARCH"],
            RiskLevel::Low,
        ));
    }

    if needs_workspace_read(&lower) {
        steps.push(PlanStep::new(
            "Inspect the local workspace",
            &["FILES_SEARCH"],
            RiskLevel::Low,
        ));
    }

    if needs_workspace_write(&lower) {
        steps.push(PlanStep::new(
            "Write a file in the local workspace",
            &["FILES_WRITE"],
            RiskLevel::Medium,
        ));
    }

    if needs_code(&lower) {
        let tool = if lower.contains("node") || lower.contains("javascript") {
            "CODE_EXECUTE"
        } else {
            "PYTHON_EXECUTE"
        };
        steps.push(PlanStep::new("Run a short sandbox snippet", &[tool], RiskLevel::Medium));
    }

    if contains_any(&lower, &["remember", "note that", "save this fact", "don't forget"])
        && !needs_workspace_write(&lower)
    {
        steps.push(PlanStep::new(
            "Write a lasting memory",
            &["MEMORY_WRITE"],
            RiskLevel::Low,
        ));
    }

    if contains_any(&lower, &["every morning", "every day", "daily", "weekly", "schedule this"])
    {
        steps.push(PlanStep::new(
            "Schedule a recurring follow-up",
            &["TASK_SCHEDULE"],
            RiskLevel::Medium,
        ));
    }

    if contains_any(&lower, &["send", "submit", "purchase", "buy", "pay"]) {
        steps.push(PlanStep::new(
            "Pause for approval before any irreversible action",
            &[],
            RiskLevel::High,
        ));
    }

    steps.push(PlanStep::new(
        "Write the answer",
        &[],
        RiskLevel::Low,
    ));

    wire_dependencies(steps)
}

pub fn replan(goal: &str, reason: &str) -> Vec<PlanStep> {
    let mut steps = plan(goal);
    if !reason.is_empty() {
        steps.insert(
            0,
            PlanStep::new(
                format!("Adjust the approach after: {reason}"),
                &[],
                RiskLevel::Low,
            ),
        );
        steps = wire_dependencies(steps);
    }
    steps
}

pub fn needs_research(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "find", "search", "research", "compare", "options", "jobs", "price",
            "look up", "lookup", "latest", "news", "weather", "forecast", "who is",
            "who was", "what is the", "what's the", "how much", "stock", "score",
            "headline", "google", "wiki", "wikipedia", "current", "today in",
            "best", "top 5", "top five",
        ],
    )
}

pub fn looks_like_time(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "what time",
            "what's the time",
            "current time",
            "date today",
            "what's the date",
            "what is the date",
            "utc time",
            "time is it",
        ],
    )
}

pub fn extract_math(goal: &str) -> Option<String> {
    let lower = goal.to_ascii_lowercase();
    if needs_research(&lower) || first_http_url(goal).is_some() {
        return None;
    }
    if let Some(expr) = percent_of(&lower) {
        return Some(expr);
    }
    let filtered: String = goal
        .chars()
        .filter(|ch| ch.is_ascii_digit() || matches!(ch, '+' | '-' | '*' | '/' | '(' | ')' | '.' | ' '))
        .collect();
    let compact: String = filtered.split_whitespace().collect();
    if compact.chars().any(|ch| ch.is_ascii_digit())
        && compact.chars().any(|ch| matches!(ch, '+' | '*' | '/'))
        && super::tools::eval_arith_for_plan(&compact).is_some()
    {
        return Some(compact);
    }
    if contains_any(&lower, &["calculate", "compute", "plus", "minus", "times", "divided"]) {
        if compact.chars().any(|ch| ch.is_ascii_digit()) {
            return Some(compact);
        }
    }
    None
}

fn percent_of(lower: &str) -> Option<String> {
    let idx = lower.find("% of ")?;
    let left = lower[..idx]
        .split_whitespace()
        .last()?
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    let right = lower[idx + 5..]
        .split_whitespace()
        .next()?
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    if left.is_empty() || right.is_empty() {
        return None;
    }
    Some(format!("({left}*{right})/100"))
}

pub fn first_http_url(goal: &str) -> Option<String> {
    goal.split_whitespace().find_map(|word| {
        let trimmed = word.trim_matches(|ch: char| "()[],.\"'".contains(ch));
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Some(trimmed.to_string())
        } else {
            None
        }
    })
}

pub fn needs_workspace_read(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "list files",
            "list the files",
            "what's in the workspace",
            "in the workspace",
            "read the file",
            "open the file",
            "search files",
            "folder",
            "directory",
            "codebase",
        ],
    )
}

pub fn needs_workspace_write(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "save to a file",
            "save as ",
            "write a file",
            "write to ",
            "create a file",
            "create file",
            "save this as",
            "put this in",
            "write it to",
        ],
    )
}

pub fn needs_code(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "run python",
            "run this code",
            "execute this",
            "python script",
            "node script",
            "```python",
            "```js",
            "```javascript",
        ],
    )
}

fn wire_dependencies(mut steps: Vec<PlanStep>) -> Vec<PlanStep> {
    for index in 1..steps.len() {
        let previous = steps[index - 1].step_id.clone();
        steps[index].dependencies = vec![previous];
    }
    steps
}

fn looks_like_summarize(lower: &str) -> bool {
    (lower.contains("summarize") || lower.contains("tldr") || lower.contains("summary of"))
        && (lower.contains("pdf")
            || lower.contains(".pdf")
            || lower.contains("document")
            || lower.contains("resume")
            || lower.contains("file"))
}

fn looks_like_remember_only(lower: &str) -> bool {
    (lower.starts_with("remember ") || lower.starts_with("remember this"))
        && !contains_any(lower, &["find", "search", "research", "compare"])
}

fn looks_like_forget_only(lower: &str) -> bool {
    lower.starts_with("forget ")
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| hay.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_goal_has_search_then_report() {
        let steps = plan("Research three laptops under ₹100,000 and compare them.");
        let tools: Vec<_> = steps.iter().flat_map(|step| step.required_tools.iter()).collect();
        assert!(tools.iter().any(|name| *name == "WEB_SEARCH"));
        assert!(steps.last().unwrap().description.contains("answer"));
        assert!(steps.iter().all(|step| step.status == super::super::types::StepStatus::Pending));
    }

    #[test]
    fn submit_creates_approval_checkpoint() {
        let steps = plan("Find 5 jobs and submit the applications");
        assert!(steps.iter().any(|step| step.risk_level == RiskLevel::High));
    }

    #[test]
    fn write_a_poem_is_not_a_file_write() {
        let tools: Vec<_> = plan("Write a short poem about rain")
            .iter()
            .flat_map(|step| step.required_tools.iter())
            .cloned()
            .collect();
        assert!(!tools.iter().any(|name| name == "FILES_WRITE"));
    }

    #[test]
    fn weather_uses_search() {
        let tools: Vec<_> = plan("What's the weather in Delhi today?")
            .iter()
            .flat_map(|step| step.required_tools.iter())
            .cloned()
            .collect();
        assert!(tools.iter().any(|name| name == "WEB_SEARCH"));
    }

    #[test]
    fn math_uses_calculator() {
        let tools: Vec<_> = plan("what's 25*4")
            .iter()
            .flat_map(|step| step.required_tools.iter())
            .cloned()
            .collect();
        assert!(tools.iter().any(|name| name == "CALCULATOR"));
    }

    #[test]
    fn summarize_pdf_uses_file_tool() {
        let tools: Vec<_> = plan("Summarize the offer letter PDF in Downloads")
            .iter()
            .flat_map(|step| step.required_tools.iter())
            .cloned()
            .collect();
        assert_eq!(tools, vec!["SUMMARIZE_FILE".to_string()]);
    }

    #[test]
    fn open_apple_music_uses_computer_tool() {
        let tools: Vec<_> = plan("Open Apple Music")
            .iter()
            .flat_map(|step| step.required_tools.iter())
            .cloned()
            .collect();
        assert_eq!(tools, vec!["OPEN_APPLICATION".to_string()]);
    }

    #[test]
    fn complex_music_request_plays_playlist() {
        let tools: Vec<_> = plan("Open Apple Music and play my workout playlist")
            .iter()
            .flat_map(|step| step.required_tools.iter())
            .cloned()
            .collect();
        assert_eq!(tools, vec!["PLAY_PLAYLIST".to_string()]);
    }
}
