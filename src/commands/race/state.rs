use poise::serenity_prelude::{self as serenity, Mentionable};
use rand::{Rng, seq::SliceRandom};
use std::cmp::Ordering;

const ANIMALS: [RaceAnimal; 12] = [
    RaceAnimal::new("cat", "🐈"),
    RaceAnimal::new("dog", "🐕"),
    RaceAnimal::new("fox", "🦊"),
    RaceAnimal::new("turtle", "🐢"),
    RaceAnimal::new("fish", "🐟"),
    RaceAnimal::new("dragon", "🐉"),
    RaceAnimal::new("rabbit", "🐇"),
    RaceAnimal::new("panda", "🐼"),
    RaceAnimal::new("penguin", "🐧"),
    RaceAnimal::new("unicorn", "🦄"),
    RaceAnimal::new("horse", "🐎"),
    RaceAnimal::new("frog", "🐸"),
];

#[derive(Clone, Copy, Debug)]
pub struct RaceAnimal {
    pub id: &'static str,
    pub emoji: &'static str,
}

impl RaceAnimal {
    pub const fn new(id: &'static str, emoji: &'static str) -> Self {
        Self { id, emoji }
    }

    pub fn all() -> &'static [RaceAnimal] {
        &ANIMALS
    }
}

pub fn random_animals(count: usize) -> Vec<RaceAnimal> {
    let mut animals = RaceAnimal::all().to_vec();
    let mut rng = rand::rng();
    animals.shuffle(&mut rng);
    animals.truncate(count.min(animals.len()));
    animals
}

#[derive(Clone)]
pub struct RaceContestant {
    pub user: serenity::User,
    pub animal: RaceAnimal,
}

#[derive(Clone)]
pub struct RaceResultEntry {
    pub user: serenity::User,
    pub animal: RaceAnimal,
    pub position: usize,
    pub finished_round: Option<usize>,
}

#[derive(Clone)]
pub struct RacerState {
    pub contestant: RaceContestant,
    pub position: usize,
    pub speed_bias: f32,
    pub finished_round: Option<usize>,
}

pub struct RaceState {
    racers: Vec<RacerState>,
    track_length: usize,
    rounds_elapsed: usize,
}

impl RaceState {
    pub fn new(participants: Vec<RaceContestant>, track_length: usize) -> Self {
        let mut rng = rand::rng();
        let racers = participants
            .into_iter()
            .map(|contestant| RacerState {
                contestant,
                position: 0,
                speed_bias: random_speed_bias(&mut rng),
                finished_round: None,
            })
            .collect();

        Self {
            racers,
            track_length,
            rounds_elapsed: 0,
        }
    }

    pub fn advance_round<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        min_step: usize,
        max_step: usize,
    ) -> bool {
        self.rounds_elapsed += 1;
        let current_round = self.rounds_elapsed;
        let mut someone_finished = false;
        for racer in &mut self.racers {
            let base_step = if min_step == max_step {
                min_step
            } else {
                rng.random_range(min_step..=max_step)
            };
            let mut step_value = base_step as f32 * racer.speed_bias;

            let swing = (rng.random::<f32>() * 2.0 - 1.0) * VARIANCE_SWING_FACTOR;
            step_value += swing;

            if rng.random::<f32>() < SURGE_CHANCE {
                step_value += rng.random_range(SURGE_BONUS_MIN..=SURGE_BONUS_MAX) as f32;
            }

            if rng.random::<f32>() < SLIP_CHANCE {
                step_value -= rng.random_range(SLIP_PENALTY_MIN..=SLIP_PENALTY_MAX) as f32;
            }

            let step = step_value
                .round()
                .max(MINIMUM_STEP as f32)
                .min(MAX_STEP_CAP as f32) as usize;

            racer.position = (racer.position + step).min(self.track_length);

            if racer.position >= self.track_length {
                someone_finished = true;
                if racer.finished_round.is_none() {
                    racer.finished_round = Some(current_round);
                }
            }
        }

        someone_finished
    }

    pub fn winners(&self) -> Vec<RaceContestant> {
        let Some(best_round) = self
            .racers
            .iter()
            .filter_map(|racer| racer.finished_round)
            .min()
        else {
            return Vec::new();
        };

        self.racers
            .iter()
            .filter(|racer| racer.finished_round == Some(best_round))
            .map(|racer| racer.contestant.clone())
            .collect()
    }

    pub fn rankings(&self) -> Vec<RaceResultEntry> {
        let mut results: Vec<_> = self
            .racers
            .iter()
            .map(|racer| RaceResultEntry {
                user: racer.contestant.user.clone(),
                animal: racer.contestant.animal,
                position: racer.position,
                finished_round: racer.finished_round,
            })
            .collect();

        results.sort_by(|a, b| {
            b.position
                .cmp(&a.position)
                .then_with(|| match (a.finished_round, b.finished_round) {
                    (Some(ar), Some(br)) => ar.cmp(&br),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    _ => Ordering::Equal,
                })
        });
        results
    }

    pub fn render_track(&self) -> String {
        let mut content = String::from("🏁 Corrida dos Animais\n");
        content.push_str("Acompanhe o avanço dos participantes:\n\n");

        for racer in &self.racers {
            let lane = build_lane(racer.position, self.track_length, racer.contestant.animal);
            content.push_str(&format!(
                "{} {} | {}\n",
                racer.contestant.animal.emoji,
                racer.contestant.user.mention(),
                lane
            ));
        }

        content
    }

    pub fn render_simple_track(&self) -> String {
        self.racers
            .iter()
            .map(|racer| {
                build_simple_lane(racer.position, self.track_length, racer.contestant.animal)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn build_lane(position: usize, track_length: usize, animal: RaceAnimal) -> String {
    let progress = position.min(track_length);
    let remaining = track_length.saturating_sub(progress);

    let mut lane = "-".repeat(progress);
    lane.push_str(animal.emoji);

    if remaining > 0 {
        lane.push_str(&".".repeat(remaining));
    }

    lane.push('🏁');
    if progress >= track_length {
        lane.push_str(" (chegou!)");
    }

    lane
}

fn build_simple_lane(position: usize, track_length: usize, animal: RaceAnimal) -> String {
    let progress = position.min(track_length);
    let mut lane = "-".repeat(progress);
    lane.push_str(animal.emoji);
    lane
}

const MIN_SPEED_BIAS: f32 = 0.65;
const MAX_SPEED_BIAS: f32 = 1.35;
const VARIANCE_SWING_FACTOR: f32 = 1.25;
const SURGE_CHANCE: f32 = 0.18;
const SURGE_BONUS_MIN: usize = 2;
const SURGE_BONUS_MAX: usize = 4;
const SLIP_CHANCE: f32 = 0.15;
const SLIP_PENALTY_MIN: usize = 1;
const SLIP_PENALTY_MAX: usize = 3;
const MAX_STEP_CAP: usize = 9;
const MINIMUM_STEP: usize = 1;

fn random_speed_bias<R: Rng + ?Sized>(rng: &mut R) -> f32 {
    let span = MAX_SPEED_BIAS - MIN_SPEED_BIAS;
    MIN_SPEED_BIAS + rng.random::<f32>() * span
}
