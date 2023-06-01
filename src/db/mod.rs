pub mod duckdb;
mod model;

pub use model::CollectionModel;
use uuid::Uuid;

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

    fn add_embeddings(&self, collection_uuid: Uuid, embeddings: Vec<EmbeddingModel>) -> Result<(), DbError>;
    fn count_embeddings(&self, collection_uuid: Uuid) -> Result<usize, DbError>;

}

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Failed to initialize DB: {0}")]
    DbInitError(Box<dyn std::error::Error>),

    #[error("Failed to execute SQL query: {0}")]
    SqlError(Box<dyn std::error::Error>),

    #[error("{0}")]
    UpdateError(String),
}
