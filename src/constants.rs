use serenity::all::{Colour, EmojiId, ReactionType};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CustomEmoji {
    id: u64,
}

impl CustomEmoji {
    pub const fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn emoji_id(&self) -> EmojiId {
        EmojiId::new(self.id)
    }

    pub fn as_reaction(&self) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: self.emoji_id(),
            name: None,
        }
    }

    pub fn as_str(&self) -> String {
        format!("<:_:{}>", self.id)
    }
}

impl fmt::Display for CustomEmoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

pub mod icon {
    use super::CustomEmoji;

    pub const CHECK: CustomEmoji = CustomEmoji::new(1434940349326954647);
    pub const ERROR: CustomEmoji = CustomEmoji::new(1434940351642079242);
    pub const BELL: CustomEmoji = CustomEmoji::new(1434941392651550824);
    pub const GEAR: CustomEmoji = CustomEmoji::new(1434940839779631204);
    pub const PLUS: CustomEmoji = CustomEmoji::new(1434953088409534544);
    pub const MINUS: CustomEmoji = CustomEmoji::new(1434953090326462637);
    pub const GIFT: CustomEmoji = CustomEmoji::new(1436430668425859253);
    pub const DOLLAR: CustomEmoji = CustomEmoji::new(1436430666366586912);
    pub const DIAMOND: CustomEmoji = CustomEmoji::new(1436430663476842659);
    pub const ALARM: CustomEmoji = CustomEmoji::new(1436442458849284127);
    pub const TIMER: CustomEmoji = CustomEmoji::new(1436442461022195753);
    pub const RSS: CustomEmoji = CustomEmoji::new(1436674873433915412);
    pub const HASTAG: CustomEmoji = CustomEmoji::new(1434940827263832096);
}

pub mod colors {
    use super::Colour;

    pub const MINT: Colour = Colour::new(0x4ECCA3);
    pub const MOON: Colour = Colour::new(0xA6B1E1);
}
