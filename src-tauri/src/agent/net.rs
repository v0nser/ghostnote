use std::net::{IpAddr, ToSocketAddrs};

use super::error::{AgentError, AgentResult};

#[derive(Debug, Clone)]
pub struct SafeUrl {
    pub raw: String,
    pub host: String,
}

pub fn parse_public_http_url(raw: &str) -> AgentResult<SafeUrl> {
    let raw = raw.trim();
    let lower = raw.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(AgentError::msg("only http and https URLs are allowed"));
    }
    let rest = raw.split_once("://").map(|(_, rest)| rest).unwrap_or(raw);
    let host_port = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = host_port
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(host_port);
    let host = host
        .trim_start_matches('[')
        .split(']')
        .next()
        .unwrap_or(host)
        .split(':')
        .next()
        .unwrap_or(host)
        .trim()
        .to_ascii_lowercase();

    if host.is_empty() || is_blocked_host(&host) {
        return Err(AgentError::msg("that host is not allowed"));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(AgentError::msg("that address is not allowed"));
        }
    } else if let Ok(addrs) = (host.as_str(), 80u16).to_socket_addrs() {
        for addr in addrs {
            if is_blocked_ip(addr.ip()) {
                return Err(AgentError::msg("that address is not allowed"));
            }
        }
    }
    Ok(SafeUrl {
        raw: raw.to_string(),
        host,
    })
}

pub fn is_blocked_host(host: &str) -> bool {
    host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host == "metadata.google.internal"
        || host.ends_with(".internal")
}

pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.octets()[0] == 169 && v4.octets()[1] == 254
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_localhost_and_metadata() {
        assert!(parse_public_http_url("http://127.0.0.1/secret").is_err());
        assert!(parse_public_http_url("http://localhost/admin").is_err());
        assert!(parse_public_http_url("file:///etc/passwd").is_err());
        assert!(parse_public_http_url("https://example.com/x").is_ok());
    }
}
