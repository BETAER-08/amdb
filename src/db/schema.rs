use rusqlite::{Connection, Result};

const CURRENT_SCHEMA_VERSION: i32 = 3;

pub fn init(conn: &Connection) -> Result<bool> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbols (
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
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS relationships (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            caller TEXT NOT NULL,
            callee TEXT NOT NULL,
            callee_file TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS warnings (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            kind TEXT NOT NULL,
            message TEXT NOT NULL,
            line INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_symbols_file_path ON symbols (file_path)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols (name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_relationships_caller ON relationships (caller)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_relationships_file_path ON relationships (file_path)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_warnings_file_path ON warnings (file_path)",
        [],
    )?;

    match conn.execute(
        "ALTER TABLE symbols ADD COLUMN is_public INTEGER NOT NULL DEFAULT 1",
        [],
    ) {
        Ok(_) => {}
        Err(e) if is_duplicate_column_error(&e) => {}
        Err(e) => return Err(e),
    }

    match conn.execute("ALTER TABLE symbols ADD COLUMN signature TEXT", []) {
        Ok(_) => {}
        Err(e) if is_duplicate_column_error(&e) => {}
        Err(e) => return Err(e),
    }

    match conn.execute("ALTER TABLE relationships ADD COLUMN callee_file TEXT", []) {
        Ok(_) => {}
        Err(e) if is_duplicate_column_error(&e) => {}
        Err(e) => return Err(e),
    }

    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    let migrated = if version < CURRENT_SCHEMA_VERSION {
        conn.execute("DELETE FROM relationships", [])?;
        conn.execute("DELETE FROM symbols", [])?;
        conn.execute_batch(&format!("PRAGMA user_version = {}", CURRENT_SCHEMA_VERSION))?;
        true
    } else {
        false
    };

    Ok(migrated)
}

fn is_duplicate_column_error(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.starts_with("duplicate column name")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_db_migrates_once() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init(&conn).unwrap());
        assert!(!init(&conn).unwrap());
    }

    #[test]
    fn duplicate_column_error_is_recognized() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE probe (id INTEGER PRIMARY KEY, x INTEGER)", [])
            .unwrap();
        let err = conn
            .execute("ALTER TABLE probe ADD COLUMN x INTEGER", [])
            .unwrap_err();
        assert!(is_duplicate_column_error(&err));
    }

    #[test]
    fn non_duplicate_error_is_not_swallowed() {
        let conn = Connection::open_in_memory().unwrap();
        let err = conn
            .execute("SELECT * FROM nonexistent_table_xyz", [])
            .unwrap_err();
        assert!(!is_duplicate_column_error(&err));
    }

    #[test]
    fn legacy_db_purges_stale_symbols_on_migration() {
        let conn = Connection::open_in_memory().unwrap();
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
            "INSERT INTO symbols (file_path, name, kind, line, docstring, is_public, signature)
             VALUES ('legacy.rs', 'stale_unique_sym', 'Function', 999, NULL, 0, NULL)",
            [],
        )
        .unwrap();
        conn.execute_batch("PRAGMA user_version = 1;").unwrap();

        let migrated = init(&conn).unwrap();
        assert!(migrated);

        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
    }
}
