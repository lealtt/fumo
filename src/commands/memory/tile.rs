pub struct Tile {
    pub emoji: &'static str,
    pub matched: bool,
}

impl Tile {
    pub const fn new(emoji: &'static str) -> Self {
        Self {
            emoji,
            matched: false,
        }
    }
}
