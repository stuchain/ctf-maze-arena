ALTER TABLE runs
    ADD COLUMN peak_frontier BIGINT CHECK (peak_frontier >= 0);

ALTER TABLE runs DROP CONSTRAINT runs_terminal_shape;
ALTER TABLE runs ADD CONSTRAINT runs_terminal_shape CHECK (
    (status = 'queued' AND started_at IS NULL AND completed_at IS NULL AND visited IS NULL AND cost IS NULL AND duration_ms IS NULL AND peak_frontier IS NULL AND error_code IS NULL)
    OR (status = 'running' AND started_at IS NOT NULL AND completed_at IS NULL AND visited IS NULL AND cost IS NULL AND duration_ms IS NULL AND peak_frontier IS NULL AND error_code IS NULL)
    OR (status = 'completed' AND started_at IS NOT NULL AND completed_at IS NOT NULL AND visited IS NOT NULL AND cost IS NOT NULL AND duration_ms IS NOT NULL AND peak_frontier IS NOT NULL AND error_code IS NULL)
    OR (status = 'failed' AND completed_at IS NOT NULL AND error_code IS NOT NULL)
    OR (status = 'cancelled' AND completed_at IS NOT NULL)
);
