use serde::{Deserialize, Serialize};

use crate::maze::Cell;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFrame {
    pub t: u32,
    pub frontier: Vec<[u32; 2]>,
    pub visited: Vec<[u32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<[u32; 2]>,
}

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
    pub maze_id: String,
    pub solver: String,
    pub seed: u64,
    pub frames: Vec<ReplayFrame>,
    pub path: Vec<[u32; 2]>,
    pub stats: ReplayStats,
}

pub fn cell_to_arr(c: Cell) -> [u32; 2] {
    [c.x as u32, c.y as u32]
}

#[cfg(test)]
mod tests {
    use super::{Replay, ReplayFrame, ReplayStats};

    #[test]
    fn replay_json_roundtrip() {
        let replay = Replay {
            maze_id: "maze-1".to_string(),
            solver: "ASTAR".to_string(),
            seed: 42,
            frames: vec![ReplayFrame {
                t: 0,
                frontier: vec![[1, 0], [0, 1]],
                visited: vec![[0, 0]],
                current: None,
            }],
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
        assert!(parsed.frames[0].current.is_none());
    }
}

