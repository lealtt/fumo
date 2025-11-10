use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Manages the plain-text message that displays the race track
pub struct RaceProgressMessage {
    channel_id: serenity::ChannelId,
    message_id: serenity::MessageId,
    stale_edits: usize,
}

const MAX_STALE_EDITS: usize = 5;

impl RaceProgressMessage {
    pub async fn create(ctx: &Context<'_>, content: impl Into<String>) -> Result<Self, Error> {
        let message = ctx
            .channel_id()
            .send_message(
                ctx.serenity_context(),
                serenity::CreateMessage::new().content(content.into()),
            )
            .await?;

        Ok(Self {
            channel_id: message.channel_id,
            message_id: message.id,
            stale_edits: 0,
        })
    }

    /// Updates the existing message and occasionally re-posts if the race scrolls away
    pub async fn refresh(
        &mut self,
        ctx: &Context<'_>,
        content: impl Into<String>,
    ) -> Result<(), Error> {
        let content = content.into();
        if self.needs_repost(ctx).await? {
            self.recreate_message(ctx, &content).await?;
        } else {
            self.edit_existing(ctx, &content).await?;
        }
        Ok(())
    }

    async fn needs_repost(&mut self, ctx: &Context<'_>) -> Result<bool, Error> {
        let builder = serenity::builder::GetMessages::new().limit(1);
        let result = self
            .channel_id
            .messages(ctx.serenity_context(), builder)
            .await;

        if let Ok(messages) = result {
            if let Some(last_message) = messages.first() {
                if last_message.id != self.message_id {
                    self.stale_edits += 1;
                } else {
                    self.stale_edits = 0;
                }
            }
        } else {
            self.stale_edits = 0;
        }

        Ok(self.stale_edits >= MAX_STALE_EDITS)
    }

    async fn edit_existing(&self, ctx: &Context<'_>, content: &str) -> Result<(), Error> {
        self.channel_id
            .edit_message(
                ctx.serenity_context(),
                self.message_id,
                serenity::EditMessage::new().content(content.to_string()),
            )
            .await?;
        Ok(())
    }

    async fn recreate_message(&mut self, ctx: &Context<'_>, content: &str) -> Result<(), Error> {
        let _ = self
            .channel_id
            .delete_message(ctx.serenity_context(), self.message_id)
            .await;

        let message = self
            .channel_id
            .send_message(
                ctx.serenity_context(),
                serenity::CreateMessage::new().content(content.to_string()),
            )
            .await?;

        self.message_id = message.id;
        self.stale_edits = 0;
        Ok(())
    }
}
