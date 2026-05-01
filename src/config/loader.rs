use crate::config::regex::{build_regex, parse_js_regex};
use crate::error::KoseiError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
struct RawConfig {
    pub environments: HashMap<String, Environment>,
}

#[derive(Debug)]
pub struct Config {
    pub environments: HashMap<String, Environment>,
    pub base_dir: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct Environment {
    pub description: Option<String>,
    pub replacements: Vec<Replacement>,
}

#[derive(Deserialize, Debug)]
pub struct Replacement {
    pub files: Vec<String>,
    pub regex: String,
    pub value: String,
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Result<Config, KoseiError> {
        let path = Self::find_config()?;
        let base_dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let file = std::fs::read_to_string(&path)
            .map_err(|e| KoseiError::ConfigReadError(e.to_string()))?;
        let raw: RawConfig =
            serde_yaml::from_str(&file).map_err(|e| KoseiError::ConfigParseError(e.to_string()))?;

        for (env_name, environment) in &raw.environments {
            for (replacement_idx, replacement) in environment.replacements.iter().enumerate() {
                if replacement.files.is_empty() {
                    return Err(KoseiError::ConfigParseError(format!(
                        "empty files list in environment '{}', replacement {}",
                        env_name, replacement_idx
                    )));
                }
                if let Err(e) = Self::validate_regex(&replacement.regex) {
                    return Err(KoseiError::ConfigParseError(format!(
                        "invalid regex in environment '{}', replacement {}: {}",
                        env_name, replacement_idx, e
                    )));
                }
            }
        }

        Ok(Config {
            environments: raw.environments,
            base_dir,
        })
    }

    /// Validate a regex by parsing the format and attempting to compile it
    fn validate_regex(raw: &str) -> Result<(), KoseiError> {
        let (pattern, flags) = parse_js_regex(raw)?;
        build_regex(&pattern, &flags)?;
        Ok(())
    }

    fn find_config() -> Result<PathBuf, KoseiError> {
        let mut dir =
            std::env::current_dir().map_err(|e| KoseiError::ConfigReadError(e.to_string()))?;

        loop {
            let candidate = dir.join("kosei.yaml");
            if candidate.exists() {
                return Ok(candidate);
            }

            if !dir.pop() {
                break;
            }
        }

        Err(KoseiError::ConfigNotFound(
            "Could not find kosei.yaml in any parent directory".to_string(),
        ))
    }
}
