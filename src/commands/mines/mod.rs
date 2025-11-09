use crate::{
    Context, Error,
    constants::{colors, icon},
    database::{self, UserModel},
    functions::{
        format::format_currency,
        ui::{
            component::{send_ephemeral_response, update_component_message},
            pretty_message::pretty_message,
        },
    },
};
use poise::serenity_prelude::{self as serenity, Mentionable};
use serenity::builder::{CreateEmbedFooter, EditMessage};
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateAutocompleteResponse, CreateButton};
use std::time::Duration;

mod game_state;
mod tile;

use game_state::{MinesGameState, RevealOutcome};

pub(super) const BOARD_COLUMNS: usize = 4;
pub(super) const BOARD_ROWS: usize = 4;
pub(super) const TOTAL_BOMBS: usize = 5;
pub(super) const CASHOUT_STEP: usize = 3;
pub(super) const FORCE_CASHOUT_AFTER: usize = 9;
pub(super) const MULTIPLIER_STEP: f64 = 0.18;
pub(super) const MAX_MULTIPLIER: f64 = 4.2;
const HIDDEN_LABEL: &str = "❔";
const MIN_WAGER: i64 = 50;
const MAX_WAGER: i64 = 50_000;
const GAME_TIMEOUT: Duration = Duration::from_secs(180);

