use anyhow::Result;
use rusqlite::params;
use crate::core::parser::CodeSymbol;
use super::ContextDb;

impl ContextDb {
    pub fn save_symbols(&mut self, file_path: &str, symbols: &[CodeSymbol]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM symbols WHERE file_path = ?1", params![file_path])?;
        
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols (file_path, kind, name, line) VALUES (?1, ?2, ?3, ?4)"
            )?;
            
            for sym in symbols {
                stmt.execute(params![file_path, sym.kind, sym.name, sym.line])?;
                count += 1;
            }
        }
        
        tx.commit()?;
        Ok(count)
    }

    pub fn get_symbols(&self, file_path: &str) -> Result<Vec<CodeSymbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, name, line FROM symbols WHERE file_path = ?1"
        )?;
        
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(CodeSymbol {
                kind: row.get(0)?,
                name: row.get(1)?,
                line: row.get(2)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn debug_print_all(&self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT file_path, kind, name FROM symbols")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for row in rows {
            let (path, kind, name) = row?;
            println!("[DB] {} :: {} -> {}", path, kind, name);
        }
        Ok(())
    }
}