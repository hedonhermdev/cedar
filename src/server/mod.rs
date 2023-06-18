use crate::db::DuckDB;

pub(crate) mod warp_server;

pub struct Server {
    pub port: u16,
    pub db : Database,
}

pub enum Database {
    DuckDB(DuckDB),
}
