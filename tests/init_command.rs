use std::fs;
use tempfile::TempDir;

#[test]
fn init_creates_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("kosei.yaml");

    // Execute init command
    kosei_cli::commands::init(&Some(temp_dir.path().to_string_lossy().to_string())).unwrap();

    // Verify file exists
    assert!(config_path.exists(), "Config file should be created");

    // Verify content structure
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(
        content.contains("environments:"),
        "Should have environments section"
    );
    assert!(
        content.contains("example:"),
        "Should have example environment"
    );
    assert!(
        content.contains("regex: \"/foo/\""),
        "Should have valid regex format with leading slash"
    );
    assert!(
        content.contains("replacements:"),
        "Should have replacements section"
    );
    assert!(content.contains("files:"), "Should have files pattern");
    assert!(
        content.contains("value: \"bar\""),
        "Should have replacement value"
    );
}

#[test]
fn init_prevents_overwrite() {
    let temp_dir = TempDir::new().unwrap();

    // First init should succeed
    kosei_cli::commands::init(&Some(temp_dir.path().to_string_lossy().to_string())).unwrap();

    // Second init should fail
    let result = kosei_cli::commands::init(&Some(temp_dir.path().to_string_lossy().to_string()));
    assert!(result.is_err(), "Should fail when config already exists");
}
