use crate::error::KoseiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Mirror the config shape with both Deserialize and Serialize so we can
// round-trip from JSON -> YAML without touching the main config module.
#[derive(Deserialize, Serialize)]
struct MigrateConfig {
    environments: HashMap<String, MigrateEnvironment>,
}

#[derive(Deserialize, Serialize)]
struct MigrateEnvironment {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    replacements: Vec<MigrateReplacement>,
}

#[derive(Deserialize, Serialize)]
struct MigrateReplacement {
    files: Vec<String>,
    regex: String,
    value: String,
}

pub fn execute() -> Result<(), KoseiError> {
    let start = std::env::current_dir().map_err(|e| KoseiError::ConfigReadError(e.to_string()))?;
    execute_in(&start)
}

fn execute_in(start: &std::path::Path) -> Result<(), KoseiError> {
    let json_path = find_json_config(start)?;
    let base_dir = json_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    println!("Found {}", json_path.display());

    let json_content = std::fs::read_to_string(&json_path)
        .map_err(|e| KoseiError::ConfigReadError(e.to_string()))?;

    let config: MigrateConfig = serde_json::from_str(&json_content).map_err(|e| {
        KoseiError::ConfigParseError(format!("failed to parse kosei.config.json: {}", e))
    })?;

    let yaml_content = serde_yaml::to_string(&config)
        .map_err(|e| KoseiError::ConfigParseError(format!("failed to serialize to YAML: {}", e)))?;

    let yaml_path = base_dir.join("kosei.yaml");

    std::fs::write(&yaml_path, &yaml_content).map_err(|e| {
        KoseiError::FileWriteError(format!("cannot write kosei.yaml: {}", e), e.kind())
    })?;

    println!("Written {}", yaml_path.display());

    std::fs::remove_file(&json_path).map_err(|e| {
        KoseiError::FileWriteError(format!("cannot delete kosei.config.json: {}", e), e.kind())
    })?;

    println!("Deleted {}", json_path.display());
    println!("Migration complete.");

    Ok(())
}

fn find_json_config(start: &std::path::Path) -> Result<PathBuf, KoseiError> {
    let mut dir = start.to_path_buf();

    loop {
        let candidate = dir.join("kosei.config.json");
        if candidate.exists() {
            return Ok(candidate);
        }

        if !dir.pop() {
            break;
        }
    }

    Err(KoseiError::ConfigNotFound(
        "Could not find kosei.config.json in any parent directory".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_json(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("kosei.config.json");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_find_json_config_not_found() {
        // Use a temp dir that definitely has no kosei.config.json above it.
        // We pass the path directly so no set_current_dir is needed.
        let temp = TempDir::new().unwrap();
        let result = find_json_config(temp.path());

        assert!(result.is_err());
        match result.unwrap_err() {
            KoseiError::ConfigNotFound(_) => {}
            e => panic!("expected ConfigNotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_migration_roundtrip() {
        let temp = TempDir::new().unwrap();
        let json = r#"{
            "environments": {
                "dev": {
                    "description": "Development",
                    "replacements": [
                        {
                            "files": [".env"],
                            "regex": "/API_URL=.*/",
                            "value": "API_URL=https://dev.example.com"
                        }
                    ]
                },
                "prod": {
                    "replacements": [
                        {
                            "files": [".env", "config.js"],
                            "regex": "/API_URL=.*/",
                            "value": "API_URL=https://prod.example.com"
                        }
                    ]
                }
            }
        }"#;

        let json_path = write_json(&temp, json);

        // Parse and re-serialize
        let config: MigrateConfig = serde_json::from_str(json).unwrap();
        let yaml = serde_yaml::to_string(&config).unwrap();

        // Verify the YAML round-trips back to the same data
        let reparsed: MigrateConfig = serde_yaml::from_str(&yaml).unwrap();
        assert!(reparsed.environments.contains_key("dev"));
        assert!(reparsed.environments.contains_key("prod"));

        let dev = reparsed.environments.get("dev").unwrap();
        assert_eq!(dev.description.as_deref(), Some("Development"));
        assert_eq!(dev.replacements.len(), 1);
        assert_eq!(dev.replacements[0].files, vec![".env"]);

        let prod = reparsed.environments.get("prod").unwrap();
        assert!(prod.description.is_none());
        assert_eq!(prod.replacements[0].files, vec![".env", "config.js"]);

        // Cleanup (json_path still exists since we didn't call execute)
        drop(json_path);
    }

    #[test]
    fn test_execute_migrates_and_deletes() {
        let temp = TempDir::new().unwrap();
        let json = r#"{"environments": {"dev": {"replacements": [{"files": [".env"], "regex": "/X=.*/", "value": "X=1"}]}}}
"#;
        write_json(&temp, json);

        // Use execute_in so we don't mutate the process-wide CWD.
        let result = execute_in(temp.path());

        assert!(result.is_ok(), "{:?}", result);

        // kosei.yaml should exist, kosei.config.json should be gone
        assert!(temp.path().join("kosei.yaml").exists());
        assert!(!temp.path().join("kosei.config.json").exists());
    }

    #[test]
    fn test_execute_invalid_json_errors() {
        let temp = TempDir::new().unwrap();
        write_json(&temp, "not valid json {");

        // Use execute_in so we don't mutate the process-wide CWD.
        let result = execute_in(temp.path());

        assert!(result.is_err());
        match result.unwrap_err() {
            KoseiError::ConfigParseError(_) => {}
            e => panic!("expected ConfigParseError, got {:?}", e),
        }
    }
}
