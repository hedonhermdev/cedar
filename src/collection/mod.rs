use std::fmt::Debug;

use serde_json::Value;

use crate::{client::Client, Documents, Embeddings};

pub struct Collection {
    pub(crate) client: Box<dyn Client>,
    pub(crate) uuid: uuid::Uuid,
    pub(crate) name: String,
}

impl Debug for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("collection")
            .field("uuid", &self.uuid)
            .field("name", &self.name)
            .finish()
    }
}

impl Collection {
    pub fn add_embeddings(_embeddings: Embeddings) -> Result<(), CollectionError> {
        todo!()
    }

    pub fn add_documents<S>(_documents: Documents<S>) -> Result<(), CollectionError> {
        todo!()
    }

    pub fn query<'a, S>(
        _queries: &'a [&'a str],
        _filter: Value,
    ) -> Result<Documents<S>, CollectionError> {
        todo!()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CollectionError {}
