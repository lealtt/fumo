use std::fmt::Display;

pub fn pretty_message(emoji: impl Display, message: impl Display) -> String {
    format!("{} | {}", emoji, message)
}
