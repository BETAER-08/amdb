use rusqlite::{Connection, Result};

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            line INTEGER NOT NULL,
            docstring TEXT,       -- [추가] 주석/Docstring
            signature TEXT,       -- [추가] 타입 시그니처 (예: (x: int) -> int)
            is_public BOOLEAN,    -- [추가] 외부 공개 여부
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS routes (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            method TEXT NOT NULL,      -- GET, POST 등
            path TEXT NOT NULL,        -- /users/:id
            handler_name TEXT,         -- 연결된 함수 이름
            UNIQUE(method, path)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}