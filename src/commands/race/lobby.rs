use super::state::{RaceAnimal, RaceContestant};
use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::{
        format::pretty_message,
        interactions::component::{send_ephemeral_response, update_component_message},
    },
};
use poise::serenity_prelude::{self as serenity, Mentionable};
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateButton};
use std::time::Duration;

const JOIN_PREFIX: &str = "race_join:";
const START_BUTTON_ID: &str = "race_start";
const CANCEL_BUTTON_ID: &str = "race_cancel";

pub struct LobbyMessageHandle {
    pub channel_id: serenity::ChannelId,
    pub message_id: serenity::MessageId,
}

pub enum LobbyOutcome {
    Started {
        participants: Vec<RaceContestant>,
        message: LobbyMessageHandle,
    },
    Cancelled,
    Timeout,
}

pub struct RaceLobby {
    host: serenity::User,
    participants: Vec<RaceContestant>,
    min_participants: usize,
    available_animals: Vec<RaceAnimal>,
}

impl RaceLobby {
    pub fn new(
        host: serenity::User,
        min_participants: usize,
        available_animals: Vec<RaceAnimal>,
    ) -> Self {
        Self {
            host,
            participants: Vec::new(),
            min_participants,
            available_animals,
        }
    }

    pub fn render_view(&self) -> (serenity::CreateEmbed, Vec<CreateActionRow>) {
        let mut embed = serenity::CreateEmbed::new()
            .colour(colors::MOON)
            .description(pretty_message(
                icon::BELL,
                "Clique em um animal para participar. O mínimo é 3 participantes.",
            ))
            .footer(serenity::CreateEmbedFooter::new(format!(
                "Host: {}",
                self.host.name.clone()
            )));

        embed = embed.field(
            format!(
                "Participantes ({}/{})",
                self.participants.len(),
                self.min_participants
            ),
            self.participant_list(),
            false,
        );

        (embed, self.build_components())
    }

    pub async fn wait_for_start(
        &mut self,
        ctx: &Context<'_>,
        handle: &LobbyMessageHandle,
        timeout: Duration,
    ) -> Result<LobbyOutcome, Error> {
        loop {
            let collector = ComponentInteractionCollector::new(ctx.serenity_context())
                .message_id(handle.message_id)
                .timeout(timeout);
            let Some(interaction) = collector.await else {
                self.handle_timeout(ctx, handle).await?;
                return Ok(LobbyOutcome::Timeout);
            };

            if let Some(animal_id) = interaction.data.custom_id.strip_prefix(JOIN_PREFIX) {
                self.handle_join(ctx, &interaction, animal_id).await?;
                continue;
            }

            match interaction.data.custom_id.as_str() {
                START_BUTTON_ID => {
                    if interaction.user.id != self.host.id {
                        continue;
                    }

                    if self.participants.len() < self.min_participants {
                        send_ephemeral_response(
                            ctx,
                            &interaction,
                            pretty_message(
                                icon::ERROR,
                                format!(
                                    "São necessários pelo menos {} participantes.",
                                    self.min_participants
                                ),
                            ),
                        )
                        .await?;
                        continue;
                    }

                    let (embed, _) = self.render_start_embed();
                    update_component_message(ctx, &interaction, embed, Vec::new()).await?;
                    return Ok(LobbyOutcome::Started {
                        participants: self.participants.clone(),
                        message: LobbyMessageHandle {
                            channel_id: handle.channel_id,
                            message_id: handle.message_id,
                        },
                    });
                }
                CANCEL_BUTTON_ID => {
                    if interaction.user.id != self.host.id {
                        continue;
                    }

                    let embed = serenity::CreateEmbed::new()
                        .colour(colors::MOON)
                        .description(pretty_message(
                            icon::ERROR,
                            "A corrida foi cancelada pelo autor.",
                        ));
                    update_component_message(ctx, &interaction, embed, Vec::new()).await?;
                    return Ok(LobbyOutcome::Cancelled);
                }
                _ => {}
            }
        }
    }

