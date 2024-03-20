use crate::api::playlist_items;
use crate::data::{Action, AppPaths, DataState};
use crate::providers::ProviderData;
use crate::throw;
use crate::utils::{MessageBlock, MessageBlocks};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::FromRow;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::env;
use std::result::Result;
use tauri::command;

pub async fn init(app_paths: &AppPaths) -> Result<SqlitePool, String> {
	let exists = match Sqlite::database_exists(&app_paths.db).await {
		Ok(exists) => exists,
		Err(e) => throw!("Could not check if database exists: {}", e),
	};
	if !exists {
		if let Err(e) = std::fs::create_dir_all(&app_paths.app_dir) {
			throw!("Error creating parent folder: {}", e.to_string());
		}
		match Sqlite::create_database(&app_paths.db).await {
			Ok(_) => {}
			Err(e) => throw!("Could not create database: {}", e),
		}
	}

	let connect_options = SqliteConnectOptions::new().filename(&app_paths.db);
	let pool = match SqlitePool::connect_with(connect_options).await {
		Ok(pool) => pool,
		Err(e) => throw!("Could not open database: {}", e),
	};

	match sqlx::migrate!("./migrations").run(&pool).await {
		Ok(_) => {}
		Err(e) => throw!("Could not run database migrations: {}", e),
	};

	Ok(pool)
}

pub async fn get_ids(
	videos: &Vec<playlist_items::Playlist>,
	pool: &SqlitePool,
) -> Result<Vec<String>, String> {
	// let mut id_placeholders = "\"?\"".to_string();
	let mut id_placeholders = "?".to_string();
	for _n in 0..(videos.len() - 1) {
		// id_placeholders.push_str(",\"?\"");
		id_placeholders.push_str(",?");
	}

	let query_str = format!("SELECT id FROM videos WHERE id IN ({});", id_placeholders);
	let mut query = sqlx::query(&query_str);
	for video in videos {
		query = query.bind(&video.contentDetails.videoId);
	}
	let rows = match query.fetch_all(pool).await {
		Ok(rows) => rows,
		Err(e) => throw!("Unable to get video IDs: {}", e),
	};
	let mut existing_ids: Vec<String> = Vec::new();
	for row in rows {
		match row.try_get(0) {
			Ok(id) => existing_ids.push(id),
			Err(e) => throw!("Unable to get video ID from database row: {}", e),
		};
	}
	Ok(existing_ids)
}

#[derive(Debug, Serialize, Type)]
#[allow(non_snake_case)]
pub struct Video {
	pub id: String,
	pub title: String,
	pub description: String,
	#[specta(type = i32)] // tauri bigint fix
	pub publishTimeMs: i64,
	/// SQLite does not support unsigned integers
	#[specta(type = i32)] // tauri bigint fix
	pub durationMs: i64,
	pub thumbnailStandard: bool,
	pub thumbnailMaxres: bool,
	pub channelId: String,
	pub channelName: String,
	pub unread: bool,
	pub archived: bool,
}
impl sqlx::FromRow<'_, SqliteRow> for Video {
	fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
		Ok(Video {
			id: row.try_get("id")?,
			title: row.try_get("title")?,
			description: row.try_get("description")?,
			publishTimeMs: row.try_get("publishTimeMs")?,
			// SQLite does not support unsigned integers
			durationMs: row.try_get("durationMs")?,
			thumbnailStandard: row.try_get("thumbnailStandard")?,
			thumbnailMaxres: row.try_get("thumbnailMaxres")?,
			channelId: row.try_get("channelId")?,
			channelName: row.try_get("channelName")?,
			unread: row.try_get("unread")?,
			archived: row.try_get("archived")?,
		})
	}
}

