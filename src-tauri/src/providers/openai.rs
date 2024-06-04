use std::error::Error;

use async_openai::{types::CreateCompletionRequestArgs, Client as OpenAIClient};
use futures::StreamExt;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIChatCompletionResponse {
	id: String,
	object: String,
	created: i64,
	model: String,
	pub choices: Vec<Choice>,
	usage: Usage,
	system_fingerprint: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
	index: i32,
	pub message: OpenAIMessage,
	logprobs: Option<serde_json::Value>,
	finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIMessage {
	role: String,
	content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Usage {
	prompt_tokens: i32,
	completion_tokens: i32,
	total_tokens: i32,
}

pub async fn send_openai_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
	let url = "https://api.openai.com/v1/chat/completions";
	let client = Client::new();
	let response: Response = client
		.post(url)
		.header("Content-Type", "application/json")
		.header("Authorization", format!("Bearer {}", &api_key))
		.json(&body)
		.send()
		.await
		.map_err(|err| err.to_string())?;

	let response_text: String = response.text().await.map_err(|err| err.to_string())?;

	let parsed_response: OpenAIChatCompletionResponse = serde_json::from_str(&response_text).map_err(|err| err.to_string())?;

	let answer: String = parsed_response.choices.get(0).ok_or("No response".to_string())?.message.content.clone();

	Ok(answer)
}

// pub async fn send_openai_message(body: Value, api_key: &str) -> Result<String, Box<dyn Error>> {
// 	let client = Client::new();

// 	let request = CreateCompletionRequestArgs::default()
// 		.model("gpt-3.5-turbo-instruct")
// 		.n(1)
// 		.prompt("Tell me a bedtime story about Optimus Prime and Bumblebee")
// 		.stream(true)
// 		.max_tokens(1024_u16)
// 		.build()?;

// 	let mut stream = client.completions().create_stream(request).await?;

// 	while let Some(response) = stream.next().await {
// 		match response {
// 			Ok(ccr) => ccr.choices.iter().for_each(|c| {
// 				print!("{}", c.text);
// 			}),
// 			Err(e) => eprintln!("{}", e),
// 		}
// 	}

// 	Ok(())
// }
