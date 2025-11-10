use crate::{
    Context, Error,
    constants::{colors, icon},
    functions::format::pretty_message,
};
use lobby::{LobbyMessageHandle, LobbyOutcome, RaceLobby};
use poise::serenity_prelude::{self as serenity, Mentionable};
use progress_message::RaceProgressMessage;
use serenity::builder::EditMessage;
use state::{RaceContestant, RaceResultEntry, RaceState};
use std::{
    collections::HashSet,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::{sync::Mutex, time::sleep};

mod lobby;
mod progress_message;
mod state;

const MIN_PARTICIPANTS: usize = 2;
const TRACK_LENGTH: usize = 40;
const LOBBY_TIMEOUT: Duration = Duration::from_secs(120);
const ROUND_DELAY: Duration = Duration::from_millis(1800);
const MIN_STEP_PER_ROUND: usize = 1;
const MAX_STEP_PER_ROUND: usize = 3;
const MAX_ANIMALS_PER_RACE: usize = 8;

static ACTIVE_CHANNELS: OnceLock<Arc<Mutex<HashSet<serenity::ChannelId>>>> = OnceLock::new();

/// Crie uma corrida de animais.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "corrida",
    aliases("race", "animais"),
    category = "Jogos",
    interaction_context = "Guild",
    on_error = "crate::commands::util::command_error_handler"
)]
pub async fn race(ctx: Context<'_>) -> Result<(), Error> {
    let Some(_) = claim_channel(ctx.channel_id()).await else {
        ctx.send(
            poise::CreateReply::default()
                .content(pretty_message(
                    icon::ERROR,
                    "Já existe uma corrida acontecendo neste canal. Aguarde terminar.",
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let available_animals = state::random_animals(MAX_ANIMALS_PER_RACE);
    let mut lobby = RaceLobby::new(ctx.author().clone(), MIN_PARTICIPANTS, available_animals);
    let (embed, components) = lobby.render_view();
    let reply = ctx
        .send(
            poise::CreateReply::default()
                .embed(embed)
                .components(components),
        )
        .await?;
    let message = reply.message().await?;
    let message_handle = LobbyMessageHandle {
        channel_id: message.channel_id,
        message_id: message.id,
    };

    match lobby
        .wait_for_start(&ctx, &message_handle, LOBBY_TIMEOUT)
        .await?
    {
        LobbyOutcome::Started {
            participants,
            message,
        } => run_race(ctx, participants, message).await?,
        LobbyOutcome::Cancelled | LobbyOutcome::Timeout => {}
    }

    Ok(())
}

async fn run_race(
    ctx: Context<'_>,
    participants: Vec<RaceContestant>,
    lobby_message: LobbyMessageHandle,
) -> Result<(), Error> {
    let mut state = RaceState::new(participants, TRACK_LENGTH);
    let mut progress_message =
        RaceProgressMessage::create(&ctx, render_track_message(&state, false)).await?;

    loop {
        sleep(ROUND_DELAY).await;
        let finished = {
            let mut rng = rand::rng();
            state.advance_round(&mut rng, MIN_STEP_PER_ROUND, MAX_STEP_PER_ROUND)
        };
        let content = render_track_message(&state, finished);
        progress_message.refresh(&ctx, content).await?;
        if finished {
            break;
        }
    }

    let winners = state.winners();
    let rankings = state.rankings();

    announce_results(&ctx, &lobby_message, &winners, &rankings).await?;

    Ok(())
}

fn render_track_message(state: &RaceState, finished: bool) -> String {
    if finished {
        let mut content = state.render_track();
        content.push_str("\n🏁 Corrida finalizada! Parabéns!");
        content
    } else {
        let content = state.render_simple_track();
        content
    }
}

async fn announce_results(
    ctx: &Context<'_>,
    handle: &LobbyMessageHandle,
    winners: &[RaceContestant],
    rankings: &[RaceResultEntry],
) -> Result<(), Error> {
    let embed = build_results_embed(winners, rankings);
    handle
        .channel_id
        .edit_message(
            ctx.serenity_context(),
            handle.message_id,
            EditMessage::new().embed(embed).components(Vec::new()),
        )
        .await?;
    Ok(())
}

fn build_results_embed(
    winners: &[RaceContestant],
    rankings: &[RaceResultEntry],
) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::new()
        .title("🏁 Corrida finalizada")
        .colour(colors::MINT)
        .description(pretty_message(
            icon::CHECK,
            "Obrigada por correr comigo! Aqui está o resultado.",
        ));

    let winners_value = if winners.is_empty() {
        "Ninguém alcançou a linha de chegada.".to_string()
    } else {
        winners
            .iter()
            .map(|winner| format!("{} {}", winner.animal.emoji, winner.user.mention()))
            .collect::<Vec<_>>()
            .join("\n")
    };

    embed = embed.field("Vencedor", winners_value, false);

    if !rankings.is_empty() {
        let standings = rankings
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                format!(
                    "{}º - {} {} | {} casas",
                    idx + 1,
                    entry.animal.emoji,
                    entry.user.mention(),
                    entry.position
                )
            })
            .take(5)
            .collect::<Vec<_>>()
            .join("\n");

        embed = embed.field("Classificação", standings, false);
    }

    embed
}

async fn claim_channel(channel_id: serenity::ChannelId) -> Option<RaceChannelGuard> {
    let active = ACTIVE_CHANNELS
        .get_or_init(|| Arc::new(Mutex::new(HashSet::new())))
        .clone();

    let mut lock = active.lock().await;
    if lock.contains(&channel_id) {
        return None;
    }
    lock.insert(channel_id);
    drop(lock);

    Some(RaceChannelGuard { channel_id, active })
}

struct RaceChannelGuard {
    channel_id: serenity::ChannelId,
    active: Arc<Mutex<HashSet<serenity::ChannelId>>>,
}

impl Drop for RaceChannelGuard {
    fn drop(&mut self) {
        let channel_id = self.channel_id;
        let active = Arc::clone(&self.active);
        tokio::spawn(async move {
            let mut lock = active.lock().await;
            lock.remove(&channel_id);
        });
    }
}
