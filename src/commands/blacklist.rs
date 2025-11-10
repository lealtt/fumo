use crate::{
    Context, Error,
    constants::{colors, icon},
    database::{self, BlacklistEntryModel},
    functions::{
        format::{
            discord::{bold, mention},
            pretty_message,
        },
        interactions::pagination::paginate,
        time,
    },
};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateEmbedFooter;
use std::time::Duration;

const LIST_PAGE_SIZE: usize = 10;
const LIST_FETCH_LIMIT: i64 = 50;
const LIST_PAGINATION_TIMEOUT: Duration = Duration::from_secs(180);

/// Ferramentas administrativas para gerenciar a blacklist global do bot.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "blacklist",
    category = "Equipe",
    interaction_context = "Guild",
    owners_only,
    on_error = "crate::commands::util::command_error_handler",
    ephemeral = true,
    subcommands(
        "blacklist_add",
        "blacklist_remove",
        "blacklist_info",
        "blacklist_list"
    )
)]
pub async fn blacklist(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Adiciona um usuário na blacklist.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "adicionar",
    category = "Equipe",
    interaction_context = "Guild",
    owners_only,
    on_error = "crate::commands::util::command_error_handler",
    ephemeral = true
)]
pub async fn blacklist_add(
    ctx: Context<'_>,
    #[description = "Usuário que será impedido de usar comandos"] user: serenity::User,
    #[description = "Motivo mostrado ao usuário"] reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if user.id == ctx.author().id {
        ctx.send(poise::CreateReply::default().content(pretty_message(
            icon::ERROR,
            "Você não pode se adicionar na blacklist.",
        )))
        .await?;
        return Ok(());
    }

    if ctx.framework().options().owners.contains(&user.id) {
        ctx.send(poise::CreateReply::default().content(pretty_message(
            icon::ERROR,
            "Você não pode adicionar um owner do bot na blacklist.",
        )))
        .await?;
        return Ok(());
    }

    let db = ctx.data().database.clone();
    let result = {
        let discord_id = user.id.get() as i64;
        if let Some(existing) = database::find_blacklist_entry(&db, discord_id).await? {
            Err(existing)
        } else {
            let moderator_id = ctx.author().id.get() as i64;
            Ok(
                database::insert_blacklist_entry(&db, discord_id, moderator_id, reason.clone())
                    .await?,
            )
        }
    };

    match result {
        Ok(entry) => {
            let embed = serenity::CreateEmbed::new()
                .title(format!("{} Usuário bloqueado", icon::CHECK))
                .colour(colors::MINT)
                .description(build_entry_description(&entry));

            ctx.send(
                poise::CreateReply::default()
                    .content(pretty_message(
                        icon::CHECK,
                        format!("{} foi adicionado à blacklist.", user.tag()),
                    ))
                    .embed(embed),
            )
            .await?;
        }
        Err(existing) => {
            let embed = serenity::CreateEmbed::new()
                .title(format!("{} Já estava na blacklist", icon::MINUS))
                .colour(colors::MOON)
                .description(build_entry_description(&existing));

            ctx.send(
                poise::CreateReply::default()
                    .content(pretty_message(
                        icon::MINUS,
                        format!("{} já estava bloqueado.", user.tag()),
                    ))
                    .embed(embed),
            )
            .await?;
        }
    }

    Ok(())
}

/// Remove um usuário da blacklist.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "remover",
    category = "Equipe",
    interaction_context = "Guild",
    owners_only,
    on_error = "crate::commands::util::command_error_handler",
    ephemeral = true
)]
pub async fn blacklist_remove(
    ctx: Context<'_>,
    #[description = "Usuário que será liberado"] user: serenity::User,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let db = ctx.data().database.clone();
    let (removed, existing_entry) = {
        let discord_id = user.id.get() as i64;
        let entry = database::find_blacklist_entry(&db, discord_id).await?;
        let removed = database::delete_blacklist_entry(&db, discord_id).await? > 0;
        (removed, entry)
    };

    if removed {
        ctx.send(poise::CreateReply::default().content(pretty_message(
            icon::CHECK,
            format!("{} foi removido da blacklist.", user.tag()),
        )))
        .await?;
    } else {
        let mut reply = poise::CreateReply::default().content(pretty_message(
            icon::MINUS,
            format!("{} não estava na blacklist.", user.tag()),
        ));

        if let Some(entry) = existing_entry {
            let embed = serenity::CreateEmbed::new()
                .title(format!("{} Último registro conhecido", icon::MINUS))
                .description(build_entry_description(&entry))
                .colour(colors::MOON);
            reply = reply.embed(embed);
        }

        ctx.send(reply).await?;
    }

    Ok(())
}

