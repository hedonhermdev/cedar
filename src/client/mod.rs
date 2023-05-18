use crate::{collection::Collection, db::DbError};

pub mod local;

pub trait Client {
    fn create_collection(&mut self, name: &str) -> Result<Collection, ClientError>;

    fn get_collection(&self, name: &str) -> Result<Option<Collection>, ClientError>;

    fn list_collection_names(&self) -> Result<Vec<String>, ClientError>;
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Failed to initialise Cedar Client: {0}")]
    InitError(Box<dyn std::error::Error>),

    #[error("Failed to perform db operation: {0}")]
    DbError(#[from] DbError),
}
