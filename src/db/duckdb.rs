use duckdb::{params, types::FromSql, Config};

use uuid::Uuid;

use crate::Embedding;

use super::{model::{CollectionModel, EmbeddingModel}, Db, DbError};

pub type DuckDBConfig = duckdb::Config;

#[derive(Debug)]
pub struct DuckDB {
    conn: duckdb::Connection,
}

impl DuckDB {
    pub fn new(config: Config) -> Result<Self, DbError> {
        let conn = duckdb::Connection::open_in_memory_with_flags(config)
            .map_err(|e| DbError::DbInitError(e.into()))?;

        Ok(DuckDB { conn })
    }

    fn init_collections_table(&self) -> Result<(), DbError> {
        self.conn.execute(
            "CREATE TABLE collections (uuid STRING, name STRING, metadata STRING)",
            [],
        )?;

        Ok(())
    }

    fn init_embeddings_table(&self) -> Result<(), DbError> {
        self.conn
        .execute(
            "CREATE TABLE embeddings (collection_uuid STRING, uuid STRING, embedding JSON, text STRING, metadata STRING)",
            []
        )?;

        Ok(())
    }
}

impl Db for DuckDB {
    fn init(&self) -> Result<(), DbError> {
        self.init_collections_table()?;
        self.init_embeddings_table()?;

        self.conn.execute("LOAD 'json';", [])?;

        Ok(())
    }

    fn get_collection(&self, name: &str) -> Result<Option<CollectionModel>, DbError> {
        let mut sql = self
            .conn
            .prepare("SELECT * FROM collections WHERE name = ?")?;
        let mut collections = sql.query_map([name], |row| CollectionModel::try_from(row))?;

        match collections.next() {
            Some(res) => match res {
                Ok(collection) => Ok(Some(collection)),
                Err(e) => Err(e.into()),
            },
            None => Ok(None),
        }
    }

    fn create_collection(&self, name: &str) -> Result<CollectionModel, DbError> {
        let collection = CollectionModel {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            metadata: "".into(),
        };

        self.conn.execute(
            "INSERT INTO collections (uuid, name, metadata) VALUES (?, ?, ?)",
            params![collection.uuid.urn().to_string(), name, ""],
        )?;

        Ok(collection)
    }

    fn get_or_create_collection(&self, name: &str) -> Result<CollectionModel, DbError> {
        match self.get_collection(name)? {
            Some(collection) => Ok(collection),
            None => self.create_collection(name),
        }
    }

    fn list_collections(&self) -> Result<Vec<CollectionModel>, DbError> {
        let mut stmt = self.conn.prepare("SELECT * FROM collections")?;
        let rows = stmt.query_map([], |row| CollectionModel::try_from(row))?;

        let mut collections = vec![];
        for row in rows {
            collections.push(row?)
        }

        Ok(collections)
    }

    fn get_collection_uuid_from_name(&self, name: &str) -> Result<Option<Uuid>, DbError> {
        match self.get_collection(name)? {
            Some(collection) => Ok(Some(collection.uuid)),
            None => Ok(None),
        }
    }

    fn update_collection(&self, uuid: Uuid, new_name: &str) -> Result<CollectionModel, DbError> {
        match self.get_collection_uuid_from_name(new_name)? {
            Some(collection_uuid) => {
                if collection_uuid != uuid {
                    return Err(DbError::UpdateError(String::from(
                        "Collection with new name already exists",
                    )));
                }

                self.conn.execute(
                    "UPDATE collections SET name = ? WHERE uuid = ?",
                    params![new_name, uuid.urn().to_string()],
                )?;
            }
            None => {
                self.conn.execute(
                    "UPDATE collections SET name = ? WHERE uuid = ?",
                    params![new_name, uuid.urn().to_string()],
                )?;
            }
        };

        match self.get_collection(new_name)? {
            Some(updated_collection) => Ok(updated_collection),
            None => Err(DbError::UpdateError(String::from(
                "Failed to fetch updated collection",
            ))),
        }
    }

