use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::ui::{
        component::{send_ephemeral_response, update_component_message},
        pretty_message::pretty_message,
        prompt::{
            ConfirmationMessageHandle, ConfirmationOutcome, ConfirmationPromptOptions,
            confirmation_prompt,
        },
    },
};
use poise::serenity_prelude::{self as serenity, Mentionable};
use serenity::builder::EditMessage;
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateButton};
use std::time::Duration;
use tokio::time::sleep;

mod game_mode;
mod game_state;
mod player_state;
mod tile;

use game_mode::Mode;
use game_state::{MemoryGameState, SelectionResult};

const BOARD_COLUMNS: usize = 4;
const MISMATCH_DELAY: Duration = Duration::from_secs(2);
const GAME_TIMEOUT: Duration = Duration::from_secs(300);
const CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(45);
const HIDDEN_LABEL: &str = "❔";

/// Jogo da memória com pares de emojis.
#[poise::command(
    slash_command,
    prefix_command,
    interaction_context = "Guild",
    category = "Jogos",
    rename = "memoria",
    subcommands("solo", "versus")
)]
pub async fn memory(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Encontre todos os pares sozinho.
#[poise::command(slash_command, prefix_command, category = "Jogos")]
pub async fn solo(ctx: Context<'_>) -> Result<(), Error> {
    let player = player_state::PlayerState::new(ctx.author().clone());
    run_game(ctx, Mode::Solo { player }, None).await
}

/// Desafie outra pessoa para ver quem encontra mais pares.
#[poise::command(slash_command, prefix_command, category = "Jogos")]
pub async fn versus(
    ctx: Context<'_>,
    #[description = "Oponente que jogará com você"] opponent: serenity::User,
) -> Result<(), Error> {
    if opponent.id == ctx.author().id {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    "Você precisa escolher alguém diferente para jogar com você.",
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
                    "Bots preferem assistir à partida, escolha um usuário humano.",
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mut prompt_options = ConfirmationPromptOptions::new(pretty_message(
        icon::BELL,
        format!(
            "{} desafiou {} para um jogo da memória. Aceita?",
            ctx.author().mention(),
            opponent.mention()
        ),
    ));
    prompt_options.timeout = CONFIRMATION_TIMEOUT;
    prompt_options.keep_message_on_accept = true;

    let confirmation = confirmation_prompt(&ctx, opponent.id, prompt_options).await?;

    match confirmation.outcome {
        ConfirmationOutcome::Accepted => {
            let players = [
                player_state::PlayerState::new(ctx.author().clone()),
                player_state::PlayerState::new(opponent),
            ];
            run_game(
                ctx,
                Mode::Versus {
                    players,
                    current_turn: 0,
                },
                confirmation.message,
            )
            .await
        }
        ConfirmationOutcome::Declined | ConfirmationOutcome::Timeout => Ok(()),
    }
}

async fn run_game(
    ctx: Context<'_>,
    mode: Mode,
    existing_message: Option<ConfirmationMessageHandle>,
) -> Result<(), Error> {
    let mut state = MemoryGameState::new(mode);
    state.set_status(pretty_message(
        icon::BELL,
        "Abra dois botões e encontre os pares!",
    ));

    let (channel_id, message_id) = if let Some(handle) = existing_message {
        let (embed, components) = render_game(&state, None);
        handle
            .channel_id
            .edit_message(
                ctx.serenity_context(),
                handle.message_id,
                EditMessage::new()
                    .content("")
                    .embed(embed)
                    .components(components),
            )
            .await?;
        (handle.channel_id, handle.message_id)
    } else {
        let (embed, components) = render_game(&state, None);
        let reply = ctx
            .send(
                poise::CreateReply::default()
                    .embed(embed)
                    .components(components),
            )
            .await?;
        let message = reply.message().await?;
        (message.channel_id, message.id)
    };

    loop {
        let mut collector = ComponentInteractionCollector::new(ctx.serenity_context())
            .message_id(message_id)
            .timeout(GAME_TIMEOUT);
        if state.mode.is_strict_single_player() {
            collector = collector.author_id(state.mode.active_player_id());
        }

        let Some(interaction) = collector.await else {
            break;
        };

        let Some(tile_index) = parse_index(&interaction.data.custom_id, &state.custom_id_prefix)
        else {
            continue;
        };

        if !state.mode.is_allowed(interaction.user.id) {
            send_ephemeral_response(
                &ctx,
                &interaction,
                pretty_message(
                    icon::ERROR,
                    "Somente os participantes do jogo podem interagir com estes botões.",
                ),
            )
            .await?;
            continue;
        }

        if state.locked {
            send_ephemeral_response(
                &ctx,
                &interaction,
                pretty_message(icon::TIMER, "Espere um instante enquanto escondo as peças."),
            )
            .await?;
            continue;
        }

        if !state.is_selectable(tile_index) {
            send_ephemeral_response(
                &ctx,
                &interaction,
                pretty_message(icon::ERROR, "Esse botão já foi utilizado. Escolha outro."),
            )
            .await?;
            continue;
        }

        if state.pending.is_none() {
            if !state.mode.is_current_player(interaction.user.id) {
                send_ephemeral_response(
                    &ctx,
                    &interaction,
                    pretty_message(icon::BELL, "Aguarde sua vez para jogar."),
                )
                .await?;
                continue;
            }
            state.pending_owner = Some(interaction.user.id);
        } else if state.pending_owner != Some(interaction.user.id) {
            send_ephemeral_response(
                &ctx,
                &interaction,
                pretty_message(icon::ERROR, "Espere o jogador atual finalizar a tentativa."),
            )
            .await?;
            continue;
        }

        match state.select(tile_index) {
            SelectionResult::FirstReveal => {
                let (embed, components) = render_game(&state, None);
                update_component_message(&ctx, &interaction, embed, components).await?;
            }
            SelectionResult::Matched { finished } => {
                state.mode.register_match(interaction.user.id);
                state.set_status(pretty_message(
                    icon::CHECK,
                    format!("{} encontrou um par!", interaction.user.mention()),
                ));

                let (embed, components) = render_game(&state, None);
                update_component_message(&ctx, &interaction, embed, components).await?;

                if finished {
                    state.set_status(state.mode.finish_message(state.attempts));
                    let (embed, components) = render_game(&state, None);
                    channel_id
                        .edit_message(
                            ctx.serenity_context(),
                            message_id,
                            EditMessage::new()
                                .content("")
                                .embed(embed)
                                .components(components),
                        )
                        .await?;
                    return Ok(());
                }
            }
            SelectionResult::Mismatch { pair } => {
                state.locked = true;
                state.set_status(pretty_message(
                    icon::ERROR,
                    format!("{} não acertou o par.", interaction.user.mention()),
                ));

                {
                    let (embed, components) = render_game(&state, Some(pair));
                    update_component_message(&ctx, &interaction, embed, components).await?;
                }

                sleep(MISMATCH_DELAY).await;
                state.locked = false;
                state.mode.advance_turn();
                state.set_status(pretty_message(icon::BELL, state.mode.turn_message()));

                let (embed, components) = render_game(&state, None);
                channel_id
                    .edit_message(
                        ctx.serenity_context(),
                        message_id,
                        EditMessage::new()
                            .content("")
                            .embed(embed)
                            .components(components),
                    )
                    .await?;
            }
        }
    }

    state.set_status(pretty_message(
        icon::ERROR,
        "Jogo encerrado por inatividade.",
    ));
    let (embed, _) = render_game(&state, None);
    channel_id
        .edit_message(
            ctx.serenity_context(),
            message_id,
            EditMessage::new()
                .content("")
                .embed(embed)
                .components(Vec::new()),
        )
        .await?;

    Ok(())
}

fn render_game(
    state: &MemoryGameState,
    mismatch: Option<[usize; 2]>,
) -> (serenity::CreateEmbed, Vec<CreateActionRow>) {
    (build_embed(state), build_components(state, mismatch))
}

fn build_embed(state: &MemoryGameState) -> serenity::CreateEmbed {
    let mut lines = vec![
        pretty_message(
            icon::CHECK,
            format!(
                "**{}/{}** pares descobertos",
                state.matches,
                state.total_pairs()
            ),
        ),
        pretty_message(icon::TIMER, format!("**{}** tentativas", state.attempts)),
    ];

    if let Some(line) = state.mode.scoreboard_line() {
        lines.push(line);
    }

    if let Some(status) = &state.status_text {
        lines.push(String::new());
        lines.push(status.clone());
    }

    let title = match &state.mode {
        Mode::Solo { player } => format!("🧠 Memória de {}", player.user.name),
        Mode::Versus { players, .. } => {
            format!("🧠 {} vs {}", players[0].user.name, players[1].user.name)
        }
    };

    serenity::CreateEmbed::new()
        .title(title)
        .colour(colors::MOON)
        .description(lines.join("\n"))
}

fn build_components(state: &MemoryGameState, mismatch: Option<[usize; 2]>) -> Vec<CreateActionRow> {
    let mut rows = Vec::new();
    for chunk_start in (0..state.tiles.len()).step_by(BOARD_COLUMNS) {
        let mut buttons = Vec::new();
        for offset in 0..BOARD_COLUMNS {
            let index = chunk_start + offset;
            if index >= state.tiles.len() {
                break;
            }

            let tile = &state.tiles[index];
            let mut button = CreateButton::new(format!("{}{}", state.custom_id_prefix, index));
            let is_mismatch = mismatch.map(|pair| pair.contains(&index)).unwrap_or(false);

            if tile.matched {
                button = button
                    .label(tile.emoji)
                    .style(serenity::ButtonStyle::Success)
                    .disabled(true);
            } else if is_mismatch {
                button = button
                    .label(tile.emoji)
                    .style(serenity::ButtonStyle::Danger)
                    .disabled(true);
            } else if state.pending == Some(index) {
                button = button
                    .label(tile.emoji)
                    .style(serenity::ButtonStyle::Secondary)
                    .disabled(true);
            } else {
                button = button
                    .label(HIDDEN_LABEL)
                    .style(serenity::ButtonStyle::Primary);
            }

            buttons.push(button);
        }

        rows.push(CreateActionRow::Buttons(buttons));
    }

    rows
}

fn parse_index(custom_id: &str, prefix: &str) -> Option<usize> {
    custom_id.strip_prefix(prefix)?.parse().ok()
}
