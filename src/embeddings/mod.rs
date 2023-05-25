use crate::Embeddings;

pub mod openai;
pub mod sentencetransformer;

pub trait EmbeddingFunction {
    fn embed<S: AsRef<str> + Sync>(self, texts: &[S]) -> Result<Embeddings, EmbeddingError>;
}

#[derive(thiserror::Error, Debug)]
#[error("failed to embed document")]
pub struct EmbeddingError {
    #[source]
    err: Box<dyn std::error::Error>,
}
