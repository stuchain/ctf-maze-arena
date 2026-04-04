use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ctf_maze_arena::maze::gen::{generate, GeneratorAlgo};
use ctf_maze_arena::solve::{AstarSolver, BfsSolver, DfsSolver, Solver};

fn bench_solvers(c: &mut Criterion) {
    let maze = generate(20, 20, 12345, GeneratorAlgo::Kruskal);

    c.bench_function("bfs_20x20", |b| {
        b.iter(|| black_box(BfsSolver.solve(&maze)))
    });
    c.bench_function("dfs_20x20", |b| {
        b.iter(|| black_box(DfsSolver.solve(&maze)))
    });
    c.bench_function("astar_20x20", |b| {
        b.iter(|| black_box(AstarSolver.solve(&maze)))
    });
}

criterion_group!(benches, bench_solvers);
criterion_main!(benches);
