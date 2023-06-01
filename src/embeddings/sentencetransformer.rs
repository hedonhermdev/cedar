use std::fmt::Debug;

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType::AllMiniLmL6V2,
};

use crate::Embedding;

use super::{EmbeddingError, EmbeddingFunction};

pub struct SentenceTransformerEmbeddings {
    model: SentenceEmbeddingsModel,
}

impl Debug for SentenceTransformerEmbeddings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SentenceTransformerEmbeddings").finish()
    }
}

impl SentenceTransformerEmbeddings {
    pub fn new() -> Self {
        let model = SentenceEmbeddingsBuilder::remote(AllMiniLmL6V2)
            .with_device(tch::Device::cuda_if_available())
            .create_model()
            .expect("failed to load embedding model");

        Self { model }
    }
}

impl EmbeddingFunction for SentenceTransformerEmbeddings {
    fn embed<S: AsRef<str> + Sync>(self, texts: &[S]) -> Result<Vec<Embedding>, EmbeddingError> {
        self.model
            .encode(texts)
            .map(|v| v.into_iter().map(|x| Into::<Embedding>::into(x)).collect())
            .map_err(|e| EmbeddingError { err: e.into() })
    }
}

#[cfg(test)]
mod test {
    use super::SentenceTransformerEmbeddings;
    use crate::embeddings::EmbeddingFunction;

    #[test]
    pub fn test_sentence_transformer_embeddings() {
        let docs = vec!["hello, this is a sentence"];
        let embeddingfn = SentenceTransformerEmbeddings::new();

        let embeddings = &embeddingfn.embed(&docs).unwrap()[0];
        assert_eq!(embeddings.dim(), 384);
    }
}
