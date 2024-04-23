#![allow(unused)]
// #[cfg(feature = "mkl")]
// extern crate intel_mkl_src;

// #[cfg(feature = "accelerate")]
// extern crate accelerate_src;

// use std::error::Error;

use anyhow::{Error as E, Result};
// use clap::Parser;

use candle_transformers::models::mistral::{Config, Model as Mistral};
use candle_transformers::models::quantized_mistral::Model as QMistral;

use candle_core::{DType, Device, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use hf_hub::{api::sync::Api, Repo, RepoType};
use minijinja::{context, Environment};
use serde_json::Value;
use tokenizers::Tokenizer;

enum Model {
	Mistral(Mistral),
	Quantized(QMistral),
}

struct TextGeneration {
	model: Model,
	device: Device,
	tokenizer: TokenOutputStream,
	logits_processor: LogitsProcessor,
	repeat_penalty: f32,
	repeat_last_n: usize,
}

impl TextGeneration {
	#[allow(clippy::too_many_arguments)]
	fn new(
		model: Model,
		tokenizer: Tokenizer,
		seed: u64,
		temp: Option<f64>,
		top_p: Option<f64>,
		top_k: Option<usize>,
		repeat_penalty: f32,
		repeat_last_n: usize,
		device: &Device,
	) -> Self {
		let logits_processor = {
			let temperature = temp.unwrap_or(0.);
			let sampling = if temperature <= 0. {
				Sampling::ArgMax
			} else {
				match (top_k, top_p) {
					(None, None) => Sampling::All { temperature },
					(Some(k), None) => Sampling::TopK { k, temperature },
					(None, Some(p)) => Sampling::TopP { p, temperature },
					(Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
				}
			};
			LogitsProcessor::from_sampling(seed, sampling)
		};

		Self {
			model,
			tokenizer: TokenOutputStream::new(tokenizer),
			logits_processor,
			repeat_penalty,
			repeat_last_n,
			device: device.clone(),
		}
	}

	fn run(&mut self, prompt: &str, sample_len: usize) -> Result<String> {
		use std::io::Write;
		self.tokenizer.clear();
		let mut tokens = self
			.tokenizer
			.tokenizer()
			.encode(prompt, true)
			.map_err(E::msg)?
			.get_ids()
			.to_vec();
		let mut answer = String::new();
		for &t in tokens.iter() {
			if let Some(t) = self.tokenizer.next_token(t)? {
				print!("{t}")
			}
		}
		std::io::stdout().flush()?;

		let mut generated_tokens = 0usize;
		let eos_token = match self.tokenizer.get_token("</s>") {
			Some(token) => token,
			None => panic!("cannot find the </s> token"),
		};
		let start_gen = std::time::Instant::now();
		for index in 0..sample_len {
			let context_size = if index > 0 { 1 } else { tokens.len() };
			let start_pos = tokens.len().saturating_sub(context_size);
			let ctxt = &tokens[start_pos..];
			let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
			let logits = match &mut self.model {
				Model::Mistral(m) => m.forward(&input, start_pos)?,
				Model::Quantized(m) => m.forward(&input, start_pos)?,
			};
			let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
			let logits = if self.repeat_penalty == 1. {
				logits
			} else {
				let start_at = tokens.len().saturating_sub(self.repeat_last_n);
				candle_transformers::utils::apply_repeat_penalty(
					&logits,
					self.repeat_penalty,
					&tokens[start_at..],
				)?
			};

			let next_token = self.logits_processor.sample(&logits)?;
			tokens.push(next_token);
			generated_tokens += 1;
			if next_token == eos_token {
				println!("</s> token generated, stopping");
				break;
			}
			if let Some(t) = self.tokenizer.next_token(next_token)? {
				print!("{t}");
				answer.push_str(&t);
				std::io::stdout().flush()?;
			}
		}
		let dt = start_gen.elapsed();
		if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
			print!("{rest}");
		}
		std::io::stdout().flush()?;
		println!(
			"\n{generated_tokens} tokens generated ({:.2} token/s)",
			generated_tokens as f64 / dt.as_secs_f64(),
		);
		Ok(answer)
	}
}

// #[derive(Clone, Debug, Copy, PartialEq, Eq)]
// enum Which {
// 	// Mistral7bV01,
// 	// Mistral7bV02,
// 	// Mistral7bInstructV01,
// 	// Mistral7bInstructV02,
// }

#[derive(Debug)]
struct Mistral7BArgs {
	/// Run on CPU rather than on GPU.
	cpu: bool,

	/// Enable tracing (generates a trace-timestamp.json file).
	// tracing: bool,
	use_flash_attn: bool,

	prompt: String,

	/// The temperature used to generate samples.
	temperature: Option<f64>,

	/// Nucleus sampling probability cutoff.
	top_p: Option<f64>,

	/// Only sample among the top K samples.
	top_k: Option<usize>,

	/// The seed to use when generating random samples.
	seed: u64,

	/// The length of the sample to generate (in tokens).
	sample_len: usize,

	/// The model size to use.
	// which: Which,

	// model_id: Option<String>,
	revision: String,

	tokenizer_file: Option<String>,

	config_file: Option<String>,

	weight_files: Option<String>,

	quantized: bool,

	/// Penalty to be applied for repeating tokens, 1. means no penalty.
	repeat_penalty: f32,

	/// The context size to consider for the repeat penalty.
	repeat_last_n: usize,
	// Use the slower dmmv cuda kernel.
	// force_dmmv: bool,
}

pub async fn send_local_message(body: Value) -> Result<String> {
	let mut env = Environment::new();
	env.add_template(
		"chat_completion_template",
		r#"
{% for message in messages %}
{{ message.role }}
{{ message.content }}

{% endfor %}
{% if add_generation_prompt %}
assistant
{% endif %}
"#,
	)
	.unwrap();
	let tmpl = env.get_template("chat_completion_template").unwrap();
	let prompt = tmpl
		.render(context! {
			messages => body.get("messages").unwrap(),
			add_generation_prompt => false,
		})
		.unwrap();

	let args = Mistral7BArgs {
		cpu: false,
		// tracing: false,
		use_flash_attn: false,
		prompt: prompt,
		temperature: None,
		top_p: None,
		top_k: None,
		seed: 0,
		sample_len: body.get("max_tokens").unwrap().to_string().parse().unwrap(),
		// which: Which::Mistral7bV01,
		// model_id: None,
		revision: "main".to_string(),
		tokenizer_file: None,
		config_file: None,
		weight_files: None,
		// quantized: false,
		quantized: true,
		repeat_penalty: 1.,
		repeat_last_n: 1,
		// force_dmmv: false,
	};
	#[cfg(feature = "cuda")]
	candle::quantized::cuda::set_force_dmmv(args.force_dmmv);

	// let _guard = if args.tracing {
	// 	let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
	// 	tracing_subscriber::registry().with(chrome_layer).init();
	// 	Some(guard)
	// } else {
	// 	None
	// };
	println!(
		"avx: {}, neon: {}, simd128: {}, f16c: {}",
		candle_core::utils::with_avx(),
		candle_core::utils::with_neon(),
		candle_core::utils::with_simd128(),
		candle_core::utils::with_f16c()
	);
	println!(
		"temp: {:.2} repeat-penalty: {:.2} repeat-last-n: {}",
		args.temperature.unwrap_or(0.),
		args.repeat_penalty,
		args.repeat_last_n
	);

	let start = std::time::Instant::now();
	let api = Api::new()?;
	let model_id = match args.quantized {
		true => "lmz/candle-mistral".to_string(),
		false => "mistralai/Mistral-7B-v0.1".to_string(),
	};
	// let model_id = "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string();
	// let model_id = match args.model_id {
	// 	Some(model_id) => model_id,
	// 	None => {
	// 		if args.quantized {
	// 			if args.which != Which::Mistral7bV01 {
	// 				// anyhow::bail!("only 7b-v0.1 is available as a quantized model for now")
	// 			}
	// 			"lmz/candle-mistral".to_string()
	// 		} else {
	// 			match args.which {
	// 				Which::Mistral7bV01 => "mistralai/Mistral-7B-v0.1".to_string(),
	// 				Which::Mistral7bV02 => "mistralai/Mistral-7B-v0.2".to_string(),
	// 				Which::Mistral7bInstructV01 => "mistralai/Mistral-7B-Instruct-v0.1".to_string(),
	// 				Which::Mistral7bInstructV02 => "mistralai/Mistral-7B-Instruct-v0.2".to_string(),
	// 			}
	// 		}
	// 	}
	// };
	let repo = api.repo(Repo::with_revision(
		model_id,
		RepoType::Model,
		args.revision,
	));
	println!("Retrieving tokenizer");
	let tokenizer_filename = match args.tokenizer_file {
		Some(file) => std::path::PathBuf::from(file),
		None => repo.get("tokenizer.json")?,
	};

	println!("retrieved the tokenizer in {:?}", start.elapsed());
	let filenames = match args.weight_files {
		Some(files) => {
			println!("CHP1");
			files
				.split(',')
				.map(std::path::PathBuf::from)
				.collect::<Vec<_>>()
		}
		None => {
			if args.quantized {
				vec![repo.get("model-q4k.gguf")?]
			} else {
				candle_examples::hub_load_safetensors(&repo, "model.safetensors.index.json")?
			}
		}
	};
	println!("retrieved the files in {:?}", start.elapsed());
	let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

	let start = std::time::Instant::now();
	let config = match args.config_file {
		Some(config_file) => serde_json::from_slice(&std::fs::read(config_file)?)?,
		None => {
			if args.quantized {
				Config::config_7b_v0_1(args.use_flash_attn)
			} else {
				let config_file = repo.get("config.json")?;
				serde_json::from_slice(&std::fs::read(config_file)?)?
			}
		}
	};
	let device = candle_examples::device(args.cpu)?;
	let (model, device) = if args.quantized {
		let filename = &filenames[0];
		let vb =
			candle_transformers::quantized_var_builder::VarBuilder::from_gguf(filename, &device)?;
		let model = QMistral::new(&config, vb)?;
		(Model::Quantized(model), device)
	} else {
		let dtype = if device.is_cuda() {
			DType::BF16
		} else {
			DType::F32
		};
		let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
		let model = Mistral::new(&config, vb)?;
		(Model::Mistral(model), device)
	};

	println!("loaded the model in {:?}", start.elapsed());

	let mut pipeline = TextGeneration::new(
		model,
		tokenizer,
		args.seed,
		args.temperature,
		args.top_p,
		args.top_k,
		args.repeat_penalty,
		args.repeat_last_n,
		&device,
	);
	let answer = pipeline.run(&args.prompt, args.sample_len)?;
	Ok(answer)
}
