use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

// Define the response structure for chat completion according to the Mistral API spec
#[derive(Serialize, Deserialize, Debug)]
pub struct GroqChatCompletionResponse {
	id: String,
	object: String,
	created: i64,
	model: String,
	pub choices: Vec<GroqChoice>,
	usage: GroqUsage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroqChoice {
	index: i32,
	pub message: GroqMessage,
	finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroqMessage {
	role: String,
	content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GroqUsage {
	prompt_tokens: i32,
	completion_tokens: i32,
	total_tokens: i32,
}

pub async fn send_groqcloud_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
	let url = "https://api.groq.com/openai/v1/chat/completions"; // URL for Groq chat completions
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

	let parsed_response: GroqChatCompletionResponse =
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
