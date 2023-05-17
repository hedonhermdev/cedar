use std::marker::PhantomData;

use hnsw::{Hnsw, Params};
use rand_pcg::Pcg64;
use space::Metric;

struct Euclidean;
impl Metric<&[f32]> for Euclidean {
    type Unit = u32;
    fn distance(&self, a: &&[f32], b: &&[f32]) -> u32 {
        a.iter()
            .zip(b.iter())
            .map(|(&a, &b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
            .to_bits()
    }
}

struct Index<T> {
    idx: Hnsw<Euclidean, T, Pcg64, 24, 48>,
}

impl<T> Index<T> {
    pub fn new() -> Self {
        let idx: Hnsw<Euclidean, T, Pcg64, 24, 48> = Hnsw::new_params(Euclidean, Params::new());

        Self { idx }
    }
}
