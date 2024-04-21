use std::path::PathBuf;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::command;

use crate::data::DataState;
use crate::utils::{highlight_code, MessageBlock};

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
pub struct Settings {
	pub default_model: String,
	pub default_provider: String,
	pub code_theme: String,
}
impl Settings {
	pub fn load(settings_file: &PathBuf) -> Self {
		let settings = match std::fs::read_to_string(settings_file) {
			Ok(settings) => settings,
			Err(_) => {
				let default_settings = Settings {
					default_model: "claude-3-opus-20240229".to_string(),
					default_provider: "anthropic".to_string(),
					code_theme: "base16-eighties.dark".to_string(),
				};
				let settings = serde_json::to_string(&default_settings).unwrap();
				std::fs::write(settings_file, &settings).unwrap();
				settings
			}
		};
		serde_json::from_str(&settings).unwrap()
	}
	pub fn save(&self, settings_file: &PathBuf) {
		let settings = serde_json::to_string(&self).unwrap();
		std::fs::write(settings_file, &settings).unwrap();
	}
}

#[command]
#[specta::specta]
pub async fn get_settings(data: DataState<'_>) -> Result<Settings, String> {
	let data = data.0.lock().await;
	Ok(Settings::load(&data.paths.settings_file))
}

#[command]
#[specta::specta]
pub async fn apply_and_save_settings( 
    new_settings: Settings,
    data: DataState<'_>,
) -> Result<(), String> {
    let mut data = data.0.lock().await;
    if data.settings.code_theme != new_settings.code_theme {
        data.settings.code_theme = new_settings.code_theme;
		let fetch_code_message_blocks_query = 
            "SELECT id, type_, language, raw_content, rendered_content, copied FROM message_blocks WHERE type_ = 'code'".to_string();
        let code_message_blocks_result = 
            sqlx::query_as::<_, MessageBlock>(&fetch_code_message_blocks_query)
                .fetch_all(&data.db_pool)
                .await;
        match code_message_blocks_result {
            Ok(mut code_message_blocks) => {
				code_message_blocks = code_message_blocks.par_iter_mut().map(|block| {
					block.rendered_content = highlight_code(&block.raw_content, &block.language.as_deref().unwrap_or("plain"), &data.settings.code_theme).unwrap();
					block.to_owned()
				}
				).collect();
				let update_code_message_blocks_query = 
					"UPDATE message_blocks SET rendered_content = $1 WHERE id = $2".to_string();
				for block in code_message_blocks {
					let _update_code_message_blocks_result = 
					sqlx::query(&update_code_message_blocks_query)
						.bind(&block.rendered_content)
						.bind(&block.id)
						.execute(&data.db_pool)
						.await;
				}
            }
            Err(err) => {
                eprintln!("Error fetching code message blocks from database: {}", err);
            }
        }
    }

    data.settings.save(&data.paths.settings_file);
    Ok(())
}
