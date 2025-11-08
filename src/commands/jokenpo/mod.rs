use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::ui::{
        pretty_message::pretty_message,
        prompt::{
            ConfirmationMessageHandle, ConfirmationOutcome, ConfirmationPromptOptions,
            confirmation_prompt,
        },
    },
};
use poise::serenity_prelude::{self as serenity, Mentionable};
use rand::prelude::IndexedRandom;
use serenity::builder::{CreateInteractionResponseMessage, EditMessage};
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateButton};
use std::{collections::HashMap, time::Duration};

mod game_move;
use game_move::GameMove;

const SOLO_TIMEOUT: Duration = Duration::from_secs(30);
const VERSUS_ROUND_TIMEOUT: Duration = Duration::from_secs(60);
const CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(45);

/// Jogue pedra, papel ou tesoura!
#[poise::command(
    slash_command,
    prefix_command,
    interaction_context = "Guild",
    category = "Jogos",
    subcommands("fumo", "versus")
)]
pub async fn jokenpo(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Enfrente a Fumo e teste sua sorte!
#[poise::command(slash_command, prefix_command, category = "Jogos")]
pub async fn fumo(ctx: Context<'_>) -> Result<(), Error> {
    let prompt = ctx
        .send(
            poise::CreateReply::default()
                .content(pretty_message(icon::BELL, "JoKenPo! Escolha sua jogada:"))
                .components(action_rows(false)),
        )
        .await?;

    let message = prompt.message().await?.id;

    let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .message_id(message)
        .timeout(SOLO_TIMEOUT)
        .await;

    match interaction {
        Some(interaction) => {
            let Some(user_move) = GameMove::from_custom_id(&interaction.data.custom_id) else {
                let response = CreateInteractionResponseMessage::new()
                    .content("Jogada desconhecida recebida. Por favor, tente novamente.")
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
                let mut rng = rand::rng();
                *GameMove::ALL
                    .choose(&mut rng)
                    .expect("Move list should not be empty")
            };

            let outcome = if user_move == bot_move {
                "Empate!"
            } else if user_move.beats(bot_move) {
                "Você venceu!"
            } else {
                "Eu venci!"
            };

            let response = CreateInteractionResponseMessage::new()
                .content(pretty_message(
                    icon::CHECK,
                    format!(
                        "Você escolheu {}. Eu escolhi {}. {}",
                        user_move, bot_move, outcome
                    ),
                ))
                .components(action_rows(true));

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
                        .content(pretty_message(
                            icon::ERROR,
                            "JoKenPo expirou. Tente novamente quando estiver pronto!",
                        ))
                        .components(Vec::new()),
                )
                .await?;
        }
    }

    Ok(())
}

