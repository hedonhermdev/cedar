use clap::Parser;
use cedar_db::{db::{ DbError, Db, duckdb::DuckDB }, embeddings::{ SentenceTransformerEmbeddings, OpenAIEmbeddingFunction }, client::LocalClient, server::Server };

#[derive(Parser, Debug)]
struct Arguments {
    #[clap(short, long)]
    db: String,

    #[clap(short, long)]
    embedding_fn: String,

    #[clap(short, long, default_value = "3030")]
    port: u16,
}

#[tokio::main]
async fn main() {
    println!("Hello, cli!");
    let args = Arguments::parse();


    let db = match args.db.as_str() {
        "DuckDB" => { 
            DuckDB::new(Default::default()).unwrap()
        },
        _ => panic!("Invalid database"),
    };
    let _ = db.init().unwrap();

    let embedding_fn = match args.embedding_fn.as_str() {
        "OpenAIEmbeddingFunction" => OpenAIEmbeddingFunction::new(String::from("test_api_key")),
        _ => { 
            panic!("Invalid embedding function");
        }
    };

    return ();

    let client = LocalClient::init(db, embedding_fn).unwrap();
    let server = Server::new(args.port, client);

    server.run().await;
}
