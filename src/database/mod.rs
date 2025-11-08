pub mod models;
pub mod reward;
pub mod user;

use crate::env;
use sqlx::{Error as SqlxError, sqlite::SqlitePool};

/// Connects to the database and runs migrations
pub async fn connect() -> Result<SqlitePool, SqlxError> {
    let database_url = env::database_url()
        .map(|opt| opt.unwrap_or_else(|| env::DEFAULT_DATABASE_URL.to_string()))
        .map_err(|err| SqlxError::Configuration(err.to_string().into()))?;

    let pool = SqlitePool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub use models::{RewardStateModel, UserModel};

pub use reward::{get_all as get_all_reward_states, upsert as upsert_reward_state};
pub use user::{get_or_create as get_or_create_user, update_balance as update_user_balance};
