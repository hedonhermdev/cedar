use openai_api_rs::v1::embedding::{EmbeddingRequest, EmbeddingResponse};

use crate::{Documents, Embeddings};

use super::EmbeddingFunction;

struct OpenAIEmbeddingFunction {
    api_key: String,
    client: openai_api_rs::v1::api::Client,
}