/// Mini game Mines: aposte e tente escapar das bombas.
#[poise::command(
    slash_command,
    prefix_command,
    category = "Jogos",
    interaction_context = "Guild"
)]
pub async fn mines(
    ctx: Context<'_>,
    #[description = "Valor da aposta"]
    #[autocomplete = "autocomplete_wager"]
    valor: i64,
) -> Result<(), Error> {
    if valor < MIN_WAGER {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    format!(
                        "O mínimo para jogar é **{}** moedas.",
                        format_currency(MIN_WAGER)
                    ),
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    if valor > MAX_WAGER {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    format!(
                        "O máximo permitido por rodada é **{}** moedas.",
                        format_currency(MAX_WAGER)
                    ),
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let player = ctx.author().clone();
    let discord_id = player.id.get() as i64;

    let mut user = {
        let db = ctx.data().database.lock().await;
        database::get_or_create_user(&db, discord_id).await?
    };
    let mut wager_transaction_id: Option<i32>;

    if user.dollars < valor {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    "Você não possui moedas suficientes para essa aposta.",
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    wager_transaction_id = {
        let db = ctx.data().database.lock().await;
        user.dollars -= valor;
        user = database::update_user_balance(&db, user.id, user.dollars, user.diamonds).await?;
        let transaction = database::insert_currency_transaction(
            &db,
            user.id,
            -valor,
            user.dollars,
            "dollars",
            "mines_wager",
            Some("Entrada no Mines".to_string()),
        )
        .await?;
        Some(transaction.id)
    };

    let mut state = MinesGameState::new(valor);
    state.set_status(pretty_message(
        icon::BELL,
        format!(
            "{} apostou **{}** moedas. Encontre {} diamantes antes de pensar em resgatar!",
            player.mention(),
            format_currency(valor),
            CASHOUT_STEP
        ),
    ));

    let (embed, components) = render_game(&state, &player);
    let reply = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed)
                .components(components),
        )
        .await?;

    let message = reply.message().await?;
    let message_id = message.id;
    let channel_id = message.channel_id;
    let cashout_id = format!("{}cashout", state.custom_id_prefix);
    let giveup_id = format!("{}giveup", state.custom_id_prefix);

    while let Some(interaction) = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(player.id)
        .message_id(message_id)
        .timeout(GAME_TIMEOUT)
        .await
    {
        if let Some(index) = parse_tile_index(&interaction.data.custom_id, &state.custom_id_prefix)
        {
            if state.is_finished() {
                send_ephemeral_response(
                    &ctx,
                    &interaction,
                    pretty_message(icon::ERROR, "Essa rodada já foi encerrada."),
                )
                .await?;
                continue;
            }

            match state.reveal(index) {
                Some(RevealOutcome::AlreadyOpened) => {
                    send_ephemeral_response(
                        &ctx,
                        &interaction,
                        pretty_message(icon::ERROR, "Esse botão já foi aberto."),
                    )
                    .await?;
                }
                Some(RevealOutcome::Diamond) => {
                    if state.can_cash_out() {
                        state.set_status(pretty_message(
                            icon::CHECK,
                            format!(
                                "{} encontrou {} diamantes. Resgate disponível!",
                                player.mention(),
                                state.revealed_safe
                            ),
                        ));
                    } else {
                        let needed = state.remaining_for_cashout();
                        state.set_status(pretty_message(
                            icon::BELL,
                            format!(
                                "{} diamante(s). Precisa de mais {} para liberar o resgate.",
                                state.revealed_safe, needed
                            ),
                        ));
                    }

                    if state.force_cashout_reached() {
                        let payout = state.projected_payout();
                        finalize_cashout(&ctx, &mut state, &mut user, payout, &player, true)
                            .await?;
                        let (embed, components) = render_game(&state, &player);
                        update_component_message(&ctx, &interaction, embed, components).await?;
                        break;
                    } else {
                        let (embed, components) = render_game(&state, &player);
                        update_component_message(&ctx, &interaction, embed, components).await?;
                    }
                }
                Some(RevealOutcome::Bomb) => {
                    state.busted = true;
                    state.reveal_all();
                    state.set_status(pretty_message(
                        icon::ERROR,
                        format!(
                            "{} pisou em uma bomba e perdeu **{}** moedas.",
                            player.mention(),
                            format_currency(state.wager)
                        ),
                    ));
                    let (embed, components) = render_game(&state, &player);
                    update_component_message(&ctx, &interaction, embed, components).await?;
                    break;
                }
                None => continue,
            }
        } else if interaction.data.custom_id == cashout_id {
            if !state.can_cash_out() {
                let needed = state.remaining_for_cashout();
                send_ephemeral_response(
                    &ctx,
                    &interaction,
                    pretty_message(
                        icon::BELL,
                        format!(
                            "Você precisa de mais {} diamantes para liberar o resgate.",
                            needed
                        ),
                    ),
                )
                .await?;
                continue;
            }

            let payout = state.projected_payout();
            finalize_cashout(&ctx, &mut state, &mut user, payout, &player, false).await?;
            let (embed, components) = render_game(&state, &player);
            update_component_message(&ctx, &interaction, embed, components).await?;
            break;
        } else if interaction.data.custom_id == giveup_id {
            if state.is_finished() {
                send_ephemeral_response(
                    &ctx,
                    &interaction,
                    pretty_message(icon::ERROR, "Seu jogo já acabou."),
                )
                .await?;
                continue;
            }

            state.gave_up = true;
            state.reveal_all();
            let transaction_id = wager_transaction_id.take();
            refund_wager(&ctx, &mut user, state.wager, transaction_id).await?;
            state.refunded = true;
            state.set_status(pretty_message(
                icon::CHECK,
                format!(
                    "{} cancelou a rodada e recuperou **{}** moedas.",
                    player.mention(),
                    format_currency(state.wager)
                ),
            ));
            let (embed, components) = render_game(&state, &player);
            update_component_message(&ctx, &interaction, embed, components).await?;
            break;
        } else {
            continue;
        }
    }

    if !state.is_finished() {
        state.gave_up = true;
        state.reveal_all();
        state.set_status(pretty_message(
            icon::ERROR,
            "Tempo esgotado. A rodada foi encerrada sem resgate.",
        ));
        let (embed, components) = render_game(&state, &player);
        channel_id
            .edit_message(
                ctx.serenity_context(),
                message_id,
                EditMessage::new().embed(embed).components(components),
            )
            .await?;
    }

    Ok(())
}

async fn finalize_cashout(
    ctx: &Context<'_>,
    state: &mut MinesGameState,
    user: &mut UserModel,
    payout: i64,
    player: &serenity::User,
    forced: bool,
) -> Result<(), Error> {
    if payout <= 0 {
        return Ok(());
    }

    {
        let db = ctx.data().database.lock().await;
        user.dollars += payout;
        *user = database::update_user_balance(&db, user.id, user.dollars, user.diamonds).await?;
        let context = if forced {
            "Resgate automático"
        } else {
            "Resgate manual"
        };
        let kind = if forced {
            "mines_autocashout"
        } else {
            "mines_cashout"
        };
        database::insert_currency_transaction(
            &db,
            user.id,
            payout,
            user.dollars,
            "dollars",
            kind,
            Some(context.to_string()),
        )
        .await?;
    }

    state.cashed_out_amount = Some(payout);
    state.reveal_all();
    let message = if forced {
        pretty_message(
            icon::GIFT,
            format!(
                "Resgate automático! {} garantiu **{}** moedas.",
                player.mention(),
                format_currency(payout)
            ),
        )
    } else {
        pretty_message(
            icon::GIFT,
            format!(
                "{} resgatou **{}** moedas antes de atingir uma bomba.",
                player.mention(),
                format_currency(payout)
            ),
        )
    };
    state.set_status(message);
    Ok(())
}

async fn refund_wager(
    ctx: &Context<'_>,
    user: &mut UserModel,
    amount: i64,
    transaction_id: Option<i32>,
) -> Result<(), Error> {
    let db = ctx.data().database.lock().await;
    user.dollars += amount;
    *user = database::update_user_balance(&db, user.id, user.dollars, user.diamonds).await?;
    if let Some(id) = transaction_id {
        database::delete_currency_transaction(&db, id).await?;
    }
    Ok(())
}

fn render_game(
    state: &MinesGameState,
    player: &serenity::User,
) -> (serenity::CreateEmbed, Vec<CreateActionRow>) {
    let mut embed = serenity::CreateEmbed::new()
        .title(format!("💣 Mines de {}", player.name))
        .colour(colors::MOON)
        .field(
            format!("{} Aposta", icon::DOLLAR),
            format!("**{}** moedas", format_currency(state.wager)),
            true,
        )
        .field(
            format!("{} Diamante(s)", icon::DIAMOND),
            format!(
                "**{} / {}** encontrados",
                state.revealed_safe, FORCE_CASHOUT_AFTER
            ),
            true,
        )
        .field(
            format!("{} Multiplicador", icon::HASTAG),
            format!("**x{:.2}**", state.current_multiplier()),
            true,
        );

    if let Some(amount) = state.cashed_out_amount {
        embed = embed.field(
            format!("{} Resultado", icon::GIFT),
            format!("Resgate final de **{}** moedas", format_currency(amount)),
            false,
        );
    } else if state.busted {
        embed = embed.field(
            format!("{} Resultado", icon::ERROR),
            "Uma bomba explodiu e encerrou a rodada.",
            false,
        );
    } else if state.gave_up {
        let message = if state.refunded {
            "Rodada cancelada manualmente. Aposta devolvida."
        } else {
            "Rodada cancelada automaticamente. Sem reembolso."
        };
        embed = embed.field(format!("{} Resultado", icon::ERROR), message, false);
    } else if state.can_cash_out() {
        embed = embed.field(
            format!("{} Resgate", icon::GIFT),
            format!(
                "Disponível agora: **{}** moedas",
                format_currency(state.projected_payout())
            ),
            false,
        );
    } else if state.revealed_safe == 0 {
        embed = embed.field(
            format!("{} Resgate", icon::BELL),
            format!(
                "Abra {} diamantes seguidos antes de pensar em resgatar.",
                CASHOUT_STEP
            ),
            false,
        );
    } else {
        embed = embed.field(
            format!("{} Resgate", icon::TIMER),
            format!(
                "Faltam {} diamantes para liberar o resgate.",
                state.remaining_for_cashout()
            ),
            false,
        );
    }

    if let Some(status) = &state.status_text {
        embed = embed.description(status.clone());
    }

    let footer_text = if state.is_finished() {
        format!("Rodada encerrada • {} bombas escondidas", TOTAL_BOMBS)
    } else {
        let remaining_auto = FORCE_CASHOUT_AFTER.saturating_sub(state.revealed_safe);
        format!(
            "Auto-resgate em {} diamantes • {} bombas no tabuleiro",
            remaining_auto, TOTAL_BOMBS
        )
    };
    embed = embed.footer(CreateEmbedFooter::new(footer_text));

    let components = build_components(state);
    (embed, components)
}

fn build_components(state: &MinesGameState) -> Vec<CreateActionRow> {
    let finished = state.is_finished();
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
            if tile.revealed {
                button = if tile.is_bomb {
                    button
                        .label("💣")
                        .style(serenity::ButtonStyle::Danger)
                        .disabled(true)
                } else {
                    button
                        .label("💎")
                        .style(serenity::ButtonStyle::Success)
                        .disabled(true)
                };
            } else if finished {
                let label = if tile.is_bomb { "💣" } else { "💎" };
                let style = if tile.is_bomb {
                    serenity::ButtonStyle::Danger
                } else {
                    serenity::ButtonStyle::Secondary
                };
                button = button.label(label).style(style).disabled(true);
            } else {
                button = button
                    .label(HIDDEN_LABEL)
                    .style(serenity::ButtonStyle::Secondary);
            }

            buttons.push(button);
        }
        rows.push(CreateActionRow::Buttons(buttons));
    }

    let mut controls = Vec::new();
    controls.push(
        CreateButton::new(format!("{}cashout", state.custom_id_prefix))
            .label("Resgatar")
            .emoji(icon::GIFT.as_reaction())
            .style(serenity::ButtonStyle::Success)
            .disabled(!state.can_cash_out()),
    );
    controls.push(
        CreateButton::new(format!("{}giveup", state.custom_id_prefix))
            .label("Desistir")
            .emoji(icon::ERROR.as_reaction())
            .style(serenity::ButtonStyle::Danger)
            .disabled(finished),
    );
    rows.push(CreateActionRow::Buttons(controls));

    rows
}

