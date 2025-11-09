use crate::commands;
use poise::serenity_prelude as serenity;
use serenity::prelude::TypeMapKey;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn gateway_intents() -> serenity::GatewayIntents {
    serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
}

pub fn prefix_options() -> poise::PrefixFrameworkOptions<Data, Error> {
    poise::PrefixFrameworkOptions {
        prefix: Some("?".into()),
        additional_prefixes: vec![
            poise::Prefix::Literal("-"),
            poise::Prefix::Literal("f!"),
            poise::Prefix::Literal("."),
        ],
        ..Default::default()
    }
}

pub struct Data {
    pub shard_manager: Arc<serenity::ShardManager>,
    pub database: Arc<Mutex<SqlitePool>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<serenity::ShardManager>;
}

/// Builds the Poise framework with all commands and the provided prefix options
pub fn build_framework(
    prefix_options: poise::PrefixFrameworkOptions<Data, Error>,
    database: SqlitePool,
) -> poise::Framework<Data, Error> {
    poise::Framework::builder()
        .options(framework_options(prefix_options))
        .setup(move |ctx, ready, framework| {
            let database = database.clone();
            Box::pin(async move { setup_framework(ctx, ready, framework, database).await })
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

fn framework_options(
    prefix_options: poise::PrefixFrameworkOptions<Data, Error>,
) -> poise::FrameworkOptions<Data, Error> {
    poise::FrameworkOptions {
        commands: commands::load_all(),
        prefix_options,
        ..Default::default()
    }
}

async fn setup_framework(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<Data, Error>,
    database: SqlitePool,
) -> Result<Data, Error> {
    register_commands(ctx, framework).await?;
    let shard_manager = extract_shard_manager(ctx).await;
    // TODO: re-enable automatic avatar rotation on startup when the feature is stable
    // functions::bot::avatar::spawn_avatar_rotation_task(ctx.http.clone());
    println!("{} is connected and ready", ready.user.display_name());

    Ok(Data {
        shard_manager,
        database: Arc::new(Mutex::new(database)),
    })
}

async fn register_commands(
    ctx: &serenity::Context,
    framework: &poise::Framework<Data, Error>,
) -> Result<(), Error> {
    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    Ok(())
}

async fn extract_shard_manager(ctx: &serenity::Context) -> Arc<serenity::ShardManager> {
    let data = ctx.data.read().await;
    data.get::<ShardManagerContainer>()
        .cloned()
        .expect("Shard manager missing from TypeMap")
}
