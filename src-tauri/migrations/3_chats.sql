CREATE TABLE IF NOT EXISTS chats
(
    id TEXT NOT NULL PRIMARY KEY,
    model TEXT,
    api_key_id TEXT,
    display_name TEXT NOT NULL,
    archived TEXT,
    creation_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_updated DATETIME 
);

CREATE INDEX idx_last_updated ON chats(last_updated DESC);