use hnsw_rs::{hnsw::Hnsw, hnswio::*, prelude::*};

use crate::{embeddings, Embedding};

pub struct Index {
    idx: Hnsw<f32, DistCosine>,
}

pub struct IndexEntry {
    e: Embedding,
    id: usize,
}

impl Index {
    pub fn new() -> Self {
        let idx = Hnsw::new(16, 100, 16, 200, DistCosine {});

        Self { idx }
    }

    pub fn add(&mut self, entry: IndexEntry) {
        self.idx.insert((&entry.e, entry.id));
    }

    pub fn nearest_k(&mut self, embedding: Embedding, k: usize) {
        // TODO: magic number 30
        self.idx.search(&embedding, k, 30);
    }
}
