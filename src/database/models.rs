use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct UserModel {
    pub id: i32,
    pub discord_id: i64,
    pub dollars: i64,
    pub diamonds: i64,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct CurrencyTransactionModel {
    pub id: i32,
    pub user_id: i32,
    pub amount: i64,
    pub balance_after: i64,
    pub currency: String,
    pub kind: String,
    pub context: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct RewardStateModel {
    pub id: i32,
    pub user_id: i32,
    pub reward_type: String,
    pub last_claimed_at: Option<String>,
    pub next_reset_at: Option<String>,
    pub total_claims: i64,
}

impl RewardStateModel {
    pub fn next_reset_datetime(&self) -> Option<DateTime<Utc>> {
        self.next_reset_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }
}
