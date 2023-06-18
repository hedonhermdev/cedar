use warp::Filter;
use crate::server::Database;
use crate::server::Server;

impl Server {
    pub fn new(port: u16, db: Database) -> Self {
        Self { port, db }
    }

    pub async fn run(&self) {
        let routes = warp::any().map(|| "Hello, World!");

        warp::serve(routes).run(([127, 0, 0, 1], self.port)).await;
    }
}
