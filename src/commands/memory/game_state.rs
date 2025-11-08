use super::game_mode::Mode;
use super::tile::Tile;
use rand::{Rng, seq::SliceRandom};

const BOARD_PAIRS: usize = 8;
const EMOJI_POOL: [&str; 40] = [
    "🍎", "🍌", "🍇", "🍒", "🍋", "🍉", "🍓", "🍑", "🥥", "🥝", "🍊", "🍍", "🥕", "🌽", "🥦", "🍪",
    "🌶️", "🍆", "🥔", "🧄", "🧅", "🍄", "🧀", "🥨", "🍿", "🍩", "🍰", "🧁", "🍫", "🍯", "🍭", "🍡",
    "🍙", "🍣", "🍤", "🍕", "🍔", "🌮", "🥐", "🥞",
];

pub struct MemoryGameState {
    pub tiles: Vec<Tile>,
    pub pending: Option<usize>,
    pub pending_owner: Option<poise::serenity_prelude::UserId>,
    pub matches: usize,
    pub attempts: u32,
    pub locked: bool,
    pub custom_id_prefix: String,
    pub mode: Mode,
    pub status_text: Option<String>,
}

impl MemoryGameState {
    pub fn new(mode: Mode) -> Self {
        let tiles = generate_tiles();
        let custom_id_prefix = format!("mem_{}_", rand::rng().random::<u64>());
        Self {
            tiles,
            pending: None,
            pending_owner: None,
            matches: 0,
            attempts: 0,
            locked: false,
            custom_id_prefix,
            mode,
            status_text: None,
        }
    }

    pub fn total_pairs(&self) -> usize {
        self.tiles.len() / 2
    }

    pub fn is_selectable(&self, index: usize) -> bool {
        index < self.tiles.len() && !self.tiles[index].matched && self.pending != Some(index)
    }

    pub fn select(&mut self, index: usize) -> SelectionResult {
        match self.pending.replace(index) {
            None => SelectionResult::FirstReveal,
            Some(first) => {
                self.pending = None;
                self.pending_owner = None;
                self.attempts += 1;

                if self.tiles[first].emoji == self.tiles[index].emoji {
                    self.tiles[first].matched = true;
                    self.tiles[index].matched = true;
                    self.matches += 1;
                    SelectionResult::Matched {
                        finished: self.matches == self.total_pairs(),
                    }
                } else {
                    SelectionResult::Mismatch {
                        pair: [first, index],
                    }
                }
            }
        }
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status_text = Some(status.into());
    }
}

pub enum SelectionResult {
    FirstReveal,
    Matched { finished: bool },
    Mismatch { pair: [usize; 2] },
}

fn generate_tiles() -> Vec<Tile> {
    let mut rng = rand::rng();
    let mut emojis = EMOJI_POOL.to_vec();
    emojis.shuffle(&mut rng);

    let mut tiles: Vec<Tile> = emojis
        .into_iter()
        .take(BOARD_PAIRS)
        .flat_map(|emoji| [Tile::new(emoji), Tile::new(emoji)])
        .collect();

    tiles.shuffle(&mut rng);
    tiles
}
