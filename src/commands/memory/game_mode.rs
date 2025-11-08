use super::player_state::PlayerState;
use crate::constants::icon;
use crate::functions::ui::pretty_message::pretty_message;
use poise::serenity_prelude as serenity;
use serenity::Mentionable;

pub enum Mode {
    Solo {
        player: PlayerState,
    },
    Versus {
        players: [PlayerState; 2],
        current_turn: usize,
    },
}

impl Mode {
    pub fn is_allowed(&self, user_id: serenity::UserId) -> bool {
        match self {
            Mode::Solo { player } => player.user.id == user_id,
            Mode::Versus { players, .. } => players.iter().any(|player| player.user.id == user_id),
        }
    }

    pub fn is_current_player(&self, user_id: serenity::UserId) -> bool {
        match self {
            Mode::Solo { player } => player.user.id == user_id,
            Mode::Versus {
                players,
                current_turn,
            } => players[*current_turn].user.id == user_id,
        }
    }

    pub fn is_strict_single_player(&self) -> bool {
        matches!(self, Mode::Solo { .. })
    }

    pub fn active_player_id(&self) -> serenity::UserId {
        match self {
            Mode::Solo { player } => player.user.id,
            Mode::Versus {
                players,
                current_turn,
            } => players[*current_turn].user.id,
        }
    }

    pub fn register_match(&mut self, user_id: serenity::UserId) {
        match self {
            Mode::Solo { player } => {
                if player.user.id == user_id {
                    player.score += 1;
                }
            }
            Mode::Versus { players, .. } => {
                if let Some(player) = players.iter_mut().find(|p| p.user.id == user_id) {
                    player.score += 1;
                }
            }
        }
    }

    pub fn advance_turn(&mut self) {
        if let Mode::Versus { current_turn, .. } = self {
            *current_turn = (*current_turn + 1) % 2;
        }
    }

    pub fn turn_message(&self) -> String {
        match self {
            Mode::Solo { player } => {
                format!("{} continua na busca pelos pares!", player.user.mention())
            }
            Mode::Versus {
                players,
                current_turn,
            } => format!(
                "Agora é a vez de {}.",
                players[*current_turn].user.mention()
            ),
        }
    }

    pub fn scoreboard_line(&self) -> Option<String> {
        match self {
            Mode::Solo { .. } => None,
            Mode::Versus { players, .. } => Some(pretty_message(
                icon::HASTAG,
                format!(
                    "{}: **{}** • {}: **{}**",
                    players[0].user.name, players[0].score, players[1].user.name, players[1].score
                ),
            )),
        }
    }

    pub fn finish_message(&self, attempts: u32) -> String {
        match self {
            Mode::Solo { player } => pretty_message(
                icon::CHECK,
                format!(
                    "{} completou todos os pares em **{}** tentativas!",
                    player.user.mention(),
                    attempts
                ),
            ),
            Mode::Versus { players, .. } => {
                let left = &players[0];
                let right = &players[1];
                if left.score == right.score {
                    pretty_message(
                        icon::HASTAG,
                        format!(
                            "Empate! {} e {} terminaram com **{}** pares.",
                            left.user.mention(),
                            right.user.mention(),
                            left.score
                        ),
                    )
                } else {
                    let (winner, loser) = if left.score > right.score {
                        (left, right)
                    } else {
                        (right, left)
                    };
                    pretty_message(
                        icon::GIFT,
                        format!(
                            "{} venceu com **{}** pares contra {} ({})!",
                            winner.user.mention(),
                            winner.score,
                            loser.user.mention(),
                            loser.score
                        ),
                    )
                }
            }
        }
    }
}
