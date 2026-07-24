use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;
use xxhash_rust::xxh3::xxh3_64;

struct McpClient {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    rx: std::sync::mpsc::Receiver<String>,
    next_id: i64,
}

impl McpClient {
    fn start(dir: &std::path::Path) -> Self {
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_amdb"))
            .current_dir(dir)
            .arg("serve")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn amdb serve");

        let stdin = child.stdin.take().expect("child stdin");
        let stdout = child.stdout.take().expect("child stdout");
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Self {
            child,
            stdin,
            rx,
            next_id: 1,
        }
    }

    fn send(&mut self, value: serde_json::Value) {
        use std::io::Write;
        let mut line = value.to_string();
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).expect("write stdin");
        self.stdin.flush().expect("flush stdin");
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }));

        loop {
            let line = self
                .rx
                .recv_timeout(std::time::Duration::from_secs(120))
                .expect("mcp response before timeout");
            let msg: serde_json::Value = serde_json::from_str(&line).expect("valid jsonrpc line");
            if msg.get("id").and_then(|v| v.as_i64()) == Some(id) {
                return msg;
            }
        }
    }

    fn initialize(&mut self) {
        self.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "amdb-contract-test", "version": "0.0.0"},
            }),
        );
        self.send(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        }));
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn test_cli_subcommands_and_flags_stable() {
    cargo_bin_cmd!("amdb")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("daemon"))
        .stdout(predicate::str::contains("generate"))
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("--verbose"));

    cargo_bin_cmd!("amdb")
        .arg("generate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--focus"))
        .stdout(predicate::str::contains("--depth"))
        .stdout(predicate::str::contains("--verbose"));

    cargo_bin_cmd!("amdb")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("[PATH]"));

    cargo_bin_cmd!("amdb")
        .arg("daemon")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("[PATH]"));
}

#[test]
fn test_mcp_tool_names_stable() {
    let temp_dir = TempDir::new().unwrap();
    let mut client = McpClient::start(temp_dir.path());
    client.initialize();

    let resp = client.request("tools/list", serde_json::json!({}));
    let tools = resp["result"]["tools"].as_array().expect("tools array");

    let mut names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("tool name"))
        .collect();
    names.sort_unstable();

    assert_eq!(
        names,
        vec!["amdb_focus", "amdb_get_context", "amdb_get_symbol"]
    );
}

#[test]
fn test_mcp_get_symbol_schema_stable() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(
        root.join("entry.rs"),
        "pub fn contract_entry_fn(x: i32) -> i32 {\n    contract_local_helper_fn();\n    contract_unique_target_fn();\n    contract_ambiguous_target_fn();\n    x\n}\nfn contract_local_helper_fn() {}\n",
    )
    .unwrap();
    fs::write(root.join("definer.rs"), "fn contract_unique_target_fn() {}").unwrap();
    fs::write(
        root.join("amb_one.rs"),
        "fn contract_ambiguous_target_fn() {}",
    )
    .unwrap();
    fs::write(
        root.join("amb_two.rs"),
        "fn contract_ambiguous_target_fn() {}",
    )
    .unwrap();
    fs::write(
        root.join("caller.rs"),
        "fn contract_outer_caller_fn() { contract_entry_fn(); }",
    )
    .unwrap();

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    let mut client = McpClient::start(root);
    client.initialize();

    let resp = client.request(
        "tools/call",
        serde_json::json!({"name": "amdb_get_symbol", "arguments": {"name": "contract_entry_fn"}}),
    );
    let result = &resp["result"];
    assert_ne!(result["isError"], true);

    let text = result["content"][0]["text"].as_str().expect("text content");
    let matches: serde_json::Value = serde_json::from_str(text).expect("json payload");
    let arr = matches.as_array().expect("array of matches");
    assert_eq!(arr.len(), 1);

    let m = &arr[0];
    assert_eq!(m["file"].as_str().expect("file is string"), "entry.rs");
    assert_eq!(
        m["name"].as_str().expect("name is string"),
        "contract_entry_fn"
    );
    assert!(m["kind"].is_string());
    assert_eq!(m["line"].as_i64().expect("line is integer"), 1);
    assert!(m["signature"]
        .as_str()
        .expect("signature is string")
        .contains("x: i32"));
    assert!(m["is_public"].as_bool().expect("is_public is boolean"));

    let callers = m["callers"].as_array().expect("callers is array");
    let outer = callers
        .iter()
        .find(|c| c["name"] == "contract_outer_caller_fn")
        .expect("outer caller present");
    assert_eq!(
        outer["file"].as_str().expect("caller file is string"),
        "caller.rs"
    );

    let callees = m["callees"].as_array().expect("callees is array");
    let by_name = |n: &str| {
        callees
            .iter()
            .find(|c| c["name"] == n)
            .unwrap_or_else(|| panic!("callee {} present", n))
    };

    let local = by_name("contract_local_helper_fn");
    assert_eq!(
        local["file"].as_str().expect("callee file is string"),
        "entry.rs"
    );
    assert_eq!(local["resolution"], "same-file");

    let unique = by_name("contract_unique_target_fn");
    assert_eq!(
        unique["file"].as_str().expect("callee file is string"),
        "definer.rs"
    );
    assert_eq!(unique["resolution"], "global-unique");

    let ambiguous = by_name("contract_ambiguous_target_fn");
    assert!(ambiguous["file"].is_null());
    assert_eq!(ambiguous["resolution"], "unresolved");
}