/// Desafie outra pessoa para uma partida.
#[poise::command(slash_command, prefix_command, category = "Jogos")]
pub async fn versus(
    ctx: Context<'_>,
    #[description = "Jogador que você deseja desafiar"] opponent: serenity::User,
) -> Result<(), Error> {
    if opponent.id == ctx.author().id {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    "Você precisa convidar outra pessoa para jogar.",
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    if opponent.bot {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    "Bots preferem assistir. Escolha um usuário humano.",
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mut prompt = ConfirmationPromptOptions::new(pretty_message(
        icon::BELL,
        format!(
            "{} desafiou {} para um JoKenPo. Aceita?",
            ctx.author().mention(),
            opponent.mention()
        ),
    ));
    prompt.timeout = CONFIRMATION_TIMEOUT;
    prompt.keep_message_on_accept = true;

    let confirmation = confirmation_prompt(&ctx, opponent.id, prompt).await?;

    match confirmation.outcome {
        ConfirmationOutcome::Accepted => {
            start_versus_match(ctx, opponent, confirmation.message).await
        }
        ConfirmationOutcome::Declined => {
            ctx.send(
                poise::CreateReply::default()
                    .content(pretty_message(
                        icon::ERROR,
                        "Convite recusado. Quem sabe mais tarde!",
                    ))
                    .ephemeral(true),
            )
            .await?;
            Ok(())
        }
        ConfirmationOutcome::Timeout => Ok(()),
    }
}

async fn start_versus_match(
    ctx: Context<'_>,
    opponent: serenity::User,
    existing_message: Option<ConfirmationMessageHandle>,
) -> Result<(), Error> {
    let challenger = ctx.author().clone();

    let (channel_id, message_id, needs_edit) = if let Some(handle) = existing_message {
        (handle.channel_id, handle.message_id, true)
    } else {
        let reply = ctx
            .send(
                poise::CreateReply::default()
                    .embed(versus_waiting_embed(&challenger, &opponent))
                    .components(action_rows(false)),
            )
            .await?;
        let message = reply.message().await?;
        (message.channel_id, message.id, false)
    };

    if needs_edit {
        channel_id
            .edit_message(
                ctx.serenity_context(),
                message_id,
                EditMessage::new()
                    .content("")
                    .embed(versus_waiting_embed(&challenger, &opponent))
                    .components(action_rows(false)),
            )
            .await?;
    }

    let mut selections: HashMap<serenity::UserId, GameMove> = HashMap::new();

    loop {
        let interaction = ComponentInteractionCollector::new(ctx.serenity_context())
            .message_id(message_id)
            .timeout(VERSUS_ROUND_TIMEOUT)
            .await;

        let Some(interaction) = interaction else {
            channel_id
                .edit_message(
                    ctx.serenity_context(),
                    message_id,
                    serenity::builder::EditMessage::new()
                        .content("")
                        .embed(versus_cancelled_embed("Partida cancelada por inatividade."))
                        .components(Vec::new()),
                )
                .await?;
            return Ok(());
        };

        if interaction.user.id != challenger.id && interaction.user.id != opponent.id {
            send_ephemeral_message(
                ctx,
                &interaction,
                pretty_message(
                    icon::ERROR,
                    "Apenas os desafiantes podem usar estes botões.",
                ),
            )
            .await?;
            continue;
        }

        let Some(chosen_move) = GameMove::from_custom_id(&interaction.data.custom_id) else {
            continue;
        };

        if selections.contains_key(&interaction.user.id) {
            send_ephemeral_message(
                ctx,
                &interaction,
                pretty_message(icon::ERROR, "Você já escolheu sua jogada."),
            )
            .await?;
            continue;
        }

        selections.insert(interaction.user.id, chosen_move);

        send_ephemeral_message(
            ctx,
            &interaction,
            pretty_message(icon::CHECK, format!("Jogada registrada: {}", chosen_move)),
        )
        .await?;

        if selections.len() == 2 {
            break;
        }
    }

    let challenger_move = selections
        .get(&challenger.id)
        .copied()
        .expect("challenger move missing");
    let opponent_move = selections
        .get(&opponent.id)
        .copied()
        .expect("opponent move missing");

    let outcome = match (challenger_move, opponent_move) {
        (a, b) if a == b => pretty_message(icon::HASTAG, "Empate!"),
        (a, b) if a.beats(b) => pretty_message(
            icon::CHECK,
            format!("{} venceu a rodada!", challenger.mention()),
        ),
        _ => pretty_message(
            icon::CHECK,
            format!("{} venceu a rodada!", opponent.mention()),
        ),
    };

    let embed = serenity::CreateEmbed::new()
        .title("🪨 JoKenPo")
        .colour(colors::MINT)
        .description(
            [
                pretty_message(
                    icon::BELL,
                    format!("{} escolheu {}", challenger.mention(), challenger_move),
                ),
                pretty_message(
                    icon::BELL,
                    format!("{} escolheu {}", opponent.mention(), opponent_move),
                ),
                String::new(),
                outcome,
            ]
            .join("\n"),
        );

    channel_id
        .edit_message(
            ctx.serenity_context(),
            message_id,
            serenity::builder::EditMessage::new()
                .content("")
                .embed(embed)
                .components(action_rows(true)),
        )
        .await?;

    Ok(())
}

async fn send_ephemeral_message(
    ctx: Context<'_>,
    interaction: &serenity::ComponentInteraction,
    content: String,
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

fn action_rows(disabled: bool) -> Vec<CreateActionRow> {
    let buttons = GameMove::ALL
        .into_iter()
        .map(|mv| {
            let style = if disabled {
                serenity::ButtonStyle::Secondary
            } else {
                serenity::ButtonStyle::Primary
            };
            CreateButton::new(mv.custom_id())
                .label(mv.label())
                .style(style)
                .disabled(disabled)
        })
        .collect();
    vec![CreateActionRow::Buttons(buttons)]
}

fn versus_waiting_embed(
    challenger: &serenity::User,
    opponent: &serenity::User,
) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title("🪨 JoKenPo")
        .colour(colors::MOON)
        .description(
            vec![
                pretty_message(
                    icon::BELL,
                    format!("{} vs {}", challenger.mention(), opponent.mention()),
                ),
                pretty_message(
                    icon::TIMER,
                    "Escolham uma jogada nos botões abaixo. O resultado aparece assim que ambos decidirem.",
                ),
            ]
            .join("\n"),
        )
}

fn versus_cancelled_embed(reason: &str) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title("🪨 JoKenPo")
        .colour(colors::MOON)
        .description(pretty_message(icon::ERROR, reason))
}
