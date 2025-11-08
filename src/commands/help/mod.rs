use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::ui::pretty_message::pretty_message,
};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateAutocompleteResponse;
use std::collections::HashSet;

pub mod command_finder;
use command_finder::CommandFinder;

/// Veja informações sobre meus comandos.
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    aliases("help", "h"),
    interaction_context = "Guild",
    rename = "ajuda",
    category = "Geral"
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Comando específico para descrever"]
    #[autocomplete = "help_autocomplete"]
    command: Option<String>,
) -> Result<(), Error> {
    match command {
        Some(name) => send_command_help(ctx, &name).await?,
        None => send_overview(ctx).await?,
    }

    Ok(())
}

async fn send_overview(ctx: Context<'_>) -> Result<(), Error> {
    let finder = CommandFinder::new(&ctx);
    let lines: Vec<String> = finder
        .get_all_commands()
        .iter()
        .map(|command| {
            let description = command.description.as_deref().unwrap_or("Sem descrição");
            let category = command.category.as_deref().unwrap_or("Sem categoria");
            pretty_message(
                icon::GEAR,
                format!(
                    "`{}` — {} _(Categoria: {})_",
                    command.name, description, category
                ),
            )
        })
        .collect();

    let mut description = vec![pretty_message(
        icon::BELL,
        "Use `/ajuda <comando>` para ver detalhes completos de um comando.",
    )];

    if lines.is_empty() {
        description.push(pretty_message(
            icon::MINUS,
            "Nenhum comando disponível ainda.",
        ));
    } else {
        description.push(String::new());
        description.push("**Comandos disponíveis**".to_string());
        description.extend(lines);
    }

    let embed = serenity::CreateEmbed::new()
        .description(description.join("\n"))
        .colour(colors::MINT);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

async fn send_command_help(ctx: Context<'_>, name: &str) -> Result<(), Error> {
    let finder = CommandFinder::new(&ctx);

    if let Some(info) = finder.find_command(name) {
        let (cmd_name, description, category, parameters, aliases) =
            if let Some(subcmd) = info.subcommand {
                let desc = subcmd
                    .help_text
                    .as_deref()
                    .unwrap_or_else(|| subcmd.description.as_deref().unwrap_or("Sem descrição"))
                    .to_string();
                let cat = subcmd.category.as_deref().unwrap_or("Sem categoria");
                let full_name = format!("{} {}", info.command.name, subcmd.name);
                (
                    full_name,
                    desc,
                    cat.to_string(),
                    &subcmd.parameters,
                    &subcmd.aliases,
                )
            } else {
                let desc = info
                    .command
                    .help_text
                    .as_deref()
                    .unwrap_or_else(|| {
                        info.command
                            .description
                            .as_deref()
                            .unwrap_or("Sem descrição")
                    })
                    .to_string();
                let cat = info.command.category.as_deref().unwrap_or("Sem categoria");
                (
                    info.command.name.to_string(),
                    desc,
                    cat.to_string(),
                    &info.command.parameters,
                    &info.command.aliases,
                )
            };

        let mut embed = serenity::CreateEmbed::new()
            .title(format!("{} /{}", icon::BELL, cmd_name))
            .description(description)
            .colour(colors::MOON)
            .field(
                format!("{} Categoria", icon::GEAR),
                format!("`{}`", category),
                true,
            );

        if !aliases.is_empty() {
            let alias_str = aliases
                .iter()
                .map(|alias| format!("`{alias}`"))
                .collect::<Vec<_>>()
                .join(", ");
            embed = embed.field(format!("{} Alias", icon::HASTAG), alias_str, true);
        }

        if !parameters.is_empty() {
            let params = parameters
                .iter()
                .map(|param| {
                    let param_desc = param.description.as_deref().unwrap_or("Sem descrição");
                    format!("`{}` — {}", param.name, param_desc)
                })
                .collect::<Vec<_>>()
                .join("\n");
            embed = embed.field(format!("{} Parâmetros", icon::PLUS), params, false);
        }

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title(format!("{} Comando não encontrado", icon::ERROR))
            .description(pretty_message(
                icon::ERROR,
                format!("Não foi possível encontrar um comando chamado `{name}`"),
            ))
            .colour(colors::MOON);

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    }

    Ok(())
}

async fn help_autocomplete(ctx: Context<'_>, partial: &str) -> CreateAutocompleteResponse {
    let finder = CommandFinder::new(&ctx);
    let lowercase = partial.to_ascii_lowercase();
    let mut seen = HashSet::new();
    let mut matches = Vec::new();

    for name in finder
        .get_all_command_names()
        .into_iter()
        .filter(|name| name.to_ascii_lowercase().starts_with(&lowercase))
    {
        if seen.insert(name.clone()) {
            matches.push(name);
        }
        if matches.len() >= 25 {
            break;
        }
    }

    let response = CreateAutocompleteResponse::new();
    if matches.is_empty() {
        response.add_string_choice("Nenhum comando encontrado", "")
    } else {
        matches.into_iter().fold(response, |acc, name| {
            acc.add_string_choice(name.clone(), name)
        })
    }
}
