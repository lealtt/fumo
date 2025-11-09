use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::CreateActionRow;
use serenity::builder::CreateInteractionResponseMessage;

/// Sends a simple ephemeral message in response to a component interaction.
pub async fn send_ephemeral_response(
    ctx: &Context<'_>,
    interaction: &serenity::ComponentInteraction,
    content: impl Into<String>,
) -> Result<(), Error> {
    let response = CreateInteractionResponseMessage::new()
        .content(content)
        .ephemeral(true);

    interaction
        .create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::Message(response),
        )
        .await?;
    Ok(())
}

/// Updates the original message tied to the component interaction.
pub async fn update_component_message(
    ctx: &Context<'_>,
    interaction: &serenity::ComponentInteraction,
    embed: serenity::CreateEmbed,
    components: Vec<CreateActionRow>,
) -> Result<(), Error> {
    let response = CreateInteractionResponseMessage::new()
        .embed(embed)
        .components(components);

    interaction
        .create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::UpdateMessage(response),
        )
        .await?;
    Ok(())
}
