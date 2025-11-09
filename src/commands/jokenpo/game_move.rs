use rand::Rng;
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
}

impl GameMove {
    pub const ALL: [GameMove; 3] = [GameMove::Rock, GameMove::Paper, GameMove::Scissors];

    pub fn custom_id(self) -> &'static str {
        match self {
            GameMove::Rock => "jkp_rock",
            GameMove::Paper => "jkp_paper",
            GameMove::Scissors => "jkp_scissors",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GameMove::Rock => "🪨 Pedra",
            GameMove::Paper => "📄 Papel",
            GameMove::Scissors => "✂️ Tesoura",
        }
    }

    pub fn beats(self, other: GameMove) -> bool {
        matches!(
            (self, other),
            (GameMove::Rock, GameMove::Scissors)
                | (GameMove::Paper, GameMove::Rock)
                | (GameMove::Scissors, GameMove::Paper)
        )
    }

    pub fn from_custom_id(id: &str) -> Option<Self> {
        GameMove::ALL.into_iter().find(|mv| mv.custom_id() == id)
    }

    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let idx = rng.random_range(0..Self::ALL.len());
        Self::ALL[idx]
    }
}

impl fmt::Display for GameMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameMove::Rock => write!(f, "🪨 Pedra"),
            GameMove::Paper => write!(f, "📄 Papel"),
            GameMove::Scissors => write!(f, "✂️ Tesoura"),
        }
    }
}
