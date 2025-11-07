use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateButton};
use std::{fmt, time::Duration};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    const ALL: [Move; 3] = [Move::Rock, Move::Paper, Move::Scissors];

    fn custom_id(self) -> &'static str {
        match self {
            Move::Rock => "jkp_rock",
            Move::Paper => "jkp_paper",
            Move::Scissors => "jkp_scissors",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Move::Rock => "🪨 Rock",
            Move::Paper => "📄 Paper",
            Move::Scissors => "✂️ Scissors",
        }
    }

    fn beats(self, other: Move) -> bool {
        matches!(
            (self, other),
            (Move::Rock, Move::Scissors)
                | (Move::Paper, Move::Rock)
                | (Move::Scissors, Move::Paper)
        )
    }

    fn from_custom_id(id: &str) -> Option<Self> {
        Move::ALL.into_iter().find(|mv| mv.custom_id() == id)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Rock => write!(f, "🪨 Rock"),
            Move::Paper => write!(f, "📄 Paper"),
            Move::Scissors => write!(f, "✂️ Scissors"),
        }
    }
}

/// Play rock-paper-scissors against me!
#[poise::command(
    slash_command,
    prefix_command,
    interaction_context = "Guild",
    category = "Games"
)]
pub async fn jokenpo(ctx: Context<'_>) -> Result<(), Error> {
    let buttons = Move::ALL
        .into_iter()
        .map(|mv| CreateButton::new(mv.custom_id()).label(mv.label()))
        .collect();
    let components = vec![CreateActionRow::Buttons(buttons)];

    let prompt = ctx
        .send(
            poise::CreateReply::default()
                .content("JoKenPo! Choose your move:")
                .components(components),
        )
        .await?;

    let message = prompt.message().await?.id;

    let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .message_id(message)
        .timeout(Duration::from_secs(30))
        .await;

    match interaction {
        Some(interaction) => {
            let Some(user_move) = Move::from_custom_id(&interaction.data.custom_id) else {
                let response = CreateInteractionResponseMessage::new()
                    .content("Unknown move received. Please try again.")
                    .components(Vec::new());

                interaction
                    .create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(response),
                    )
                    .await?;
                return Ok(());
            };

            let bot_move = {
                let mut rng = thread_rng();
                *Move::ALL
                    .choose(&mut rng)
                    .expect("Move list should not be empty")
            };

            let outcome = if user_move == bot_move {
                "It's a tie!"
            } else if user_move.beats(bot_move) {
                "You win!"
            } else {
                "I win!"
            };

            let response = CreateInteractionResponseMessage::new()
                .content(format!(
                    "You chose {}. I chose {}. {}",
                    user_move, bot_move, outcome
                ))
                .components(Vec::new());

            interaction
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(response),
                )
                .await?;
        }
        None => {
            prompt
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("JoKenPo timed out. Try again when you're ready!")
                        .components(Vec::new()),
                )
                .await?;
        }
    }

    Ok(())
}
