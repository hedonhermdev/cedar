use std::{cell::RefCell, collections::HashMap};

use duckdb::{params, Config};

use uuid::Uuid;

use crate::{
    index::{Index, IndexEntry},
    Embedding, QueryResult,
};

use super::{
    model::{CollectionModel, EmbeddingModel},
    Db, DbError,
};

use serde_json::Value;

pub type DuckDBConfig = duckdb::Config;

#[derive(Debug)]
pub struct DuckDB {
    conn: duckdb::Connection,
    index: RefCell<HashMap<Uuid, Index>>,
}

impl DuckDB {
    pub fn new(config: Config) -> Result<Self, DbError> {
        let conn = duckdb::Connection::open_in_memory_with_flags(config)
            .map_err(|e| DbError::DbInitError(e.into()))?;
        let index = HashMap::new().into();

        Ok(DuckDB { conn, index })
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

    fn get_nearest_neighbors(
        &self,
        collection_uuid: Uuid,
        embeddings: &[Embedding],
        k: usize,
    ) -> Result<Vec<Vec<(Uuid, f32)>>, DbError> {
        let index = self.index.borrow();
        let idx = index
            .get(&collection_uuid)
            .expect("index does not exist for collection");

        // TODO: add metadata filtering
        let mut stmt = self
            .conn
            .prepare("SELECT uuid FROM embeddings WHERE collection_uuid = ?")?;

        let mapped_rows = stmt.query_map([collection_uuid.urn().to_string()], |row| {
            row.get::<_, String>(0)
        })?;

        let mut uuids = Vec::new();
        for row in mapped_rows {
            uuids.push(row?.parse().expect("failed to parse uuid from string"))
        }

        let uuids = idx.get_nearest_neighbors(embeddings, k, &uuids);

        Ok(uuids)
    }

    fn get_embedding_from_uuid(&self, uuid: Uuid) -> Result<EmbeddingModel, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM embeddings WHERE uuid = ?")?;

        Ok(stmt.query_row([uuid.urn().to_string()], |row| {
            EmbeddingModel::try_from(row)
        })?)
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

        self.index
            .borrow_mut()
            .insert(collection.uuid, Index::new());

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

    fn add_embeddings(
        &self,
        collection_uuid: Uuid,
        embeddings: Vec<EmbeddingModel>,
    ) -> Result<(), DbError> {
        let mut stmt = self.conn.prepare("INSERT INTO embeddings (collection_uuid, uuid, embedding, metadata, text) VALUES (?,?,?,?,?)")?;

        let mut index = self.index.borrow_mut();

        let idx = index
            .get_mut(&collection_uuid)
            .expect("index does not exist for collection");

        embeddings.iter().try_for_each(|e| {
            let embedding_json =
                serde_json::ser::to_string(&e.embedding).expect("failed to serialize vec");
            let v = params![
                collection_uuid.urn().to_string(),
                e.uuid.urn().to_string(),
                embedding_json,
                e.metadata,
                e.text
            ];

            idx.add(IndexEntry {
                e: Embedding {
                    e: e.embedding.clone(),
                },
                uuid: e.uuid,
            });
            stmt.execute(v)?;

            Ok::<(), DbError>(())
        })?;

        Ok(())
    }

    fn count_embeddings(&self, collection_uuid: Uuid) -> Result<usize, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT() FROM embeddings WHERE collection_uuid = ?")?;

        Ok(stmt.query_row(params![collection_uuid.urn().to_string()], |row| row.get(0))?)
    }

    fn get_embeddings(&self, collection_uuid: Uuid) -> Result<Vec<EmbeddingModel>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM embeddings WHERE collection_uuid = ?")?;

        let mapped_rows = stmt.query_map([collection_uuid.urn().to_string()], |row| {
            EmbeddingModel::try_from(row)
        })?;

        let mut embeddings = Vec::new();

        for row in mapped_rows {
            embeddings.push(row?);
        }

