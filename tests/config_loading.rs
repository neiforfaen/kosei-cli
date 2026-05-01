use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

// Ensure tests don't interfere with each other when changing directories
static TEST_DIR_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_config_loader_finds_kosei_yaml() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    fs::write(
        &config_path,
        "environments:\n  dev:\n    description: Development\n    replacements: []\n",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.environments.contains_key("dev"));
}

#[test]
fn test_config_loader_walks_up_directory() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    fs::write(
        &config_path,
        "environments:\n  prod:\n    description: Production\n    replacements: []\n",
    )
    .unwrap();

    let nested_dir = temp_dir.path().join("src").join("nested");
    fs::create_dir_all(&nested_dir).unwrap();

    // Canonicalize the expected path before changing directories
    let expected_base = std::fs::canonicalize(temp_dir.path()).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested_dir).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.environments.contains_key("prod"));
    // Canonicalize the result path to handle macOS symlink resolution
    assert_eq!(
        std::fs::canonicalize(&config.base_dir).unwrap(),
        expected_base
    );
}

#[test]
fn test_config_loader_parses_replacement_fields() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    fs::write(
        &config_path,
        "environments:\n  test:\n    description: Test Environment\n    replacements:\n      - files:\n          - \"*.rs\"\n        regex: \"/old_pattern/\"\n        value: \"new_value\"\n",
    ).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    let config = result.unwrap();
    let test_env = config.environments.get("test").unwrap();

    assert_eq!(test_env.description, Some("Test Environment".to_string()));
    assert_eq!(test_env.replacements.len(), 1);

    let replacement = &test_env.replacements[0];
    assert_eq!(replacement.files, vec!["*.rs".to_string()]);
    assert_eq!(replacement.regex, "/old_pattern/");
    assert_eq!(replacement.value, "new_value");
}

#[test]
fn test_config_loader_handles_missing_file() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_config_loader_handles_invalid_yaml() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    fs::write(
        &config_path,
        "invalid: yaml: content:\n  - broken\n    structure\n",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_config_loader_validates_regex_at_load_time() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    // Invalid regex: missing opening slash
    fs::write(
        &config_path,
        "environments:\n  test:\n    description: Test Environment\n    replacements:\n      - files:\n          - \"*.rs\"\n        regex: \"invalid_regex\"\n        value: \"new_value\"\n",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    // Should fail because regex is invalid
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("invalid regex in environment 'test'"));
}

#[test]
fn test_config_loader_validates_invalid_regex_pattern() {
    let _lock = TEST_DIR_LOCK.lock().unwrap();

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    // Invalid regex: malformed pattern
    fs::write(
        &config_path,
        "environments:\n  test:\n    description: Test Environment\n    replacements:\n      - files:\n          - \"*.rs\"\n        regex: \"/[invalid/\"\n        value: \"new_value\"\n",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = kosei_cli::ConfigLoader::load();

    std::env::set_current_dir(original_dir).unwrap();

    // Should fail because regex pattern is invalid
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("invalid regex in environment 'test'"));
}
