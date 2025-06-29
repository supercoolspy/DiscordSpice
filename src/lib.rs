pub mod discord;
pub mod minecraft;

use crate::discord::{DiscordState, SendMessageInfo};
use crate::minecraft::Config;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use poise::serenity_prelude::ChannelId;
use pumpkin::plugin::Context;
use pumpkin::plugin::EventPriority;
use pumpkin::plugin::player::player_chat::PlayerChatEvent;
use pumpkin::{plugin::EventHandler, server::Server};
use pumpkin_api_macros::with_runtime;
use pumpkin_api_macros::{plugin_impl, plugin_method};
use std::sync::Arc;
use tokio::sync::OnceCell;

type ClonableArcSwap<T> = Arc<ArcSwap<T>>;

struct PlayerChatHandler {
    discord_state: ClonableArcSwap<DiscordState>,
}

#[with_runtime(global)]
#[async_trait]
impl EventHandler<PlayerChatEvent> for PlayerChatHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerChatEvent) {
        if event.cancelled {
            return;
        }

        let discord_state = &self.discord_state.load_full();
        let config = &discord_state.config.load().discord.chat_channels;

        let global_chat = config.get("global");

        if global_chat.is_none() {
            log::warn!("Global chat channel is none");
            return;
        }

        let global_chat = *global_chat.unwrap();

        if global_chat == 0 {
            log::warn!("Global chat channel is default");
            return;
        }

        let global_channel = ChannelId::from(global_chat);

        let res = discord_state.sender.send(SendMessageInfo::new(
            event.player.clone(),
            global_channel,
            event.message.to_string(),
        ));

        if let Some(err) = res.err() {
            log::error!("Failed to send message to discord: {}", err);
            return;
        }
    }
}

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    pumpkin::init_log!();

    log::debug!("Getting config");

    let config: &Arc<ArcSwap<Config>> = self
        .config
        .get_or_init(async move || {
            let config = minecraft::init_config(server.get_data_folder().join("config.toml"))
                .await
                .map_err(|e| e.to_string())
                .unwrap();

            let config_swap = Arc::new(ArcSwap::new(Arc::new(config)));

            config_swap
        })
        .await;

    log::info!("Loaded config");

    let discord_state = discord::DiscordState::init(server, config.clone())
        .await
        .map_err(|e| e.to_string())
        .unwrap();

    log::debug!("Registering events");
    server
        // Highest since we want to not send messages if they are cancelled
        .register_event(
            Arc::new(PlayerChatHandler {
                discord_state: discord_state.clone(),
            }),
            EventPriority::Highest,
            true,
        )
        .await;

    log::info!("Registered Events");

    Ok(())
}

#[plugin_impl]
pub struct DiscordSpice {
    config: OnceCell<ClonableArcSwap<Config>>,
}

impl DiscordSpice {
    pub fn new() -> Self {
        DiscordSpice {
            config: OnceCell::new(),
        }
    }
}

impl Default for DiscordSpice {
    fn default() -> Self {
        Self::new()
    }
}
