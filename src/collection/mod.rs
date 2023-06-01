use std::{fmt::{Debug, Display}, hash::Hash, collections::HashSet};

use serde_json::Value;

use crate::{client::Client, index::Index, Documents, Document, Embedding};

pub struct Collection {
    pub(crate) client: Box<dyn Client>,
    pub(crate) index: Index,
    pub(crate) uuid: uuid::Uuid,
    pub(crate) name: String,
    pub(crate) dim: Option<usize>,
}

impl Debug for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("collection")
            .field("uuid", &self.uuid)
            .field("name", &self.name)
            .finish()
    }
}

impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("collection")
            .field("uuid", &self.uuid)
            .field("name", &self.name)
            .finish()
    }
}

impl Collection {
    pub fn add_embeddings(&mut self, embeddings: &[Embedding]) -> Result<(), CollectionError> {
        validate_embeddings(embeddings, self.dim)?;

        todo!()
    }

    pub fn add_documents(&mut self, documents: &[Document]) -> Result<(), CollectionError> {
        validate_documents(documents)?;
        todo!()
    }

    pub fn query<'a>(
        _queries: &'a [&'a str],
        _filter: Value,
    ) -> Result<Documents, CollectionError> {
        todo!()
    }
}

fn validate_documents(docs: &[Document]) -> Result<(), CollectionError> {
    has_dups(docs.iter().map(|d| d.id)).then_some(()).ok_or(CollectionError::DuplicateError)
}

fn validate_embeddings(embeddings: &[Embedding], dim: Option<usize>) -> Result<(), CollectionError> {
    match dim {
        Some(dim) => embeddings.iter().all(|e| e.dim() == dim).then_some(()).ok_or(CollectionError::DimensionError),
        None => embeddings.iter().all(|e| e.dim() == embeddings[0].dim()).then_some(()).ok_or(CollectionError::DimensionError),
    }
}

fn has_dups<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}

#[derive(thiserror::Error, Debug)]
pub enum CollectionError {
    #[error("Found duplicates in given set of documents")]
    DuplicateError,

    #[error("Invalid dimensions for given set of embeddings")]
    DimensionError,
}
