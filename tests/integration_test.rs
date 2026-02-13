use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::env;
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
    fs::write(&config_path, "ignore_patterns = [\"secret_vault\"]\n")?;

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

#[test]
fn test_config_toml_override_db_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let config_path = root.join("amdb.toml");
    fs::write(&config_path, "db_path = \".custom_toml_db\"\nignore_patterns = [\"test_ignore\"]")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    assert!(root.join(".custom_toml_db").exists());
    Ok(())
}

#[test]
fn test_config_env_override_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init
        .current_dir(root)
        .env("AMDB_DB_PATH", ".custom_env_db")
        .arg("init")
        .arg(".")
        .assert()
        .success();

    assert!(root.join(".custom_env_db").exists());
    Ok(())
}

#[test]
fn test_depth_control_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let main_rs = root.join("main.rs");
    fs::write(&main_rs, "fn main() { func_a(); }")?;

    let a_rs = root.join("a.rs");
    fs::write(&a_rs, "fn func_a() { func_b(); }")?;

    let b_rs = root.join("b.rs");
    fs::write(&b_rs, "fn func_b() {}")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init.current_dir(root).arg("init").arg(".").assert().success();

    let mut cmd_depth0 = cargo_bin_cmd!("amdb");
    cmd_depth0.current_dir(root)
        .arg("generate")
        .arg("--focus").arg("main")
        .arg("--depth").arg("0")
        .assert().success();

    let context_md = root.join(".amdb/main.md");
    let content_0 = fs::read_to_string(&context_md)?;
    assert!(content_0.contains("main.rs"));
    assert!(!content_0.contains("a.rs"));
    assert!(!content_0.contains("b.rs"));

    let mut cmd_depth1 = cargo_bin_cmd!("amdb");
    cmd_depth1.current_dir(root)
        .arg("generate")
        .arg("--focus").arg("main")
        .arg("--depth").arg("1")
        .assert().success();

    let content_1 = fs::read_to_string(&context_md)?;
    assert!(content_1.contains("main.rs"));
    assert!(content_1.contains("a.rs"));
    assert!(!content_1.contains("b.rs"));

    let mut cmd_depth2 = cargo_bin_cmd!("amdb");
    cmd_depth2.current_dir(root)
        .arg("generate")
        .arg("--focus").arg("main")
        .arg("--depth").arg("2")
        .assert().success();

    let content_2 = fs::read_to_string(&context_md)?;
    assert!(content_2.contains("main.rs"));
    assert!(content_2.contains("a.rs"));
    assert!(content_2.contains("b.rs"));

    Ok(())
}
#[test]
fn test_hybrid_search_incoming_edge() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let def_rs = root.join("def.rs");
    fs::write(&def_rs, "fn target_func() {}")?;

    let user_rs = root.join("user.rs");
    fs::write(&user_rs, "fn main() { target_func(); }")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init.current_dir(root).arg("init").arg(".").assert().success();

    let mut cmd_gen = cargo_bin_cmd!("amdb");
    cmd_gen.current_dir(root)
        .arg("generate")
        .arg("--focus").arg("target_func")
        .arg("--depth").arg("1")
        .assert().success();

    let context_md = root.join(".amdb/target_func.md");
    let content = fs::read_to_string(&context_md)?;

    assert!(content.contains("def.rs"));
    assert!(content.contains("user.rs"));

    Ok(())
}