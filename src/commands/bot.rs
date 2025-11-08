use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::pretty_message::pretty_message,
};
use poise::serenity_prelude as serenity;

/// Comandos utilitários relacionados a mim.
#[poise::command(
    slash_command,
    prefix_command,
    interaction_context = "Guild",
    category = "Utilidades",
    subcommands("info")
)]
pub async fn bot(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Exibe estatísticas sobre mim.
#[poise::command(slash_command, prefix_command, category = "Utilidades")]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let cache = ctx.serenity_context().cache.clone();

    let (bot_tag, bot_id) = {
        let current = cache.current_user();
        let display = current.display_name().to_string();
        let username = current.name.clone();
        let tag = if display == username {
            display
        } else {
            format!("{display} (@{username})")
        };
        (tag, current.id)
    };

    let guild_count = cache.guild_count();
    let user_count = cache.user_count();
    let channel_count = cache.guild_channel_count();
    let unknown_members = cache.unknown_members();
    let shard_total = cache.shard_count();

    let data = ctx.data();
    let shard_manager = data.shard_manager.clone();
    let runners = shard_manager.runners.lock().await;
    let connected_shards = runners.len();
    let mut latencies = Vec::new();
    for runner in runners.values() {
        if let Some(latency) = runner.latency {
            latencies.push(latency.as_millis());
        }
    }
    let avg_latency_ms = if latencies.is_empty() {
        None
    } else {
        Some(latencies.iter().copied().sum::<u128>() / (latencies.len() as u128))
    };
    let max_latency_ms = latencies.into_iter().max();
    drop(runners);

    let rust_version = option_env!("FUMO_RUSTC_VERSION").unwrap_or("unknown");
    let binary_version = env!("CARGO_PKG_VERSION");

    let runtime_field = format!(
        "{} Rust `{rust_version}`\n{} Binário v{binary_version}",
        icon::CHECK,
        icon::PLUS
    );

    let cache_field = format!(
        "{} Servidores **{guild_count}**\n{} Usuários **{user_count}**\n{} Canais **{channel_count}**",
        icon::GEAR,
        icon::GEAR,
        icon::GEAR
    );

    let mut latency_lines = Vec::new();
    if let Some(avg) = avg_latency_ms {
        latency_lines.push(format!("{} Média: **{avg}ms**", icon::TIMER));
    }
    if let Some(max) = max_latency_ms {
        latency_lines.push(format!("{} Pico: **{max}ms**", icon::ALARM));
    }
    let latency_field = if latency_lines.is_empty() {
        format!("{} Sem dados de latência", icon::MINUS)
    } else {
        latency_lines.join("\n")
    };

    let avatar_url = {
        let current = cache.current_user();
        current
            .avatar_url()
            .unwrap_or_else(|| current.default_avatar_url())
    };

    let description = vec![
        pretty_message(icon::BELL, format!("ID `{bot_id}`")),
        pretty_message(
            icon::GEAR,
            format!("Shards ativos {connected_shards}/{shard_total}"),
        ),
        pretty_message(
            icon::PLUS,
            format!("Membros desconhecidos: {unknown_members}"),
        ),
    ]
    .join("\n");

    let embed = serenity::CreateEmbed::new()
        .title(format!("{bot_tag} • Status"))
        .thumbnail(avatar_url)
        .description(description)
        .colour(colors::MINT)
        .field(format!("{} Runtime", icon::CHECK), runtime_field, true)
        .field(format!("{} Cache", icon::GEAR), cache_field, true)
        .field(format!("{} Latência", icon::TIMER), latency_field, true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
