use crate::{
    env,
    models::{RewardStateModel, UserModel},
};
use chrono::{DateTime, Utc};
use sqlx::{Error as SqlxError, sqlite::SqlitePool};

pub async fn connect() -> Result<SqlitePool, SqlxError> {
    let database_url = env::database_url()
        .map(|opt| opt.unwrap_or_else(|| env::DEFAULT_DATABASE_URL.to_string()))
        .map_err(|err| SqlxError::Configuration(err.to_string().into()))?;

    let pool = SqlitePool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Finds a user by discord id.
pub async fn find_user_by_discord_id(
    pool: &SqlitePool,
    discord_id: i64,
) -> Result<Option<UserModel>, SqlxError> {
    sqlx::query_as::<_, UserModel>(
        "SELECT id, discord_id, dollars, diamonds, created_at FROM users WHERE discord_id = ?",
    )
    .bind(discord_id)
    .fetch_optional(pool)
    .await
}

/// Creates a new user row and returns the stored model.
pub async fn create_user(pool: &SqlitePool, discord_id: i64) -> Result<UserModel, SqlxError> {
    let created_at = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO users (discord_id, dollars, diamonds, created_at) VALUES (?, 0, 0, ?)",
    )
    .bind(discord_id)
    .bind(&created_at)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid() as i32;

    Ok(UserModel {
        id,
        discord_id,
        dollars: 0,
        diamonds: 0,
        created_at,
    })
}

/// Gets an existing user or creates a new one if it doesn't exist
pub async fn get_or_create_user(
    pool: &SqlitePool,
    discord_id: i64,
) -> Result<UserModel, SqlxError> {
    if let Some(existing) = find_user_by_discord_id(pool, discord_id).await? {
        Ok(existing)
    } else {
        create_user(pool, discord_id).await
    }
}

/// Updates user's balance
pub async fn update_user_balance(
    pool: &SqlitePool,
    user_id: i32,
    dollars: i64,
    diamonds: i64,
) -> Result<UserModel, SqlxError> {
    sqlx::query("UPDATE users SET dollars = ?, diamonds = ? WHERE id = ?")
        .bind(dollars)
        .bind(diamonds)
        .bind(user_id)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, UserModel>(
        "SELECT id, discord_id, dollars, diamonds, created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Gets a reward state for a user and reward type
pub async fn get_reward_state(
    pool: &SqlitePool,
    user_id: i32,
    reward_type: &str,
) -> Result<Option<RewardStateModel>, SqlxError> {
    sqlx::query_as::<_, RewardStateModel>(
        "SELECT id, user_id, reward_type, last_claimed_at, next_reset_at, total_claims
         FROM reward_states WHERE user_id = ? AND reward_type = ?",
    )
    .bind(user_id)
    .bind(reward_type)
    .fetch_optional(pool)
    .await
}

/// Gets all reward states for a user
pub async fn get_all_reward_states(
    pool: &SqlitePool,
    user_id: i32,
) -> Result<Vec<RewardStateModel>, SqlxError> {
    sqlx::query_as::<_, RewardStateModel>(
        "SELECT id, user_id, reward_type, last_claimed_at, next_reset_at, total_claims
         FROM reward_states WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Updates or creates a reward state
pub async fn upsert_reward_state(
    pool: &SqlitePool,
    user_id: i32,
    reward_type: &str,
    last_claimed_at: Option<DateTime<Utc>>,
    next_reset_at: Option<DateTime<Utc>>,
    total_claims: i64,
) -> Result<RewardStateModel, SqlxError> {
    let last_claimed_str = last_claimed_at.map(|dt| dt.to_rfc3339());
    let next_reset_str = next_reset_at.map(|dt| dt.to_rfc3339());

    sqlx::query(
        "INSERT INTO reward_states (user_id, reward_type, last_claimed_at, next_reset_at, total_claims)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(user_id, reward_type) DO UPDATE SET
            last_claimed_at = excluded.last_claimed_at,
            next_reset_at = excluded.next_reset_at,
            total_claims = excluded.total_claims",
    )
    .bind(user_id)
    .bind(reward_type)
    .bind(&last_claimed_str)
    .bind(&next_reset_str)
    .bind(total_claims)
    .execute(pool)
    .await?;

    get_reward_state(pool, user_id, reward_type)
        .await?
        .ok_or(SqlxError::RowNotFound)
}
