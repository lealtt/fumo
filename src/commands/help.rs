use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateAutocompleteResponse;
use std::collections::HashSet;

/// Unified help command showing either an overview.
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    interaction_context = "Guild",
    category = "General"
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to describe"]
    #[autocomplete = "help_autocomplete"]
    mut command: Option<String>,
) -> Result<(), Error> {
    if ctx.invoked_command_name() != "help" {
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
            let description = command
                .description
                .as_deref()
                .unwrap_or("No description provided");
            let category = command.category.as_deref().unwrap_or("Uncategorized");
            format!(
                "`{}` — {} _(Category: {})_",
                command.name, description, category
            )
        })
        .collect();

    let content = if lines.is_empty() {
        "No commands available yet.".to_string()
    } else {
        format!("**Available commands**\n{}", lines.join("\n"))
    };

    ctx.send(
        poise::CreateReply::default()
            .content(content)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

async fn send_command_help(ctx: Context<'_>, name: &str) -> Result<(), Error> {
    if let Some(command) = find_command(&ctx, name) {
        let mut description = command
            .help_text
            .as_deref()
            .unwrap_or_else(|| {
                command
                    .description
                    .as_deref()
                    .unwrap_or("No description provided")
            })
            .to_string();
        let category = command.category.as_deref().unwrap_or("Uncategorized");

        if !command.parameters.is_empty() {
            description.push_str("\n\n**Parameters**\n");
            for param in &command.parameters {
                let param_desc = param
                    .description
                    .as_deref()
                    .unwrap_or("No description provided");
                description.push_str(&format!("`{}` — {}\n", param.name, param_desc));
            }
        }

        let content = format!(
            "**{}**\n_Category: {}_\n{}",
            command.name, category, description
        );
        ctx.send(
            poise::CreateReply::default()
                .content(content)
                .ephemeral(true),
        )
        .await?;
    } else {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("Couldn't find a command named `{name}`"))
                .ephemeral(true),
        )
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
        response.add_string_choice("No matching commands", "")
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
        .flat_map(|cmd| {
            let mut names = vec![cmd.name.to_string()];
            names.extend(cmd.aliases.iter().map(|alias| alias.to_string()));
            names
        })
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
