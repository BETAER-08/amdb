use crate::core::parser::{CodeSymbol, CodeWarning};
use crate::core::symbol::SymbolRef;
use crate::db::schema;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Relationship {
    pub caller: SymbolRef,
    pub callee: String,
}

pub struct ContextDb {
    conn: Connection,
    pub needs_reindex: bool,
}

impl ContextDb {
    pub fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("context.db");
        let conn = Connection::open(db_path)?;

        let needs_reindex = schema::init(&conn)?;

        Ok(Self { conn, needs_reindex })
    }

    pub fn save_symbols(&mut self, file_path: &str, symbols: &[CodeSymbol]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file_path],
        )?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols (file_path, name, kind, line, docstring, is_public, signature) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )?;

            for sym in symbols {
                stmt.execute(params![
                    file_path,
                    sym.name,
                    sym.kind,
                    sym.line,
                    sym.docstring,
                    sym.is_public as i64,
                    sym.signature
                ])?;
            }
        }
        tx.commit()
    }

    pub fn save_relationships(
        &mut self,
        file_path: &str,
        graph: &crate::core::graph::DependencyGraph,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM relationships WHERE file_path = ?1",
            params![file_path],
        )?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO relationships (file_path, caller, callee) VALUES (?1, ?2, ?3)",
            )?;

            for (caller, callees) in &graph.edges {
                for callee in callees {
                    stmt.execute(params![file_path, caller.name, callee])?;
                }
            }
        }
        tx.commit()
    }

    pub fn save_warnings(&mut self, file_path: &str, warnings: &[CodeWarning]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM warnings WHERE file_path = ?1",
            params![file_path],
        )?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO warnings (file_path, kind, message, line) VALUES (?1, ?2, ?3, ?4)",
            )?;

            for w in warnings {
                stmt.execute(params![file_path, w.kind, w.message, w.line])?;
            }
        }
        tx.commit()
    }

    pub fn remove_file_data(&mut self, file_path: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.execute(
            "DELETE FROM relationships WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.execute(
            "DELETE FROM warnings WHERE file_path = ?1",
            params![file_path],
        )?;
        tx.commit()
    }

    pub fn get_symbols(&self, file_path: &str) -> Result<Vec<CodeSymbol>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, kind, line, docstring, is_public, signature FROM symbols WHERE file_path = ?1")?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(CodeSymbol {
                name: row.get(0)?,
                kind: row.get(1)?,
                line: row.get(2)?,
                docstring: row.get(3)?,
                is_public: row.get::<_, i64>(4).unwrap_or(1) != 0,
                signature: row.get(5)?,
            })
        })?;

        let mut symbols = Vec::new();
        for sym in rows {
            symbols.push(sym?);
        }
        Ok(symbols)
    }

    #[allow(dead_code)]
    pub fn get_relationships(&self, file_path: &str) -> Result<Vec<Relationship>> {
        let mut stmt = self
            .conn
            .prepare("SELECT file_path, caller, callee FROM relationships WHERE file_path = ?1")?;
        let rows = stmt.query_map(params![file_path], |row| {
            let file: String = row.get(0)?;
            let caller: String = row.get(1)?;
            Ok(Relationship {
                caller: SymbolRef::new(file, caller),
                callee: row.get(2)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in rows {
            edges.push(edge?);
        }
        Ok(edges)
    }

    #[allow(dead_code)]
    pub fn get_warnings(&self, file_path: &str) -> Result<Vec<CodeWarning>> {
        let mut stmt = self
            .conn
            .prepare("SELECT kind, message, line FROM warnings WHERE file_path = ?1")?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(CodeWarning {
                kind: row.get(0)?,
                message: row.get(1)?,
                line: row.get(2)?,
            })
        })?;

        let mut warnings = Vec::new();
        for w in rows {
            warnings.push(w?);
        }
        Ok(warnings)
    }

    pub fn get_all_files(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT file_path FROM symbols ORDER BY file_path")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }
        Ok(files)
    }

    pub fn get_all_relationships(&self) -> Result<Vec<Relationship>> {
        let mut stmt = self
            .conn
            .prepare("SELECT file_path, caller, callee FROM relationships")?;
        let rows = stmt.query_map([], |row| {
            let file: String = row.get(0)?;
            let caller: String = row.get(1)?;
            Ok(Relationship {
                caller: SymbolRef::new(file, caller),
                callee: row.get(2)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in rows {
            edges.push(edge?);
        }
        Ok(edges)
    }
}
