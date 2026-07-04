use rusqlite::{Connection, Result};

const CURRENT_SCHEMA_VERSION: i32 = 2;

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
            callee TEXT NOT NULL
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
        Err(rusqlite::Error::SqliteFailure(_, _)) => {}
        Err(e) => return Err(e),
    }

    match conn.execute("ALTER TABLE symbols ADD COLUMN signature TEXT", []) {
        Ok(_) => {}
        Err(rusqlite::Error::SqliteFailure(_, _)) => {}
        Err(e) => return Err(e),
    }

    let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    let migrated = if version < CURRENT_SCHEMA_VERSION {
        conn.execute("DELETE FROM relationships", [])?;
        conn.execute_batch(&format!("PRAGMA user_version = {}", CURRENT_SCHEMA_VERSION))?;
        true
    } else {
        false
    };

    Ok(migrated)
}
