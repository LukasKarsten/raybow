use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::Rng;
use raybow::vector::Vector3x8;

pub fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot product");

    group.bench_function(
        BenchmarkId::new("auto vectorization", "Two Vector3x8s"),
        |b| {
            b.iter_batched_ref(
                || {
                    let mut rng = rand::thread_rng();
                    let v1 = Vector3x8::new(rng.gen(), rng.gen(), rng.gen());
                    let v2 = Vector3x8::new(rng.gen(), rng.gen(), rng.gen());
                    (v1, v2)
                },
                |(v1, v2)| v1.dot(v2),
                BatchSize::SmallInput,
            )
        },
    );

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
