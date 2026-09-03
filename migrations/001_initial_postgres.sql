CREATE TABLE users (
    id UUID PRIMARY KEY,
    github_subject TEXT NOT NULL UNIQUE,
    display_name TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_github_subject_not_blank CHECK (BTRIM(github_subject) <> '')
);

CREATE TABLE mazes (
    id UUID PRIMARY KEY,
    width SMALLINT NOT NULL CHECK (width BETWEEN 5 AND 100),
    height SMALLINT NOT NULL CHECK (height BETWEEN 5 AND 100),
    seed BIGINT NOT NULL CHECK (seed >= 0),
    generator_algo TEXT NOT NULL CHECK (generator_algo IN ('KRUSKAL', 'PRIM', 'DFS')),
    content_version SMALLINT NOT NULL DEFAULT 1 CHECK (content_version > 0),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE runs (
    id UUID PRIMARY KEY,
    maze_id UUID NOT NULL REFERENCES mazes(id) ON DELETE RESTRICT,
    owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    solver TEXT NOT NULL CHECK (solver IN ('BFS', 'DFS', 'ASTAR', 'DP_KEYS')),
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled')),
    visited BIGINT CHECK (visited >= 0),
    cost BIGINT CHECK (cost >= 0),
    duration_ms BIGINT CHECK (duration_ms >= 0),
    error_code TEXT,
    request_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    CONSTRAINT runs_terminal_shape CHECK (
        (status = 'queued' AND started_at IS NULL AND completed_at IS NULL AND visited IS NULL AND cost IS NULL AND duration_ms IS NULL AND error_code IS NULL)
        OR (status = 'running' AND started_at IS NOT NULL AND completed_at IS NULL AND visited IS NULL AND cost IS NULL AND duration_ms IS NULL AND error_code IS NULL)
        OR (status = 'completed' AND started_at IS NOT NULL AND completed_at IS NOT NULL AND visited IS NOT NULL AND cost IS NOT NULL AND duration_ms IS NOT NULL AND error_code IS NULL)
        OR (status = 'failed' AND completed_at IS NOT NULL AND error_code IS NOT NULL)
        OR (status = 'cancelled' AND completed_at IS NOT NULL)
    )
);

CREATE TABLE replays (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL UNIQUE REFERENCES runs(id) ON DELETE CASCADE,
    protocol_version SMALLINT NOT NULL CHECK (protocol_version > 0),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE leaderboard_submissions (
    run_id UUID PRIMARY KEY REFERENCES runs(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX runs_maze_created_idx ON runs (maze_id, created_at DESC, id);
CREATE INDEX runs_owner_created_idx ON runs (owner_user_id, created_at DESC, id);
CREATE INDEX runs_status_created_idx ON runs (status, created_at, id);
CREATE INDEX runs_leaderboard_sort_idx
    ON runs (maze_id, cost, duration_ms, visited, id)
    WHERE status = 'completed';
CREATE INDEX leaderboard_user_accepted_idx
    ON leaderboard_submissions (user_id, accepted_at DESC, run_id);
