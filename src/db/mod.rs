pub mod duckdb;

pub use self::duckdb::DuckDB;

pub(crate) mod model;

pub use model::CollectionModel;
use uuid::Uuid;

use crate::{Embedding, QueryResult};

use self::model::EmbeddingModel;

pub trait Db {
    fn init(&self) -> Result<(), DbError>;
    fn reset(&self) -> Result<(), DbError>;
    fn persist(&self) -> Result<(), DbError>;

    fn get_collection(&self, name: &str) -> Result<Option<CollectionModel>, DbError>;
    fn create_collection(&self, name: &str) -> Result<CollectionModel, DbError>;
    fn get_or_create_collection(&self, name: &str) -> Result<CollectionModel, DbError>;
    fn list_collections(&self) -> Result<Vec<CollectionModel>, DbError>;
    fn get_collection_uuid_from_name(&self, name: &str) -> Result<Option<uuid::Uuid>, DbError>;
    fn update_collection(
        &self,
        uuid: uuid::Uuid,
        new_name: &str,
    ) -> Result<CollectionModel, DbError>;

    fn add_embeddings(
        &self,
        collection_uuid: Uuid,
        embeddings: Vec<EmbeddingModel>,
    ) -> Result<(), DbError>;
    fn count_embeddings(&self, collection_uuid: Uuid) -> Result<usize, DbError>;

    fn get_embeddings(&self, collection_uuid: Uuid) -> Result<Vec<EmbeddingModel>, DbError>;

    fn query(
        &self,
        collection_uuid: Uuid,
        embeddings: &[Embedding],
        _where: serde_json::Value,
        k: usize,
    ) -> Result<Vec<Vec<QueryResult>>, DbError>;

    fn format_where(
        &self,
        where_map: &serde_json::Map<String, serde_json::Value>,
        result: &mut Vec<String>,
    ) -> Result<(), DbError>;
}

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to initialize DB: {0}")]
    DbInitError(Box<dyn std::error::Error>),

    #[error("Failed to execute SQL query: {0}")]
    SqlError(Box<dyn std::error::Error>),

    #[error("{0}")]
    UpdateError(String),

    #[error("{0}")]
    OperandError(String),

    #[error("{0}")]
    OperatorError(String),

    #[error("{0}")]
    InvalidValueError(String),
}