#[test]
fn test_config_keys_match_documented() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    let toml_block = readme
        .split("```toml")
        .nth(1)
        .expect("README documents an amdb.toml block")
        .split("```")
        .next()
        .unwrap();

    let mut documented_keys: Vec<&str> = toml_block
        .lines()
        .filter_map(|line| {
            let key = line.split('=').next()?.trim();
            if key.is_empty() {
                None
            } else {
                Some(key)
            }
        })
        .collect();
    documented_keys.sort_unstable();
    assert_eq!(documented_keys, vec!["db_path", "ignore_patterns"]);

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(
        root.join("amdb.toml"),
        "db_path = \".contract_docs_db\"\nignore_patterns = [\"contract_hidden_dir\"]\n",
    )
    .unwrap();

    let hidden = root.join("contract_hidden_dir");
    fs::create_dir(&hidden).unwrap();
    fs::write(hidden.join("secret.rs"), "fn contract_hidden_fn() {}").unwrap();
    fs::write(root.join("visible.rs"), "fn contract_visible_fn() {}").unwrap();

    cargo_bin_cmd!("amdb")
        .current_dir(root)
        .arg("init")
        .arg(".")
        .assert()
        .success();

    assert!(root.join(".contract_docs_db/context.db").exists());
    assert!(!root.join(".database").exists());

    let conn = Connection::open(root.join(".contract_docs_db/context.db")).unwrap();
    let visible: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'contract_visible_fn'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let hidden_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'contract_hidden_fn'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(visible, 1);
    assert_eq!(hidden_count, 0);
}

#[test]
fn test_migration_chain_from_each_prior_version() {
    for version in [0i32, 1, 2] {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let source = "fn migration_probe_current_fn() {}";
        fs::write(root.join("probe.rs"), source).unwrap();
        let hash = format!("{:016x}", xxh3_64(source.as_bytes()));

        fs::create_dir(root.join(".database")).unwrap();
        {
            let conn = Connection::open(root.join(".database/context.db")).unwrap();
            if version < 2 {
                conn.execute(
                    "CREATE TABLE symbols (
                        id INTEGER PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        name TEXT NOT NULL,
                        kind TEXT NOT NULL,
                        line INTEGER NOT NULL,
                        docstring TEXT
                    )",
                    [],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO symbols (file_path, name, kind, line, docstring)
                     VALUES ('legacy.rs', 'stale_legacy_sym', 'Function', 1, NULL)",
                    [],
                )
                .unwrap();
            } else {
                conn.execute(
                    "CREATE TABLE symbols (
                        id INTEGER PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        name TEXT NOT NULL,
                        kind TEXT NOT NULL,
                        line INTEGER NOT NULL,
                        docstring TEXT,
                        is_public INTEGER NOT NULL DEFAULT 1,
                        signature TEXT
                    )",
                    [],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO symbols (file_path, name, kind, line, docstring, is_public, signature)
                     VALUES ('legacy.rs', 'stale_legacy_sym', 'Function', 1, NULL, 0, NULL)",
                    [],
                )
                .unwrap();
            }
            conn.execute(
                "CREATE TABLE relationships (
                    id INTEGER PRIMARY KEY,
                    file_path TEXT NOT NULL,
                    caller TEXT NOT NULL,
                    callee TEXT NOT NULL
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO relationships (file_path, caller, callee)
                 VALUES ('legacy.rs', 'stale_legacy_sym', 'stale_callee')",
                [],
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE file_hashes (
                    file_path TEXT PRIMARY KEY,
                    hash TEXT NOT NULL
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO file_hashes (file_path, hash) VALUES ('probe.rs', ?1)",
                rusqlite::params![hash],
            )
            .unwrap();
            conn.execute_batch(&format!("PRAGMA user_version = {}", version))
                .unwrap();
        }

        cargo_bin_cmd!("amdb")
            .current_dir(root)
            .arg("init")
            .arg(".")
            .assert()
            .success();

        let conn = Connection::open(root.join(".database/context.db")).unwrap();

        let user_version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, 3, "from version {}", version);

        let stale: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE name = 'stale_legacy_sym'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stale, 0, "from version {}", version);

        let current: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE name = 'migration_probe_current_fn'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, 1, "from version {}", version);
    }
}
