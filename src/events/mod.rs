use crate::{Data, Error};
use poise::{self, BoxFuture, serenity_prelude as serenity};

pub mod mention;

pub type EventHandler = for<'a> fn(
    poise::FrameworkContext<'a, Data, Error>,
    &'a serenity::FullEvent,
) -> BoxFuture<'a, Result<(), Error>>;

/// Returns the list of registered event handlers
pub fn load_all() -> &'static [EventHandler] {
    &[mention::event_handler]
}

/// Dispatches the incoming event to every registered handler in order
pub fn dispatch<'a>(
    framework: poise::FrameworkContext<'a, Data, Error>,
    event: &'a serenity::FullEvent,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        for handler in load_all() {
            handler(framework, event).await?;
        }
        Ok(())
    })
}
