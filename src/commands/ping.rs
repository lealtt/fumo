use crate::{Context, Error};
use std::time::Instant;

/// Reply with pong 🏓!
#[poise::command(
    slash_command,
    prefix_command,
    interaction_context = "Guild",
    category = "Utility"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();

    let msg = ctx.say("🏓 Pinging...").await?;
    let elapsed = start.elapsed();

    let manager = ctx.data().shard_manager.clone();
    let runners = manager.runners.lock().await;
    let shard_id = ctx.serenity_context().shard_id;
    let latency = runners
        .get(&shard_id)
        .and_then(|runner| runner.latency)
        .unwrap_or_default();

    msg.edit(
        ctx,
        poise::CreateReply::default().content(format!(
            "🏓 Pong!\nWebSocket latency: {} ms\nAPI latency: {} ms",
            latency.as_millis(),
            elapsed.as_millis()
        )),
    )
    .await?;

    Ok(())
}
