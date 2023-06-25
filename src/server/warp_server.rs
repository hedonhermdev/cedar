use std::sync::Arc;
use warp::Filter;
use crate::client::Client;
use crate::server::Server;

impl<C> Server<C> where C: Client {
    pub fn new(port: u16, client: C) -> Self {
        Self { port, client: Arc::new(client) }
    }

    pub async fn run(&self) {
        let routes = warp::any().map(|| "Hello, World!");

        warp::serve(routes).run(([127, 0, 0, 1], self.port)).await;
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_create_server() {

    }
}