    fn ready_to_start(&self) -> bool {
        self.participants.len() >= self.min_participants
    }

    fn participant_list(&self) -> String {
        if self.participants.is_empty() {
            return "Ninguém entrou ainda.".into();
        }

        self.participants
            .iter()
            .enumerate()
            .map(|(idx, participant)| {
                format!(
                    "{}. {} {}",
                    idx + 1,
                    participant.animal.emoji,
                    participant.user.mention()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn build_components(&self) -> Vec<CreateActionRow> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();

        for animal in self.available_animals.iter().take(8) {
            let is_taken = self.is_animal_taken(animal.id);
            let button = CreateButton::new(format!("{JOIN_PREFIX}{}", animal.id))
                .label(animal.emoji)
                .style(serenity::ButtonStyle::Primary)
                .disabled(is_taken);

            current_row.push(button);
            if current_row.len() == 4 {
                rows.push(CreateActionRow::Buttons(current_row));
                current_row = Vec::new();
            }
        }

        if !current_row.is_empty() {
            rows.push(CreateActionRow::Buttons(current_row));
        }

        let start = CreateButton::new(START_BUTTON_ID)
            .label("Iniciar")
            .style(serenity::ButtonStyle::Secondary)
            .disabled(!self.ready_to_start())
            .emoji(icon::CHECK.as_reaction());
        let cancel = CreateButton::new(CANCEL_BUTTON_ID)
            .label("Cancelar")
            .style(serenity::ButtonStyle::Danger)
            .emoji(icon::ERROR.as_reaction());

        rows.push(CreateActionRow::Buttons(vec![start, cancel]));
        rows
    }

    async fn handle_join(
        &mut self,
        ctx: &Context<'_>,
        interaction: &serenity::ComponentInteraction,
        animal_id: &str,
    ) -> Result<(), Error> {
        let Some(animal) = self.find_animal(animal_id) else {
            return Ok(());
        };

        let user = interaction.user.clone();

        if !self
            .participants
            .iter()
            .any(|participant| participant.user.id == user.id)
            && self.participants.len() >= self.available_animals.len()
        {
            return Ok(());
        }

        if self
            .participants
            .iter()
            .any(|participant| participant.animal.id == animal.id && participant.user.id != user.id)
        {
            return Ok(());
        }

        let updated = if let Some(existing) = self
            .participants
            .iter_mut()
            .find(|participant| participant.user.id == user.id)
        {
            if existing.animal.id == animal.id {
                false
            } else {
                existing.animal = animal;
                true
            }
        } else {
            self.participants.push(RaceContestant { user, animal });
            true
        };

        if !updated {
            return Ok(());
        }

        let (embed, components) = self.render_view();
        update_component_message(ctx, interaction, embed, components).await
    }

    fn render_start_embed(&self) -> (serenity::CreateEmbed, Vec<CreateActionRow>) {
        let embed = serenity::CreateEmbed::new()
            .title("🏁 Corrida começando!")
            .colour(colors::MINT)
            .field("Participantes", self.participant_list(), false);
        (embed, Vec::new())
    }

    async fn handle_timeout(
        &self,
        ctx: &Context<'_>,
        handle: &LobbyMessageHandle,
    ) -> Result<(), Error> {
        if handle
            .channel_id
            .message(ctx.serenity_context(), handle.message_id)
            .await
            .is_err()
        {
            return Ok(());
        }

        let _ = handle
            .channel_id
            .delete_message(ctx.serenity_context(), handle.message_id)
            .await;
        Ok(())
    }

    fn is_animal_taken(&self, animal_id: &str) -> bool {
        self.participants
            .iter()
            .any(|participant| participant.animal.id == animal_id)
    }

    fn find_animal(&self, animal_id: &str) -> Option<RaceAnimal> {
        self.available_animals
            .iter()
            .copied()
            .find(|animal| animal.id == animal_id)
    }
}
