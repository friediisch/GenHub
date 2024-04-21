CREATE TABLE IF NOT EXISTS providers
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    api_key TEXT NOT NULL,
    display_name TEXT NOT NULL
);

INSERT INTO providers (provider_name, api_key, display_name) VALUES ('local', '', 'Local');
INSERT INTO providers (provider_name, api_key, display_name) VALUES ('openai', '', 'OpenAI');
INSERT INTO providers (provider_name, api_key, display_name) VALUES ('anthropic', '', 'Anthropic');
INSERT INTO providers (provider_name, api_key, display_name) VALUES ('mistralai', '', 'Mistral AI');
INSERT INTO providers (provider_name, api_key, display_name) VALUES ('groqcloud', '', 'Groq Cloud');