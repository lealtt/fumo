use crate::{Data, Error};

pub mod help;
pub mod jokenpo;
pub mod ping;

pub fn load_all() -> Vec<poise::Command<Data, Error>> {
    vec![help::help(), ping::ping(), jokenpo::jokenpo()]
}
