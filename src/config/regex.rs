use crate::error::KoseiError;
use regex::RegexBuilder;

/// Parse a JavaScript-style regex string into pattern and flags.
/// Format: `/pattern/flags`
pub fn parse_js_regex(raw: &str) -> Result<(String, String), KoseiError> {
    if !raw.starts_with('/') {
        return Err(KoseiError::RegexParseError(format!(
            "regex must use /pattern/flags format, got: `{}`",
            raw
        )));
    }

    let inner = &raw[1..];
    let close = inner.rfind('/').ok_or_else(|| {
        KoseiError::RegexParseError(format!("regex is missing a closing `/`: `{}`", raw))
    })?;

    let pattern = inner[..close].to_string();
    let flags = inner[close + 1..].to_string();

    Ok((pattern, flags))
}

/// Build a compiled Regex from a pattern and flags string.
/// Handles flag parsing and applies appropriate RegexBuilder settings.
///
/// Supported flags:
/// - 'i': case-insensitive matching
/// - 'm': multi-line mode (^ and $ match line boundaries)
/// - 's': dot matches newline
///
/// Silently ignored flags (for backward compatibility with TypeScript version):
/// - 'g': global flag - Rust's replace_all already replaces all occurrences globally
/// - 'u': unicode flag - Unicode handling is implicit in Rust regexes
/// - 'y': sticky flag - Sticky flag doesn't apply to replace_all semantics
pub fn build_regex(pattern: &str, flags: &str) -> Result<regex::Regex, KoseiError> {
    let mut builder = RegexBuilder::new(pattern);

    for ch in flags.chars() {
        match ch {
            'g' => {}
            'u' => {}
            'y' => {}
            'i' => {
                builder.case_insensitive(true);
            }
            'm' => {
                builder.multi_line(true);
            }
            's' => {
                builder.dot_matches_new_line(true);
            }
            other => {
                return Err(KoseiError::RegexParseError(format!(
                    "unsupported regex flag: `{}`",
                    other
                )));
            }
        }
    }

    builder.build().map_err(|e| {
        KoseiError::RegexParseError(format!(
            "failed to compile regex pattern `{}`: {}",
            pattern, e
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_js_regex_simple() {
        let result = parse_js_regex("/hello/");
        assert!(result.is_ok());
        let (pattern, flags) = result.unwrap();
        assert_eq!(pattern, "hello");
        assert_eq!(flags, "");
    }

    #[test]
    fn test_parse_js_regex_with_flags() {
        let result = parse_js_regex("/world/im");
        assert!(result.is_ok());
        let (pattern, flags) = result.unwrap();
        assert_eq!(pattern, "world");
        assert_eq!(flags, "im");
    }

    #[test]
    fn test_parse_js_regex_missing_start_slash() {
        let result = parse_js_regex("hello/");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, KoseiError::RegexParseError(_)));
        assert!(err.to_string().contains("must use /pattern/flags format"));
    }

    #[test]
    fn test_parse_js_regex_missing_end_slash() {
        let result = parse_js_regex("/hello");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, KoseiError::RegexParseError(_)));
        assert!(err.to_string().contains("missing a closing `/`"));
    }

    #[test]
    fn test_build_regex_simple() {
        let result = build_regex("hello", "");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_regex_case_insensitive() {
        let result = build_regex("hello", "i");
        assert!(result.is_ok());
        let re = result.unwrap();
        assert!(re.is_match("HELLO"));
        assert!(re.is_match("Hello"));
    }

    #[test]
    fn test_build_regex_multiline() {
        let result = build_regex("^test", "m");
        assert!(result.is_ok());
        let re = result.unwrap();
        assert!(re.is_match("test"));
        assert!(re.is_match("\ntest"));
    }

    #[test]
    fn test_build_regex_ignores_g_flag() {
        let result = build_regex("foo", "g");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_regex_ignores_u_flag() {
        let result = build_regex("foo", "u");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_regex_ignores_y_flag() {
        let result = build_regex("foo", "y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_regex_ignores_multiple_ignored_flags() {
        let result = build_regex("foo", "guy");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_regex_mixed_flags() {
        let result = build_regex("hello", "igum");
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_regex_invalid_pattern() {
        let result = build_regex("[invalid", "");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to compile regex pattern"));
    }

    #[test]
    fn test_build_regex_unsupported_flag() {
        let result = build_regex("foo", "x");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unsupported regex flag"));
    }
}
