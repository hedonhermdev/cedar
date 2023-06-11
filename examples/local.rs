use cedar::{
    client::Client,
    client::LocalClient,
    db::{Db, DuckDB},
    embeddings::OpenAIEmbeddingFunction,
    Document,
};
use serde_json::json;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. initialize db
    let db = DuckDB::new(Default::default())?;
    db.init()?;

    // 2. initialize embedding function (could be OpenAI, Chrome, etc)
    // let embedding_fn = SentenceTransformerEmbeddings::new();
    let embedding_fn = OpenAIEmbeddingFunction::new(
        "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".to_string(),
    );

    // 3. initialize client
    let mut client = LocalClient::init(db, embedding_fn)?;

    // 4. create a collection
    let mut collection = client.create_collection("collection1")?;

    let docs = &[
        Document {
            text: "this is about macbooks".to_string(),
            metadata: json!({}),
            id: Uuid::new_v4(),
        },
        Document {
            text: "apples are the best fruit ever".to_string(),
            metadata: json!({}),
            id: Uuid::new_v4(),
        },
    ];

    collection.add_documents(docs)?;

    let res = collection.query_documents(&["what is the best fruit?"], 1, json!({}))?;

    println!("text: {:?}", res[0][0].text);
    println!("score: {:?}", 1.0 - res[0][0].distance);

    Ok(())
}
