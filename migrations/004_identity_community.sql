ALTER TABLE users
    ADD COLUMN deleted_at TIMESTAMPTZ,
    ADD CONSTRAINT users_subject_length CHECK (CHAR_LENGTH(github_subject) <= 255),
    ADD CONSTRAINT users_display_name_length CHECK (display_name IS NULL OR CHAR_LENGTH(display_name) <= 100),
    ADD CONSTRAINT users_avatar_url_safe CHECK (avatar_url IS NULL OR (CHAR_LENGTH(avatar_url) <= 2048 AND avatar_url LIKE 'https://%'));

CREATE TABLE daily_challenges (
    id UUID PRIMARY KEY,
    challenge_date DATE NOT NULL,
    version SMALLINT NOT NULL CHECK (version > 0),
    seed BIGINT NOT NULL CHECK (seed >= 0),
    width SMALLINT NOT NULL CHECK (width BETWEEN 5 AND 100),
    height SMALLINT NOT NULL CHECK (height BETWEEN 5 AND 100),
    generator_algo TEXT NOT NULL CHECK (generator_algo IN ('KRUSKAL', 'PRIM', 'DFS')),
    feature_preset TEXT NOT NULL CHECK (feature_preset IN ('classic', 'keys')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (challenge_date, version)
);

ALTER TABLE mazes
    ADD COLUMN daily_challenge_id UUID REFERENCES daily_challenges(id) ON DELETE RESTRICT;

ALTER TABLE runs
    ADD COLUMN race_id UUID;

CREATE TABLE achievement_definitions (
    achievement_key TEXT NOT NULL,
    version SMALLINT NOT NULL CHECK (version > 0),
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    PRIMARY KEY (achievement_key, version)
);

INSERT INTO achievement_definitions (achievement_key, version, name, description) VALUES
    ('first_finish', 1, 'First Finish', 'Complete and submit an authoritative solve.'),
    ('efficient', 1, 'Efficient', 'Submit a solve that visits fewer than 100 nodes.'),
    ('astar_optimal', 1, 'A* Optimal', 'Submit a completed A* solve.'),
    ('key_master', 1, 'Key Master', 'Submit a completed key-aware solve.'),
    ('daily_challenger', 1, 'Daily Challenger', 'Complete and submit a daily challenge.');

CREATE TABLE user_achievements (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_key TEXT NOT NULL,
    achievement_version SMALLINT NOT NULL,
    run_id UUID NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    awarded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, achievement_key, achievement_version),
    FOREIGN KEY (achievement_key, achievement_version)
        REFERENCES achievement_definitions(achievement_key, version) ON DELETE RESTRICT
);

CREATE INDEX mazes_daily_challenge_idx ON mazes (daily_challenge_id, created_at, id)
    WHERE daily_challenge_id IS NOT NULL;
CREATE INDEX runs_race_idx ON runs (race_id, cost, duration_ms, visited, id)
    WHERE race_id IS NOT NULL AND status = 'completed';
CREATE INDEX achievements_user_awarded_idx ON user_achievements (user_id, awarded_at DESC);
CREATE INDEX daily_challenges_date_idx ON daily_challenges (challenge_date DESC, version DESC);
