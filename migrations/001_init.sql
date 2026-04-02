-- Mazes: stored as JSON for simplicity
CREATE TABLE IF NOT EXISTS mazes (
    id TEXT PRIMARY KEY,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    seed INTEGER NOT NULL,
    generator_algo TEXT NOT NULL,
    walls_json TEXT NOT NULL,
    keys_json TEXT,
    doors_json TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Runs: metadata + stats
CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    maze_id TEXT NOT NULL,
    solver TEXT NOT NULL,
    stats_json TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (maze_id) REFERENCES mazes(id)
);

-- Replays: full replay as JSON blob
CREATE TABLE IF NOT EXISTS replays (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL UNIQUE,
    replay_json TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (run_id) REFERENCES runs(id)
);

CREATE INDEX idx_runs_maze_id ON runs(maze_id);
CREATE INDEX idx_runs_status ON runs(status);

