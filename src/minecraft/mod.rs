use pumpkin::plugin::Context;
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
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
    pub(crate) chat_channels: HashMap<String, u64>,
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

pub(crate) async fn send_message(
    server: Arc<Server>,
    name: String,
    role_name: String,
    message: String,
) -> Result<(), MinecraftError> {
    server
        .broadcast_message(
            &TextComponent::chat_decorated(
                format!("[{} {{DISPLAY_NAME}} {{MESSAGE}}", role_name),
                name,
                message,
            ),
            &TextComponent::text(""),
            0,
            None,
        )
        .await;

    Ok(())
}