pub async fn insert_video(video: &Video, pool: &SqlitePool) -> Result<(), String> {
	let query_str =
		"INSERT INTO videos (id,title,description,publishTimeMs,durationMs,thumbnailStandard,thumbnailMaxres,channelId,channelName) \
		VALUES (?,?,?,?,?,?,?,?,?)";
	let query = sqlx::query(query_str)
		.bind(&video.id)
		.bind(&video.title)
		.bind(&video.description)
		.bind(video.publishTimeMs)
		.bind(video.durationMs)
		.bind(video.thumbnailStandard)
		.bind(video.thumbnailMaxres)
		.bind(&video.channelId)
		.bind(&video.channelName);
	let rows_affected = match query.execute(pool).await {
		Ok(result_rows) => result_rows.rows_affected(),
		Err(e) => throw!("Error saving video: {}", e),
	};
	if rows_affected != 1 {
		throw!("Error saving video: {} rows affected", rows_affected);
	}
	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Options {
	show_all: bool,
	show_archived: bool,
	channel_filter: String,
	tag: Option<String>,
	limit: u16,
}
#[derive(Debug, Serialize, Deserialize, Type)]
#[allow(non_snake_case)]
pub struct After {
	#[specta(type = i32)] // tauri bigint fix
	publishTimeMs: i64,
	id: String,
}

#[command]
#[specta::specta]
pub async fn get_videos(
	options: Options,
	after: Option<After>,
	data: DataState<'_>,
) -> Result<Vec<Video>, String> {
	let data = data.0.lock().await;
	let mut selects: Vec<&str> = vec!["*"];
	let mut wheres: Vec<&str> = Vec::new();
	let mut bindings: Vec<&str> = Vec::new();
	if options.channel_filter != "" {
		selects.push("INSTR(LOWER(channelName), LOWER(?)) channelFilter");
		wheres.push("channelFilter > 0");
		bindings.push(&options.channel_filter);
	}
	if !options.show_all {
		if options.show_archived {
			wheres.push("archived = 1");
		} else {
			wheres.push("archived = 0");
		}
	}
	let after_publish_time_ms;
	if let Some(after) = &after {
		wheres.push("(publishTimeMs,id) < (?,?)");
		after_publish_time_ms = after.publishTimeMs.to_string();
		bindings.push(&after_publish_time_ms);
		bindings.push(&after.id);
	}

	let q;
	if let Some(tag) = &options.tag {
		let mut question_marks: Vec<&str> = Vec::new();
		for channel in &data.settings_ref().channels {
			if channel.tags.contains(tag) {
				bindings.push(&channel.id);
				question_marks.push("?");
			}
		}
		q = format!("channelId IN ({})", question_marks.join(","));
		wheres.push(&q);
	}

	let mut query_str = "SELECT ".to_owned() + &selects.join(",");
	query_str.push_str(" FROM videos");
	if !wheres.is_empty() {
		query_str.push_str(" WHERE ");
		query_str.push_str(&wheres.join(" AND "));
	}
	query_str.push_str(" ORDER BY publishTimeMs DESC, id DESC");
	query_str.push_str(&format!(" LIMIT {}", options.limit));

	let mut query = sqlx::query_as(&query_str);
	for binding in bindings {
		query = query.bind(binding);
	}
	let videos: Vec<Video> = match query.fetch_all(&data.db_pool).await {
		Ok(videos) => videos,
		Err(e) => throw!("Error getting videos: {}", e),
	};
	Ok(videos)
}

async fn set_archived(pool: &SqlitePool, id: &str, value: bool) -> Result<(), String> {
	let query = sqlx::query("UPDATE videos SET archived = ? WHERE id = ?")
		.bind(value)
		.bind(id);
	let rows_affected = match query.execute(pool).await {
		Ok(result_rows) => result_rows.rows_affected(),
		Err(e) => throw!("{}", e),
	};
	if rows_affected != 1 {
		throw!("{} rows affected", rows_affected);
	}
	Ok(())
}

#[command]
#[specta::specta]
pub async fn archive(id: String, data: DataState<'_>) -> Result<(), String> {
	let mut data = data.0.lock().await;
	match set_archived(&data.db_pool, &id, true).await {
		Ok(()) => (),
		Err(e) => throw!("Error archiving video: {}", e),
	}
	data.user_history.push(Action::Archive(id));
	Ok(())
}

#[command]
#[specta::specta]
pub async fn unarchive(id: String, data: DataState<'_>) -> Result<(), String> {
	let mut data = data.0.lock().await;
	match set_archived(&data.db_pool, &id, false).await {
		Ok(()) => (),
		Err(e) => throw!("Error unarchiving video: {}", e),
	}
	data.user_history.push(Action::Unarchive(id));
	Ok(())
}

#[command]
#[specta::specta]
pub async fn load_providers(data: DataState<'_>) -> Result<Vec<ProviderData>, String> {
	let data = data.0.lock().await;
	let query = "SELECT provider_name, api_key, display_name FROM providers";
	let providers = sqlx::query_as::<_, ProviderData>(&query);
	match providers.fetch_all(&data.db_pool).await {
		Ok(providers) => return Ok(providers),
		Err(e) => throw!("Error getting providers: {}", e),
	};
}

#[command]
#[specta::specta]
pub async fn set_api_keys(providers: Vec<ProviderData>, data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	for provider in providers.iter() {
		let query = sqlx::query("UPDATE providers SET api_key = ? WHERE provider_name = ?")
			.bind(&provider.api_key)
			.bind(&provider.provider_name);

		// Execute the update query for the current provider
		match query.execute(&data.db_pool).await {
			Ok(_) => println!(
				"Successfully updated API key for provider: {}",
				&provider.provider_name
			),
			Err(e) => {
				eprintln!(
					"Error updating API key for provider {}: {:?}",
					&provider.provider_name, e
				);
				return Err(format!(
					"Error updating API key for provider {}: {:?}",
					&provider.provider_name, e
				));
			}
		}
	}

	Ok(())
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct Message {
	pub id: String,
	pub role: String,
	pub content: String,
	pub model_name: String,
	pub blocks: Option<MessageBlocks>,
}

impl sqlx::FromRow<'_, SqliteRow> for Message {
	fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
		Ok(Message {
			id: row.try_get("id")?,
			role: row.try_get("role")?,
			content: row.try_get("content")?,
			model_name: row.try_get("model_name")?,
			blocks: None,
		})
	}
}

