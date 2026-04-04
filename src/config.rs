use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub cleanup: CleanupConfig,
    pub webhook: WebhookConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub admin_api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanupConfig {
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
    #[serde(default = "default_auto_delete_days")]
    pub auto_delete_expired_days: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookConfig {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_check_interval() -> u64 { 1 }
fn default_auto_delete_days() -> u64 { 30 }
fn default_timeout() -> u64 { 10 }
fn default_max_retries() -> u32 { 3 }

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
