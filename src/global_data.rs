// src/global_data.rs

use chrono::Utc;
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};
use teloxide::prelude::Bot;
use teloxide::types::UserId;
use tokio::sync::Mutex;

use crate::{
    db::services::chat::fetch_chat_demand_counts,
    hyperliquid::fetch_price::{fetch_token_data, TokenInfo},
};

use lazy_static::lazy_static;

use once_cell::sync::OnceCell;
pub type TokenMapping = HashMap<String, TokenInfo>;

#[derive(Clone)]
pub struct TokenThatPumped {
    pub when: i64,
    pub price: f64,
}
pub type ChatDemandMap = HashMap<i64, u8>;
// pub type PumpedMap = HashMap<String, u8>;

lazy_static! {
    pub static ref MY_ID: UserId = {
        let token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN not set");
        let me :u64= token
            .split(':')
            .next()
            .expect("Could not split the token")
            .parse().unwrap();

        UserId(me)
    };
    // Global data variables wrapped in Mutex
    pub static ref CHAT_DEMAND_MAP: Mutex<ChatDemandMap> = Mutex::new(HashMap::new());
    pub static ref TOKEN_MAP: Mutex<TokenMapping> = Mutex::new(HashMap::new());
    pub static ref TOKEN_THAT_PUMPED: Mutex<HashMap<String, TokenThatPumped>> = Mutex::new(HashMap::new());
    pub static ref TOKEN_ARRAY: Mutex<Vec<String>> = Mutex::new(Vec::new());
    pub static ref POOL: OnceCell<Arc<Pool<Postgres>>> = OnceCell::new();

    pub static ref BOT: OnceCell<Arc<Bot>> = OnceCell::new();

}

/// Obtient une référence globale au Bot.
/// Panique si le Bot n'est pas encore initialisé.
pub fn get_bot() -> Arc<Bot> {
    BOT.get().expect("Bot n'est pas initialisé").clone()
}

pub async fn get_token_array() -> Vec<String> {
    TOKEN_ARRAY.lock().await.clone()
}
pub async fn get_last_token_map() -> TokenMapping {
    TOKEN_MAP.lock().await.clone()
}
pub async fn get_token_that_pumped(key: &str) -> Option<TokenThatPumped> {
    TOKEN_THAT_PUMPED.lock().await.get(key).cloned()
}
const H12: i64 = 60 * 60 * 12;
const H24: i64 = H12 * 2;
pub async fn check_token_that_pumped(key: &str) -> Option<TokenThatPumped> {
    let now = Utc::now().timestamp();
    let ret = get_token_that_pumped(key).await;
    if let Some(token) = ret {
        if now - token.when >= H24 {
            TOKEN_THAT_PUMPED.lock().await.remove(key);
            return None;
        }
        return Some(token);
    }
    None
}

pub const REFERRAL_LINK: &str = "https://t.me/HypurrFunBot?start=ref_2262836c-trade_";

pub async fn update_token_data() -> Result<(), Box<dyn std::error::Error>> {
    let (map, token_array) = fetch_token_data().await?;

    {
        let mut global_mapping = TOKEN_MAP.lock().await;
        *global_mapping = map;
    }
    {
        let mut global_array = TOKEN_ARRAY.lock().await;
        *global_array = token_array;
    }
    debug!("Token array initiated");
    Ok(())
}

pub async fn update_demand_data() -> Result<(), Box<dyn std::error::Error>> {
    let map = fetch_chat_demand_counts().await?;

    {
        let mut global_mapping = CHAT_DEMAND_MAP.lock().await;
        *global_mapping = map;
    }

    Ok(())
}

pub fn get_pool() -> Arc<Pool<Postgres>> {
    POOL.get().expect("Pool has not been initialized").clone()
}
pub async fn get_amount_from_map_for_chat_id(id: i64) -> u8 {
    CHAT_DEMAND_MAP.lock().await.get(&id).cloned().unwrap_or(0)
}

pub async fn decrease_chat_demand(chat_id: i64) -> u8 {
    let mut map = CHAT_DEMAND_MAP.lock().await;
    map.entry(chat_id)
        .and_modify(|count| {
            if *count > 0 {
                *count -= 1;
            }
        })
        .or_insert(0);
    *map.get(&chat_id).unwrap()
}

pub async fn increase_chat_demand(chat_id: i64) -> u8 {
    let mut map = CHAT_DEMAND_MAP.lock().await;
    *map.entry(chat_id)
        .and_modify(|count| *count += 1)
        .or_insert(1)
}

//Can you do me a page like this for token with  Vec< pub struct TokenInfo { pub name: String, pub full_name: Option, pub price: f64, pub pair_number: Option, pub market_cap: u32, }>  with the interval it's for and the unix date in minutes
