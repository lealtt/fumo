pub mod blacklist;
pub mod models;
pub mod reward;
pub mod transaction;
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

pub use models::{BlacklistEntryModel, CurrencyTransactionModel, RewardStateModel, UserModel};

pub use blacklist::{
    delete_by_discord_id as delete_blacklist_entry, find_by_discord_id as find_blacklist_entry,
    insert as insert_blacklist_entry, list_recent as list_blacklist_entries,
};
pub use reward::{get_all as get_all_reward_states, upsert as upsert_reward_state};
pub use transaction::{
    delete_by_id as delete_currency_transaction, insert as insert_currency_transaction,
    list_recent_by_user as list_currency_transactions,
};
pub use user::{get_or_create as get_or_create_user, update_balance as update_user_balance};
