use pumpkin::plugin::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Error, Debug)]
pub enum MinecraftError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("TOML Deserialize Error: {0}")]
    TOMLDe(#[from] toml::de::Error),
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub(crate) discord: DiscordConfig,
    pub(crate) lib_sql: LibSQL,
}

#[derive(Serialize, Deserialize)]
pub struct DiscordConfig {
    pub(crate) token: String,
    pub(crate) chat_format: String,
}

#[derive(Serialize, Deserialize)]
pub struct LibSQL {
    pub(crate) remote: bool,
    pub(crate) token: String,
}

pub(super) async fn init_config(config_file_path: PathBuf) -> Result<Config, MinecraftError> {
    if !config_file_path.exists() {
        let mut file = File::create(&config_file_path).await?;

        file.write_all(include_bytes!("../../assets/default_config.toml"))
            .await?;
    }

    get_config(config_file_path).await
}

pub(super) async fn get_config(config_path: PathBuf) -> Result<Config, MinecraftError> {
    let config: Config = toml::from_str(&tokio::fs::read_to_string(config_path).await?)?;

    Ok(config)
}
