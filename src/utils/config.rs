use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub use_gpio: bool,
    pub log_level: String,
    pub log_dir: String,
    pub log_file_name: String,
    pub log_rotate_size: usize,
    pub log_rotate_count: usize,
}

pub const DEFAULT_LOG_ROTATE_SIZE: usize = 5_000_000;
pub const DEFAULT_LOG_ROTATE_COUNT: usize = 5;

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            log_level: env::var("log_level").unwrap_or_else(|_| "info".to_string()),
            log_dir: env::var("log_dir").unwrap_or_else(|_| "logs".to_string()),
            log_file_name: env::var("log_file_name").unwrap_or_else(|_| "baymax".to_string()),
            log_rotate_size: env::var("log_rotate_size")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_LOG_ROTATE_SIZE),
            log_rotate_count: env::var("log_rotate_count")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_LOG_ROTATE_COUNT),
            use_gpio: std::env::var("use_gpio").unwrap_or("false".to_string()) == "true"
        }
    }
}

/// ✅ ดึงชื่อ Agent จาก .env (หรือใช้ "baymax" เป็น default)
pub fn get_agent_name() -> String {
    dotenvy::dotenv().ok();
    env::var("agent_name").unwrap_or_else(|_| "baymax".to_string())
}