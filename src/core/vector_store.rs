use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::io::{BufReader, BufWriter};
use std::cmp::Ordering; // 추가됨

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub file_path: String,
    pub text: String,
    pub vector: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct VectorStore {
    records: HashMap<String, VectorRecord>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file_path = path.join("vectors.bin");
        if !file_path.exists() {
            return Ok(Self::new());
        }

        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        let store: VectorStore = bincode::deserialize_from(reader)?;

        Ok(store)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file_path = path.join("vectors.bin");

        let file = fs::File::create(file_path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)?;

        Ok(())
    }

    pub fn add(&mut self, id: String, file_path: String, text: String, vector: Vec<f32>) {
        self.records.insert(id.clone(), VectorRecord {
            id,
            file_path,
            text,
            vector,
        });
    }

    pub fn remove_by_file(&mut self, file_path: &str) {
        let keys_to_remove: Vec<String> = self.records.iter()
            .filter(|(_, record)| record.file_path == file_path)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            self.records.remove(&key);
        }
    }

    pub fn search(&self, query_vec: &[f32], limit: usize) -> Vec<(f64, VectorRecord)> {
        let mut results: Vec<(f64, VectorRecord)> = self.records.values()
            .map(|record| {
                let score = cosine_similarity(query_vec, &record.vector);
                (score, record.clone())
            })
            .collect();

        results.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal)
        });

        results.into_iter().take(limit).collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    (dot_product / (norm_a * norm_b)) as f64
}