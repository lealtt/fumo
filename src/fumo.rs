use crate::commands;
use poise::serenity_prelude as serenity;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub struct Data {
    pub shard_manager: Arc<serenity::ShardManager>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<serenity::ShardManager>;
}

/// Builds the Poise framework with all commands and the provided prefix options.
pub fn build_framework(
    prefix_options: poise::PrefixFrameworkOptions<Data, Error>,
) -> poise::Framework<Data, Error> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::load_all(),
            prefix_options,
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let shard_manager = {
                    let data = ctx.data.read().await;
                    data.get::<ShardManagerContainer>()
                        .cloned()
                        .expect("Shard manager missing from TypeMap")
                };

                println!("{} is connected and ready", ready.user.display_name());
                Ok(Data { shard_manager })
            })
        })
        .build()
}

pub async fn run_client(
    token: String,
    intents: serenity::GatewayIntents,
    framework: poise::Framework<Data, Error>,
) -> Result<(), Error> {
    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    client.start_autosharded().await?;

    Ok(())
}
