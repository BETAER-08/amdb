use rusqlite::{Connection, Result};
use serde::Serialize; // [추가] API 응답용
use crate::core::parser::CodeSymbol;
use crate::core::graph::DependencyGraph;

// [신규] 관계 데이터 운반용 구조체
#[derive(Debug, Serialize)]
pub struct Relationship {
    pub caller: String,
    pub callee: String,
}

pub struct ContextDb {
    conn: Connection,
}
// ... (impl ContextDb 시작)

impl ContextDb {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let db_path = path.join("context.db");
        let conn = Connection::open(db_path)?;
        
        // 스키마 초기화
        super::schema::init(&conn)?;
        
        Ok(Self { conn })
    }

    // [기존 기능] 심볼 저장
    pub fn save_symbols(&mut self, file_path: &str, symbols: &[CodeSymbol]) -> Result<()> {
        let tx = self.conn.transaction()?;
        
        tx.execute("DELETE FROM symbols WHERE file_path = ?", [file_path])?;
        
        {
            let mut stmt = tx.prepare(
                "INSERT INTO symbols (file_path, kind, name, line) VALUES (?, ?, ?, ?)"
            )?;
            
            for s in symbols {
                stmt.execute((file_path, &s.kind, &s.name, s.line))?;
            }
        }
        
        tx.commit()
    }

    // [복구됨] 심볼 조회 (이게 없어서 에러가 났었습니다)
    pub fn get_symbols(&self, file_path: &str) -> Result<Vec<CodeSymbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT kind, name, line FROM symbols WHERE file_path = ?"
        )?;
        
        let rows = stmt.query_map([file_path], |row| {
            Ok(CodeSymbol {
                kind: row.get(0)?,
                name: row.get(1)?,
                line: row.get(2)?,
            })
        })?;

        let mut symbols = Vec::new();
        for symbol in rows {
            symbols.push(symbol?);
        }
        
        Ok(symbols)
    }

    // [신규 기능] 관계 저장
    pub fn save_relationships(&mut self, file_path: &str, graph: &DependencyGraph) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute("DELETE FROM relationships WHERE source_file = ?", [file_path])?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO relationships (source_file, caller_name, callee_name) VALUES (?, ?, ?)"
            )?;

            for (caller_key, callees) in &graph.edges {
                let parts: Vec<&str> = caller_key.split("::").collect();
                if parts.len() < 2 { continue; }
                let caller_name = parts[1];

                for callee in callees {
                    stmt.execute((file_path, caller_name, callee))?;
                }
            }
        }

        tx.commit()
    }
// ... (save_relationships 밑에 추가)

    pub fn get_relationships(&self, file_path: &str) -> Result<Vec<Relationship>> {
        let mut stmt = self.conn.prepare(
            "SELECT caller_name, callee_name FROM relationships WHERE source_file = ?"
        )?;

        let rows = stmt.query_map([file_path], |row| {
            Ok(Relationship {
                caller: row.get(0)?,
                callee: row.get(1)?,
            })
        })?;

        let mut rels = Vec::new();
        for r in rows {
            rels.push(r?);
        }

        Ok(rels)
    }
} // impl 끝