use rusqlite::{Connection, Result};

pub fn init(conn: &Connection) -> Result<()> {
    // 1. 기존 심볼 테이블
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS relationships (
            id INTEGER PRIMARY KEY,
            source_file TEXT NOT NULL,
            caller_name TEXT NOT NULL,
            callee_name TEXT NOT NULL,
            UNIQUE(source_file, caller_name, callee_name)
        )",
        [],
    )?;

    Ok(())
}