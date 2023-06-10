use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
};



use crate::{
    client::{Client, ClientError},
    index::Index,
    Document, Embedding, QueryResult,
};

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
    pub fn add_documents(&mut self, documents: &[Document]) -> Result<(), CollectionError> {
        validate_documents(documents)?;

        self.client.add_documents(self.uuid, documents)?;

        Ok(())
    }

    pub fn query_documents(
        &self,
        queries: &[&str],
        k: usize,
        _where: serde_json::Value
    ) -> Result<Vec<Vec<QueryResult>>, CollectionError> {
        Ok(self.client.query(self.uuid, queries, k)?)
    }
}

fn validate_documents(docs: &[Document]) -> Result<(), CollectionError> {
    has_dups(docs.iter().map(|d| d.id))
        .then_some(())
        .ok_or(CollectionError::DuplicateError)
}

fn validate_embeddings(
    embeddings: &[Embedding],
    dim: Option<usize>,
) -> Result<(), CollectionError> {
    match dim {
        Some(dim) => embeddings
            .iter()
            .all(|e| e.dim() == dim)
            .then_some(())
            .ok_or(CollectionError::DimensionError),
        None => embeddings
            .iter()
            .all(|e| e.dim() == embeddings[0].dim())
            .then_some(())
            .ok_or(CollectionError::DimensionError),
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

    #[error("Client operation failed: {0}")]
    ClientError(#[from] ClientError),
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use uuid::Uuid;

    use crate::{
        client::{local::LocalClient, Client},
        db::{duckdb::DuckDB, Db},
        embeddings::sentencetransformer::SentenceTransformerEmbeddings,
        Document,
    };

    #[test]
    pub fn test_collection() {
        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();

        let embedding_fn = SentenceTransformerEmbeddings::new();

        let mut client = LocalClient::init(db, embedding_fn).unwrap();

        let mut collection = client.create_collection("collection1").unwrap();

        let docs = vec![Document {
            text: "hello world!".to_string(),
            metadata: json!({}),
            id: Uuid::new_v4(),
        }];

        collection.add_documents(&docs).unwrap();

        let res = collection.query_documents(&["hello"], 1, json!({ "source": "notion" })).unwrap();
        assert_eq!(res[0][0].text, docs[0].text);

        dbg!(res);
    }
}
