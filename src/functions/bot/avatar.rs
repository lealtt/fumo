// use crate::Error;
// use poise::serenity_prelude as serenity;
// use rand::prelude::IndexedRandom;
// use serenity::builder::{CreateAttachment, EditProfile};
// use serenity::http::Http;
// use std::fs;
// use std::path::PathBuf;
// use std::sync::Arc;
// use std::time::Duration;

// const AVATAR_ROTATION_INTERVAL_SECS: u64 = 60 * 60; // 1 hour
// const ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");

// /// Spawns a background task that periodically swaps the bot avatar.
// pub fn spawn_avatar_rotation_task(http: Arc<Http>) {
//     tokio::spawn(async move {
//         if let Err(err) = avatar_rotation_loop(http).await {
//             eprintln!("Avatar rotation task exited: {err}");
//         }
//     });
// }

// async fn avatar_rotation_loop(http: Arc<Http>) -> Result<(), Error> {
//     if let Err(err) = set_random_avatar(http.clone()).await {
//         eprintln!("Failed to change avatar: {err}");
//     }

//     let mut ticker = tokio::time::interval(Duration::from_secs(AVATAR_ROTATION_INTERVAL_SECS));
//     ticker.tick().await; // consume immediate tick so the next one fires after the interval

//     loop {
//         ticker.tick().await;
//         if let Err(err) = set_random_avatar(http.clone()).await {
//             eprintln!("Failed to change avatar: {err}");
//         }
//     }
// }

// async fn set_random_avatar(http: Arc<Http>) -> Result<(), Error> {
//     let candidates = collect_avatar_candidates()?;
//     if candidates.is_empty() {
//         eprintln!("Avatar rotation skipped: no files found in the assets directory ({ASSETS_DIR})");
//         return Ok(());
//     }

//     let path = {
//         let mut rng = rand::rng();
//         match candidates.choose(&mut rng) {
//             Some(path) => path.clone(),
//             None => return Ok(()),
//         }
//     };

//     let attachment = CreateAttachment::path(&path).await?;
//     let mut current_user = http.get_current_user().await?;
//     current_user
//         .edit(http.clone(), EditProfile::new().avatar(&attachment))
//         .await?;

//     Ok(())
// }

// fn collect_avatar_candidates() -> Result<Vec<PathBuf>, std::io::Error> {
//     let mut files = Vec::new();
//     for entry in fs::read_dir(ASSETS_DIR)? {
//         let entry = entry?;
//         if entry.file_type()?.is_file() {
//             files.push(entry.path());
//         }
//     }

//     Ok(files)
// }
