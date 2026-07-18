use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct EmbeddingEngine {
    model: TextEmbedding,
    calls: AtomicU64,
}

impl EmbeddingEngine {
    pub fn new() -> Result<Self> {
        let mut options = InitOptions::new(EmbeddingModel::BGESmallENV15);
        options.show_download_progress = true;

        let model = TextEmbedding::try_new(options)?;
        Ok(Self {
            model,
            calls: AtomicU64::new(0),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let documents = vec![text.to_string()];
        let embeddings = self.model.embed(documents, None)?;
        Ok(embeddings[0].clone())
    }

    pub fn call_count(&self) -> u64 {
        self.calls.load(Ordering::Relaxed)
    }
}
