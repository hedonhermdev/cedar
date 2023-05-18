use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub struct CollectionModel {
    pub(crate) name: String,
    pub(crate) uuid: Uuid,
    pub(crate) metadata: serde_json::Value,
}
