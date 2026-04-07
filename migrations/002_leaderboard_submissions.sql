ALTER TABLE runs ADD COLUMN user_id TEXT;
CREATE INDEX IF NOT EXISTS idx_runs_user_id ON runs(user_id);

CREATE TABLE IF NOT EXISTS leaderboard_submissions (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    user_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (run_id) REFERENCES runs(id)
);

CREATE INDEX IF NOT EXISTS idx_leaderboard_submissions_run_id ON leaderboard_submissions(run_id);
CREATE INDEX IF NOT EXISTS idx_leaderboard_submissions_user_id ON leaderboard_submissions(user_id);
