use crate::settings::{Settings, VersionedSettings};
use crate::{background, throw};
use atomicwrites::{AtomicFile, OverwriteBehavior};

use serde::Serialize;
use specta::Type;
use sqlx::SqlitePool;

use std::convert::TryInto;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Config, State};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppPaths {
	pub app_dir: PathBuf,
	pub settings_file: PathBuf,
	pub db: String,
}
impl AppPaths {
	pub fn from_tauri_config(config: &Config) -> Self {
		let app_dir = match env::var("DEVELOPMENT").is_ok() {
			true => env::current_dir().unwrap().join("appdata"),
			false => tauri::api::path::app_data_dir(config).unwrap(),
		};
		AppPaths {
			app_dir: app_dir.clone(),
			settings_file: app_dir.join("Settings.json"),
			db: app_dir.join("genhub.sqlite").to_string_lossy().to_string(),
		}
	}
}

pub struct Data {
	pub bg_handle: Option<background::BgHandle>,
	pub db_pool: SqlitePool,
	pub versioned_settings: VersionedSettings,
	pub paths: AppPaths,
	pub window: tauri::Window,
	pub user_history: UndoHistory,
}
impl Data {
	pub fn settings_ref(&self) -> &Settings {
		self.versioned_settings.unwrap_ref()
	}
}

pub fn ensure_parent_exists(file_path: &Path) -> Result<(), String> {
	if let Some(parent) = file_path.parent() {
		if let Err(e) = std::fs::create_dir_all(parent) {
			throw!("Error creating parent folder: {}", e.to_string());
		}
	}
	Ok(())
}

pub fn write_atomically(file_path: &PathBuf, buf: &[u8]) -> Result<(), String> {
	ensure_parent_exists(file_path)?;
	let af = AtomicFile::new(file_path, OverwriteBehavior::AllowOverwrite);
	match af.write(|f| f.write_all(buf)) {
		Ok(_) => Ok(()),
		Err(e) => Err(e.to_string()),
	}
}

pub type DataState<'a> = State<'a, ArcData>;
pub struct ArcData(pub Arc<Mutex<Data>>);
impl ArcData {
	pub fn new(data: Data) -> Self {
		Self(Arc::new(Mutex::new(data)))
	}
}

#[derive(Serialize, Clone, Type)]
pub struct UndoHistory {
	pub entries: Vec<(u32, Action)>,
}

impl UndoHistory {
	pub fn new() -> Self {
		Self { entries: vec![] }
	}
	pub fn push(&mut self, action: Action) {
		let time: u32 = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.try_into()
			.unwrap();
		self.entries.push((time, action));
		if self.entries.len() > 100 {
			self.entries.remove(0);
		}
	}
}

#[derive(Serialize, Clone, Type)]
pub enum Action {
	Archive(String),
	Unarchive(String),
}
