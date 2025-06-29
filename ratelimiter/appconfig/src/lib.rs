use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub ratelimiter: RatelimiterConfig,
}

impl Config {
    pub fn parse() -> Result<Config, config::ConfigError> {
        let config: Config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?
            .try_deserialize()?;
        Ok(config)
    }
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}
impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}

#[derive(Deserialize)]
pub struct ClientConfig {
    pub requests_per_sec: u32,
}
impl ClientConfig {
    pub fn frequency(&self) -> Duration {
        Duration::from_secs_f64(1.0 / (self.requests_per_sec as f64))
    }
}

#[derive(Deserialize)]
pub struct RatelimiterConfig {
    pub port: u16,
    pub token_bucket: TokenBucketConfig,
}
impl RatelimiterConfig {
    pub fn addr(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}

#[derive(Deserialize)]
pub struct TokenBucketConfig {
    pub capacity: u32,
    pub rate: u64,
}
