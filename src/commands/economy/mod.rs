use crate::{
    Context, Error,
    constants::{colors, icon},
    database,
    functions::{
        pretty_message::pretty_message,
        time::{self, ResetTime},
    },
    models::{RewardStateModel, UserModel},
};
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use rand::Rng;
use serenity::builder::CreateInteractionResponseMessage;
use serenity::collector::ComponentInteractionCollector;
use serenity::{CreateActionRow, CreateButton};
use std::time::Duration;

mod reward_kind;
use reward_kind::RewardKind;

const RESET_CONFIG: ResetTime = ResetTime {
    hour: 21,
    minute: 0,
    timezone_offset_secs: -3 * 60 * 60,
};

/// Gerencie sua economia e recompensas.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "economia",
    category = "Economia",
    interaction_context = "Guild",
    subcommands("rewards", "balance")
)]
pub async fn economy(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Veja seu saldo, diamantes e cooldown das recompensas.
#[poise::command(slash_command, prefix_command, rename = "saldo", category = "Economia")]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let discord_id = ctx.author().id.get() as i64;

    let db = ctx.data().database.lock().await;
    let user = database::get_or_create_user(&db, discord_id).await?;
    let reward_states = database::get_all_reward_states(&db, user.id).await?;

    let now = Utc::now();

    let mut description_lines = vec![
        pretty_message(
            icon::DOLLAR,
            format!("Saldo: **{}** moedas", user.dollars.max(0)),
        ),
        pretty_message(
            icon::DIAMOND,
            format!("Diamantes: **{}**", user.diamonds.max(0)),
        ),
    ];

    description_lines.push(String::new());
    description_lines.push("**Recompensas disponíveis:**".to_string());

    for kind in RewardKind::ALL {
        let state = reward_states
            .iter()
            .find(|s| s.reward_type == kind.db_name());
        let available = is_reward_available(state, now);

        let status = if available {
            pretty_message(icon::CHECK, "Disponível agora")
        } else if let Some(state) = state {
            if let Some(next_time) = state.next_reset_datetime() {
                pretty_message(icon::TIMER, time::describe_relative(next_time))
            } else {
                pretty_message(icon::CHECK, "Disponível agora")
            }
        } else {
            pretty_message(icon::CHECK, "Disponível agora")
        };

        description_lines.push(format!("**{}:** {}", kind.field_title(), status));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!(
            "{} Carteira de {}",
            icon::DOLLAR,
            ctx.author().name
        ))
        .colour(colors::MINT)
        .description(description_lines.join("\n"));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}

/// Resgate suas recompensas!
#[poise::command(
    slash_command,
    prefix_command,
    rename = "recompensas",
    category = "Economia"
)]
pub async fn rewards(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let discord_id = ctx.author().id.get() as i64;

    let (mut user, mut reward_states) = {
        let db = ctx.data().database.lock().await;
        let user = database::get_or_create_user(&db, discord_id).await?;
        let states = database::get_all_reward_states(&db, user.id).await?;
        (user, states)
    };

    let now = Utc::now();
    let (embed, components) = build_rewards_message(&user, &reward_states, now, None);

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .ephemeral(true)
                .embed(embed)
                .components(components.clone()),
        )
        .await?;

    let message_id = reply.message().await?.id;

    while let Some(interaction) = ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .message_id(message_id)
        .timeout(Duration::from_secs(90))
        .await
    {
        let Some(kind) = RewardKind::from_custom_id(&interaction.data.custom_id) else {
            continue;
        };

        let now = Utc::now();
        let state = reward_states
            .iter()
            .find(|s| s.reward_type == kind.db_name());
        let available = is_reward_available(state, now);
        let response_text;

        if available {
            let (money, diamonds) = roll_reward(kind);
            user.dollars += money;
            if let Some(amount) = diamonds {
                user.diamonds += amount;
            }

            let total_claims = state.map(|s| s.total_claims + 1).unwrap_or(1);
            let next_reset = time::next_reset_from(now, kind.reset_period(), &RESET_CONFIG);

            {
                let db = ctx.data().database.lock().await;
                user = database::update_user_balance(&db, user.id, user.dollars, user.diamonds)
                    .await?;
                let new_state = database::upsert_reward_state(
                    &db,
                    user.id,
                    kind.db_name(),
                    Some(now),
                    Some(next_reset),
                    total_claims,
                )
                .await?;

                if let Some(existing) = reward_states
                    .iter_mut()
                    .find(|s| s.reward_type == kind.db_name())
                {
                    *existing = new_state;
                } else {
                    reward_states.push(new_state);
                }
            }

            response_text = format_claim_message(money, diamonds);
        } else {
            response_text = if let Some(state) = state {
                if let Some(next_time) = state.next_reset_datetime() {
                    format_cooldown_message(next_time)
                } else {
                    pretty_message(icon::ERROR, "Erro ao verificar cooldown")
                }
            } else {
                pretty_message(icon::ERROR, "Estado de recompensa não encontrado")
            };
        }

        let (embed, components) =
            build_rewards_message(&user, &reward_states, now, Some(&response_text));
        let response = CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components);

        interaction
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(response),
            )
            .await?;
    }

    let (embed, _) = build_rewards_message(&user, &reward_states, Utc::now(), None);
    reply
        .edit(
            ctx,
            poise::CreateReply::default()
                .embed(embed)
                .components(Vec::new()),
        )
        .await?;

    Ok(())
}

