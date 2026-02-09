use assert_cmd::Command;
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

    let mut cmd_init = Command::cargo_bin("amdb")?;

    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project indexed successfully"));

    assert!(root.join(".database").exists());
    assert!(root.join(".database/vector/vectors.db").exists());

    let mut cmd_gen = Command::cargo_bin("amdb")?;

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

    let mut cmd_init = Command::cargo_bin("amdb")?;
    cmd_init.current_dir(root).arg("init").arg(".").assert().success();

    let mut cmd_focus = Command::cargo_bin("amdb")?;
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