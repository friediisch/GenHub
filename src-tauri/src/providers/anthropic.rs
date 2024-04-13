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

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicErrorResponse {
	#[serde(rename = "type")]
	error_type: String,
	error: AnthropicErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicErrorDetails {
	#[serde(rename = "type")]
	error_type: String,
	message: String,
}

pub async fn send_anthropic_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
	let url = "https://api.anthropic.com/v1/messages";
	let client = Client::new();
	let response = client
		.post(url)
		.header("Content-Type", "application/json")
		.header("anthropic-version", "2023-06-01")
		.header("x-api-key", api_key)
		.json(&body)
		.send()
		.await;

	let response_text = match response {
		Ok(response) => response.text().await.unwrap(),
		Err(err) => {
			eprintln!("Error: {:?}", err);
			return Ok(format!("Error: {:?}", err));
		}
	};

	let result = serde_json::from_str::<AnthropicChatResponse>(&response_text);
	let answer = match result {
		Ok(parsed_response) => parsed_response
			.content
			.get(0)
			.ok_or("Invalid response".to_string())
			.unwrap()
			.text
			.clone(),
		Err(_e) => {
			let error_result = serde_json::from_str::<AnthropicErrorResponse>(&response_text);
			match error_result {
				Ok(parsed_error) => format!(
					"{}: {}",
					parsed_error.error.error_type, parsed_error.error.message
				),
				Err(_e) => "Unknown error".to_string(),
			}
		}
	};
	Ok(answer)
}
