use crate::KoseiError;
use colored::Colorize;

pub fn execute(path: &Option<String>) -> Result<(), KoseiError> {
    let target_path = path
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let config_path = target_path.join("kosei.yaml");

    if config_path.exists() {
        return Err(KoseiError::FileWriteError(
            format!("kosei.yaml already exists at {}", config_path.display()),
            std::io::ErrorKind::AlreadyExists,
        ));
    }

    let default_config = r#"environments:
  example:
    description: "An example environment"
    replacements:
      - files: ["**/*.txt"]
        regex: "/foo/"
        value: "bar"
        "#;

    std::fs::write(&config_path, default_config).map_err(|e| {
        KoseiError::FileWriteError(format!("cannot write kosei.yaml: {}", e), e.kind())
    })?;

    println!(
        "{}",
        format!(
            "{} {}",
            "✓ kosei.yaml initialized @".green(),
            config_path.display().to_string().bold()
        )
    );
    Ok(())
}
