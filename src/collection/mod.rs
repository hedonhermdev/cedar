use std::{sync::Arc, fmt::Debug};

use crate::client::Client;

pub struct Collection {
    pub(crate) client: Box<dyn Client>,
    pub(crate) uuid: uuid::Uuid,
    pub(crate) name: String,
}

impl Debug for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("collection").field("uuid", &self.uuid).field("name", &self.name).finish()
    }
}
