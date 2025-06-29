use crate::ClonableArcSwap;
use crate::minecraft::Config;
use arc_swap::ArcSwap;
use poise::serenity_prelude as serenity;
use pumpkin::{plugin::Context, server::Server};
use std::sync::{Arc, OnceLock};
use poise::serenity_prelude::Http;
use thiserror::Error;

pub(crate) struct DiscordState {
    server: Arc<Server>,
    config: Arc<ArcSwap<Config>>,
    http: OnceLock<Arc<Http>>
}

#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("Serenity Error")]
    Serenity(#[from] serenity::Error),
    
    #[error("Discord Token Not Set")]
    DefaultToken
}

impl DiscordState {
    pub(super) async fn init(
        mc_ctx: &Context,
        config: Arc<ArcSwap<Config>>,
    ) -> Result<ClonableArcSwap<DiscordState>, DiscordError> {
        log::debug!("Starting discord bot");
        
        let token = &config.load_full().tokens.discord;
        
        if token.trim().is_empty() || token == "BOT_TOKEN" {
            return Err(DiscordError::DefaultToken)
        }
        
        let intents = serenity::GatewayIntents::all();

        let server = mc_ctx.server.clone();

        let state = Arc::new(ArcSwap::new(Arc::new(DiscordState { server, config, http: OnceLock::new() })));

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

        state.load_full().http.get_or_init(move || {
            http
        });

        client.start().await?;
        
        log::info!("Started discord bot");

        Ok(state)
    }
}
