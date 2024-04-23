use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;
use std::fmt;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Serialize, Deserialize, Debug, FromRow, Type, Clone)]
pub struct MessageBlock {
	pub id: Option<i32>,
	pub type_: String,
	pub language: Option<String>,
	pub raw_content: String,
	pub rendered_content: String,
	pub copied: Option<bool>,
}

impl fmt::Display for MessageBlock {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.language {
			Some(lang) => write!(
				f,
				"{{\"type\": \"{}\", \"language\": \"{}\", \"content\": \"{}\"}}",
				self.type_, lang, self.rendered_content
			),
			None => write!(
				f,
				"{{\"type\": \"{}\", \"content\": \"{}\"}}",
				self.type_, self.rendered_content
			),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct MessageBlocks {
	pub blocks: Vec<MessageBlock>,
}

pub async fn render_message(message: &str, code_theme: &str) -> MessageBlocks {
	let mut message_blocks: MessageBlocks = MessageBlocks { blocks: vec![] };
	let regex = Regex::new(r"```([a-zA-Z]*\n[\s\S]*?)```").unwrap();
	let mut last_index = 0;

	for cap in regex.captures_iter(message) {
		let match_str = cap.get(0).unwrap().as_str();
		let code_with_lang = cap.get(1).unwrap().as_str();
		let offset = cap.get(0).unwrap().start();

		// Add the text before the code block, if any
		if offset > last_index {
			let raw_content = message[last_index..offset].trim().to_string();
			let rendered_content = process_text(raw_content.clone());
			message_blocks.blocks.push(MessageBlock {
				id: None,
				type_: "text".to_string(),
				language: None,
				raw_content: raw_content,
				rendered_content: rendered_content,
				copied: Some(false),
			});
		}

		// Extract the language from the code block, if present
		let mut lines = code_with_lang.lines();
		let first_line = lines.next().unwrap_or("");
		let (language, code) =
			if !first_line.is_empty() && first_line.chars().all(|c| c.is_alphabetic()) {
				(
					first_line.to_lowercase(),
					lines.collect::<Vec<_>>().join("\n"),
				)
			} else {
				("plain".to_string(), code_with_lang.to_string())
			};

		let highlighted_code: String = match highlight_code(&code, &language, code_theme) {
			Ok(highlighted_code) => highlighted_code.to_string(),
			Err(_) => code.to_string(),
		};

		// Add the code block
		message_blocks.blocks.push(MessageBlock {
			id: None,
			type_: "code".to_string(),
			language: Some(language),
			raw_content: code,
			rendered_content: highlighted_code,
			copied: Some(false),
		});

		// Update last_index to the end of the current match
		last_index = offset + match_str.len();
	}

	// Add any remaining text after the last code block
	if last_index < message.len() {
		let raw_content = message[last_index..].trim().to_string();
		let rendered_content = process_text(raw_content.clone());
		message_blocks.blocks.push(MessageBlock {
			id: None,
			type_: "text".to_string(),
			language: None,
			raw_content: raw_content,
			rendered_content: rendered_content,
			copied: Some(false),
		});
	}
	message_blocks
}

pub fn highlight_code(code: &str, language: &str, code_theme: &str) -> Result<String, String> {
	let ps: SyntaxSet = SyntaxSet::load_defaults_newlines();
	let ts: ThemeSet = ThemeSet::load_defaults();
	let capitalized_language: String = capitalize_first_letter(&language);
	let syntax: &syntect::parsing::SyntaxReference = ps
		.find_syntax_by_token(&capitalized_language)
		.unwrap_or_else(|| ps.find_syntax_by_extension("js").unwrap());
	let theme = &ts.themes[code_theme];
	let mut h: HighlightLines<'_> = HighlightLines::new(syntax, theme);
	let bgc = theme
		.settings
		.background
		.unwrap_or_else(|| syntect::highlighting::Color::BLACK);
	let mut result: String = String::new();
	result.push_str(&format!(
		"<div style=\"background-color:#{:02x}{:02x}{:02x};padding:0.75rem;border-bottom-right-radius:0.375rem;border-bottom-left-radius:0.375rem;overflow-x:scroll\">",
		bgc.r, bgc.g, bgc.b
	));
	result.push_str("<pre><code>");
	for line in LinesWithEndings::from(code) {
		let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
		let html = styled_line_to_highlighted_html(&ranges[..], IncludeBackground::Yes).unwrap();
		result.push_str(&html);
	}
	result.push_str("</code></pre>");
	result.push_str("</div>");
	Ok(result)
}

fn capitalize_first_letter(word: &str) -> String {
	let mut chars = word.chars();
	match chars.next() {
		None => word.to_string(),
		Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
	}
}

fn process_text(mut text: String) -> String {
	text = escape_html_tags(text);
	text = replace_backticks_with_span(text);
	text = replace_markdown_headers_with_html(&text);
	text = replace_linebreaks_with_br(text);
	text = replace_bold_with_html(text);
	text.to_string()
}

fn replace_backticks_with_span(str: String) -> String {
	let regex = Regex::new(r"`([^`]+)`").unwrap();
	regex
		.replace_all(&str, |caps: &regex::Captures| {
			format!(
				"<span class=\"bg-gray2 text-codefontcolor text-sm rounded-xs font-mono px-1\">{}</span>",
				&caps[1]
			)
		})
		.to_string()
}

fn replace_bold_with_html(input: String) -> String {
	// Regular expression to find text enclosed in '**'
	let bold_re = Regex::new(r"\*\*(.*?)\*\*").unwrap();

	// Replace '**text**' with '<span class="font-bold">text</span>'
	let result = bold_re.replace_all(&input, r#"<span class="font-bold">$1</span>"#);

	result.into_owned()
}

fn replace_linebreaks_with_br(input: String) -> String {
	input.replace("\n", "<br>")
}

fn replace_markdown_headers_with_html(text: &str) -> String {
	let mut result = String::new();

	for line in text.lines() {
		let trimmed_line = line.trim_start();
		let mut header_level = 0;

		// Count the number of consecutive '#' characters at the beginning of the line
		for c in trimmed_line.chars() {
			if c == '#' {
				header_level += 1;
			} else {
				break;
			}
		}

		// Replace Markdown headers with HTML headers if '#' characters are found
		let header_content: &str = &trimmed_line[header_level..].trim_start();
		let class: &str = match header_level {
			1 => "text-4xl font-bold",
			2 => "text-3xl font-bold",
			3 => "text-2xl font-bold",
			4 => "text-xl font-bold",
			5 => "text-lg font-bold",
			6 => "text-base font-bold",
			_ => "",
		};
		let new_line = format!("<span class=\"{}\">{}</span>\n", class, header_content);
		result.push_str(&new_line);
	}

	result
}

pub fn escape_html_tags(input: String) -> String {
	let re = Regex::new(r"<([^<>]+)>").unwrap();
	let escaped = re.replace_all(&input, "&lt;$1&gt;");
	escaped.to_string()
}

#[allow(dead_code)]
pub fn truncate_string(s: &str, max_chars: usize) -> &str {
	match s.char_indices().nth(max_chars) {
		Some((idx, _)) => &s[0..idx],
		None => s,
	}
}
