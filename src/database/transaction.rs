use super::models::CurrencyTransactionModel;
use chrono::Utc;
use sqlx::{Error as SqlxError, sqlite::SqlitePool};

/// Inserts a new entry into the currency transaction ledger
pub async fn insert(
    pool: &SqlitePool,
    user_id: i32,
    amount: i64,
    balance_after: i64,
    currency: &str,
    kind: &str,
    context: Option<String>,
) -> Result<CurrencyTransactionModel, SqlxError> {
    let created_at = Utc::now().to_rfc3339();
    let context_ref = context.as_deref();

    let result = sqlx::query(
        "INSERT INTO currency_transactions \
        (user_id, amount, balance_after, currency, kind, context, created_at) \
        VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(amount)
    .bind(balance_after)
    .bind(currency)
    .bind(kind)
    .bind(context_ref)
    .bind(&created_at)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid() as i32;

    sqlx::query_as::<_, CurrencyTransactionModel>(
        "SELECT id, user_id, amount, balance_after, currency, kind, context, created_at \
        FROM currency_transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

/// Returns the most recent transactions for a user, ordered from newest to oldest
pub async fn list_recent_by_user(
    pool: &SqlitePool,
    user_id: i32,
    limit: i64,
) -> Result<Vec<CurrencyTransactionModel>, SqlxError> {
    let capped_limit = limit.clamp(1, 200);
    sqlx::query_as::<_, CurrencyTransactionModel>(
        "SELECT id, user_id, amount, balance_after, currency, kind, context, created_at \
        FROM currency_transactions \
        WHERE user_id = ? \
        ORDER BY id DESC \
        LIMIT ?",
    )
    .bind(user_id)
    .bind(capped_limit)
    .fetch_all(pool)
    .await
}

/// Removes a specific transaction entry, used when rolling back pending operations
pub async fn delete_by_id(pool: &SqlitePool, transaction_id: i32) -> Result<(), SqlxError> {
    sqlx::query("DELETE FROM currency_transactions WHERE id = ?")
        .bind(transaction_id)
        .execute(pool)
        .await?;
    Ok(())
}