fn build_rewards_message(
    user: &UserModel,
    reward_states: &[RewardStateModel],
    now: DateTime<Utc>,
    status: Option<&str>,
) -> (serenity::CreateEmbed, Vec<CreateActionRow>) {
    let mut description_lines = vec![
        pretty_message(icon::DOLLAR, format!("Saldo: {}", user.dollars.max(0))),
        pretty_message(
            icon::DIAMOND,
            format!("Diamantes: {}", user.diamonds.max(0)),
        ),
    ];

    if let Some(status) = status {
        description_lines.push(String::new());
        description_lines.push(status.to_string());
    }

    let mut embed: serenity::CreateEmbed = serenity::CreateEmbed::new()
        .colour(colors::MOON)
        .description(description_lines.join("\n"));

    for kind in RewardKind::ALL {
        let state = reward_states
            .iter()
            .find(|s| s.reward_type == kind.db_name());
        embed = embed.field(kind.field_title(), reward_field(state, kind, now), true);
    }

    let buttons = RewardKind::ALL
        .iter()
        .map(|kind| {
            let state = reward_states
                .iter()
                .find(|s| s.reward_type == kind.db_name());
            CreateButton::new(kind.custom_id())
                .label(kind.button_label())
                .emoji(kind.button_emoji().as_reaction())
                .style(serenity::ButtonStyle::Secondary)
                .disabled(!is_reward_available(state, now))
        })
        .collect();
    let components = vec![CreateActionRow::Buttons(buttons)];

    (embed, components)
}

fn reward_field(state: Option<&RewardStateModel>, kind: RewardKind, now: DateTime<Utc>) -> String {
    let available = is_reward_available(state, now);
    let (min_cash, max_cash) = kind.money_range();
    let payout_line = pretty_message(icon::DOLLAR, format!("{} - {} moedas", min_cash, max_cash));

    if available {
        let upcoming_reset = time::next_reset_from(now, kind.reset_period(), &RESET_CONFIG);
        let reset_line = pretty_message(icon::ALARM, time::describe_absolute(upcoming_reset));
        let ready_line = pretty_message(icon::CHECK, "Pronto para coletar");
        format!("{ready_line}\n{payout_line}\n{reset_line}")
    } else if let Some(state) = state {
        if let Some(next_time) = state.next_reset_datetime() {
            let cooldown_line = pretty_message(icon::TIMER, time::describe_relative(next_time));
            let reset_line = pretty_message(icon::ALARM, time::describe_absolute(next_time));
            format!("{payout_line}\n{reset_line}\n{cooldown_line}")
        } else {
            format!(
                "{payout_line}\n{}",
                pretty_message(icon::ERROR, "Erro no cooldown")
            )
        }
    } else {
        let upcoming_reset = time::next_reset_from(now, kind.reset_period(), &RESET_CONFIG);
        let reset_line = pretty_message(icon::ALARM, time::describe_absolute(upcoming_reset));
        let ready_line = pretty_message(icon::CHECK, "Pronto para coletar");
        format!("{ready_line}\n{payout_line}\n{reset_line}")
    }
}

fn is_reward_available(state: Option<&RewardStateModel>, now: DateTime<Utc>) -> bool {
    match state {
        None => true,
        Some(state) => match state.next_reset_datetime() {
            None => true,
            Some(ready_at) => now >= ready_at,
        },
    }
}

fn roll_reward(kind: RewardKind) -> (i64, Option<i64>) {
    let (min_cash, max_cash) = kind.money_range();
    let mut rng = rand::rng();
    let money = rng.random_range(min_cash..=max_cash);
    let diamonds = if rng.random_bool(0.20) {
        Some(rng.random_range(1..=5))
    } else {
        None
    };
    (money, diamonds)
}

fn format_claim_message(money: i64, diamonds: Option<i64>) -> String {
    let money_line = pretty_message(icon::DOLLAR, format!("+{money} moedas"));
    let diamond_line = diamonds
        .map(|amount| pretty_message(icon::DIAMOND, format!("+{amount} diamantes")))
        .unwrap_or_else(|| pretty_message(icon::DIAMOND, "Sem diamantes desta vez"));
    format!(
        "{}\n{money_line}\n{diamond_line}",
        pretty_message(icon::GIFT, "Recompensa coletada")
    )
}

fn format_cooldown_message(next_time: DateTime<Utc>) -> String {
    let absolute = pretty_message(icon::ALARM, time::describe_absolute(next_time));
    let relative = pretty_message(icon::TIMER, time::describe_relative(next_time));
    format!(
        "{}\n{}\n{}",
        pretty_message(icon::ERROR, "Tempo de espera ativo"),
        absolute,
        relative
    )
}
