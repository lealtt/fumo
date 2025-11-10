use crate::{
    Context, Error,
    constants::{colors, icon},
    database::{self, BlacklistEntryModel},
    functions::{
        format::{discord::mention, pretty_message},
        time,
    },
};
use poise::serenity_prelude as serenity;

/// Global command check to prevent blacklisted users from using the bot
pub async fn enforce_global_blacklist(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.author().bot {
        return Ok(true);
    }

    let discord_id = ctx.author().id.get() as i64;
    let db = ctx.data().database.clone();
    let entry = database::find_blacklist_entry(&db, discord_id).await?;

    if let Some(entry) = entry {
        send_blacklist_notice(ctx, &entry).await?;
        return Ok(false);
    }

    Ok(true)
}

async fn send_blacklist_notice(ctx: Context<'_>, entry: &BlacklistEntryModel) -> Result<(), Error> {
    let mut description = vec![pretty_message(
        icon::ERROR,
        "Você está na blacklist e não pode usar meus comandos, seu bocó.",
    )];

    if let Some(reason) = entry.reason.as_deref() {
        description.push(pretty_message(icon::HASTAG, format!("Motivo: {reason}")));
    }

    description.push(pretty_message(
        icon::HAMMER,
        format!("Responsável: {}", mention(entry.moderator_id)),
    ));
    let registered_at = time::describe_relative_from_str(&entry.created_at)
        .unwrap_or_else(|| entry.created_at.clone());
    description.push(pretty_message(
        icon::TIMER,
        format!("Registrado {registered_at}"),
    ));

    description.push(String::new());
    description.push(pretty_message(
        icon::PLUS,
        "Procure a minha equipe caso acredite que isso é um erro.",
    ));

    let embed = serenity::CreateEmbed::new()
        .title(format!("{} Acesso bloqueado", icon::ERROR))
        .description(description.join("\n"))
        .colour(colors::MOON);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
