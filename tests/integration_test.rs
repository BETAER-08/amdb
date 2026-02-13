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

        // SECRET_KEY = "AKIA1234567890ABCDEF" (This comment should be ignored)
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
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

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
fn test_config_exclude_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let config_content = r#"
server_port = 3000
exclude_patterns = ["secret_data", "ignored.rs"]
"#;
    fs::write(root.join("amdb.toml"), config_content)?;

    fs::create_dir(root.join("secret_data"))?;
    fs::write(root.join("secret_data/api_keys.rs"), "fn get_stripe_key() {}")?;
    fs::write(root.join("ignored.rs"), "fn ignored_func() {}")?;
    fs::write(root.join("public.rs"), "fn public_login() {}")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let mut cmd_gen = cargo_bin_cmd!("amdb");
    cmd_gen
        .current_dir(root)
        .arg("generate")
        .assert()
        .success();

    let context_md = root.join(".amdb/context.md");
    let content = fs::read_to_string(context_md)?;

    assert!(content.contains("public_login"));
    assert!(!content.contains("get_stripe_key"));
    assert!(!content.contains("ignored_func"));

    Ok(())
}