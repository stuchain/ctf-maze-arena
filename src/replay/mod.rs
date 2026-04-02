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

/// Keep every `step`'th frame, plus always keep the first and last frames.
pub fn decimate_frames(frames: Vec<ReplayFrame>, step: u32) -> Vec<ReplayFrame> {
    if frames.is_empty() {
        return frames;
    }
    if frames.len() == 1 {
        return frames;
    }
    if step == 0 {
        return frames;
    }

    let last_index = frames.len() - 1;
    let mut out = vec![frames[0].clone()];
    for (i, f) in frames.iter().enumerate().skip(1) {
        if i == last_index {
            out.push(f.clone());
            break;
        }
        if (i as u32) % step == 0 {
            out.push(f.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{decimate_frames, Replay, ReplayFrame, ReplayStats};

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

    #[test]
    fn decimate_frames_empty_and_singleton() {
        assert!(decimate_frames(vec![], 5).is_empty());

        let single = vec![ReplayFrame {
            t: 123,
            frontier: vec![],
            visited: vec![],
            current: None,
        }];
        assert_eq!(decimate_frames(single.clone(), 5).len(), 1);
        assert_eq!(decimate_frames(single, 5)[0].t, 123);
    }

    #[test]
    fn decimate_frames_keeps_first_and_last() {
        let frames = (0..4)
            .map(|t| ReplayFrame {
                t,
                frontier: vec![],
                visited: vec![],
                current: None,
            })
            .collect::<Vec<_>>();

        let out = decimate_frames(frames, 10);
        assert_eq!(out.len(), 2);
        assert_eq!(out.first().unwrap().t, 0);
        assert_eq!(out.last().unwrap().t, 3);
    }

    #[test]
    fn decimate_frames_step_10_produces_11_frames_for_100() {
        let frames = (0..100)
            .map(|t| ReplayFrame {
                t,
                frontier: vec![],
                visited: vec![],
                current: None,
            })
            .collect::<Vec<_>>();

        let out = decimate_frames(frames, 10);
        assert_eq!(out.len(), 11);
        assert_eq!(out.first().unwrap().t, 0);
        assert_eq!(out.last().unwrap().t, 99);
    }
}

