use criterion::{Criterion, black_box, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("chain 30", |b| b.iter(|| todo!()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
