CREATE TABLE IF NOT EXISTS models
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_display_name TEXT NOT NULL, 
    show BOOLEAN DEFAULT 1
);

INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('openai', 'gpt-4-turbo-preview', 'GPT-4');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('openai', 'gpt-4-32k', 'GPT-4');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('openai', 'gpt-4', 'GPT-4');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('openai', 'gpt-3.5-turbo', 'GPT-3.5');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('anthropic', 'claude-3-opus-20240229', 'Claude');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('anthropic', 'claude-3-sonnet-20240229', 'Claude');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('anthropic', 'claude-3-haiku-20240307', 'Claude');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('mistralai', 'open-mistral-7b', 'Mistral');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('mistralai', 'open-mixtral-8x7b', 'Mixtral');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('mistralai', 'mistral-small-latest', 'Mistral');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('mistralai', 'mistral-medium-latest', 'Mistral');
INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('mistralai', 'mistral-large-latest', 'Mistral');
-- INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('local', 'Mistral-7B-v0.1-local', 'Mistral');