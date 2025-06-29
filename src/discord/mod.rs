use crate::ClonableArcSwap;
use crate::minecraft::Config;
use arc_swap::ArcSwap;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, CreateMessage, Http};
use pumpkin::entity::player::Player;
use pumpkin::{plugin::Context, server::Server};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use strfmt::strfmt;
use thiserror::Error;

/// State for the Discord Bot
pub(crate) struct DiscordState {
    server: Arc<Server>,
    config: Arc<ArcSwap<Config>>,
    http: OnceLock<Arc<Http>>,
}

/// Info for messages being sent to the discord as if they were sent by a player
pub struct SendMessageInfo {
    player: Arc<Player>,
    channel: ChannelId,
    message: String,
}

#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("Serenity Error {0}")]
    Serenity(#[from] serenity::Error),

    #[error("Format Error {0}")]
    Format(#[from] strfmt::FmtError),

    #[error("Discord Token Not Set")]
    DefaultToken,
}

impl DiscordState {
    pub(super) async fn init(
        mc_ctx: &Context,
        config: Arc<ArcSwap<Config>>,
    ) -> Result<ClonableArcSwap<DiscordState>, DiscordError> {
        log::debug!("Starting discord bot");

        let token = &config.load_full().discord.token;

        if token.trim().is_empty() || token == "BOT_TOKEN" {
            return Err(DiscordError::DefaultToken);
        }

        let intents = serenity::GatewayIntents::all();

        let server = mc_ctx.server.clone();

        let state = Arc::new(ArcSwap::new(Arc::new(DiscordState {
            server,
            config,
            http: OnceLock::new(),
        })));

        let dc_state = state.clone();

        let framework = poise::Framework::<ClonableArcSwap<DiscordState>, DiscordError>::builder()
            .options(poise::FrameworkOptions {
                ..Default::default()
            })
            .setup(move |ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    Ok(dc_state)
                })
            })
            .build();

        let mut client = serenity::ClientBuilder::new(token, intents)
            .framework(framework)
            .await?;

        let http = client.http.clone();

        state.load_full().http.get_or_init(move || http);

        client.start().await?;

        log::info!("Started discord bot");

        Ok(state)
    }

    /// Send a message that a player sent to the discord
    pub async fn send_message(
        &self,
        player: Arc<Player>,
        channel: ChannelId,
        message: String,
    ) -> Result<(), DiscordError> {
        // Get needed values
        let http = self.http.get().expect("HTTP Should be initialized");

        let discord_config = &self.config.load().discord;

        let profile = player.gameprofile.clone();

        let mut vars = HashMap::new();
        
        // Add vars to replace
        vars.insert("uuid", profile.id.to_string());
        vars.insert("name", profile.name);
        vars.insert("message", message);

        let message_content = strfmt::strfmt(&discord_config.chat_format, &vars)?;

        drop(vars);

        channel
            .send_message(http, CreateMessage::new().content(message_content))
            .await?;

        Ok(())
    }
}
