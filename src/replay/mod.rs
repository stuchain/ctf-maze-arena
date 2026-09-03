use crate::maze::Cell;
use crate::realtime::{ReplayEvent, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayStats {
    pub visited: usize,
    pub cost: usize,
    pub ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Replay {
    pub protocol_version: u16,
    pub maze_id: String,
    pub solver: String,
    pub seed: u64,
    pub events: Vec<ReplayEvent>,
    pub path: Vec<[u32; 2]>,
    pub stats: ReplayStats,
}

pub fn cell_to_arr(c: Cell) -> [u32; 2] {
    [c.x as u32, c.y as u32]
}

use crate::solve::SolveResult;

/// Build a `Replay` object from a solver run.
///
pub fn build_replay(
    maze_id: impl Into<String>,
    solver_name: impl Into<String>,
    seed: u64,
    result: SolveResult,
    events: Vec<ReplayEvent>,
) -> Replay {
    let path = result.path.into_iter().map(cell_to_arr).collect();
    let stats = ReplayStats {
        visited: result.stats.visited,
        cost: result.stats.cost,
        ms: result.stats.ms,
    };

    Replay {
        protocol_version: PROTOCOL_VERSION,
        maze_id: maze_id.into(),
        solver: solver_name.into(),
        seed,
        events,
        path,
        stats,
    }
}

pub fn to_json(replay: &Replay) -> Result<String, serde_json::Error> {
    serde_json::to_string(replay)
}

pub fn from_json(s: &str) -> Result<Replay, serde_json::Error> {
    serde_json::from_str::<Replay>(s)
}

#[cfg(test)]
mod tests {
    use super::{build_replay, Replay, ReplayStats};
    use super::{from_json, to_json};
    use crate::maze::{generate, GeneratorAlgo};
    use crate::solve::bfs::BfsSolver;
    use crate::solve::Solver;

    #[test]
    fn replay_json_roundtrip() {
        let replay = Replay {
            protocol_version: 1,
            maze_id: "maze-1".to_string(),
            solver: "ASTAR".to_string(),
            seed: 42,
            events: vec![],
            path: vec![[0, 0], [1, 0]],
            stats: ReplayStats {
                visited: 10,
                cost: 1,
                ms: 2,
            },
        };

        let json = serde_json::to_string(&replay).expect("to_json works");
        let parsed: Replay = serde_json::from_str(&json).expect("from_json works");

        assert_eq!(parsed.maze_id, replay.maze_id);
        assert_eq!(parsed.solver, replay.solver);
        assert_eq!(parsed.seed, replay.seed);
        assert_eq!(parsed.path, replay.path);
        assert_eq!(parsed.stats.visited, replay.stats.visited);
    }

    #[test]
    fn build_replay_from_bfs_result_has_basic_invariants() {
        let maze = generate(6, 6, 42, GeneratorAlgo::Kruskal);
        let result = BfsSolver.solve(&maze);
        let replay = build_replay("maze-1", "BFS", 42, result, vec![]);

        assert!(!replay.path.is_empty());
        assert_eq!(replay.stats.cost, replay.path.len().saturating_sub(1));
    }

    #[test]
    fn replay_to_json_and_back_roundtrip() {
        let replay = Replay {
            protocol_version: 1,
            maze_id: "maze-1".to_string(),
            solver: "ASTAR".to_string(),
            seed: 42,
            events: vec![],
            path: vec![[0, 0]],
            stats: ReplayStats {
                visited: 0,
                cost: 0,
                ms: 0,
            },
        };

        let json = to_json(&replay).expect("to_json works");
        let parsed = from_json(&json).expect("from_json works");

        assert_eq!(parsed.maze_id, replay.maze_id);
        assert_eq!(parsed.solver, replay.solver);
        assert_eq!(parsed.seed, replay.seed);
        assert_eq!(parsed.path, replay.path);
    }
}
