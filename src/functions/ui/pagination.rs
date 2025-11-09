use std::time::Duration;

use poise::serenity_prelude as serenity;
use serenity::collector::ComponentInteractionCollector;
use serenity::{ButtonStyle, CreateActionRow, CreateButton};

use crate::{Context, Error};

use super::component::update_component_message;

/// Provides a generic paginator that can be reused across commands
pub async fn paginate<F>(
    ctx: Context<'_>,
    total_pages: usize,
    timeout: Duration,
    ephemeral: bool,
    mut build_page: F,
) -> Result<(), Error>
where
    F: FnMut(usize, usize) -> (serenity::CreateEmbed, Vec<CreateActionRow>),
{
    if total_pages == 0 {
        return Ok(());
    }

    let mut current_page = 0_usize;
    let buttons = PaginationButtons::new(ctx.id());

    let (embed, mut components) = build_page(current_page, total_pages);
    components.push(build_navigation_row(&buttons, current_page, total_pages));

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed)
                .components(components)
                .ephemeral(ephemeral),
        )
        .await?;
    let message = reply.message().await?;

    while let Some(interaction) = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .message_id(message.id)
        .timeout(timeout)
        .await
    {
        let last_index = total_pages - 1;
        let next_index = match interaction.data.custom_id.as_str() {
            id if id == buttons.first => 0,
            id if id == buttons.prev => current_page.saturating_sub(1),
            id if id == buttons.home => 0,
            id if id == buttons.next => (current_page + 1).min(last_index),
            id if id == buttons.last => last_index,
            _ => continue,
        };

        current_page = next_index;
        let (embed, mut components) = build_page(current_page, total_pages);
        components.push(build_navigation_row(&buttons, current_page, total_pages));
        update_component_message(&ctx, &interaction, embed, components).await?;
    }

    Ok(())
}

fn build_navigation_row(
    buttons: &PaginationButtons,
    current_page: usize,
    total_pages: usize,
) -> CreateActionRow {
    let disable_back = total_pages <= 1 || current_page == 0;
    let disable_forward = total_pages <= 1 || current_page + 1 >= total_pages;

    let row = vec![
        CreateButton::new(buttons.first.clone())
            .style(ButtonStyle::Secondary)
            .emoji(crate::constants::icon::CARET_DOUBLE_LEFT.as_reaction())
            .disabled(disable_back),
        CreateButton::new(buttons.prev.clone())
            .style(ButtonStyle::Secondary)
            .emoji(crate::constants::icon::CARET_LEFT.as_reaction())
            .disabled(disable_back),
        CreateButton::new(buttons.home.clone())
            .style(ButtonStyle::Secondary)
            .emoji(crate::constants::icon::HOUSE.as_reaction())
            .disabled(current_page == 0),
        CreateButton::new(buttons.next.clone())
            .style(ButtonStyle::Secondary)
            .emoji(crate::constants::icon::CARET_RIGHT.as_reaction())
            .disabled(disable_forward),
        CreateButton::new(buttons.last.clone())
            .style(ButtonStyle::Secondary)
            .emoji(crate::constants::icon::CARET_DOUBLE_RIGHT.as_reaction())
            .disabled(disable_forward),
    ];

    CreateActionRow::Buttons(row)
}

struct PaginationButtons {
    first: String,
    prev: String,
    home: String,
    next: String,
    last: String,
}

impl PaginationButtons {
    fn new(ctx_id: u64) -> Self {
        Self {
            first: format!("{ctx_id}_pg_first"),
            prev: format!("{ctx_id}_pg_prev"),
            home: format!("{ctx_id}_pg_home"),
            next: format!("{ctx_id}_pg_next"),
            last: format!("{ctx_id}_pg_last"),
        }
    }
}
