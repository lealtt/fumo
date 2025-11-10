use std::vec;

use crate::{
    Data, Error,
    constants::{colors, icon, links},
    functions::format::discord::{inline_code, mention},
};
use ::serenity::all::{CreateActionRow, CreateButton};
use poise::{
    self, BoxFuture,
    serenity_prelude::{self as serenity, Mentionable},
};
use tokio::time::{Duration, sleep};

pub fn event_handler<'a>(
    framework: poise::FrameworkContext<'a, Data, Error>,
    event: &'a serenity::FullEvent,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move { handle_ping(framework, event).await })
}

async fn handle_ping(
    framework: poise::FrameworkContext<'_, Data, Error>,
    event: &serenity::FullEvent,
) -> Result<(), Error> {
    let serenity::FullEvent::Message { new_message } = event else {
        return Ok(());
    };

    if new_message.author.bot || new_message.guild_id.is_none() {
        return Ok(());
    }

    let bot_id = framework.bot_id();
    let trimmed = new_message.content.trim();
    let mention_variants = [
        mention(bot_id.get() as i64),
        format!("<@!{}>", bot_id.get()),
    ];

    if !mention_variants.iter().any(|mention| mention == trimmed) {
        return Ok(());
    }

    let literal_prefixes = collect_literal_prefixes(&framework.options.prefix_options);
    let prefix_sentence = format_prefix_sentence(&literal_prefixes);
    let help_hint = format_help_hint(&literal_prefixes);

    let version = env!("CARGO_PKG_VERSION");
    let rust_version = option_env!("FUMO_RUSTC_VERSION").unwrap_or("unknown");
    let serenity_version = option_env!("FUMO_SERENITY_VERSION").unwrap_or("unknown");
    let poise_version = option_env!("FUMO_POISE_VERSION").unwrap_or("unknown");
    let shard_manager = framework.shard_manager();
    let shard_count = shard_manager.runners.lock().await.len();
    drop(shard_manager);
    let cache = &framework.serenity_context.cache;
    let guild_count = cache.guild_count();
    let user_count = cache.user_count();
    let avatar_url = {
        let current = cache.current_user();
        current
            .avatar_url()
            .unwrap_or_else(|| current.default_avatar_url())
    };

    let version_field = format!(
        "{} {}\n{} {}\n{} {}\n{} {}",
        inline_code("app"),
        inline_code(version),
        inline_code("rust"),
        inline_code(rust_version),
        inline_code("serenity"),
        inline_code(serenity_version),
        inline_code("poise"),
        inline_code(poise_version),
    );

    let stats_field = format!(
        "{} guilda(s)\n{} usuário(s)\n{} shard(s)",
        inline_code(guild_count.to_string()),
        inline_code(user_count.to_string()),
        inline_code(shard_count.to_string()),
    );

    let description = format!(
        "{}\n{}\n{}",
        format!("Oi {}, {}", new_message.author.mention(), prefix_sentence),
        help_hint,
        "Aqui estão alguns detalhes rápidos:"
    );

    let embed = serenity::CreateEmbed::new()
        .description(description)
        .thumbnail(avatar_url)
        .color(colors::MOON)
        .field("Versões", version_field, false)
        .field("Estatísticas", stats_field, false)
        .footer(serenity::CreateEmbedFooter::new(
            "Resposta automática • removo em 30s",
        ));

    let row = vec![CreateActionRow::Buttons(vec![
        CreateButton::new_link(links::GITHUB_REPO)
            .label("Github")
            .emoji(icon::GITHUB_LOGO.as_reaction()),
    ])];

    let response = new_message
        .channel_id
        .send_message(
            framework.serenity_context,
            serenity::CreateMessage::new().embed(embed).components(row),
        )
        .await?;

    let http = framework.serenity_context.http.clone();
    let channel_id = response.channel_id;
    let message_id = response.id;
    tokio::spawn(async move {
        sleep(Duration::from_secs(30)).await;
        let _ = channel_id.delete_message(&http, message_id).await;
    });

    Ok(())
}

fn collect_literal_prefixes(options: &poise::PrefixFrameworkOptions<Data, Error>) -> Vec<String> {
    let mut prefixes = Vec::new();

    if let Some(prefix) = options.prefix.as_deref() {
        prefixes.push(prefix.to_string());
    }

    for additional in &options.additional_prefixes {
        if let poise::Prefix::Literal(prefix) = additional {
            prefixes.push((*prefix).to_string());
        }
    }

    prefixes
}

fn format_prefix_sentence(prefixes: &[String]) -> String {
    match prefixes.len() {
        0 => "ainda não tenho prefixos configurados.".to_string(),
        1 => format!("meu prefixo atual é {}.", inline_code(&prefixes[0])),
        _ => {
            let formatted = format_prefix_list(prefixes);
            format!("meus prefixos atuais são {}.", formatted)
        }
    }
}

fn format_prefix_list(prefixes: &[String]) -> String {
    let quoted: Vec<String> = prefixes.iter().map(|prefix| inline_code(prefix)).collect();
    match quoted.len() {
        0 => String::new(),
        1 => quoted[0].clone(),
        2 => format!("{} e {}", quoted[0], quoted[1]),
        _ => {
            let (head, last) = quoted.split_at(quoted.len() - 1);
            format!("{}, e {}", head.join(", "), &last[0])
        }
    }
}

fn format_help_hint(prefixes: &[String]) -> String {
    if let Some(prefix) = prefixes.first() {
        format!(
            "Use {} ou {} para ver os comandos.",
            inline_code(format!("{prefix}help")),
            inline_code("/help")
        )
    } else {
        format!("Use {} para ver os comandos.", inline_code("/help"))
    }
}
