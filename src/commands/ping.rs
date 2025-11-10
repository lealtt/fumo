use crate::{Context, Error, constants::icon, functions::format::pretty_message};
use std::time::Instant;

/// Responde com pong 🏓!
#[poise::command(
    slash_command,
    prefix_command,
    aliases("pong", "latencia", "p"),
    interaction_context = "Guild",
    category = "Utilidades",
    on_error = "crate::commands::util::command_error_handler"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();

    let msg = ctx.say(pretty_message("🏓", "Pong!")).await?;
    let elapsed = start.elapsed();

    let manager = ctx.data().shard_manager.clone();
    let runners = manager.runners.lock().await;
    let shard_id = ctx.serenity_context().shard_id;
    let latency = runners
        .get(&shard_id)
        .and_then(|runner| runner.latency)
        .unwrap_or_default();

    let content = format!(
        "{}\n{}\n{}",
        pretty_message(icon::CHECK, "Pong!"),
        pretty_message(
            icon::RSS,
            format!("Latência WebSocket: {} ms", latency.as_millis())
        ),
        pretty_message(
            icon::RSS,
            format!("Latência API: {} ms", elapsed.as_millis())
        ),
    );

    msg.edit(ctx, poise::CreateReply::default().content(content))
        .await?;

    Ok(())
}
