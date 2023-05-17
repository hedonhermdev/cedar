use crate::collection::Collection;

pub mod duckdb;

pub trait Db {
    fn init(&self) -> Result<(), DbError>;

    fn get_collection(&self, name: &str) -> Result<Option<Collection>, DbError>;
    fn create_collection(&self, name: &str) -> Result<Collection, DbError>;
    fn get_or_create_collection(&self, name: &str) -> Result<Collection, DbError>;
    fn list_collections(&self) -> Result<Vec<Collection>, DbError>;
    fn get_collection_uuid_from_name(&self, name: &str) -> Result<Option<uuid::Uuid>, DbError>;
    fn update_collection(&self, uuid: uuid::Uuid, new_name: &str) -> Result<Collection, DbError>;
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
