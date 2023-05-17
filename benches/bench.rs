use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cedar::embeddings::EmbeddingFunction;

use cedar::embeddings::sentencetransformer::SentenceTransformerEmbeddings;

pub fn sentence_transformer_benchmark(c: &mut Criterion) {
    c.bench_function("sentence transformer", |b| {
        b.iter(|| {
            let embeddingfn = SentenceTransformerEmbeddings::new();
            black_box(embeddingfn.embed(&vec!["this is a very long piece of text."])).unwrap();
        })
    });
}

criterion_group!(benches, sentence_transformer_benchmark);
criterion_main!(benches);
