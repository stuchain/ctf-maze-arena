use crate::maze::{generate, GeneratorAlgo};
use crate::solve::astar::AstarSolver;
use crate::solve::bfs::BfsSolver;
use crate::solve::dfs::DfsSolver;
use crate::solve::Solver;

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