fn parse_tile_index(custom_id: &str, prefix: &str) -> Option<usize> {
    custom_id.strip_prefix(prefix)?.parse().ok()
}

async fn autocomplete_wager(ctx: Context<'_>, partial: &str) -> CreateAutocompleteResponse {
    let cleaned_input: String = partial.chars().filter(|c| c.is_ascii_digit()).collect();
    let discord_id = ctx.author().id.get() as i64;
    let balance = {
        let db = ctx.data().database.lock().await;
        database::get_or_create_user(&db, discord_id)
            .await
            .map(|user| user.dollars)
            .unwrap_or(0)
    };

    let suggestions = build_wager_suggestions(balance);
    let mut choices = Vec::new();
    for value in suggestions {
        let value_text = value.to_string();
        if !cleaned_input.is_empty() && !value_text.starts_with(&cleaned_input) {
            continue;
        }
        let label = if balance < value {
            format!(
                "{} moedas (saldo: {})",
                format_currency(value),
                format_currency(balance.max(0))
            )
        } else {
            format!("{} moedas", format_currency(value))
        };
        choices.push(serenity::AutocompleteChoice::new(label, value));
        if choices.len() >= 25 {
            break;
        }
    }

    if choices.is_empty() {
        let fallback = cleaned_input
            .parse::<i64>()
            .unwrap_or(MIN_WAGER)
            .clamp(MIN_WAGER, MAX_WAGER);
        let label = if balance < fallback {
            format!(
                "{} moedas (saldo: {})",
                format_currency(fallback),
                format_currency(balance.max(0))
            )
        } else {
            format!("{} moedas", format_currency(fallback))
        };
        choices.push(serenity::AutocompleteChoice::new(label, fallback));
    }

    CreateAutocompleteResponse::new().set_choices(choices)
}

fn build_wager_suggestions(balance: i64) -> Vec<i64> {
    let cap = balance.min(MAX_WAGER);
    if cap < MIN_WAGER {
        return vec![MIN_WAGER];
    }

    let mut values = vec![MIN_WAGER, cap];
    for pct in [0.2, 0.35, 0.5, 0.75, 1.0] {
        let mut value = ((cap as f64) * pct).round() as i64;
        if value < MIN_WAGER {
            value = MIN_WAGER;
        }
        if value > cap {
            value = cap;
        }
        values.push(value);
    }

    values.sort_unstable();
    values.dedup();
    values
}