/// Mostra detalhes da entrada de um usuário.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "ver",
    category = "Equipe",
    interaction_context = "Guild",
    owners_only,
    on_error = "crate::commands::util::command_error_handler",
    ephemeral = true
)]
pub async fn blacklist_info(
    ctx: Context<'_>,
    #[description = "Usuário para consultar"] user: serenity::User,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let db = ctx.data().database.clone();
    let entry = database::find_blacklist_entry(&db, user.id.get() as i64).await?;

    if let Some(entry) = entry {
        let embed = serenity::CreateEmbed::new()
            .title(format!("{} Entrada encontrada", icon::CHECK))
            .colour(colors::MINT)
            .description(build_entry_description(&entry));

        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::CHECK,
                    format!("{} está bloqueado.", user.tag()),
                ))
                .embed(embed),
        )
        .await?;
    } else {
        ctx.send(poise::CreateReply::default().content(pretty_message(
            icon::MINUS,
            format!("{} não possui uma entrada na blacklist.", user.tag()),
        )))
        .await?;
    }

    Ok(())
}

/// Lista as entradas mais recentes da blacklist.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "listar",
    category = "Equipe",
    interaction_context = "Guild",
    owners_only,
    on_error = "crate::commands::util::command_error_handler",
    ephemeral = true
)]
pub async fn blacklist_list(
    ctx: Context<'_>,
    #[description = "Página inicial (1 = mais recente)"] page: Option<u32>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let db = ctx.data().database.clone();
    let entries = database::list_blacklist_entries(&db, LIST_FETCH_LIMIT, 0).await?;

    if entries.is_empty() {
        ctx.send(poise::CreateReply::default().content(pretty_message(
            icon::MINUS,
            "Não há usuários na blacklist no momento.",
        )))
        .await?;
        return Ok(());
    }

    let pages = build_blacklist_pages(&entries);
    let requested_page = page.unwrap_or(1).max(1) as usize - 1;
    let initial_page = requested_page.min(pages.len() - 1);

    paginate_blacklist(ctx, pages, initial_page, entries.len()).await
}

async fn paginate_blacklist(
    ctx: Context<'_>,
    pages: Vec<String>,
    initial_page: usize,
    total_entries: usize,
) -> Result<(), Error> {
    if pages.is_empty() {
        return Ok(());
    }

    let total_pages = pages.len();
    let visible_entries = total_entries.min(LIST_FETCH_LIMIT as usize);

    paginate(
        ctx,
        total_pages,
        LIST_PAGINATION_TIMEOUT,
        true,
        initial_page,
        move |current_page, total_pages| {
            let embed = build_blacklist_embed(
                &pages[current_page],
                current_page,
                total_pages,
                visible_entries,
            );
            (embed, Vec::new())
        },
    )
    .await
}

fn build_blacklist_pages(entries: &[BlacklistEntryModel]) -> Vec<String> {
    entries
        .chunks(LIST_PAGE_SIZE)
        .enumerate()
        .map(|(page_idx, chunk)| {
            let start_index = page_idx * LIST_PAGE_SIZE;
            chunk
                .iter()
                .enumerate()
                .map(|(idx, entry)| {
                    let position = start_index + idx + 1;
                    format!(
                        "{}\n{}",
                        bold(format!("{position}º")),
                        build_entry_description(entry)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .collect()
}

fn build_blacklist_embed(
    page_content: &str,
    current_page: usize,
    total_pages: usize,
    visible_entries: usize,
) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(format!("{} Usuários bloqueados", icon::BELL))
        .colour(colors::MOON)
        .description(page_content.to_string())
        .footer(CreateEmbedFooter::new(format!(
            "Página {}/{} • últimas {} entradas (máx. {})",
            current_page + 1,
            total_pages,
            visible_entries,
            LIST_FETCH_LIMIT
        )))
}

fn build_entry_description(entry: &BlacklistEntryModel) -> String {
    build_entry_lines(entry).join("\n")
}

fn build_entry_lines(entry: &BlacklistEntryModel) -> Vec<String> {
    let registered = time::describe_relative_from_str(&entry.created_at)
        .unwrap_or_else(|| entry.created_at.clone());

    vec![
        pretty_message(
            icon::BELL,
            format!("Usuário: {}", mention(entry.discord_id)),
        ),
        pretty_message(icon::HASTAG, format!("Motivo: {}", reason_text(entry))),
        pretty_message(
            icon::PLUS,
            format!("Responsável: {}", mention(entry.moderator_id)),
        ),
        pretty_message(icon::TIMER, format!("Registrado {registered}")),
    ]
}

fn reason_text(entry: &BlacklistEntryModel) -> &str {
    entry.reason.as_deref().unwrap_or("Sem motivo informado")
}
