use std::error::Error;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnthropicChatResponse {
	content: Vec<MessageContent>,
	id: String,
	model: String,
	role: String,
	stop_reason: String,
	stop_sequence: Option<String>,
	#[serde(rename = "type")]
	message_type: String,
	usage: AnthropicUsage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageContent {
	text: String,
	#[serde(rename = "type")]
	content_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AnthropicUsage {
	input_tokens: i32,
	output_tokens: i32,
}

// struct for anthropic error response:
// {"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}

// pub struct AnthropicErrorResponse {
// 	error: AnthropicError,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct AnthropicError {
// 	type_: String,
// 	message: String,
// }

pub async fn send_anthropic_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
	let url = "https://api.anthropic.com/v1/messages";
	let client = Client::new();
	let response: Response = client
		.post(url)
		.header("Content-Type", "application/json")
		.header("anthropic-version", "2023-06-01")
		.header("x-api-key", api_key)
		.json(&body)
		.send()
		.await
		.map_err(|err| err.to_string())?;

	let response_text: String = response.text().await.map_err(|err| err.to_string())?;

	let parsed_response: AnthropicChatResponse =
		serde_json::from_str(&response_text).map_err(|err| err.to_string())?;

	let answer: String = parsed_response
		.content
		.get(0)
		.ok_or("No response".to_string())?
		.text
		.clone();

	Ok(answer)
}
