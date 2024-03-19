CREATE TABLE IF NOT EXISTS message_blocks
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER,
    type_ TEXT NOT NULL,
    language TEXT,
    raw_content TEXT NOT NULL,
    rendered_content TEXT NOT NULL,
    copied INTEGER DEFAULT FALSE
);