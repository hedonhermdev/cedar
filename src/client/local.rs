use std::sync::Arc;

use crate::{collection::Collection, db::Db, embeddings::EmbeddingFunction};

use super::{Client, ClientError};

#[derive(Debug)]
pub struct LocalClient<D: Db, E: EmbeddingFunction> {
    db: Arc<D>,
    embedding_fn: Arc<E>,
}

impl<D: Db, E: EmbeddingFunction> Clone for LocalClient<D, E> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            embedding_fn: self.embedding_fn.clone(),
        }
    }
}

impl<D, E> LocalClient<D, E>
where
    D: Db,
    E: EmbeddingFunction,
{
    pub fn init(db: D, embedding_fn: E) -> Result<Self, ClientError> {
        Ok(Self {
            db: db.into(),
            embedding_fn: embedding_fn.into(),
        }
        .into())
    }
}

impl<D, E> Client for LocalClient<D, E>
where
    D: Db + 'static,
    E: EmbeddingFunction + 'static,
    LocalClient<D, E>: Clone,
{
    fn create_collection(&mut self, name: &str) -> Result<Collection, ClientError> {
        let client = Box::new(self.clone());
        let model = self.db.create_collection(name)?;
        let collection = Collection {
            name: model.name,
            uuid: model.uuid,
            client,
        };

        Ok(collection)
    }

    fn get_collection(&self, name: &str) -> Result<Option<Collection>, ClientError> {
        Ok(match self.db.get_collection(name)? {
            Some(model) => {
                let client = Box::new(self.clone());
                Some(Collection {
                    name: model.name,
                    uuid: model.uuid,
                    client,
                })
            }
            None => None,
        })
    }

    fn list_collection_names(&self) -> Result<Vec<String>, ClientError> {
        Ok(self
            .db
            .list_collections()?
            .into_iter()
            .map(|c| c.name)
            .collect())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::{duckdb::DuckDB, Db},
        embeddings::sentencetransformer::SentenceTransformerEmbeddings,
    };

    use super::Client;
    use super::LocalClient;

    #[test]
    pub fn test_create_collection_local() {
        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();

        let embedding_fn = SentenceTransformerEmbeddings::new();

        let mut client = LocalClient::init(db, embedding_fn).unwrap();

        let collection = client.create_collection("collection1").unwrap();

        let collection1 = client.get_collection("collection1").unwrap().unwrap();

        assert_eq!(collection.name, collection1.name);
        assert_eq!(collection.uuid, collection1.uuid);
    }
}
