CREATE TABLE IF NOT EXISTS messages
(
    id TEXT NOT NULL PRIMARY KEY,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    chat_id TEXT NOT NULL,
    model_name TEXT NOT NULL
);