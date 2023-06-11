use serde::Deserialize;
use serde_json::{json};

use crate::Embedding;

use super::{EmbeddingError, EmbeddingFunction};

const OPENAI_EMBEDDING_MODEL: &str = "text-embedding-ada-002";
const OPENAI_EMBEDDING_ENDPOINT: &str = "https://api.openai.com/v1/embeddings";

pub struct OpenAIEmbeddingFunction {
    client: reqwest::blocking::Client,
}

impl OpenAIEmbeddingFunction {
    pub fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
        );
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_str("application/json").unwrap(),
        );

        let client = reqwest::blocking::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();
        Self { client }
    }
}

impl EmbeddingFunction for OpenAIEmbeddingFunction {
    fn embed<I, S: AsRef<str> + Sync>(&self, texts: I) -> Result<Vec<Embedding>, EmbeddingError>
    where
        I: IntoIterator<Item = S>,
    {
        let texts: Vec<_> = texts.into_iter().map(|s| s.as_ref().to_string()).collect();

        let payload = json!({
            "model": OPENAI_EMBEDDING_MODEL,
            "input": texts
        });

        let res: ApiResponse = self
            .client
            .post(OPENAI_EMBEDDING_ENDPOINT)
            .json(&payload)
            .send().map_err(|e| EmbeddingError{ err: e.into() })?
            .json().map_err(|e| EmbeddingError{ err: e.into() })?;

        Ok(res.data.into_iter().map(|e| e.into()).collect())
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<EmbeddingObject>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingObject {
    embedding: Vec<f32>,
}

impl From<EmbeddingObject> for Embedding {
    fn from(val: EmbeddingObject) -> Self {
        Embedding { e: val.embedding }
    }
}
