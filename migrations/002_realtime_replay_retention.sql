ALTER TABLE replays
    ADD COLUMN payload_bytes INTEGER NOT NULL,
    ADD COLUMN expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),
    ADD CONSTRAINT replays_payload_size CHECK (payload_bytes BETWEEN 1 AND 8388608);

CREATE INDEX replays_expiry_idx ON replays (expires_at);
