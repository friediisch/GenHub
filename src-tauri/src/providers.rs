use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use specta::Type;
use sqlx::prelude::FromRow;
use tauri::command;

use crate::{
	data::DataState,
	db::{insert_message, insert_message_blocks, Message, MessageHistory},
	utils::{render_message, truncate_string, MessageBlocks},
};

use self::{
	anthropic::send_anthropic_message, local::send_local_message,
	mistralai::send_mistralai_message, openai::send_openai_message,
};

pub mod anthropic;
pub mod local;
pub mod mistralai;
pub mod openai;

#[derive(Serialize, Deserialize, Clone, Debug, FromRow, Type)]
pub struct ProviderData {
	pub provider_name: String,
	pub api_key: String,
	pub display_name: String,
}

#[command]
#[specta::specta]
pub async fn get_message(
	msg: String,
	chat_id: String,
	model_name: String,
	data: DataState<'_>,
) -> Result<String, String> {
	let messages: MessageHistory;
	let provider_name: String;
	let mut api_key: String = "".to_string();
	let mut chats_result: Result<Option<(String,)>, sqlx::Error>;
	{
		let data = data.0.lock().await;
		let new_message_id = uuid::Uuid::new_v4().to_string();
		insert_message(
			&new_message_id,
			"user",
			&msg,
			&chat_id,
			&model_name,
			&data.db_pool,
		)
		.await;
		insert_message_blocks(
			&new_message_id,
			&render_message(&msg, &data.settings.code_theme).await,
			&data.db_pool,
		)
		.await;

		println!("New message inserted into the database");
		// emit event that a new message is in the database
		let _ = data.window.emit("newMessage", &chat_id);

		let chat_display_name_query: &str = "SELECT display_name FROM chats WHERE id = $1";
		chats_result = sqlx::query_as(chat_display_name_query)
			.bind(&chat_id)
			.fetch_optional(&data.db_pool)
			.await;

		match &chats_result {
			Ok(Some(_display_name)) => {}
			Ok(None) => {
				let insert_chat_display_name_query = "INSERT INTO chats (id, model, api_key_id, display_name, archived, last_updated) VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)";
				let query_result = sqlx::query(insert_chat_display_name_query)
					.bind(&chat_id)
					.bind(&model_name)
					.bind("NA")
					.bind(format!("unnamed_new_chat_{}", &chat_id))
					.bind("false")
					.execute(&data.db_pool)
					.await;
				match query_result {
					Ok(_) => {
						println!("New message inserted into the database #2");
						// emit event that a new message is in the database
						let _ = data.window.emit("newMessage", &chat_id);
					}
					Err(e) => {
						eprintln!("Error inserting display name into database: {}", e);
					}
				}
			}
			Err(e) => {
				eprintln!("Error fetching display name from database: {}", e);
			}
		}

		let _ = data.window.emit("newMessage", &chat_id);

		// Get the provider name from the models table
		// TODO: Now there are multiple providers, e.g. for "Mistral-7B"
		let provider_name_query: &str = "SELECT provider_name FROM models WHERE model_name = $1";
		(provider_name,) = sqlx::query_as::<_, (String,)>(provider_name_query)
			.bind(&model_name)
			.fetch_one(&data.db_pool)
			.await
			.map_err(|e| e.to_string())?;

		if provider_name != "local" {
			// Get the API key from the providers table
			let api_key_query: &str = "SELECT api_key FROM providers WHERE provider_name = $1";
			(api_key,) = sqlx::query_as::<_, (String,)>(api_key_query)
				.bind(&provider_name)
				.fetch_one(&data.db_pool)
				.await
				.map_err(|e| e.to_string())?;
		}

		// Get the messages for the current chat from the messages table (including the latest user's message)
		let messages_query: &str =
			"SELECT id, role, content, model_name FROM messages WHERE chat_id = $1";
		messages = MessageHistory {
			messages: sqlx::query_as::<_, Message>(messages_query)
				.bind(&chat_id)
				.fetch_all(&data.db_pool)
				.await
				.map_err(|e| {
					eprintln!("Error getting messages: {}", e);
					e.to_string()
				})?,
		};
	}

	let mut answer: String = "Not implemented yet".to_string();
	match provider_name.as_str() {
		"openai" => {
			let body: Value = json!({
				"model": &model_name,
				"messages": messages.render("openai"),
				"temperature": 0.7,
				"max_tokens": 4096
			});
			answer = match send_openai_message(body, &api_key).await {
				Ok(answer) => answer,
				Err(e) => {
					eprintln!("Error sending message to OpenAI: {}", e);
					e.to_string()
				}
			};
		}
		"anthropic" => {
			let body: Value = json!({
				"model": &model_name,
				"max_tokens": 4096,
				"messages": messages.render("anthropic"),
			});
			answer = match send_anthropic_message(body, &api_key).await {
				Ok(answer) => answer,
				Err(e) => {
					eprintln!("Error sending message to Anthropic: {}", e);
					e.to_string()
				}
			};
		}
		"mistralai" => {
			let body: Value = json!({
				"model": &model_name,
				"messages": messages.render("openai"),
				"temperature": 0.7,
				"max_tokens": 4096
			});
			answer = match send_mistralai_message(body, &api_key).await {
				Ok(answer) => answer,
				Err(e) => {
					eprintln!("Error sending message to Mistral: {}", e);
					e.to_string()
				}
			};
		}
		"local" => {
			let body: Value = json!({
				"model": &model_name,
				"messages": messages.render("openai"),
				"temperature": 0.7,
				"max_tokens": 512
			});
			answer = match send_local_message(body).await {
				Ok(answer) => answer,
				Err(e) => {
					eprintln!("Error sending message to Local: {}", e);
					e.to_string()
				}
			};
		}
		"google" => {}
		_ => {
			return Err("Provider not found".to_string());
		}
	}

	println!("Sleeping #2");
	std::thread::sleep(Duration::from_secs(3));

	{
		let data = data.0.lock().await;

		let new_answer_id = uuid::Uuid::new_v4().to_string();
		insert_message(
			&new_answer_id,
			"assistant",
			&answer,
			&chat_id,
			&model_name,
			&data.db_pool,
		)
		.await;
		let rendered_answer: MessageBlocks =
			render_message(&answer, &data.settings.code_theme).await;
		insert_message_blocks(&new_answer_id, &rendered_answer, &data.db_pool).await;

		// emit event that a new message is in the database
		let _ = data.window.emit("newMessage", &chat_id);

		let chat_display_name_query: &str = "SELECT display_name FROM chats WHERE id = $1";
		chats_result = sqlx::query_as(chat_display_name_query)
			.bind(&chat_id)
			.fetch_optional(&data.db_pool)
			.await;
	}

	const MAX_DISPLAY_NAME_LENGTH: usize = 32;

	match chats_result {
		Ok(Some((display_name,))) => {
			match display_name.starts_with("unnamed_new_chat_") {
				true => {
					let display_name_messages: MessageHistory = MessageHistory {
						messages: vec![Message {
							id: "none".to_string(),
							role: "user".to_string(),
							content: format!(
								"Please respond with the topic of the thread for these two messages:
								'user': '{msg}',
								'assistant': '{answer}'
								Your response will be used to name the chat, therefore omit any other content from your response, keep it short and use the language used in the prompt.
								Do not use quotation marks. Capitalize the first letter of your answer."
							),
							model_name: "".into(),
							blocks: None,
						}],
					};

					let new_chat_display_name: String;
					match provider_name.as_str() {
						"openai" => {
							let body = json!({
								"model": &model_name,
								"messages": display_name_messages.render("openai"),
								"temperature": 0.7,
								"max_tokens": &MAX_DISPLAY_NAME_LENGTH,
							});
							new_chat_display_name = match send_openai_message(body, &api_key).await
							{
								Ok(answer) => answer,
								Err(e) => {
									answer = format!("Error fetching chat name from OpenAI: {}", e);
									eprintln!("{}", &answer);
									return Ok(answer);
								}
							}
						}
						"anthropic" => {
							let body = json!({
								"model": &model_name,
								"max_tokens": &MAX_DISPLAY_NAME_LENGTH,
								"messages": display_name_messages.render("anthropic"),
							});
							new_chat_display_name = match send_anthropic_message(body, &api_key)
								.await
							{
								Ok(answer) => answer,
								Err(e) => {
									answer =
										format!("Error fetching chat name from Anthropic: {}", e);
									eprintln!("{}", &answer);
									return Ok(answer);
								}
							}
						}
						"mistralai" => {
							let body = json!({
								"model": &model_name,
								"messages": display_name_messages.render("openai"),
								"temperature": 0.7,
								"max_tokens": &MAX_DISPLAY_NAME_LENGTH,
							});
							new_chat_display_name =
								match send_mistralai_message(body, &api_key).await {
									Ok(answer) => answer,
									Err(e) => {
										answer =
											format!("Error fetching chat name from Mistral: {}", e);
										eprintln!("{}", &answer);
										return Ok(answer);
									}
								}
						}
						"local" => {
							new_chat_display_name = truncate_string(&chat_id, 10).to_string()
						}
						_ => new_chat_display_name = "Error fetching chat name".to_string(),
					}
					let data = data.0.lock().await;
					// update the display_name field in the chats database
					let update_chat_display_name_query: &str =
						"UPDATE chats SET display_name = $1 WHERE id = $2";
					let _ = sqlx::query(update_chat_display_name_query)
						.bind(&new_chat_display_name)
						.bind(&chat_id)
						.execute(&data.db_pool)
						.await
						.map_err(|e| {
							eprintln!("Error updating display name in database: {}", e);
							e.to_string()
						})?;
					// emit event saying there are new chats
					let _ = data.window.emit("newChat", ());
					//let _ = data.window.emit("newMessage", &chat_id);
				}
				false => {
					let data = data.0.lock().await;
					// update the last_updated field in the chats database to the current time
					let update_last_updated_query: &str =
						"UPDATE chats SET last_updated = CURRENT_TIMESTAMP WHERE id = $1";
					let _ = sqlx::query(update_last_updated_query)
						.bind(&chat_id)
						.execute(&data.db_pool)
						.await
						.map_err(|e| e.to_string())?;
				}
			}
		}
		Ok(None) => {
			eprintln!("Chat not found in the database");
		}
		Err(e) => {
			eprintln!("Error fetching display name from database: {}", e);
		}
	}
	Ok(answer)
}
