use criterion::{black_box, criterion_group, criterion_main, Criterion};
use superdiff::comp::levenshtein_distance;

fn criterion_lev_check(c: &mut Criterion) {
    let a = ['a'; 100].iter().collect();
    let b = ['b'; 100].iter().collect();
    let t = 75;
    let closure = || levenshtein_distance(black_box(&a), black_box(&b), black_box(t));
    c.bench_function("lev len=100 vs len=100", |b| b.iter(closure));
}

criterion_group!(benches, criterion_lev_check);
criterion_main!(benches);
