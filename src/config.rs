use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DEFAULT_ADMIN_API_KEY: &str = "admin-secret-key-change-me";

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
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    #[serde(default = "default_admin_username")]
    pub admin_username: String,
    #[serde(default = "default_admin_key")]
    pub admin_api_key: String,
    pub admin_password: String,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_exp_hours")]
    pub jwt_exp_hours: u64,
}

fn default_admin_username() -> String {
    std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string())
}

fn default_admin_key() -> String {
    std::env::var("ADMIN_API_KEY").unwrap_or_else(|_| {
        DEFAULT_ADMIN_API_KEY.to_string()
    })
}

fn default_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        "change-this-jwt-secret-in-production".to_string()
    })
}

fn default_jwt_exp_hours() -> u64 { 24 }

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

fn default_host() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 8080 }
fn default_check_interval() -> u64 { 1 }
fn default_auto_delete_days() -> u64 { 210 }
fn default_timeout() -> u64 { 10 }
fn default_max_retries() -> u32 { 3 }

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    // 优先从环境变量读取配置路径
    let config_path = std::env::var("CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| path.to_path_buf());

    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.auth.admin_api_key == DEFAULT_ADMIN_API_KEY {
            anyhow::bail!(
                "admin_api_key 仍在使用默认值，请在配置文件或环境变量中设置安全的 API Key"
            );
        }
        if self.cleanup.check_interval_hours == 0 {
            anyhow::bail!("check_interval_hours 必须大于 0");
        }
        if self.webhook.timeout_seconds == 0 {
            anyhow::bail!("webhook.timeout_seconds 必须大于 0");
        }
        if self.webhook.max_retries == 0 {
            anyhow::bail!("webhook.max_retries 必须大于 0");
        }
        Ok(())
    }
}

use std::path::PathBuf;
