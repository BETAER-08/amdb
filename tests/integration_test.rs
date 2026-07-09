use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

fn open_context_db(root: &std::path::Path) -> Connection {
    Connection::open(root.join(".database/context.db")).expect("open context.db")
}

fn parse_mermaid_edges(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .filter(|line| line.contains("-->"))
        .filter_map(|line| {
            let trimmed = line.trim().trim_end_matches(';');
            let mut parts = trimmed.splitn(2, "-->");
            let left = parts.next()?.trim().to_string();
            let right = parts.next()?.trim().to_string();
            Some((left, right))
        })
        .collect()
}

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
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

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
    fs::write(
        &config_path,
        "db_path = \".custom_toml_db\"\nignore_patterns = [\"test_ignore\"]",
    )?;

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
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let mut cmd_depth0 = cargo_bin_cmd!("amdb");
    cmd_depth0
        .current_dir(root)
        .arg("generate")
        .arg("--focus")
        .arg("main")
        .arg("--depth")
        .arg("0")
        .assert()
        .success();

    let context_md = root.join(".amdb/main.md");
    let content_0 = fs::read_to_string(&context_md)?;
    assert!(content_0.contains("main.rs"));
    assert!(!content_0.contains("a.rs"));
    assert!(!content_0.contains("b.rs"));

    let mut cmd_depth1 = cargo_bin_cmd!("amdb");
    cmd_depth1
        .current_dir(root)
        .arg("generate")
        .arg("--focus")
        .arg("main")
        .arg("--depth")
        .arg("1")
        .assert()
        .success();

    let content_1 = fs::read_to_string(&context_md)?;
    assert!(content_1.contains("main.rs"));
    assert!(content_1.contains("a.rs"));
    assert!(!content_1.contains("b.rs"));

    let mut cmd_depth2 = cargo_bin_cmd!("amdb");
    cmd_depth2
        .current_dir(root)
        .arg("generate")
        .arg("--focus")
        .arg("main")
        .arg("--depth")
        .arg("2")
        .assert()
        .success();

    let content_2 = fs::read_to_string(&context_md)?;
    assert!(content_2.contains("main.rs"));
    assert!(content_2.contains("a.rs"));
    assert!(content_2.contains("b.rs"));

    Ok(())
}

#[test]
fn test_directional_graph_excludes_callers() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    std::fs::write(root.join("def.rs"), "fn probe_alpha_unique_fn() {}")?;
    std::fs::write(
        root.join("user.rs"),
        "fn probe_caller_unique_fn() { probe_alpha_unique_fn(); }",
    )?;

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
        .arg("--focus")
        .arg("probe_alpha_unique_fn")
        .arg("--depth")
        .arg("1")
        .assert()
        .success();

    let content = std::fs::read_to_string(root.join(".amdb/probe_alpha_unique_fn.md"))?;
    assert!(content.contains("def.rs"));
    assert!(!content.contains("user.rs"));

    Ok(())
}

#[test]
fn test_symbol_fields_persisted() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let src = root.join("lib.rs");
    std::fs::write(&src, "pub fn public_func(x: i32) -> i32 { x }")?;

    let mut cmd_init = cargo_bin_cmd!("amdb");
    cmd_init
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let mut cmd_gen = cargo_bin_cmd!("amdb");
    cmd_gen.current_dir(root).arg("generate").assert().success();

    let content = std::fs::read_to_string(root.join(".amdb/context.md"))?;
    assert!(content.contains("public_func"));

    Ok(())
}

#[test]
fn test_vector_store_save_no_panic() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let src = root.join("main.rs");
    std::fs::write(&src, "fn main() {}")?;

    let mut cmd = cargo_bin_cmd!("amdb");
    cmd.current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_depth_directional_no_reverse() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    std::fs::write(root.join("core_impl.rs"), "fn unique_core_fn() {}")?;
    std::fs::write(root.join("caller.rs"), "fn entry() { unique_core_fn(); }")?;
    std::fs::write(root.join("unrelated.rs"), "fn unrelated() { entry(); }")?;

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
        .arg("--focus")
        .arg("unique_core_fn")
        .arg("--depth")
        .arg("0")
        .assert()
        .success();

    let content = std::fs::read_to_string(root.join(".amdb/unique_core_fn.md"))?;
    assert!(content.contains("core_impl.rs"));
    assert!(!content.contains("unrelated.rs"));

    Ok(())
}

#[test]
fn test_symbol_line_number_is_exact() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("line_position_check.rs"),
        "\n\n\nfn line_check_unique_fn() {\n    ()\n}\n",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);
    let line: i64 = conn.query_row(
        "SELECT line FROM symbols WHERE name = ?1",
        rusqlite::params!["line_check_unique_fn"],
        |row| row.get(0),
    )?;

    assert_eq!(line, 4);
    Ok(())
}

