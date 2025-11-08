use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::pretty_message::pretty_message,
};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateAutocompleteResponse;
use std::collections::HashSet;

/// Veja informações sobre meus comandos.
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    aliases("ajuda", "h"),
    interaction_context = "Guild",
    category = "Geral"
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Comando específico para descrever"]
    #[autocomplete = "help_autocomplete"]
    mut command: Option<String>,
) -> Result<(), Error> {
    if ctx.invoked_command_name() != "ajuda" {
        command = match command {
            Some(rest) => Some(format!("{} {}", ctx.invoked_command_name(), rest)),
            None => Some(ctx.invoked_command_name().to_string()),
        };
    }

    match command {
        Some(name) => send_command_help(ctx, &name).await?,
        None => send_overview(ctx).await?,
    }

    Ok(())
}

async fn send_overview(ctx: Context<'_>) -> Result<(), Error> {
    let lines: Vec<String> = ctx
        .framework()
        .options()
        .commands
        .iter()
        .filter(|cmd| !cmd.hide_in_help)
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
    if let Some(command) = find_command(&ctx, name) {
        let description = command
            .help_text
            .as_deref()
            .unwrap_or_else(|| command.description.as_deref().unwrap_or("Sem descrição"))
            .to_string();
        let category = command.category.as_deref().unwrap_or("Sem categoria");

        let mut embed = serenity::CreateEmbed::new()
            .title(format!("{} /{}", icon::BELL, command.name))
            .description(description)
            .colour(colors::MOON)
            .field(
                format!("{} Categoria", icon::GEAR),
                format!("`{}`", category),
                true,
            );

        if !command.aliases.is_empty() {
            let aliases = command
                .aliases
                .iter()
                .map(|alias| format!("`{alias}`"))
                .collect::<Vec<_>>()
                .join(", ");
            embed = embed.field(format!("{} Alias", icon::HASTAG), aliases, true);
        }

        if !command.parameters.is_empty() {
            let params = command
                .parameters
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
    let lowercase = partial.to_ascii_lowercase();
    let mut seen = HashSet::new();
    let mut matches = Vec::new();

    for name in available_command_names(&ctx)
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

fn available_command_names(ctx: &Context<'_>) -> Vec<String> {
    ctx.framework()
        .options()
        .commands
        .iter()
        .map(|cmd| cmd.name.to_string())
        .collect()
}

fn find_command<'a>(
    ctx: &'a Context<'_>,
    name: &str,
) -> Option<&'a poise::Command<crate::Data, crate::Error>> {
    let needle = name.to_ascii_lowercase();
    ctx.framework().options().commands.iter().find(|cmd| {
        cmd.name.to_ascii_lowercase() == needle
            || cmd
                .aliases
                .iter()
                .any(|alias| alias.to_ascii_lowercase() == needle)
    })
}
