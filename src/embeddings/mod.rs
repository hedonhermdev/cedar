use crate::Embedding;

pub mod openai;
pub mod sentencetransformer;

pub trait EmbeddingFunction {
    fn embed<I, S: AsRef<str> + Sync>(&self, texts: I) -> Result<Vec<Embedding>, EmbeddingError>
    where
    I: IntoIterator<Item = S>;
}

#[derive(thiserror::Error, Debug)]
#[error("failed to embed document")]
pub struct EmbeddingError {
    #[source]
    err: Box<dyn std::error::Error>,
}
