use clap::Parser;
use warp::Filter;
use cedar_db::{ server::Database, db::DuckDB, db::DbError };
use cedar_db::server::Server;

#[derive(Parser, Debug)]
struct Arguments {
    #[clap(short, long)]
    db: String,

    #[clap(short, long, default_value = "3030")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), DbError> {
    let args = Arguments::parse();

    let db = match args.db.as_str() {
        "duckdb" => Database::DuckDB(DuckDB::new(Default::default())?),
        _ => panic!("Unknown database"),
    };

    let server = Server::new(args.port, db);
    server.run().await;

    Ok(())
}
