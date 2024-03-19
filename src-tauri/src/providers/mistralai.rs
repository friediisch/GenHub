use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

// Define the response structure for chat completion according to the Mistral API spec
#[derive(Serialize, Deserialize, Debug)]
pub struct MistralChatCompletionResponse {
	id: String,
	object: String,
	created: i64,
	model: String,
	pub choices: Vec<MistralChoice>,
	usage: MistralUsage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MistralChoice {
	index: i32,
	pub message: MistralMessage,
	finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MistralMessage {
	role: String,
	content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MistralUsage {
	prompt_tokens: i32,
	completion_tokens: i32,
	total_tokens: i32,
}

// Function to send a message to the Mistral API
pub async fn send_mistralai_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
	let url = "https://api.mistral.ai/v1/chat/completions"; // URL for Mistral chat completions
	let client = Client::new();
	let response: Response = client
		.post(url)
		.header("Content-Type", "application/json")
		.header("Authorization", format!("Bearer {}", api_key))
		.json(&body)
		.send()
		.await
		.map_err(|err| err.to_string())?;

	let response_text: String = response.text().await.map_err(|err| err.to_string())?;

	let parsed_response: MistralChatCompletionResponse =
		serde_json::from_str(&response_text).map_err(|err| err.to_string())?;

	let answer: String = parsed_response
		.choices
		.get(0)
		.ok_or("No response".to_string())?
		.message
		.content
		.clone();

	Ok(answer)
}
