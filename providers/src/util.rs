//! Shared utilities for puzzle providers.

/// User-Agent string for HTTP requests.
pub const USER_AGENT: &str = concat!("cruciverbal/", env!("CARGO_PKG_VERSION"));

/// Create a configured reqwest client with standard headers.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap_or_default()
}

/// Decode URL-encoded strings (percent encoding).
///
/// Handles common percent-encoded characters and `+` as space.
pub fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_decode_basic() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("hello+world"), "hello world");
        assert_eq!(url_decode("100%25"), "100%");
    }

    #[test]
    fn test_url_decode_passthrough() {
        assert_eq!(url_decode("hello"), "hello");
        assert_eq!(url_decode(""), "");
    }
}
