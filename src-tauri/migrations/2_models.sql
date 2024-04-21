CREATE TABLE IF NOT EXISTS models
(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_display_name TEXT NOT NULL, 
    show BOOLEAN DEFAULT 1,
    max_tokens INTEGER DEFAULT 4096,
    context_window INTEGER DEFAULT 1024
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
INSERT INTO models (provider_name, model_name, model_display_name, context_window) VALUES ('groqcloud', 'llama3-8b-8192', 'LLaMA3 8b', 8192);
INSERT INTO models (provider_name, model_name, model_display_name, context_window) VALUES ('groqcloud', 'llama3-70b-8192', 'LLaMA3 70b', 8192);
INSERT INTO models (provider_name, model_name, model_display_name, context_window) VALUES ('groqcloud', 'llama2-70b-4096', 'LLaMA2 70b', 4096);
INSERT INTO models (provider_name, model_name, model_display_name, context_window) VALUES ('groqcloud', 'mixtral-8x7b-32768', 'Mixtral 8x7b', 32768);
INSERT INTO models (provider_name, model_name, model_display_name, context_window) VALUES ('groqcloud', 'gemma-7b-it', 'Gemma 7b', 8192);
-- INSERT INTO models (provider_name, model_name, model_display_name) VALUES ('local', 'Mistral-7B-v0.1-local', 'Mistral');