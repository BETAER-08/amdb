use anyhow::Result;

#[cfg(feature = "embeddings")]
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

pub struct EmbeddingEngine {
    #[cfg(feature = "embeddings")]
    model: TextEmbedding,
}

impl EmbeddingEngine {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "embeddings")]
        {
            let mut options = InitOptions::new(EmbeddingModel::BGESmallENV15);
            options.show_download_progress = true;

            let model = TextEmbedding::try_new(options)?;
            Ok(Self { model })
        }
        
        #[cfg(not(feature = "embeddings"))]
        {
            Ok(Self {})
        }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        #[cfg(feature = "embeddings")]
        {
            let documents = vec![text.to_string()];
            let embeddings = self.model.embed(documents, None)?;
            Ok(embeddings[0].clone())
        }
        
        #[cfg(not(feature = "embeddings"))]
        {
            anyhow::bail!("Embedding feature is not available. On Windows, embeddings are not supported due to ONNX Runtime dependency limitations. On other platforms, rebuild with default features enabled.")
        }
    }
}
