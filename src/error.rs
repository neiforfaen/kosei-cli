use std::fmt;
use std::io;

#[derive(Debug)]
pub enum KoseiError {
    ConfigNotFound(String),
    ConfigReadError(String),
    ConfigParseError(String),
    EnvironmentNotFound(String),
    FileReadError(String, io::ErrorKind),
    FileWriteError(String, io::ErrorKind),
    RegexParseError(String),
    RegexApplyError(String),
}

impl fmt::Display for KoseiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KoseiError::ConfigNotFound(msg) => write!(f, "Config not found: {}", msg),
            KoseiError::ConfigReadError(msg) => write!(f, "Failed to read config: {}", msg),
            KoseiError::ConfigParseError(msg) => write!(f, "Failed to parse config: {}", msg),
            KoseiError::EnvironmentNotFound(msg) => {
                write!(f, "Environment name not found: {}", msg)
            }
            KoseiError::FileReadError(msg, kind) => {
                let context = match kind {
                    io::ErrorKind::NotFound => "file does not exist",
                    io::ErrorKind::PermissionDenied => "permission denied reading",
                    _ => "cannot read file",
                };
                write!(f, "Failed to read file: {} ({})", msg, context)
            }
            KoseiError::FileWriteError(msg, kind) => {
                let context = match kind {
                    io::ErrorKind::PermissionDenied => "permission denied writing",
                    io::ErrorKind::NotFound => "target directory does not exist",
                    _ => "cannot write file",
                };
                write!(f, "Failed to write file: {} ({})", msg, context)
            }
            KoseiError::RegexParseError(msg) => write!(f, "Failed to parse regex: {}", msg),
            KoseiError::RegexApplyError(msg) => write!(f, "Failed to apply regex: {}", msg),
        }
    }
}

impl std::error::Error for KoseiError {}
