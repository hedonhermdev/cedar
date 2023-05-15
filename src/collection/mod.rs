#[derive(Debug, PartialEq, Eq)]
pub struct Collection {
    pub(crate) uuid: uuid::Uuid,
    pub(crate) name: String,
}
