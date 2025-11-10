use crate::{
    Data, Error,
    constants::{colors, icon},
    functions::format::pretty_message,
};
use poise::FrameworkError;

/// Default error handler for application commands
pub async fn command_error_handler(error: FrameworkError<'_, Data, Error>) {
    match error {
        FrameworkError::Command { ctx, error, .. } => {
            eprintln!("Command `{}` failed: {error:?}", ctx.command().name);

            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .embed(
                            poise::serenity_prelude::CreateEmbed::new()
                                .colour(colors::MOON)
                                .description(pretty_message(
                                    icon::ERROR,
                                    "Algo deu errado ao executar este comando. Tente novamente em instantes.",
                                )),
                        )
                        .ephemeral(true),
                )
                .await;
        }
        other => {
            if let Err(err) = poise::builtins::on_error(other).await {
                eprintln!("Error while handling command error: {err:?}");
            }
        }
    }
}
