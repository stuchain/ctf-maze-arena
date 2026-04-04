use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ctf_maze_arena::maze::gen::{generate, GeneratorAlgo};

fn bench_kruskal_10(c: &mut Criterion) {
    c.bench_function("kruskal_10x10", |b| {
        b.iter(|| black_box(generate(10, 10, 42, GeneratorAlgo::Kruskal)))
    });
}

fn bench_kruskal_50(c: &mut Criterion) {
    c.bench_function("kruskal_50x50", |b| {
        b.iter(|| black_box(generate(50, 50, 42, GeneratorAlgo::Kruskal)))
    });
}

fn bench_prim_10(c: &mut Criterion) {
    c.bench_function("prim_10x10", |b| {
        b.iter(|| black_box(generate(10, 10, 42, GeneratorAlgo::Prim)))
    });
}

fn bench_dfs_10(c: &mut Criterion) {
    c.bench_function("dfs_10x10", |b| {
        b.iter(|| black_box(generate(10, 10, 42, GeneratorAlgo::Dfs)))
    });
}

criterion_group!(
    benches,
    bench_kruskal_10,
    bench_kruskal_50,
    bench_prim_10,
    bench_dfs_10
);
criterion_main!(benches);
