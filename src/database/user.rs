use super::models::UserModel;
use sqlx::{Error as SqlxError, sqlite::SqlitePool};

/// Finds a user by discord id
pub async fn find_by_discord_id(
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

/// Creates a new user row and returns the stored model
pub async fn create(pool: &SqlitePool, discord_id: i64) -> Result<UserModel, SqlxError> {
    let created_at = chrono::Utc::now().to_rfc3339();

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
pub async fn get_or_create(pool: &SqlitePool, discord_id: i64) -> Result<UserModel, SqlxError> {
    if let Some(existing) = find_by_discord_id(pool, discord_id).await? {
        Ok(existing)
    } else {
        create(pool, discord_id).await
    }
}

/// Updates user's balance
pub async fn update_balance(
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
