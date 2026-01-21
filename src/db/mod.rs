pub mod schema;
pub mod query;

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct ContextDb {
    conn: Connection,
}

impl ContextDb {
    pub fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("context.db");
        let conn = Connection::open(db_path)?;
        
        schema::init(&conn)?;
        
        Ok(Self { conn })
    }

    pub fn get_connection(&mut self) -> &Connection {
        &self.conn
    }

    pub fn get_connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}