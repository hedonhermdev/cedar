use std::{cmp::min, collections::HashMap, fmt::Debug};

use hnsw_rs::{hnsw::Hnsw, prelude::*};
use uuid::Uuid;

use crate::Embedding;

pub struct Index {
    idx: Hnsw<f32, DistCosine>,
    id_to_uuid: HashMap<usize, uuid::Uuid>,
    last_id: usize,
}

impl Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index").finish()
    }
}

pub struct IndexEntry {
    pub(crate) e: Embedding,
    pub(crate) uuid: Uuid,
}

impl Index {
    pub fn new() -> Self {
        let idx = Hnsw::new(16, 100, 16, 200, DistCosine {});
        let id_to_uuid = HashMap::new();
        let last_id = 0;
        let num_elems = 0;

        Self {
            idx,
            id_to_uuid,
            last_id,
        }
    }

    pub fn add(&mut self, entry: IndexEntry) {
        let id = self.last_id + 1;
        self.last_id = id;
        self.id_to_uuid.insert(id, entry.uuid);
        self.idx.insert((entry.e.get(), id));
    }

    pub fn nearest_k(&self, embedding: Embedding, k: usize) {
        // TODO: magic number 30
        self.idx.search(&embedding.get(), k, 30);
    }

    pub fn get_nearest_neighbors(
        &self,
        embeddings: &[Embedding],
        k: usize,
        uuids: &[Uuid],
    ) -> Vec<Vec<(Uuid, f32)>> {
        let k = min(k, self.last_id);

        embeddings
            .iter()
            .map(|embedding| {
                self.idx
                    .search(embedding.get(), k, 30)
                    .into_iter()
                    .map(|neigh| (neigh.get_origin_id(), neigh.distance))
                    .filter_map(|(id, dist)| self.id_to_uuid.get(&id).map(|&uuid| (uuid, dist)))
                    .filter(|(uuid, dist)| uuids.contains(uuid))
                    .collect()
            })
            .collect()
    }
}
