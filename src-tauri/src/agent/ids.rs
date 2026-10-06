use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQ: AtomicU64 = AtomicU64::new(1);

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn new_id(prefix: &str) -> String {
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}_{:x}_{seq:x}", now_ms())
}

pub fn arguments_hash(value: &serde_json::Value) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    value.to_string().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn sanitize_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, child) in map {
                let lowered = key.to_ascii_lowercase();
                if looks_secret(&lowered) {
                    out.insert(key.clone(), serde_json::Value::String("[redacted]".into()));
                } else {
                    out.insert(key.clone(), sanitize_value(child));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(sanitize_value).collect())
        }
        serde_json::Value::String(text) if looks_secret_value(text) => {
            serde_json::Value::String("[redacted]".into())
        }
        other => other.clone(),
    }
}

fn looks_secret(key: &str) -> bool {
    ["password", "secret", "token", "api_key", "apikey", "authorization", "cookie", "credit"]
        .iter()
        .any(|needle| key.contains(needle))
}

fn looks_secret_value(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("sk-") || lower.contains("bearer ") || lower.contains("-----begin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_token_fields() {
        let value = serde_json::json!({"token": "secret-value", "query": "laptops"});
        let clean = sanitize_value(&value);
        assert_eq!(clean["token"], "[redacted]");
        assert_eq!(clean["query"], "laptops");
    }
}
