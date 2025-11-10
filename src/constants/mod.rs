pub mod color;
pub mod emoji;

pub mod links {
    pub const GITHUB_REPO: &str = "https://github.com/lealtt/fumo";
}

pub mod colors {
    pub use super::color::{MINT, MOON};
}

pub use emoji::{CustomEmoji, icon};