#[derive(Serialize, Deserialize, Debug, FromRow, Type)]
pub struct MessageHistory {
	pub messages: Vec<Message>,
}

impl MessageHistory {
	pub fn render(&self, provider_name: &str) -> Value {
		match provider_name {
			"openai" => {
				serde_json::json!(self
					.messages
					.iter()
					.map(|message| {
						serde_json::json!({
							"role": message.role,
							"content": message.content
						})
					})
					.collect::<Vec<_>>())
			}
			"anthropic" => {
				serde_json::json!(self
					.messages
					.iter()
					.map(|message| {
						serde_json::json!({
							"role": message.role,
							"content": message.content
						})
					})
					.collect::<Vec<_>>())
			}
			"google" => serde_json::Value::Null,
			"alephalpha" => serde_json::Value::Null,
			_ => serde_json::Value::Null,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Type, FromRow)]
pub struct Model {
	pub provider_name: String,
	pub model_name: String,
	pub model_display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Type, FromRow)]
pub struct Models {
	models: Vec<Model>,
}

#[command]
#[specta::specta]
pub async fn get_models(data: DataState<'_>) -> Result<Models, String> {
	let data = data.0.lock().await;
	let models_query = "SELECT provider_name, model_name, model_display_name FROM models WHERE provider_name IN (SELECT provider_name FROM providers WHERE api_key != '')";
	let models_query_result = sqlx::query_as::<_, Model>(models_query)
		.fetch_all(&data.db_pool)
		.await;
	match models_query_result {
		Ok(models) => {
			// println!("Fetched models from database: {:?}", models);
			Ok(Models { models })
		}
		Err(e) => {
			println!("Error fetching models from database: {}", e.to_string());
			Err(e.to_string())
		}
	}
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Chat {
	pub id: String,
	pub display_name: String,
	pub creation_date: String,
	pub last_updated: String,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct Chats {
	pub chats: Vec<Chat>,
}

#[command]
#[specta::specta]
pub async fn get_chats(data: DataState<'_>) -> Result<Chats, String> {
	let data = data.0.lock().await;
	let fetch_query =
		"SELECT id, display_name, creation_date, last_updated FROM chats WHERE archived = 'false'";
	let chats_result = sqlx::query_as::<_, (String, String, String, String)>(fetch_query)
		.fetch_all(&data.db_pool)
		.await
		.map_err(|e| e.to_string())?;

	let mut chats: Chats = Chats { chats: Vec::new() };
	for (id, display_name, creation_date, last_updated) in chats_result {
		chats.chats.push(Chat {
			id: id,
			display_name: display_name,
			creation_date: creation_date,
			last_updated: last_updated,
		});
	}
	chats
		.chats
		.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));

	Ok(chats)
}

