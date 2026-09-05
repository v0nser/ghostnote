//! Parse the copilot's XML reply, including mid-stream fragments.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    pub question: String,
    pub answer: String,
}

/// Finished reply. Falls back to the raw text if the model skipped the tags.
pub fn parse_reply(raw: &str) -> Draft {
    let mut draft = parse_partial(raw);
    if draft.answer.is_empty() {
        let stripped = strip_tags(raw);
        if stripped.chars().count() >= 12 {
            draft.answer = stripped;
        }
    }
    draft
}

/// Whatever `<detected_question>` and `<answer>` have arrived so far.
pub fn parse_partial(raw: &str) -> Draft {
    Draft {
        question: inner_tag(raw, "detected_question").unwrap_or_default(),
        answer: inner_tag(raw, "answer").unwrap_or_default(),
    }
}

fn inner_tag(raw: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = raw.find(&open)? + open.len();
    let rest = &raw[start..];
    let body = if let Some(end) = rest.find(&close) {
        &rest[..end]
    } else {
        rest
    };
    let text = collapse_ws(body);
    (!text.is_empty()).then_some(text)
}

fn strip_tags(raw: &str) -> String {
    let mut out = raw.to_string();
    for tag in [
        "<detected_question>",
        "</detected_question>",
        "<answer>",
        "</answer>",
    ] {
        out = out.replace(tag, " ");
    }
    collapse_ws(&out)
}

fn collapse_ws(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_finished_xml() {
        let draft = parse_reply(
            "<detected_question>What is a closure in JavaScript?</detected_question>\n\
             <answer>A closure is a function that remembers the variables from the scope where it was created. I use them for private state in hooks.</answer>",
        );
        assert_eq!(draft.question, "What is a closure in JavaScript?");
        assert!(draft.answer.starts_with("A closure is a function"));
    }

    #[test]
    fn streams_an_open_answer() {
        let draft = parse_partial(
            "<detected_question>What is a closure?</detected_question><answer>A closure is a function that",
        );
        assert_eq!(draft.question, "What is a closure?");
        assert_eq!(draft.answer, "A closure is a function that");
    }

    #[test]
    fn ignores_an_unopened_answer() {
        let draft = parse_partial("<detected_question>What is a closure?");
        assert_eq!(draft.question, "What is a closure?");
        assert!(draft.answer.is_empty());
    }
}
