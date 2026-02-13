use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub file_path: String,
    pub text: String,
    pub vector: Vec<f32>,
}

pub struct VectorStore {
    conn: Connection,
}

struct SearchCandidate {
    score: f64,
    record: VectorRecord,
}

impl VectorStore {
    pub fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("vectors.db");
        let conn = Connection::open(&db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vectors (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                text TEXT NOT NULL,
                vector BLOB NOT NULL
            )",
            [],
        )?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        Ok(Self { conn })
    }

    pub fn begin_transaction(&mut self) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION;", [])?;
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        self.conn.execute("COMMIT;", [])?;
        Ok(())
    }

    pub fn add(
        &mut self,
        file_path: String,
        id: String,
        text: String,
        vector: Vec<f32>,
    ) -> Result<()> {
        let vector_blob = bincode::serialize(&vector)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO vectors (id, file_path, text, vector) VALUES (?1, ?2, ?3, ?4)",
            params![id, file_path, text, vector_blob],
        )?;
        Ok(())
    }

    pub fn remove_by_file(&mut self, file_path: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM vectors WHERE file_path = ?1",
            params![file_path],
        )?;
        Ok(())
    }

    pub fn search(&self, query_vec: &[f32], limit: usize) -> Result<Vec<(f64, VectorRecord)>> {
        let mut stmt = self.conn.prepare("SELECT id, file_path, text, vector FROM vectors")?;

        let mut rows = stmt.query([])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let text: String = row.get(2)?;
            let vector_blob: Vec<u8> = row.get(3)?;
            records.push((id, file_path, text, vector_blob));
        }

        let mut evaluated: Vec<SearchCandidate> = records.into_par_iter()
            .filter_map(|(id, file_path, text, vector_blob)| {
                if let Ok(vector) = bincode::deserialize::<Vec<f32>>(&vector_blob) {
                    let score = cosine_similarity(query_vec, &vector);
                    Some(SearchCandidate {
                        score,
                        record: VectorRecord { id, file_path, text, vector }
                    })
                } else {
                    None
                }
            })
            .collect();

        evaluated.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        evaluated.truncate(limit);

        Ok(evaluated.into_iter().map(|c| (c.score, c.record)).collect())
    }

    pub fn save(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot_product / (norm_a * norm_b)) as f64
}