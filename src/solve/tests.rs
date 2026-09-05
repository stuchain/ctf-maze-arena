use crate::maze::Maze;
use crate::maze::{generate, GeneratorAlgo};
use crate::solve::astar::AstarSolver;
use crate::solve::bfs::BfsSolver;
use crate::solve::dfs::DfsSolver;
use crate::solve::{default_registry, Solver, StubSolver};

#[test]
fn bfs_finds_path() {
    let maze = generate(10, 10, 42, GeneratorAlgo::Kruskal);
    let r = BfsSolver.solve(&maze);
    assert!(!r.path.is_empty());
    assert_eq!(r.path[0], maze.start);
    assert_eq!(r.path.last().copied(), Some(maze.goal));
}

#[test]
fn dfs_finds_path() {
    let maze = generate(10, 10, 42, GeneratorAlgo::Kruskal);
    let r = DfsSolver.solve(&maze);
    assert!(!r.path.is_empty());
    assert_eq!(r.path[0], maze.start);
    assert_eq!(r.path.last().copied(), Some(maze.goal));
}

#[test]
fn astar_optimal_matches_bfs_cost() {
    let maze = generate(10, 10, 42, GeneratorAlgo::Kruskal);
    let bfs = BfsSolver.solve(&maze);
    let astar = AstarSolver.solve(&maze);
    assert_eq!(bfs.stats.cost, astar.stats.cost);
}

#[test]
fn solver_trait_object_compiles_and_runs() {
    let maze = Maze::new(2, 2);
    let s: Box<dyn Solver> = Box::new(StubSolver);
    let r = s.solve(&maze);
    assert!(r.path.is_empty());
    assert_eq!(r.stats.visited, 0);
    assert_eq!(r.stats.cost, 0);
    assert_eq!(r.stats.ms, 0);
}

#[test]
fn bfs_registered_in_default_registry() {
    let maze = Maze::new(2, 2);
    let registry = default_registry();
    let solver = registry.get("BFS").expect("BFS solver missing");
    let _ = solver.solve(&maze);
}

#[test]
fn bfs_and_astar_optimality_holds_across_deterministic_generators_and_seeds() {
    for algorithm in [
        GeneratorAlgo::Kruskal,
        GeneratorAlgo::Prim,
        GeneratorAlgo::Dfs,
    ] {
        for seed in 0..64 {
            let maze = generate(15, 15, seed, algorithm);
            let bfs = BfsSolver.solve(&maze);
            let astar = AstarSolver.solve(&maze);
            assert_eq!(
                bfs.stats.cost, astar.stats.cost,
                "algorithm={algorithm:?}, seed={seed}"
            );
            assert!(!bfs.path.is_empty());
            assert!(bfs.stats.peak_frontier > 0);
            assert!(astar.stats.peak_frontier > 0);
        }
    }
}

#[test]
fn race_competitors_receive_equivalent_immutable_maze_clones() {
    let maze = generate(20, 20, 991, GeneratorAlgo::Prim);
    let canonical = serde_json::to_value(&maze).unwrap();
    for _ in 0..4 {
        let competitor_input = maze.clone();
        assert_eq!(serde_json::to_value(competitor_input).unwrap(), canonical);
    }
}
