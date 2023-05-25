use serde_json::Value;
use uuid::Uuid;

pub mod client;
pub mod collection;
pub mod db;
pub mod embeddings;
pub mod index;

// pub type Document<S> = S;
pub type Embedding = Vec<f32>;
pub type Embeddings = Vec<Embedding>;

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
