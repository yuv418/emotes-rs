use serde::Deserialize;

#[derive(Deserialize)]
pub struct EmotesConfig {
    pub db_url: String,
    pub db_max_connections: u32,
    pub data_dir: String,
    #[serde(default = "default_bind")]
    pub http_bind: String,
}

fn default_bind() -> String {
    "127.0.0.1:8080".to_owned()
}
