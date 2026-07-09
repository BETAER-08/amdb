use crate::core::parser::{CodeSymbol, CodeWarning};
use crate::core::symbol::SymbolRef;
use crate::db::schema;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Relationship {
    pub caller: SymbolRef,
    pub callee: String,
    pub callee_file: Option<String>,
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

        Ok(Self {
            conn,
            needs_reindex,
        })
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
        let mut stmt = self.conn.prepare(
            "SELECT file_path, caller, callee, callee_file FROM relationships WHERE file_path = ?1",
        )?;
        let rows = stmt.query_map(params![file_path], |row| {
            let file: String = row.get(0)?;
            let caller: String = row.get(1)?;
            Ok(Relationship {
                caller: SymbolRef::new(file, caller),
                callee: row.get(2)?,
                callee_file: row.get(3)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in rows {
            edges.push(edge?);
        }
        Ok(edges)
    }

    pub fn update_relationship_callee_files(
        &mut self,
        file_path: &str,
        resolved: &[(String, String, Option<String>)],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "UPDATE relationships SET callee_file = ?1 WHERE file_path = ?2 AND caller = ?3 AND callee = ?4",
            )?;

            for (caller, callee, callee_file) in resolved {
                stmt.execute(params![callee_file, file_path, caller, callee])?;
            }
        }
        tx.commit()
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

    pub fn get_symbols_by_name(&self, name: &str) -> Result<Vec<(String, CodeSymbol)>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_path, name, kind, line, docstring, is_public, signature FROM symbols WHERE name = ?1 ORDER BY file_path, line",
        )?;
        let rows = stmt.query_map(params![name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                CodeSymbol {
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    line: row.get(3)?,
                    docstring: row.get(4)?,
                    is_public: row.get::<_, i64>(5).unwrap_or(1) != 0,
                    signature: row.get(6)?,
                },
            ))
        })?;

        let mut symbols = Vec::new();
        for sym in rows {
            symbols.push(sym?);
        }
        Ok(symbols)
    }

    pub fn get_callers_of(&self, callee: &str, callee_file: &str) -> Result<Vec<SymbolRef>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT file_path, caller FROM relationships WHERE callee = ?1 AND (callee_file = ?2 OR callee_file IS NULL) ORDER BY file_path, caller",
        )?;
        let rows = stmt.query_map(params![callee, callee_file], |row| {
            Ok(SymbolRef::new(
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        })?;

        let mut callers = Vec::new();
        for caller in rows {
            callers.push(caller?);
        }
        Ok(callers)
    }

    pub fn get_callees_of(
        &self,
        file: &str,
        caller: &str,
    ) -> Result<Vec<(String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT callee, callee_file FROM relationships WHERE file_path = ?1 AND caller = ?2 ORDER BY callee",
        )?;
        let rows = stmt.query_map(params![file, caller], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;

        let mut callees = Vec::new();
        for callee in rows {
            callees.push(callee?);
        }
        Ok(callees)
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
            .prepare("SELECT file_path, caller, callee, callee_file FROM relationships")?;
        let rows = stmt.query_map([], |row| {
            let file: String = row.get(0)?;
            let caller: String = row.get(1)?;
            Ok(Relationship {
                caller: SymbolRef::new(file, caller),
                callee: row.get(2)?,
                callee_file: row.get(3)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in rows {
            edges.push(edge?);
        }
        Ok(edges)
    }
}
