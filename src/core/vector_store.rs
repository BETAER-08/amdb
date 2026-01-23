use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
        let file_path = path.join("vectors.json");
        if !file_path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(file_path)?;
        let store = serde_json::from_str(&content)?;
        Ok(store)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file_path = path.join("vectors.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(file_path, content)?;
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

    pub fn search(&self, query_vec: &[f32], limit: usize) -> Vec<(f64, VectorRecord)> {
        let mut results: Vec<(f64, VectorRecord)> = self.records.values()
            .map(|record| {
                let score = cosine_similarity(query_vec, &record.vector);
                (score, record.clone())
            })
            .collect();

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        results.into_iter().take(limit).collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    (dot_product / (norm_a * norm_b)) as f64
}