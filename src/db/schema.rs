use rusqlite::{Connection, Result};

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            line INTEGER NOT NULL,
            docstring TEXT
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

    Ok(())
}