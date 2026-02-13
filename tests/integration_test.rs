use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_end_to_end_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let src_dir = root.join("src");
    fs::create_dir(&src_dir)?;

    let main_rs = src_dir.join("main.rs");
    let code = r#"
        fn main() {
            println!("Hello, AMDB!");
        }
    "#;
    fs::write(&main_rs, code)?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project indexed successfully"));

    assert!(root.join(".database").exists());
    assert!(root.join(".database/vector/vectors.db").exists());

    let mut cmd_gen = cargo_bin_cmd!("amdb");
    cmd_gen
        .current_dir(root)
        .arg("generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated:"));

    let context_md = root.join(".amdb/context.md");
    assert!(context_md.exists());

    let content = fs::read_to_string(context_md)?;
    assert!(content.contains("main"));
    Ok(())
}

#[test]
fn test_focus_mode() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let file_path = root.join("auth.rs");
    fs::write(&file_path, "fn login() {}")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init.current_dir(root).arg("init").arg(".").assert().success();

    let mut cmd_focus = cargo_bin_cmd!("amdb");
    cmd_focus
        .current_dir(root)
        .arg("generate")
        .arg("--focus")
        .arg("login")
        .assert()
        .success();

    assert!(root.join(".amdb/login.md").exists());
    Ok(())
}

#[test]
#[cfg(unix)]
fn test_error_handling_unreadable_file() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let broken_file = root.join("broken.rs");
    fs::write(&broken_file, "fn broken() {}")?;

    let mut perms = fs::metadata(&broken_file)?.permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&broken_file, perms)?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Failed to read file"));

    Ok(())
}

#[test]
fn test_dynamic_config_exclude() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let config_path = root.join("amdb.toml");
    fs::write(&config_path, "exclude_patterns = [\"secret_vault\"]\n")?;

    let secret_dir = root.join("secret_vault");
    fs::create_dir(&secret_dir)?;
    let secret_file = secret_dir.join("hidden.rs");
    fs::write(&secret_file, "fn top_secret() {}")?;

    let public_file = root.join("public.rs");
    fs::write(&public_file, "fn public_api() {}")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init.current_dir(root).arg("init").arg(".").assert().success();

    let mut cmd_gen = cargo_bin_cmd!("amdb");
    cmd_gen.current_dir(root).arg("generate").assert().success();

    let context_md = root.join(".amdb/context.md");
    let content = fs::read_to_string(context_md)?;

    assert!(content.contains("public_api"));
    assert!(!content.contains("top_secret"));

    Ok(())
}