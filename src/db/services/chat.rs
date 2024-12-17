use crate::global_data::get_pool;
use anyhow::anyhow;
use sqlx::{Pool, Postgres, Row};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn insert_chat(chat_id_no: i64) -> anyhow::Result<()> {
    let pool: Arc<Pool<Postgres>> = get_pool();
    debug!("Inserting chat with id {:?}", chat_id_no);

    sqlx::query("INSERT INTO chat (id) VALUES ($1) ON CONFLICT (id) DO NOTHING")
        .bind(chat_id_no)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            error!("Error while inserting chat: {:?}", e);
            anyhow!(e)
        })?;

    Ok(())
}

pub async fn fetch_chat_demand_counts() -> anyhow::Result<HashMap<i64, u8>> {
    let pool: Arc<Pool<Postgres>> = get_pool();

    let rows =
        sqlx::query("SELECT chat_id, COUNT(*) AS demand_count FROM demands GROUP BY chat_id")
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| {
                let e_str = format!("Error while getting chat demand counts: {:?}", e);
                error!("{e_str}");
                anyhow!(e_str)
            })?;

    let mut map = HashMap::new();

    for row in rows {
        let chat_id: i64 = row.try_get("chat_id")?;
        let demand_count_i64: i64 = row.try_get("demand_count")?;

        // Convert the demand count to u8 if possible
        let demand_count_u8: u8 = demand_count_i64.try_into().map_err(|_| {
            anyhow!(
                "The number of demands for chat_id {} ({}) exceeds the capacity of a u8 (255)",
                chat_id,
                demand_count_i64
            )
        })?;

        map.insert(chat_id, demand_count_u8);
    }

    Ok(map)
}
