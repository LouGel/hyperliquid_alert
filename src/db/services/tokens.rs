use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Row};

use std::ops::Deref;

use crate::global_data::{get_pool, TokenMapping};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct TokensAt {
    pub timestamp_in_min: i32, // Matches INTEGER
    pub times: Vec<String>,    // Matches TEXT[]
    pub tokens: TokenMapping,  // Matches JSONB
}

// Custom FromRow implementation to handle JSONB field deserialization for TokensAt
impl<'r> FromRow<'r, sqlx::postgres::PgRow> for TokensAt {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let timestamp_in_min: i32 = row.try_get("timestamp_in_min")?;
        let times: Vec<String> = row.try_get("times")?;

        let tokens_value: Value = row.try_get("tokens")?;
        let tokens: TokenMapping =
            serde_json::from_value(tokens_value).map_err(|e| sqlx::Error::ColumnDecode {
                index: "tokens".into(),
                source: Box::new(e),
            })?;

        Ok(Self {
            timestamp_in_min,
            times,
            tokens,
        })
    }
}

impl TokensAt {
    pub async fn insert(&self) -> anyhow::Result<u64> {
        let pool = get_pool();

        sqlx::query(
            r#"
            INSERT INTO tokens_at (timestamp_in_min, times, tokens)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(self.timestamp_in_min)
        .bind(&self.times)
        .bind(
            serde_json::to_value(&self.tokens)
                .map_err(|e| anyhow!("Failed to serialize tokens: {}", e))?,
        )
        .execute(pool.deref())
        .await
        .map(|result| result.rows_affected())
        .map_err(|e| anyhow!("Failed to insert tokens: {}", e))
    }
}

pub async fn fetch_latest_tokens_at(time: &str) -> anyhow::Result<Option<TokensAt>> {
    let pool = get_pool();

    let row = sqlx::query(
        r#"
        SELECT timestamp_in_min, times, tokens
        FROM tokens_at
        WHERE $1 = ANY(times)
        ORDER BY timestamp_in_min DESC
        LIMIT 1
        "#,
    )
    .bind(time)
    .fetch_optional(pool.deref())
    .await
    .map_err(|e| anyhow!("Query failed: {}", e))?;

    // Manually map the result to TokensAt if found
    if let Some(row) = row {
        let tokens_at = TokensAt::from_row(&row)?;
        Ok(Some(tokens_at))
    } else {
        Ok(None)
    }
}
