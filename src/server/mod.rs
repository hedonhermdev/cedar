use crate::client::Client;
use std::sync::Arc;

pub(crate) mod warp_server;

#[derive(Debug)]
pub struct Server<C: Client> {
    pub port: u16,
    pub client: Arc<C>,
}
