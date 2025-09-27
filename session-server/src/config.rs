use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub heartbeat_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: i64,
    pub max_failed_attempts: u32,
    pub lockout_duration_mins: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub max_sessions: usize,
    pub session_timeout_mins: u64,
    pub cleanup_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 1000,
                heartbeat_interval_secs: 30,
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key-change-this-in-production".to_string(),
                token_expiry_hours: 24,
                max_failed_attempts: 3,
                lockout_duration_mins: 15,
            },
            session: SessionConfig {
                max_sessions: 100,
                session_timeout_mins: 60,
                cleanup_interval_secs: 300, // 5 minutes
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn create_default_config(path: &str) -> Result<()> {
        let config = Config::default();
        config.save(path)
    }
}