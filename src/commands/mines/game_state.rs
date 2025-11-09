use super::{
    BOARD_COLUMNS, BOARD_ROWS, CASHOUT_STEP, FORCE_CASHOUT_AFTER, MAX_MULTIPLIER, MULTIPLIER_STEP,
    TOTAL_BOMBS, tile::Tile,
};
use rand::{Rng, seq::SliceRandom};

pub struct MinesGameState {
    pub tiles: Vec<Tile>,
    pub custom_id_prefix: String,
    pub revealed_safe: usize,
    pub wager: i64,
    pub status_text: Option<String>,
    pub busted: bool,
    pub gave_up: bool,
    pub cashed_out_amount: Option<i64>,
    pub refunded: bool,
}

impl MinesGameState {
    pub fn new(wager: i64) -> Self {
        let tiles = generate_tiles();
        let custom_id_prefix = format!("mines_{}_", rand::rng().random::<u64>());
        Self {
            tiles,
            custom_id_prefix,
            revealed_safe: 0,
            wager,
            status_text: None,
            busted: false,
            gave_up: false,
            cashed_out_amount: None,
            refunded: false,
        }
    }

    pub fn reveal(&mut self, index: usize) -> Option<RevealOutcome> {
        let tile = self.tiles.get_mut(index)?;
        if tile.revealed {
            return Some(RevealOutcome::AlreadyOpened);
        }
        tile.revealed = true;
        if tile.is_bomb {
            Some(RevealOutcome::Bomb)
        } else {
            self.revealed_safe += 1;
            Some(RevealOutcome::Diamond)
        }
    }

    pub fn reveal_all(&mut self) {
        for tile in &mut self.tiles {
            tile.revealed = true;
        }
    }

    pub fn can_cash_out(&self) -> bool {
        self.revealed_safe >= CASHOUT_STEP && !self.is_finished()
    }

    pub fn force_cashout_reached(&self) -> bool {
        self.revealed_safe >= FORCE_CASHOUT_AFTER && !self.is_finished()
    }

    pub fn remaining_for_cashout(&self) -> usize {
        if self.revealed_safe >= CASHOUT_STEP {
            0
        } else {
            CASHOUT_STEP.saturating_sub(self.revealed_safe)
        }
    }

    pub fn projected_payout(&self) -> i64 {
        if self.revealed_safe == 0 {
            0
        } else {
            ((self.wager as f64) * self.current_multiplier()).floor() as i64
        }
    }

    pub fn current_multiplier(&self) -> f64 {
        if self.revealed_safe == 0 {
            1.0
        } else {
            let growth = self.revealed_safe as f64 * MULTIPLIER_STEP;
            (1.0 + growth).min(MAX_MULTIPLIER)
        }
    }

    pub fn is_finished(&self) -> bool {
        self.busted || self.gave_up || self.cashed_out_amount.is_some()
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status_text = Some(status.into());
    }
}

pub enum RevealOutcome {
    Diamond,
    Bomb,
    AlreadyOpened,
}

fn generate_tiles() -> Vec<Tile> {
    let total_tiles = BOARD_COLUMNS * BOARD_ROWS;
    let mut layout = vec![false; total_tiles];
    for slot in 0..TOTAL_BOMBS.min(total_tiles) {
        layout[slot] = true;
    }

    let mut rng = rand::rng();
    layout.shuffle(&mut rng);

    layout.into_iter().map(Tile::new).collect()
}
