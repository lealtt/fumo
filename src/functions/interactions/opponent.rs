use crate::{Context, Error, constants::icon, functions::format::pretty_message};
use poise::serenity_prelude as serenity;

/// Custom error messages used when validating PvP opponents
pub struct OpponentValidationMessages<'a> {
    pub self_error: &'a str,
    pub bot_error: &'a str,
}

impl<'a> OpponentValidationMessages<'a> {
    pub fn new(self_error: &'a str, bot_error: &'a str) -> Self {
        Self {
            self_error,
            bot_error,
        }
    }
}

/// Ensures the opponent is not the command author and not a bot
pub async fn ensure_valid_opponent(
    ctx: &Context<'_>,
    opponent: &serenity::User,
    messages: OpponentValidationMessages<'_>,
) -> Result<bool, Error> {
    if opponent.id == ctx.author().id {
        send_error(ctx, messages.self_error).await?;
        return Ok(false);
    }

    if opponent.bot {
        send_error(ctx, messages.bot_error).await?;
        return Ok(false);
    }

    Ok(true)
}

async fn send_error(ctx: &Context<'_>, message: &str) -> Result<(), Error> {
    ctx.send(
        poise::CreateReply::default()
            .content(pretty_message(icon::ERROR, message))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
