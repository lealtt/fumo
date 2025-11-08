use super::models::RewardStateModel;
use chrono::{DateTime, Utc};
use sqlx::{Error as SqlxError, sqlite::SqlitePool};

/// Gets a reward state for a user and reward type
pub async fn get(
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
pub async fn get_all(pool: &SqlitePool, user_id: i32) -> Result<Vec<RewardStateModel>, SqlxError> {
    sqlx::query_as::<_, RewardStateModel>(
        "SELECT id, user_id, reward_type, last_claimed_at, next_reset_at, total_claims
         FROM reward_states WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Updates or creates a reward state
pub async fn upsert(
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

    get(pool, user_id, reward_type)
        .await?
        .ok_or(SqlxError::RowNotFound)
}
