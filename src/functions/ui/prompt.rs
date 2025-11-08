use crate::constants::icon;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use rand::Rng;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateButton};
use std::time::Duration;

pub enum ConfirmationOutcome {
    Accepted,
    Declined,
    Timeout,
}

pub struct ConfirmationPromptOptions {
    pub content: String,
    pub timeout: Duration,
    pub keep_message_on_accept: bool,
}

impl ConfirmationPromptOptions {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            timeout: Duration::from_secs(45),
            keep_message_on_accept: false,
        }
    }
}

pub struct ConfirmationMessageHandle {
    pub channel_id: serenity::ChannelId,
    pub message_id: serenity::MessageId,
}

pub struct ConfirmationResult {
    pub outcome: ConfirmationOutcome,
    pub message: Option<ConfirmationMessageHandle>,
}

pub async fn confirmation_prompt(
    ctx: &Context<'_>,
    target_user: serenity::UserId,
    options: ConfirmationPromptOptions,
) -> Result<ConfirmationResult, Error> {
    let base_id = format!("confirm_{}", rand::rng().random::<u64>());
    let accept_id = format!("{base_id}_ok");
    let deny_id = format!("{base_id}_no");

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content(&options.content)
                .components(create_buttons(&accept_id, &deny_id, false)),
        )
        .await?;
    let message = reply.message().await?;
    let message_id = message.id;
    let channel_id = message.channel_id;

    let collector = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(target_user)
        .message_id(message_id)
        .timeout(options.timeout);

    let mut handle = Some(ConfirmationMessageHandle {
        channel_id,
        message_id,
    });

    let outcome = if let Some(interaction) = collector.await {
        let accepted = interaction.data.custom_id == accept_id;
        let response = CreateInteractionResponseMessage::new()
            .content(&options.content)
            .components(create_buttons(&accept_id, &deny_id, true));
        interaction
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(response),
            )
            .await?;

        if !options.keep_message_on_accept || !accepted {
            let _ = channel_id
                .delete_message(ctx.serenity_context(), message_id)
                .await;
            handle = None;
        }

        if accepted {
            ConfirmationOutcome::Accepted
        } else {
            ConfirmationOutcome::Declined
        }
    } else {
        let _ = channel_id
            .delete_message(ctx.serenity_context(), message_id)
            .await;
        handle = None;
        ConfirmationOutcome::Timeout
    };

    if !matches!(outcome, ConfirmationOutcome::Accepted) {
        if let Some(handle_ref) = &handle {
            let _ = handle_ref
                .channel_id
                .delete_message(ctx.serenity_context(), handle_ref.message_id)
                .await;
        }
        handle = None;
    }

    Ok(ConfirmationResult {
        outcome,
        message: handle,
    })
}

fn create_buttons(accept_id: &str, deny_id: &str, disabled: bool) -> Vec<CreateActionRow> {
    let accept = CreateButton::new(accept_id)
        .label("Aceitar")
        .style(serenity::ButtonStyle::Success)
        .disabled(disabled)
        .emoji(icon::CHECK.as_reaction());
    let deny = CreateButton::new(deny_id)
        .label("Recusar")
        .style(serenity::ButtonStyle::Danger)
        .disabled(disabled)
        .emoji(icon::ERROR.as_reaction());

    vec![CreateActionRow::Buttons(vec![accept, deny])]
}
