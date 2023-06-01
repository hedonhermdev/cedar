use serde_json::Value;
use uuid::Uuid;

pub mod client;
pub mod collection;
pub mod db;
pub mod embeddings;
pub mod index;

pub struct Embedding {
    e: Vec<f32>,
}

impl Embedding {
    pub fn new(e: Vec<f32>) -> Self {
        Self { e }
    }

    pub fn dim(&self) -> usize {
        self.e.len()
    }

    pub fn get(&self) -> &Vec<f32> {
        &self.e
    }

    pub fn get_mut(&mut self) -> &mut Vec<f32> {
        &mut self.e
    }
}

impl Into<Vec<f32>> for Embedding {
    fn into(self) -> Vec<f32> {
        self.e
    }
}

impl From<Vec<f32>> for Embedding {
    fn from(e: Vec<f32>) -> Self {
        Embedding { e }
    }
}

pub struct Document {
    text: String,
    metadata: Value,
    id: Uuid,
}

impl Document {
    pub fn new(text: String, metadata: Value) -> Self {
        Document {
            text,
            metadata,
            id: Uuid::new_v4(),
        }
    }

    pub fn new_with_id(text: String, metadata: Value, id: Uuid) -> Self {
        Document { text, metadata, id }
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub fn set_metadata(&mut self, metadata: Value) {
        self.metadata = metadata;
    }

    pub fn metadata(&self) -> &Value {
        &self.metadata
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

pub type Documents<'a> = &'a [Document];
