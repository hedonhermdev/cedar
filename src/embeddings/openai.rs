struct OpenAIEmbeddingFunction {
    api_key: String,
    client: openai_api_rs::v1::api::Client,
}