        Ok(embeddings)
    }

    fn query(
        &self,
        collection_uuid: Uuid,
        embeddings: &[Embedding],
        k: usize,
    ) -> Result<Vec<Vec<QueryResult>>, DbError> {
        let neighs = self.get_nearest_neighbors(collection_uuid, embeddings, k)?;

        // let stmt = self.conn.prepare("SELECT * from embeddings WHERE collection_uuid = ? AND uuid = ?");

        let mut res = vec![];

        for row in neighs {
            let mut row_docs = vec![];

            for (uuid, dist) in row {
                let emb = self.get_embedding_from_uuid(uuid)?;
                row_docs.push(QueryResult {
                    embedding: emb.embedding,
                    distance: dist,
                    text: emb.text,
                    uuid,
                });
            }
            res.push(row_docs)
        }

        Ok(res)
    }

    fn format_where(
        &self,
        where_map: &serde_json::Map<String, serde_json::Value>,
        result: &mut Vec<String>,
    ) -> Result<(), DbError> {
        let operator = where_map.keys().next().unwrap().clone();
        match operator.as_str() {
            "$contains" => {
                let temp = where_map.get(&operator).unwrap().as_str().unwrap();
                result.push(format!("position(document, '{}') > 0", temp));
            }
            "$or" | "$and" => {
                let mut all_subresults = vec![];
                let subresults = where_map.get(&operator).unwrap().as_array().unwrap();

                for subwhere in subresults {
                    let mut subresult = vec![];
                    let _ = self.format_where(subwhere.as_object().unwrap(), &mut subresult)?;
                    all_subresults.push(subresult[0].clone());
                }

                if operator == "$or" {
                    result.push(format!("({})", all_subresults.join(" OR ")));
                }

                if operator == "$and" {
                    result.push(format!("({})", all_subresults.join(" AND ")));
                }
            }
            _ => {
                for (key, value) in where_map {
                    let has_key_and = |clause: &str| -> String {
                        format!("(JSONHas(metadata, '{}') = 1 AND {})", key, clause)
                    };

                    match value {
                        Value::Number(val) => {
                            if val.is_i64() {
                                let actual_val = val.as_i64().unwrap();
                                result.push(has_key_and(&format!(
                                    "JSONExtractInt(metadata, '{}') = {}",
                                    key, actual_val
                                )));
                            } else if val.is_f64() {
                                let actual_val = val.as_f64().unwrap();
                                result.push(has_key_and(&format!(
                                    "JSONExtractFloat(metadata, '{}') = {}",
                                    key, actual_val
                                )));
                            }
                        }
                        Value::String(val) => {
                            result.push(has_key_and(&format!(
                                "JSONExtractString(metadata, '{}') = {}",
                                key, val
                            )));
                        }
                        Value::Object(val) => {
                            let (operator, operand) = val.iter().next().unwrap();

                            match operator.as_str() {
                                "$eq" => {
                                    if let Value::String(op_str) = operand {
                                        result.push(has_key_and(&format!(
                                            "JSONExtractString(metadata, '{}') == '{}'",
                                            key, op_str
                                        )));
                                    } else if let Value::Number(op_num) = operand {
                                        result.push(has_key_and(&format!(
                                            "JSONExtractFloat(metadata, '{}') == '{}'",
                                            key, op_num
                                        )));
                                    } else {
                                        return Err(DbError::OperandError(format!(
                                            "Operand {} not valid for $eq",
                                            operand
                                        )));
                                    }
                                }
                                "$gt" => {
                                    result.push(has_key_and(&format!(
                                        "JSONExtractFloat(metadata, '{}') > {}",
                                        key, operand
                                    )));
                                }
                                "$gte" => {
                                    result.push(has_key_and(&format!(
                                        "JSONExtractFloat(metadata, '{}') >= {}",
                                        key, operand
                                    )));
                                }
                                "$lt" => {
                                    result.push(has_key_and(&format!(
                                        "JSONExtractFloat(metadata, '{}') < {}",
                                        key, operand
                                    )));
                                }
                                "$lte" => {
                                    result.push(has_key_and(&format!(
                                        "JSONExtractFloat(metadata, '{}') >= {}",
                                        key, operand
                                    )));
                                }
                                "$ne" => {
                                    if let Value::String(op_str) = operand {
                                        result.push(has_key_and(&format!(
                                            "JSONExtractString(metadata, '{}') != '{}'",
                                            key, op_str
                                        )));
                                    } else if let Value::Number(op_num) = operand {
                                        result.push(has_key_and(&format!(
                                            "JSONExtractFloat(metadata, '{}') != {}",
                                            key, op_num
                                        )));
                                    } else {
                                        return Err(DbError::OperandError(format!(
                                            "Operand {} not valid for $ne",
                                            operand
                                        )));
                                    }
                                }
                                _ => {
                                    return Err(DbError::OperatorError(format!("Invalid operator: expected one of $eq, $ne, $lt, $lte, $gt, $gte, found {}", operator)));
                                }
                            }
                        }
                        Value::Array(val) => {
                            let mut all_subresults = vec![];
                            for subwhere in val {
                                let mut subresults = vec![];
                                let _ = self
                                    .format_where(subwhere.as_object().unwrap(), &mut subresults)?;
                                all_subresults.push(subresults[0].clone());
                            }

                            match key.as_str() {
                                "$and" => {
                                    result.push(format!("({})", all_subresults.join(" AND ")));
                                }
                                "$or" => {
                                    result.push(format!("({})", all_subresults.join(" OR ")));
                                }
                                _ => {
                                    return Err(DbError::OperatorError(format!(
                                        "Invalid operator: expected one of $and, $or, found {}",
                                        key
                                    )));
                                }
                            }
                        }
                        _ => {
                            return Err(DbError::InvalidValueError(format!("Invalid value type")));
                        }
                    }
                }
            }
        }

        Ok(())
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

impl TryFrom<&duckdb::Row<'_>> for EmbeddingModel {
    type Error = duckdb::Error;

    // "CREATE TABLE embeddings (collection_uuid STRING, uuid STRING, embedding JSON, text STRING, metadata STRING)",
    fn try_from(row: &duckdb::Row<'_>) -> Result<Self, Self::Error> {
        let embedding_json: String = row.get(2)?;
        let embedding =
            serde_json::from_str(&embedding_json).expect("failed to deserialize vec from db");

        println!("here");

        let model = EmbeddingModel {
            uuid: row
                .get::<_, String>(1)?
                .parse()
                .expect("failed to read uuid from db"),
            embedding,
            text: row.get(3)?,
            metadata: row.get(4)?,
        };

        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::db::{model::EmbeddingModel, Db};

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
            text: "hello, this is a sentence".to_string(),
        };

        db.add_embeddings(collection_uuid, vec![e_model]).unwrap();
    }

    #[test]
    pub fn test_count_embeddings() {
        let db = DuckDB::new(Default::default()).unwrap();

        db.init().unwrap();

        let collection = db.create_collection("collection1").unwrap();
        let collection_uuid = collection.uuid;

        assert_eq!(0, db.count_embeddings(collection_uuid).unwrap());

        let e_model = EmbeddingModel {
            embedding: vec![0.0; 384],
            uuid: Uuid::new_v4(),
            metadata: serde_json::json!({"id": "102"}),
            text: "hello, this is a sentence".to_string(),
        };

        db.add_embeddings(collection_uuid, vec![e_model]).unwrap();

        assert_eq!(1, db.count_embeddings(collection_uuid).unwrap());
    }

    #[test]
    pub fn test_get_embeddings() {
        let db = DuckDB::new(Default::default()).unwrap();

        db.init().unwrap();

        let collection = db.create_collection("collection1").unwrap();
        let collection_uuid = collection.uuid;

        assert!(db.get_embeddings(collection_uuid).unwrap().is_empty());

        let e_model = EmbeddingModel {
            embedding: vec![0.0; 384],
            uuid: Uuid::new_v4(),
            metadata: serde_json::json!({"id": "102"}),
            text: "hello, this is a sentence".to_string(),
        };

        db.add_embeddings(collection_uuid, vec![e_model.clone()])
            .unwrap();

        let embeddings = db.get_embeddings(collection_uuid).unwrap();

        assert_eq!(embeddings[0], e_model);
    }

    #[test]
    pub fn test_query_fmt() {
        use crate::client::Client;

        let db = DuckDB::new(Default::default()).unwrap();
        db.init().unwrap();
        
        let embedding_fn = crate::embeddings::SentenceTransformerEmbeddings::new();

        let mut client = crate::client::LocalClient::init(db, embedding_fn).unwrap();

        let mut collection = client.create_collection("collection1").unwrap();

        let docs = &[
            crate::Document {
                text: "this is about macbooks".to_string(),
                metadata: serde_json::json!({ "source": "laptops" }),
                id: Uuid::new_v4(),
            },
            crate::Document {
                text: "lychees are better than mangoes".to_string(),
                metadata: serde_json::json!({ "source": "facts" }),
                id: Uuid::new_v4(),
            },
        ];
        collection.add_documents(docs).unwrap();

        let k = 1;
        let res = collection.query_documents(&["which one is the better fruit?"], k, serde_json::json!({ "source": "facts" })).unwrap();
        panic!("{}", format!("{:?}", res));
        assert_eq!("hello", "hello");
    }
}
