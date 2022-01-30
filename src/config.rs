use anyhow::{Context, Result};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;

lazy_static! {
    pub static ref EMOTES_CONFIG: EmotesConfig = serde_json::from_reader(
        File::open(
            &dotenv::var("EMOTES_CONFIG_FILE")
                .with_context(|| "Failed to read emotes config file env-var")
                .unwrap()
        )
        .with_context(|| "Failed to open specified emotes config file")
        .unwrap()
    )
    .with_context(|| "Failed to parse specified emotes config file")
    .unwrap();
}

#[derive(Deserialize)]
pub struct EmotesConfig {
    pub db_url: String,
    pub db_max_connections: u32,
    pub data_dir: PathBuf,
    #[serde(default = "default_bind")]
    pub http_bind: String,
}

fn default_bind() -> String {
    "127.0.0.1:8080".to_owned()
}
