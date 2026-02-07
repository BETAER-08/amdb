use rusqlite::{params, Connection, Result};
use std::path::Path;
use crate::core::parser::{CodeSymbol, CodeWarning};
use crate::db::schema;

pub struct Relationship {
    pub caller: String,
    pub callee: String,
}

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

    pub fn save_symbols(&mut self, file_path: &str, symbols: &[CodeSymbol]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM symbols WHERE file_path = ?1", params![file_path])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols (file_path, name, kind, line, docstring) VALUES (?1, ?2, ?3, ?4, ?5)"
            )?;

            for sym in symbols {
                stmt.execute(params![file_path, sym.name, sym.kind, sym.line, sym.docstring])?;
            }
        }
        tx.commit()
    }

    pub fn save_relationships(&mut self, file_path: &str, graph: &crate::core::graph::DependencyGraph) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM relationships WHERE file_path = ?1", params![file_path])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO relationships (file_path, caller, callee) VALUES (?1, ?2, ?3)"
            )?;

            for (caller, callees) in &graph.edges {
                for callee in callees {
                    stmt.execute(params![file_path, caller, callee])?;
                }
            }
        }
        tx.commit()
    }

    pub fn save_warnings(&mut self, file_path: &str, warnings: &[CodeWarning]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM warnings WHERE file_path = ?1", params![file_path])?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO warnings (file_path, kind, message, line) VALUES (?1, ?2, ?3, ?4)"
            )?;

            for w in warnings {
                stmt.execute(params![file_path, w.kind, w.message, w.line])?;
            }
        }
        tx.commit()
    }

    // 추가됨: 파일 관련 모든 데이터 삭제
    pub fn remove_file_data(&mut self, file_path: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM symbols WHERE file_path = ?1", params![file_path])?;
        tx.execute("DELETE FROM relationships WHERE file_path = ?1", params![file_path])?;
        tx.execute("DELETE FROM warnings WHERE file_path = ?1", params![file_path])?;
        tx.commit()
    }

    pub fn get_symbols(&self, file_path: &str) -> Result<Vec<CodeSymbol>> {
        let mut stmt = self.conn.prepare("SELECT name, kind, line, docstring FROM symbols WHERE file_path = ?1")?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(CodeSymbol {
                name: row.get(0)?,
                kind: row.get(1)?,
                line: row.get(2)?,
                docstring: row.get(3)?,
                is_public: true,
                signature: None,
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
        let mut stmt = self.conn.prepare("SELECT caller, callee FROM relationships WHERE file_path = ?1")?;
        let rows = stmt.query_map(params![file_path], |row| {
            Ok(Relationship {
                caller: row.get(0)?,
                callee: row.get(1)?,
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
        let mut stmt = self.conn.prepare("SELECT kind, message, line FROM warnings WHERE file_path = ?1")?;
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
        let mut stmt = self.conn.prepare("SELECT DISTINCT file_path FROM symbols ORDER BY file_path")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }
        Ok(files)
    }

    pub fn get_all_relationships(&self) -> Result<Vec<Relationship>> {
        let mut stmt = self.conn.prepare("SELECT caller, callee FROM relationships")?;
        let rows = stmt.query_map([], |row| {
            Ok(Relationship {
                caller: row.get(0)?,
                callee: row.get(1)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in rows {
            edges.push(edge?);
        }
        Ok(edges)
    }
}