#[test]
fn test_is_public_reflects_visibility() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("visibility_check.rs"),
        "pub fn alpha_unique_sym() {}\nfn beta_unique_sym() {}\n",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);

    let alpha_is_public: i64 = conn.query_row(
        "SELECT is_public FROM symbols WHERE name = ?1",
        rusqlite::params!["alpha_unique_sym"],
        |row| row.get(0),
    )?;
    let beta_is_public: i64 = conn.query_row(
        "SELECT is_public FROM symbols WHERE name = ?1",
        rusqlite::params!["beta_unique_sym"],
        |row| row.get(0),
    )?;

    assert_eq!(alpha_is_public, 1);
    assert_eq!(beta_is_public, 0);
    Ok(())
}

#[test]
fn test_signature_round_trip_excludes_body() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("signature_check.rs"),
        "pub fn sig_check_unique_fn(x: i32, y: i32) -> i32 {\n    let leaked_body_marker = 12345;\n    x + y + leaked_body_marker\n}\n",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);
    let signature: String = conn.query_row(
        "SELECT signature FROM symbols WHERE name = ?1",
        rusqlite::params!["sig_check_unique_fn"],
        |row| row.get(0),
    )?;

    assert!(signature.contains("x: i32, y: i32"));
    assert!(signature.contains("-> i32"));
    assert!(!signature.contains("leaked_body_marker"));
    Ok(())
}

#[test]
fn test_mermaid_graph_contains_cross_file_edge() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("caller_module.rs"),
        "fn alpha_unique_sym() { beta_unique_sym(); }",
    )?;
    fs::write(root.join("callee_module.rs"), "fn beta_unique_sym() {}")?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success();

    let content = fs::read_to_string(root.join(".amdb/context.md"))?;
    assert!(content.contains("```mermaid"));

    let has_edge_arrow_between = content.lines().any(|line| {
        line.contains("-->")
            && line.contains("alpha_unique_sym")
            && line.contains("beta_unique_sym")
    });

    assert!(
        has_edge_arrow_between,
        "expected a mermaid edge arrow between alpha_unique_sym and beta_unique_sym, got:\n{}",
        content
    );
    Ok(())
}

#[test]
fn test_scoped_path_call_does_not_corrupt_edge_identity() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("scoped_call_check.rs"),
        "fn scoped_call_unique_fn() {\n    let _m: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();\n}\n",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);
    let mut stmt = conn.prepare("SELECT caller, callee FROM relationships")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (caller, callee) = row?;
        assert!(
            !caller.contains("::"),
            "caller identity leaked a joined path: {}",
            caller
        );
        assert!(
            !callee.contains("::"),
            "callee identity leaked a joined path: {}",
            callee
        );
    }

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success();

    let content = fs::read_to_string(root.join(".amdb/context.md"))?;
    assert!(content.contains("scoped_call_unique_fn"));

    Ok(())
}

#[test]
fn test_mermaid_node_id_symmetry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("alpha_module.rs"),
        "fn alpha_unique_sym() { beta_unique_sym(); }",
    )?;
    fs::write(
        root.join("beta_module.rs"),
        "fn beta_unique_sym() { gamma_unique_sym(); }",
    )?;
    fs::write(root.join("gamma_module.rs"), "fn gamma_unique_sym() {}")?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success();

    let content = fs::read_to_string(root.join(".amdb/context.md"))?;
    let edges = parse_mermaid_edges(&content);

    let callee_side_beta_id = edges
        .iter()
        .find(|(left, _)| left.contains("alpha_unique_sym"))
        .map(|(_, right)| right.clone())
        .expect("alpha_unique_sym --> beta_unique_sym edge missing");

    let caller_side_beta_id = edges
        .iter()
        .find(|(left, _)| left.contains("beta_unique_sym"))
        .map(|(left, _)| left.clone())
        .expect("beta_unique_sym --> gamma_unique_sym edge missing");

    assert_eq!(
        callee_side_beta_id, caller_side_beta_id,
        "beta_unique_sym has two different node IDs depending on caller/callee position: {} vs {}",
        callee_side_beta_id, caller_side_beta_id
    );

    Ok(())
}

#[test]
fn test_ambiguous_callee_resolution_is_pinned() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("first_definer.rs"),
        "fn shared_name() { first_definer_marker_unique(); }\nfn first_definer_marker_unique() {}",
    )?;
    fs::write(root.join("second_definer.rs"), "fn shared_name() {}")?;
    fs::write(
        root.join("caller_of_shared.rs"),
        "fn caller_of_shared_unique() { shared_name(); }",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);

    let definer_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM symbols WHERE name = ?1",
        rusqlite::params!["shared_name"],
        |row| row.get(0),
    )?;
    assert_eq!(definer_count, 2);

    let callee: String = conn.query_row(
        "SELECT callee FROM relationships WHERE caller = ?1",
        rusqlite::params!["caller_of_shared_unique"],
        |row| row.get(0),
    )?;
    assert_eq!(callee, "shared_name");

    let matching_files: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT file_path) FROM symbols WHERE name = ?1",
        rusqlite::params!["shared_name"],
        |row| row.get(0),
    )?;
    assert_eq!(
        matching_files, 2,
        "callee identity for a same-named symbol in {} files is inherently ambiguous with the current String-only callee representation",
        matching_files
    );

    Ok(())
}

