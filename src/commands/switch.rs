use crate::config::Config;
use crate::error::KoseiError;
use crate::replacer::{self, diff};
use colored::Colorize;

pub fn execute(environment: &str, config: &Config, dry_run: bool) -> Result<(), KoseiError> {
    let env = config.environments.get(environment).ok_or_else(|| {
        let mut available: Vec<&String> = config.environments.keys().collect();
        available.sort();
        let available_list = available
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        KoseiError::EnvironmentNotFound(format!(
            "environment `{}` not found in kosei.yaml\navailable: {}",
            environment, available_list
        ))
    })?;

    if env.replacements.is_empty() {
        eprintln!(
            "warning: environment '{}' has no replacements defined — nothing will be changed",
            environment
        );
        return Ok(());
    }

    if dry_run {
        println!("{}", format!("[dry-run]\n").yellow());
    }

    let mut missing_files: Vec<String> = Vec::new();
    for replacement in &env.replacements {
        for file in &replacement.files {
            let path = config.base_dir.join(file);
            if !path.exists() {
                missing_files.push(file.clone());
            }
        }
    }

    if !missing_files.is_empty() {
        let mut msg = "There were issues resolving files from paths:\n".to_string();
        for file in &missing_files {
            msg.push_str(&format!("  - {}\n", file));
        }
        return Err(KoseiError::FileReadError(msg, std::io::ErrorKind::NotFound));
    }

    let mut any_change = false;

    for replacement in &env.replacements {
        for file in &replacement.files {
            let path = config.base_dir.join(file);

            let content = std::fs::read_to_string(&path).map_err(|e| {
                let kind = e.kind();
                KoseiError::FileReadError(format!("cannot read `{}`: {}", path.display(), e), kind)
            })?;

            let updated = replacer::apply(&content, replacement).map_err(|e| {
                KoseiError::RegexApplyError(format!("replacement failed for `{}`: {}", file, e))
            })?;

            if dry_run {
                if content != updated {
                    any_change = true;
                    diff::print_diff(file, &content, &updated);
                }
            } else {
                std::fs::write(&path, &updated).map_err(|e| {
                    let kind = e.kind();
                    KoseiError::FileWriteError(
                        format!("cannot write `{}`: {}", path.display(), e),
                        kind,
                    )
                })?;
            }
        }
    }

    if dry_run && !any_change {
        println!("{}", format!("{}", "✗ No changes detected".yellow()));
    } else if !dry_run {
        println!(
            "{}",
            format!(
                "{} {} {}",
                "✓ Switched to".green(),
                environment.green().bold(),
                "environment".green()
            )
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Environment, Replacement};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_env(replacements: Vec<Replacement>) -> Environment {
        Environment {
            description: None,
            replacements,
        }
    }

    fn create_test_config(
        temp_dir: &TempDir,
        environments: HashMap<String, Environment>,
    ) -> Config {
        Config {
            environments,
            base_dir: temp_dir.path().to_path_buf(),
        }
    }

    #[test]
    fn test_execute_environment_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut envs = HashMap::new();
        envs.insert("prod".to_string(), create_test_env(vec![]));
        let config = create_test_config(&temp_dir, envs);

        let result = execute("dev", &config, false);
        assert!(result.is_err());

        match result.unwrap_err() {
            KoseiError::EnvironmentNotFound(msg) => {
                assert!(msg.contains("dev"));
                assert!(msg.contains("prod"));
            }
            _ => panic!("Expected EnvironmentNotFound error"),
        }
    }

    #[test]
    fn test_execute_dry_run_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello world").unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["test.txt".to_string()],
                regex: "/nonexistent/".to_string(),
                value: "replaced".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, true);
        assert!(result.is_ok());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_execute_applies_changes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello world").unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["test.txt".to_string()],
                regex: "/world/".to_string(),
                value: "universe".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_ok());

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "hello universe");
    }

    #[test]
    fn test_execute_file_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["nonexistent.txt".to_string()],
                regex: "/test/".to_string(),
                value: "replaced".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_err());

        match result.unwrap_err() {
            KoseiError::FileReadError(msg, _) => {
                assert!(msg.contains("nonexistent.txt"));
            }
            _ => panic!("Expected FileReadError"),
        }
    }

    #[test]
    fn test_execute_invalid_regex() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello world").unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["test.txt".to_string()],
                regex: "/[invalid/".to_string(),
                value: "replaced".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_err());

        match result.unwrap_err() {
            KoseiError::RegexApplyError(msg) => {
                assert!(msg.contains("replacement failed"));
            }
            _ => panic!("Expected RegexApplyError"),
        }
    }

    #[test]
    fn test_execute_empty_replacements() {
        let temp_dir = TempDir::new().unwrap();
        let mut envs = HashMap::new();
        envs.insert("test".to_string(), create_test_env(vec![]));
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_missing_files_error() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.txt");
        fs::write(&existing_file, "content").unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["exists.txt".to_string(), "missing.txt".to_string()],
                regex: "/content/".to_string(),
                value: "updated".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_err());

        let content = fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn test_execute_all_files_missing_error() {
        let temp_dir = TempDir::new().unwrap();
        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["missing1.txt".to_string(), "missing2.txt".to_string()],
                regex: "/test/".to_string(),
                value: "replaced".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_dry_run_with_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.txt");
        fs::write(&existing_file, "content").unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["exists.txt".to_string(), "missing.txt".to_string()],
                regex: "/content/".to_string(),
                value: "updated".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, true);
        assert!(result.is_err());

        let content = fs::read_to_string(&existing_file).unwrap();
        assert_eq!(content, "content");
    }

    #[test]
    fn test_execute_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "foo bar").unwrap();
        fs::write(&file2, "foo baz").unwrap();

        let mut envs = HashMap::new();
        envs.insert(
            "test".to_string(),
            create_test_env(vec![Replacement {
                files: vec!["file1.txt".to_string(), "file2.txt".to_string()],
                regex: "/foo/".to_string(),
                value: "FOO".to_string(),
            }]),
        );
        let config = create_test_config(&temp_dir, envs);

        let result = execute("test", &config, false);
        assert!(result.is_ok());

        let content1 = fs::read_to_string(&file1).unwrap();
        let content2 = fs::read_to_string(&file2).unwrap();
        assert_eq!(content1, "FOO bar");
        assert_eq!(content2, "FOO baz");
    }
}
