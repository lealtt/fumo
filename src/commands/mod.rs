use crate::{Data, Error};

pub mod blacklist;
pub mod economy;
pub mod help;
pub mod jokenpo;
pub mod memory;
pub mod mines;
pub mod ping;
pub mod race;
pub mod util;

pub fn load_all() -> Vec<poise::Command<Data, Error>> {
    vec![
        help::help(),
        ping::ping(),
        jokenpo::jokenpo(),
        economy::economy(),
        memory::memory(),
        mines::mines(),
        race::race(),
        blacklist::blacklist(),
    ]
}
