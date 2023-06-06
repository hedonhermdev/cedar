use uuid::Uuid;

use crate::{
    collection::Collection, db::DbError, embeddings::EmbeddingError, Document, Embedding,
    QueryResult,
};

pub mod local;

pub trait Client {
    fn create_collection(&mut self, name: &str) -> Result<Collection, ClientError>;

    fn get_collection(&self, name: &str) -> Result<Option<Collection>, ClientError>;

    fn list_collection_names(&self) -> Result<Vec<String>, ClientError>;

    fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, ClientError>;

    fn add_documents(&self, collection_uuid: Uuid, docs: &[Document]) -> Result<(), ClientError>;

    fn query(
        &self,
        collection_uuid: Uuid,
        queries: &[&str],
        k: usize,
    ) -> Result<Vec<Vec<QueryResult>>, ClientError>;
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Failed to initialise Cedar Client: {0}")]
    InitError(Box<dyn std::error::Error>),

    #[error("Failed to perform db operation: {0}")]
    DbError(#[from] DbError),

    #[error("Embedding function failed to embed texts: {0}")]
    EmbeddingFnError(#[from] EmbeddingError),
}
