-- Add migration script here
-- create_tokens.sql
CREATE TABLE IF NOT EXISTS tokens (
    id TEXT PRIMARY KEY,
    expired_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- seed db with some local test data
INSERT INTO tokens
(id, expired_at)
VALUES
('LET_ME_IN', (CURRENT_TIMESTAMP + INTERVAL '15 minutes') AT TIME ZONE 'utc');