pub struct Tile {
    pub is_bomb: bool,
    pub revealed: bool,
}

impl Tile {
    pub const fn new(is_bomb: bool) -> Self {
        Self {
            is_bomb,
            revealed: false,
        }
    }
}
