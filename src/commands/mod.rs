use crate::{Data, Error};

pub mod bot;
pub mod economy;
pub mod help;
pub mod jokenpo;
pub mod memory;
pub mod ping;

pub fn load_all() -> Vec<poise::Command<Data, Error>> {
    vec![
        help::help(),
        ping::ping(),
        jokenpo::jokenpo(),
        bot::bot(),
        economy::economy(),
        memory::memory(),
    ]
}
