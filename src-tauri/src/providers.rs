// use candle_transformers::models::whisper::model;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use specta::Type;
use sqlx::prelude::FromRow;
use tauri::command;

use crate::{
	data::DataState,
	db::{get_api_key, get_chat_display_name, get_messages, insert_chat_display_name, insert_message, insert_message_blocks, Message, MessageHistory},
	utils::{render_message, MessageBlocks},
};

// use self::local::send_local_message;
// use crate::utils::truncate_string;

use self::{anthropic::send_anthropic_message, groqcloud::send_groqcloud_message, mistralai::send_mistralai_message, openai::send_openai_message};

pub mod anthropic;
pub mod groqcloud;
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
pub async fn get_message(msg: String, chat_id: String, provider_name: String, model_name: String, data: DataState<'_>) -> Result<String, String> {
	let messages: MessageHistory;
	let mut api_key: String = "".to_string();

	let new_message_id = uuid::Uuid::new_v4().to_string();
	insert_message(&new_message_id, "user", &msg, &chat_id, &model_name, data.clone()).await;
	let code_theme = &data.0.lock().await.settings.code_theme.clone();
	insert_message_blocks(&new_message_id, &render_message(&msg, code_theme).await, data.clone()).await;

	// emit event that a new message is in the database
	let _ = data.0.lock().await.window.emit("newMessage", &chat_id);

	match &get_chat_display_name(&chat_id, data.clone()).await {
		// If the display name exists, do nothing
		Ok(Some(_display_name)) => {}
		// If the display name does not exist, insert a new one
		Ok(None) => {
			match insert_chat_display_name(&chat_id, &model_name, &format!("unnamed_new_chat_{}", &chat_id), data.clone()).await {
				Ok(_) => {
					// emit event that a new chat is in the database
					let _ = data.0.lock().await.window.emit("newChat", &chat_id);
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

	if provider_name != "local" {
		// Get the API key from the providers table
		api_key = match get_api_key(&provider_name, data.clone()).await {
			Ok(s) => s,
			Err(e) => return Err(e),
		}
	}

	// Get the messages for the current chat from the messages table (including the latest user's message)
	messages = match get_messages(&chat_id, data.clone()).await {
		Ok(messages) => messages,
		Err(e) => {
			return Err(e);
		}
	};

	let mut answer: String;
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
		"groqcloud" => {
			let body: Value = json!({
				"model": &model_name,
				"messages": messages.render("openai"),
				"temperature": 0.7,
				"max_tokens": 4096
			});
			answer = match send_groqcloud_message(body, &api_key).await {
				Ok(answer) => answer,
				Err(e) => {
					eprintln!("Error sending message to Groq: {}", e);
					e.to_string()
				}
			};
		}
		// "local" => {
		// 	let body: Value = json!({
		// 		"model": &model_name,
		// 		"messages": messages.render("openai"),
		// 		"temperature": 0.7,
		// 		"max_tokens": 512
		// 	});
		// 	answer = match send_local_message(body).await {
		// 		Ok(answer) => answer,
		// 		Err(e) => {
		// 			eprintln!("Error sending message to Local: {}", e);
		// 			e.to_string()
		// 		}
		// 	};
		// }
		// "google" => {}
		_ => {
			answer = format!("Provider {provider_name} not implemented");
		}
	}

	let new_answer_id = uuid::Uuid::new_v4().to_string();
	insert_message(&new_answer_id, "assistant", &answer, &chat_id, &model_name, data.clone()).await;
	let rendered_answer: MessageBlocks = render_message(&answer, &data.0.lock().await.settings.code_theme).await;
	insert_message_blocks(&new_answer_id, &rendered_answer, data.clone()).await;

	// emit event that a new message is in the database
	let _ = data.0.lock().await.window.emit("newMessage", &chat_id);

	let chats_result = get_chat_display_name(&chat_id, data.clone()).await;

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
							new_chat_display_name = match send_openai_message(body, &api_key).await {
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
							new_chat_display_name = match send_anthropic_message(body, &api_key).await {
								Ok(answer) => answer,
								Err(e) => {
									answer = format!("Error fetching chat name from Anthropic: {}", e);
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
							new_chat_display_name = match send_mistralai_message(body, &api_key).await {
								Ok(answer) => answer,
								Err(e) => {
									answer = format!("Error fetching chat name from Mistral: {}", e);
									eprintln!("{}", &answer);
									return Ok(answer);
								}
							}
						}
						"groqcloud" => {
							let body = json!({
								"model": &model_name,
								"messages": display_name_messages.render("openai"),
								"temperature": 0.7,
								"max_tokens": &MAX_DISPLAY_NAME_LENGTH,
							});
							new_chat_display_name = match send_groqcloud_message(body, &api_key).await {
								Ok(answer) => answer,
								Err(e) => {
									answer = format!("Error fetching chat name from Groq: {}", e);
									eprintln!("{}", &answer);
									return Ok(answer);
								}
							}
						}
						// "local" => {
						// 	new_chat_display_name = truncate_string(&chat_id, 10).to_string()
						// }
						_ => new_chat_display_name = "Error fetching chat name".to_string(),
					}
					// let data = data.0.lock().await;
					// update the display_name field in the chats database
					let update_chat_display_name_query: &str = "UPDATE chats SET display_name = $1 WHERE id = $2";
					let _ = sqlx::query(update_chat_display_name_query)
						.bind(&new_chat_display_name)
						.bind(&chat_id)
						.execute(&data.0.lock().await.db_pool)
						.await
						.map_err(|e| {
							eprintln!("Error updating display name in database: {}", e);
							e.to_string()
						})?;
					// emit event saying there are new chats
					let _ = &data.0.lock().await.window.emit("newChat", ());
					//let _ = data.window.emit("newMessage", &chat_id);
				}
				false => {
					//let data = data.0.lock().await;
					// update the last_updated field in the chats database to the current time
					let update_last_updated_query: &str = "UPDATE chats SET last_updated = CURRENT_TIMESTAMP WHERE id = $1";
					let _ = sqlx::query(update_last_updated_query)
						.bind(&chat_id)
						.execute(&data.0.lock().await.db_pool)
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
