use anyhow::Result;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

pub struct EmbeddingEngine {
    model: TextEmbedding,
}

impl EmbeddingEngine {
    pub fn new() -> Result<Self> {
        let mut options = InitOptions::new(EmbeddingModel::BGESmallENV15);
        options.show_download_progress = true;

        let model = TextEmbedding::try_new(options)?;
        Ok(Self { model })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let documents = vec![text.to_string()];
        let embeddings = self.model.embed(documents, None)?;
        Ok(embeddings[0].clone())
    }
}