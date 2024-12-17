use sqlx::Row;
use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use std::ops::Deref;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, ThreadId};

use crate::bot::{send_error_to_moderator, send_message, send_message_with_button};
use crate::global_data::{decrease_chat_demand, get_pool, increase_chat_demand};
use crate::{
    db::services::chat::insert_chat,
    global_data::CHAT_DEMAND_MAP,
    types::commands::{ALERT, SPECIAL},
};

#[derive(Debug, Default, Clone)]
pub struct Demand {
    pub chat_id: i64,
    pub thread_id: Option<i32>,
    pub type_of: String,
    pub token: String,
    pub percentage: i16,
    pub interval: String,
}

impl Demand {
    pub fn new(chat_id: i64, type_of: impl Into<String>, thread_id: Option<i32>) -> Self {
        Self {
            chat_id,
            thread_id,
            type_of: type_of.into(),
            ..Default::default()
        }
    }
    pub fn get_composite_id(&self) -> String {
        let composite = format!(
            "{}_{}_{}_{}_{}",
            self.chat_id, self.type_of, self.token, self.percentage, self.interval
        );
        base64.encode(composite.as_bytes())
    }

    pub fn parse_composite_id(id: &str) -> Option<(i64, String, String, i16, String)> {
        let bytes = base64.decode(id).ok()?;
        let composite = String::from_utf8(bytes).ok()?;

        let parts: Vec<&str> = composite.split('_').collect();
        if parts.len() != 5 {
            return None;
        }

        let chat_id = parts[0].parse().ok()?;
        let type_of = parts[1].to_string();
        let token = parts[2].to_string();
        let percentage = parts[3].parse().ok()?;
        let interval = parts[4].to_string();

        Some((chat_id, type_of, token, percentage, interval))
    }
    pub async fn delete_demand(self) -> anyhow::Result<()> {
        delete_demand_by_composite_id(&self.get_composite_id()).await
    }

    pub async fn insert_to_db(self) -> anyhow::Result<()> {
        let pool = get_pool();
        let _ = insert_chat(self.chat_id).await;

        // First do the DB insert
        sqlx::query(
            "INSERT INTO demands (chat_id, thread_id, type_of, token, percentage, interval)
         VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(self.chat_id)
        .bind(self.thread_id)
        .bind(&self.type_of)
        .bind(&self.token)
        .bind(self.percentage)
        .bind(&self.interval)
        .execute(pool.deref())
        .await
        .map_err(|e| {
            if let Some(db_error) = e.as_database_error() {
                if db_error.code().as_deref() == Some("23505") {
                    return anyhow::anyhow!("Demand already exists");
                }
            }
            anyhow::anyhow!("Failed to insert demand: {}", e)
        })?;

        // Then update the map in a separate block to ensure lock is held
        increase_chat_demand(self.chat_id).await;

        Ok(())
    }
    //Delete by composite ID
}

// Database operations implementation

pub async fn delete_demand_by_composite_id(composite_id: &str) -> anyhow::Result<()> {
    if let Some((chat_id, type_of, token, percentage, interval)) =
        Demand::parse_composite_id(composite_id)
    {
        let pool = get_pool();
        sqlx::query(
            "DELETE FROM demands 
             WHERE chat_id = $1 
             AND type_of = $2 
             AND token = $3 
             AND percentage = $4 
             AND interval = $5",
        )
        .bind(chat_id)
        .bind(type_of)
        .bind(token)
        .bind(percentage)
        .bind(interval)
        .execute(pool.deref())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete demand: {}", e))?;

        decrease_chat_demand(chat_id).await;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid composite ID format"))
    }
}
pub async fn delete_demands_for_chat(chat_id_param: i64) -> anyhow::Result<()> {
    let pool = get_pool();

    sqlx::query("DELETE FROM demands WHERE chat_id = $1")
        .bind(chat_id_param)
        .execute(pool.deref())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete demands: {}", e))?;

    CHAT_DEMAND_MAP.lock().await.remove(&chat_id_param);
    Ok(())
}

pub async fn get_all_special_chat_id() -> anyhow::Result<Vec<i64>> {
    let pool = get_pool();

    let rows = sqlx::query("SELECT chat_id FROM demands WHERE type_of = $1")
        .bind(SPECIAL)
        .fetch_all(pool.deref())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get pump check chat ids: {}", e))?;

    Ok(rows.into_iter().map(|row| row.get(0)).collect())
}

pub async fn fetch_last_regular_demands_by_time(time: &str) -> anyhow::Result<Vec<Demand>> {
    let pool = get_pool();

    let rows = sqlx::query(
        "SELECT chat_id, thread_id, type_of, token, percentage, interval
         FROM demands
         WHERE interval = $1 AND type_of = $2",
    )
    .bind(time)
    .bind(ALERT)
    .fetch_all(pool.deref())
    .await
    .map_err(|e| anyhow::anyhow!("Error fetching demands: {}", e))?;

    let demands = rows
        .into_iter()
        .map(|row| Demand {
            chat_id: row.get("chat_id"),
            thread_id: row.get("thread_id"),
            type_of: row.get("type_of"),
            token: row.get("token"),
            percentage: row.get("percentage"),
            interval: row.get("interval"),
        })
        .collect();

    Ok(demands)
}

pub async fn batch_fetch_last_demands_by_time(
    times: Vec<String>,
) -> anyhow::Result<HashMap<String, Vec<Demand>>> {
    let mut map = HashMap::new();

    for time in times {
        match fetch_last_regular_demands_by_time(&time).await {
            Ok(demands) => {
                map.insert(time.clone(), demands);
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error fetching demands for time {}: {}",
                    time,
                    e
                ));
            }
        }
    }
    Ok(map)
}

