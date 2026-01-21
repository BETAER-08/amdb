use anyhow::Result;
use rusqlite::Connection;

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            line INTEGER NOT NULL,
            UNIQUE(file_path, name, kind)
        )",
        [],
    )?;
    Ok(())
}