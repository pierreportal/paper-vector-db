use crate::config::DEFAULT_EMBEDDNG_BATCH_SIZE;
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

pub struct Embedder {
    pub embedding_model: TextEmbedding,
}

#[allow(unused)]
pub enum BatchSize {
    None,
    Default,
    Custom(usize),
}

#[allow(unused)]
impl Embedder {
    pub fn new() -> Result<Self, String> {
        let fastembed_model = EmbeddingModel::AllMiniLML6V2;
        match TextEmbedding::try_new(TextInitOptions::new(fastembed_model)) {
            Ok(embedding_model) => Ok(Self { embedding_model }),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn embed(&mut self, document: &str, batch_size: BatchSize) -> Result<Vec<f32>, String> {
        let bs: Option<usize> = match batch_size {
            BatchSize::None => None,
            BatchSize::Default => Some(DEFAULT_EMBEDDNG_BATCH_SIZE),
            BatchSize::Custom(n) => Some(n),
        };

        match self.embedding_model.embed([document], bs) {
            Ok(embeddings) => Ok(embeddings[0].clone()),
            Err(e) => Err(e.to_string()),
        }
    }
}