#[test]
fn test_duplicate_name_functions_are_split_by_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("alpha_module.rs"),
        "fn alpha_entry() { shared_dup_fn(); }\nfn shared_dup_fn() {}",
    )?;
    fs::write(
        root.join("beta_module.rs"),
        "fn beta_entry() { shared_dup_fn(); }\nfn shared_dup_fn() {}",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success();

    let content = fs::read_to_string(root.join(".amdb/context.md"))?;
    let edges = parse_mermaid_edges(&content);

    let alpha_callee_id = edges
        .iter()
        .find(|(left, _)| left.contains("alpha_entry"))
        .map(|(_, right)| right.clone())
        .expect("alpha_entry --> shared_dup_fn edge missing");

    let beta_callee_id = edges
        .iter()
        .find(|(left, _)| left.contains("beta_entry"))
        .map(|(_, right)| right.clone())
        .expect("beta_entry --> shared_dup_fn edge missing");

    assert_ne!(
        alpha_callee_id, beta_callee_id,
        "both shared_dup_fn definitions collapsed into one mermaid node: {}",
        alpha_callee_id
    );
    assert!(alpha_callee_id.contains("alpha_module"));
    assert!(beta_callee_id.contains("beta_module"));

    Ok(())
}

#[test]
fn test_callee_file_persisted_for_unique_symbol() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("definer_module.rs"),
        "fn unique_defined_once_fn() {}",
    )?;
    fs::write(
        root.join("caller_module.rs"),
        "fn unique_defined_once_fn_caller() { unique_defined_once_fn(); }",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);
    let callee_file: String = conn.query_row(
        "SELECT callee_file FROM relationships WHERE caller = ?1",
        rusqlite::params!["unique_defined_once_fn_caller"],
        |row| row.get(0),
    )?;

    assert_eq!(callee_file, "definer_module.rs");

    Ok(())
}

#[test]
fn test_generate_without_init_exits_nonzero() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    cargo_bin_cmd!("amdb")
        .current_dir(temp_dir.path())
        .arg("generate")
        .assert()
        .failure()
        .code(1);

    Ok(())
}

#[test]
fn test_successful_command_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    fs::write(temp_dir.path().join("main.rs"), "fn main() {}")?;

    cargo_bin_cmd!("amdb")
        .current_dir(temp_dir.path())
        .arg("init")
        .arg(".")
        .assert()
        .success()
        .code(0);

    Ok(())
}

#[test]
fn test_exit_code_visible_to_shell() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_amdb"))
        .current_dir(temp_dir.path())
        .arg("generate")
        .output()?;

    assert_eq!(output.status.code(), Some(1));

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("Database not found"));

    Ok(())
}

#[test]
fn test_all_subcommands_agree_on_custom_db_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(root.join("amdb.toml"), "db_path = \".custom_shared_db\"\n")?;
    fs::write(
        root.join("shared_path_check.rs"),
        "fn shared_path_unique_fn() {}",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    assert!(root.join(".custom_shared_db/context.db").exists());
    assert!(root.join(".custom_shared_db/vector/vectors.db").exists());
    assert!(!root.join(".database").exists());

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("generate")
        .assert()
        .success();

    assert!(!root.join(".database").exists());

    let content = fs::read_to_string(root.join(".amdb/context.md"))?;
    assert!(content.contains("shared_path_unique_fn"));

    Ok(())
}

#[test]
fn test_ambiguous_callee_marked_unresolved_or_split() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    fs::write(
        root.join("ambiguous_def_one.rs"),
        "fn ambiguous_dup_fn() {}",
    )?;
    fs::write(
        root.join("ambiguous_def_two.rs"),
        "fn ambiguous_dup_fn() {}",
    )?;
    fs::write(
        root.join("ambiguous_caller_module.rs"),
        "fn ambiguous_caller_fn() { ambiguous_dup_fn(); }",
    )?;

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let conn = open_context_db(root);
    let callee_file: Option<String> = conn.query_row(
        "SELECT callee_file FROM relationships WHERE caller = ?1",
        rusqlite::params!["ambiguous_caller_fn"],
        |row| row.get(0),
    )?;

    match callee_file {
        None => {}
        Some(file) => {
            assert!(
                file == "ambiguous_def_one.rs" || file == "ambiguous_def_two.rs",
                "callee_file {} does not match either defining file",
                file
            );
        }
    }

    Ok(())
}