#[command]
#[specta::specta]
pub async fn load_chat(chat_id: String, data: DataState<'_>) -> Result<Vec<Message>, String> {
	let data = data.0.lock().await;
	let fetch_query = "SELECT id, role, content, model_name FROM messages WHERE chat_id = $1";
	let messages_result = sqlx::query_as::<_, Message>(fetch_query)
		.bind(&chat_id)
		.fetch_all(&data.db_pool)
		.await;

	match messages_result {
		Ok(mut messages) => {
			let message_blocks_fetch_query =
				"SELECT type_, language, raw_content, rendered_content, copied FROM message_blocks WHERE message_id = $1";
			for message in messages.iter_mut() {
				let _ = match sqlx::query_as::<_, MessageBlock>(message_blocks_fetch_query)
					.bind(&message.id)
					.fetch_all(&data.db_pool)
					.await
				{
					Ok(message_blocks) => {
						message.blocks = Some(MessageBlocks {
							blocks: message_blocks,
						})
					}
					Err(err) => {
						eprintln!("Error fetching message blocks from database: {}", err);
					}
				};
			}

			return Ok(messages);
		}
		Err(e) => {
			eprintln!("Error fetching messages from database: {}", e);
			Err(e.to_string())
		}
	}
}

pub async fn insert_message(
	new_message_id: &str,
	role: &str,
	message: &str,
	chat_id: &str,
	model_name: &str,
	connection_pool: &Pool<Sqlite>,
) {
	let insert_message_query: &str =
		"INSERT INTO messages (id, role, content, chat_id, model_name) VALUES ($1, $2, $3, $4, $5)";
	let _ = sqlx::query(insert_message_query)
		.bind(&new_message_id)
		.bind(&role)
		.bind(&message)
		.bind(&chat_id)
		.bind(&model_name)
		.execute(connection_pool)
		.await;
}

pub async fn insert_message_blocks(
	message_id: &str,
	message_blocks: &MessageBlocks,
	connection_pool: &Pool<Sqlite>,
) {
	let insert_message_blocks_query: &str =
		"INSERT INTO message_blocks (message_id, type_, language, raw_content, rendered_content, copied) VALUES ($1, $2, $3, $4, $5, $6)";
	for block in message_blocks.blocks.iter() {
		let insert_message_blocks_query_result = sqlx::query(insert_message_blocks_query)
			.bind(&message_id)
			.bind(&block.type_)
			.bind(&block.language)
			.bind(&block.raw_content)
			.bind(&block.rendered_content)
			.bind(0)
			.execute(connection_pool)
			.await;
		match insert_message_blocks_query_result {
			Ok(_) => (),
			Err(e) => {
				eprintln!("Error inserting message blocks into database: {}", e);
			}
		}
	}
}

#[command]
#[specta::specta]
pub async fn read_api_keys_from_env(data: DataState<'_>) -> Result<(), String> {
	let data = data.0.lock().await;
	dotenv().ok();
	let mut api_keys = HashMap::new();
	api_keys.insert("google", env::var("google").unwrap_or("".to_string()));
	api_keys.insert("openai", env::var("openai").unwrap_or("".to_string()));
	api_keys.insert("anthropic", env::var("anthropic").unwrap_or("".to_string()));
	api_keys.insert("mistralai", env::var("mistralai").unwrap_or("".to_string()));
	let insert_api_keys_query: &str = "UPDATE providers SET api_key=$1 WHERE provider_name = $2";
	for (provider_name, api_key) in api_keys.iter() {
		match sqlx::query(insert_api_keys_query)
			.bind(&api_key)
			.bind(&provider_name)
			.execute(&data.db_pool)
			.await
		{
			Ok(_) => {
				// println!(
				// 	"API key for provider {} saved to the database",
				// 	&provider_name
				// );
			}
			Err(e) => {
				eprintln!(
					"Error saving API key for provider {}: {}",
					&provider_name, e
				);
				return Err(format!(
					"Error saving API key for provider {}: {}",
					&provider_name, e
				));
			}
		}
	}
	return Ok(());
}
