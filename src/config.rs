use crate::fumo::{Data, Error};
use poise::serenity_prelude as serenity;

/// Declares which gateway intents the bot subscribes to.
pub fn gateway_intents() -> serenity::GatewayIntents {
    serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
}

/// Builds the prefix configuration shared by the Poise framework.
pub fn prefix_options() -> poise::PrefixFrameworkOptions<Data, Error> {
    poise::PrefixFrameworkOptions {
        prefix: Some("?".into()),
        additional_prefixes: vec![
            poise::Prefix::Literal("-"),
            poise::Prefix::Literal("f!"),
            poise::Prefix::Literal("."),
        ],
        ..Default::default()
    }
}
