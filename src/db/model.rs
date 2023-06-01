use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub struct CollectionModel {
    pub(crate) name: String,
    pub(crate) uuid: Uuid,
    pub(crate) metadata: serde_json::Value,
}

pub struct EmbeddingModel {
    pub(crate) embedding: Vec<f32>,
    pub (crate) uuid: Uuid,
    pub (crate) metadata: serde_json::Value,
    pub (crate) text: String
}
