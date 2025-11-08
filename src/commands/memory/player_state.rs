use poise::serenity_prelude as serenity;

#[derive(Clone)]
pub struct PlayerState {
    pub user: serenity::User,
    pub score: u32,
}

impl PlayerState {
    pub fn new(user: serenity::User) -> Self {
        Self { user, score: 0 }
    }
}
