use super::models::BlacklistEntryModel;
use chrono::Utc;
use sqlx::{Error as SqlxError, sqlite::SqlitePool};

/// Finds a blacklist entry by Discord ID, if it exists
pub async fn find_by_discord_id(
    pool: &SqlitePool,
    discord_id: i64,
) -> Result<Option<BlacklistEntryModel>, SqlxError> {
    sqlx::query_as::<_, BlacklistEntryModel>(
        r#"
        SELECT id, discord_id, moderator_id, reason, created_at
        FROM blacklist_entries
        WHERE discord_id = ?
        "#,
    )
    .bind(discord_id)
    .fetch_optional(pool)
    .await
}

/// Creates a new blacklist entry
pub async fn insert(
    pool: &SqlitePool,
    discord_id: i64,
    moderator_id: i64,
    reason: Option<String>,
) -> Result<BlacklistEntryModel, SqlxError> {
    let created_at = Utc::now().to_rfc3339();
    let reason_clone = reason.clone();

    let result = sqlx::query(
        r#"
        INSERT INTO blacklist_entries (discord_id, moderator_id, reason, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(discord_id)
    .bind(moderator_id)
    .bind(reason_clone.as_deref())
    .bind(&created_at)
    .execute(pool)
    .await?;

    Ok(BlacklistEntryModel {
        id: result.last_insert_rowid() as i32,
        discord_id,
        moderator_id,
        reason,
        created_at,
    })
}

/// Removes a user from the blacklist. Returns the number of affected rows
pub async fn delete_by_discord_id(pool: &SqlitePool, discord_id: i64) -> Result<u64, SqlxError> {
    let result = sqlx::query(
        r#"
        DELETE FROM blacklist_entries
        WHERE discord_id = ?
        "#,
    )
    .bind(discord_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Lists the most recent blacklist entries with pagination support
pub async fn list_recent(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<BlacklistEntryModel>, SqlxError> {
    sqlx::query_as::<_, BlacklistEntryModel>(
        r#"
        SELECT id, discord_id, moderator_id, reason, created_at
        FROM blacklist_entries
        ORDER BY created_at DESC
        LIMIT ?
        OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