    fn add_embeddings(&self, collection_uuid: Uuid, embeddings: Vec<EmbeddingModel>) -> Result<(), DbError> {
        let mut stmt = self.conn.prepare("INSERT INTO embeddings (collection_uuid, uuid, embedding, metadata, text) VALUES (?,?,?,?,?)")?;

        
        embeddings.iter().try_for_each(|e| {
            let v = params![collection_uuid.urn().to_string(), e.uuid.urn().to_string(), serde_json::ser::to_vec(&e.embedding).expect("failed to serialize vec"), e.metadata, e.text];

            stmt.execute(v)?;

            Ok::<(), DbError>(())
        })?;
        
        Ok(())
    }

    fn count_embeddings(&self, collection_uuid: Uuid) -> Result<usize, DbError> {
        let mut stmt = self.conn.prepare("SELECT COUNT() FROM embeddings WHERE collection_uuid = ?")?;

        Ok(stmt.query_row(params![collection_uuid.urn().to_string()], |row| row.get(0))?)
    }

    fn reset(&self) -> Result<(), DbError> {
        todo!()
    }

    fn persist(&self) -> Result<(), DbError> {
        todo!()
    }
}

impl From<duckdb::Error> for DbError {
    fn from(value: duckdb::Error) -> Self {
        DbError::SqlError(value.into())
    }
}

impl TryFrom<&duckdb::Row<'_>> for CollectionModel {
    type Error = duckdb::Error;

    fn try_from(row: &duckdb::Row) -> Result<Self, Self::Error> {
        let uuid: String = row.get(0)?;
        Ok(CollectionModel {
            uuid: Uuid::parse_str(&uuid).expect("invalid UUID found in database"),
            name: row.get(1)?,
            metadata: "".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::db::{Db, model::EmbeddingModel};

    use super::DuckDB;

    #[test]
    pub fn test_create_collection() {
        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();

        let name = "collection1";

        let collection_create = db.create_collection(name).unwrap();

        let collection_get = db.get_collection(name).unwrap().unwrap();

        assert_eq!(collection_create, collection_get);
    }

    #[test]
    pub fn test_list_collections() {
        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();

        let mut collections = vec![];

        collections.push(db.create_collection("collection1").unwrap());
        collections.push(db.create_collection("collection2").unwrap());

        let mut listed_collections = db.list_collections().unwrap();

        collections.sort_by_cached_key(|c| c.uuid);
        listed_collections.sort_by_cached_key(|c| c.uuid);

        assert_eq!(collections, listed_collections)
    }

    #[test]
    pub fn test_get_collection_uuid_from_name() {
        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();

        let name = "collection1";

        let collection_create_uuid = db.create_collection(name).unwrap().uuid;

        let collection_get_uuid = db.get_collection(name).unwrap().unwrap().uuid;

        assert_eq!(collection_create_uuid, collection_get_uuid);
    }

    #[test]
    pub fn test_update_collection() {
        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();

        let name = "collection1";

        let collection_create_uuid = db.create_collection(name).unwrap().uuid;
        let new_name = "new_collection1";

        let updated_collection = db
            .update_collection(collection_create_uuid, new_name)
            .unwrap();
        let updated_name = updated_collection.name;

        assert_eq!(new_name, updated_name);
    }

    #[test]
    pub fn test_add_embeddings() {
        let db = DuckDB::new(Default::default()).unwrap();

        db.init().unwrap();

        let collection = db.create_collection("collection1").unwrap();
        let collection_uuid = collection.uuid;
        
        let e_model = EmbeddingModel {
            embedding: vec![0.0; 384],
            uuid: Uuid::new_v4(),
            metadata: serde_json::json!({"id": "102"}),
            text: "hello, this is a sentence".to_string()
        };

        db.add_embeddings(collection_uuid, vec![e_model]).unwrap();
    }

    #[test]
    pub fn test_count_embeddings() {
        let db = DuckDB::new(Default::default()).unwrap();

        db.init().unwrap();

        let collection = db.create_collection("collection1").unwrap();
        let collection_uuid = collection.uuid;
        dbg!(collection_uuid);
        
        assert_eq!(0, db.count_embeddings(collection_uuid).unwrap());
        
        let e_model = EmbeddingModel {
            embedding: vec![0.0; 384],
            uuid: Uuid::new_v4(),
            metadata: serde_json::json!({"id": "102"}),
            text: "hello, this is a sentence".to_string()
        };

        db.add_embeddings(collection_uuid, vec![e_model]).unwrap();

        assert_eq!(1, db.count_embeddings(collection_uuid).unwrap());
    }
}
