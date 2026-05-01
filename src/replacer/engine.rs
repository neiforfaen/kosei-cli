use crate::config::regex::{build_regex, parse_js_regex};
use crate::config::Replacement;
use crate::error::KoseiError;

pub fn apply(content: &str, replacement: &Replacement) -> Result<String, KoseiError> {
    let (pattern, flags) = parse_js_regex(&replacement.regex)?;
    let re = build_regex(&pattern, &flags)?;

    Ok(re
        .replace_all(content, replacement.value.as_str())
        .into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_simple_replacement() {
        let replacement = Replacement {
            files: vec![],
            regex: "/foo/".to_string(),
            value: "bar".to_string(),
        };
        let result = apply("foo is foo", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bar is bar");
    }

    #[test]
    fn test_apply_case_insensitive() {
        let replacement = Replacement {
            files: vec![],
            regex: "/hello/i".to_string(),
            value: "hi".to_string(),
        };
        let result = apply("HELLO world Hello", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hi world hi");
    }

    #[test]
    fn test_apply_with_multiline() {
        let replacement = Replacement {
            files: vec![],
            regex: "/^test/m".to_string(),
            value: "pass".to_string(),
        };
        let result = apply("test line\ntest another", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pass line\npass another");
    }

    #[test]
    fn test_apply_ignored_g_flag() {
        let replacement = Replacement {
            files: vec![],
            regex: "/foo/g".to_string(),
            value: "bar".to_string(),
        };
        let result = apply("foo bar foo", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bar bar bar");
    }

    #[test]
    fn test_apply_ignored_u_flag() {
        let replacement = Replacement {
            files: vec![],
            regex: "/foo/u".to_string(),
            value: "bar".to_string(),
        };
        let result = apply("foo bar foo", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bar bar bar");
    }

    #[test]
    fn test_apply_ignored_y_flag() {
        let replacement = Replacement {
            files: vec![],
            regex: "/foo/y".to_string(),
            value: "bar".to_string(),
        };
        let result = apply("foo bar foo", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bar bar bar");
    }

    #[test]
    fn test_apply_mixed_ignored_and_supported_flags() {
        let replacement = Replacement {
            files: vec![],
            regex: "/HELLO/igu".to_string(),
            value: "hi".to_string(),
        };
        let result = apply("HELLO hello Hello", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hi hi hi");
    }

    #[test]
    fn test_apply_invalid_pattern() {
        let replacement = Replacement {
            files: vec![],
            regex: "/[invalid/".to_string(),
            value: "bar".to_string(),
        };
        let result = apply("test content", &replacement);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, KoseiError::RegexParseError(_)));
        assert!(err.to_string().contains("failed to compile regex pattern"));
    }

    #[test]
    fn test_apply_with_capture_group() {
        let replacement = Replacement {
            files: vec![],
            regex: "/(\\w+)@(\\w+)/".to_string(),
            value: "$2.$1".to_string(),
        };
        let result = apply("user@domain", &replacement);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "domain.user");
    }
}
