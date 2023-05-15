use duckdb::{Config, params};

use uuid::Uuid;

use crate::collection::Collection;

use super::{Db, DbError};

pub type DuckDBConfig = duckdb::Config;

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
        self.conn
            .execute(
                "CREATE TABLE collections (uuid STRING, name STRING, metadata STRING)",
                [],
            )?;

        Ok(())
    }

    fn init_embeddings_table(&self) -> Result<(), DbError> {
        self.conn
            .execute(
                "CREATE TABLE embeddings (collection_uuid STRING, uuid STRING, embedding DOUBLE[], document STRING, id STRING, metadata STRING)",
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

    fn get_collection(&self, name: &str) -> Result<Option<Collection>, DbError> {
        let mut sql = self.conn.prepare("SELECT * FROM collections WHERE name = ?")?;
        let mut collections = sql.query_map([name], |row| {
            Collection::try_from(row)
        })?;

        match collections.next() {
            Some(res) => match res {
                Ok(collection) => Ok(Some(collection)),
                Err(e) => Err(e.into())
            },
            None => Ok(None)
        }
    }

    fn create_collection(&self, name: &str) -> Result<Collection, DbError> {
        let collection = Collection {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
        };

        self.conn.execute("INSERT INTO collections (uuid, name, metadata) VALUES (?, ?, ?)", params![collection.uuid.urn().to_string(), collection.name, ""])?;

        Ok(collection)
    }

    fn get_or_create_collection(&self, name: &str) -> Result<Collection, DbError> {
        match self.get_collection(name)? {
            Some(collection) => Ok(collection),
            None => self.create_collection(name)
        }
    }

    fn list_collections(&self) -> Result<Vec<Collection>, DbError> {
        let mut stmt = self.conn.prepare("SELECT * FROM collections")?;
        let rows = stmt.query_map([], |row| {
            Collection::try_from(row)
        })?;

        let mut collections = vec![];
        for row in rows {
            collections.push(row?)
        }

        Ok(collections)
    }
}

impl From<duckdb::Error> for DbError {
    fn from(value: duckdb::Error) -> Self {
        DbError::SqlError(value.into())
    }
}

impl TryFrom<&duckdb::Row<'_>> for Collection {
    type Error = duckdb::Error;

    fn try_from(row: &duckdb::Row) -> Result<Self, Self::Error> {
        let uuid: String = row.get(0)?;
        Ok(Collection {
            uuid: Uuid::parse_str(&uuid).expect("invalid UUID found in database"),
            name: row.get(1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Db;

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
}