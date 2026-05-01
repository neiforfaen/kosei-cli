pub mod diff;
pub mod engine;

pub use engine::apply;

/// Parse a JavaScript-style regex string into pattern and flags.
/// Must be paired with [build_regex] to handle flag conversion.
pub use crate::config::regex::parse_js_regex;
