pub mod client;
pub mod collection;
pub mod db;
pub mod embeddings;

mod index;

pub type Document<S: AsRef<str> + Sync> = S;
pub type Documents<'a, S> = Vec<Document<S>>;
pub type Embedding = Vec<f32>;
pub type Embeddings = Vec<Embedding>;