pub async fn get_demands_by_chat_id(chat_id: i64) -> anyhow::Result<Vec<Demand>> {
    let pool = get_pool();
    let rows = sqlx::query(
        "SELECT chat_id, thread_id, type_of, token, percentage, interval 
         FROM demands 
         WHERE chat_id = $1 
         ORDER BY type_of, token, percentage, interval", // ordered for consistency
    )
    .bind(chat_id)
    .fetch_all(pool.deref())
    .await
    .map_err(|e| anyhow::anyhow!("Failed to fetch demands for chat_id {}: {}", chat_id, e))?;

    let demands = rows
        .into_iter()
        .map(|row| Demand {
            chat_id: row.get("chat_id"),
            thread_id: row.get("thread_id"),
            type_of: row.get("type_of"),
            token: row.get("token"),
            percentage: row.get("percentage"),
            interval: row.get("interval"),
        })
        .collect();

    Ok(demands)
}

pub fn send_demands_for(chat_id: ChatId, thread_id: Option<ThreadId>, demands: Vec<Demand>) {
    if demands.len() == 0 {
        return send_message(chat_id, "No alert sert for now", thread_id);
    }

    let mut message = format!("__*Here is your alerts*__:\n");
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for (i, demand) in demands.iter().enumerate() {
        message.push_str(&format!(
            "--------- \n__{i}__: {}\n",
            format_demand_for_message(demand)
        ));
        keyboard.push(vec![InlineKeyboardButton::callback(
            &format!("{i}"),
            &format!("{}_{}", chat_id.0, demand.get_composite_id()),
        )]);
    }
    message.push_str("\n*Chose which one you want to delete*:");

    let clavier = InlineKeyboardMarkup::new(keyboard);
    send_message_with_button(chat_id, &message, thread_id, clavier);
}

pub fn format_demand_for_message(demands: &Demand) -> String {
    let end_str = match demands.percentage {
        0 => String::new(),
        x => format!("for {x}%"),
    };
    match demands.type_of.as_str() {
        ALERT => format!(
            "*{}* price change {end_str} for {}",
            demands.token, demands.interval
        ),
        SPECIAL => format!("*Special* demand"),
        _ => {
            send_error_to_moderator(format!("demands.type_of {}", demands.type_of));
            format!("Unexpected demand")
        }
    }
}
