use chrono::Utc;

use crate::{
    bot::{broadcast_message, send_error_to_moderator},
    constants::pumpcheck::{MIN_MARKET_CAP, OVER_SPECIAL_PERCENTAGE, SPECIAL_PERCENTAGE},
    db::services::demands::get_all_special_chat_id,
    global_data::{
        check_token_that_pumped, get_last_token_map, TokenThatPumped, REFERRAL_LINK,
        TOKEN_THAT_PUMPED,
    },
    hyperliquid::fetch_price::TokenInfo,
    // types::twitter_client::{utils, TwitterBot},
};
// use std::env;

const PUMP_HEADER: &str = "__*📈 WAGMI Pump Alert:*__\n\n";
const PUMP_ERROR_HEADER: &str = "PUMP_ERROR\n";

pub async fn check_and_send_pump() {
    // Generate alert message
    let alert_message = match generate_pump_alert().await {
        None => return,
        Some(message) => message,
    };
    broadcast_to_chats(alert_message).await;
}

// Part 1: Generate pump alert message
async fn generate_pump_alert() -> Option<String> {
    let token_map = get_last_token_map().await;
    let mut alert_message = PUMP_HEADER.to_string();
    let now = Utc::now().timestamp();
    for (key, value) in token_map {
        let mut pump = check_pump(&value);
        if let Some(token_that_pumped) = check_token_that_pumped(&key).await {
            if !check_over_pump(value.price, token_that_pumped.price) {
                pump = 0.0;
            }
        }

        if pump == 0.0 {
            continue;
        }
        let message = &format!(
            "__[{}]({}{})__: Price has risen by {}% in the last 24h: {}$\n------------------------\n",
            key,
            REFERRAL_LINK,
            value.pair_number.unwrap_or(0),
            pump,value.price
        );

        let token = TokenThatPumped {
            when: now,
            price: value.price,
        };
        {
            TOKEN_THAT_PUMPED.lock().await.insert(key, token);
        }
        info!("{message}");
        alert_message.push_str(message);
    }

    if alert_message == PUMP_HEADER {
        None
    } else {
        Some(alert_message)
    }
}

async fn broadcast_to_chats(alert_message: String) {
    match get_all_special_chat_id().await {
        Err(e) => send_error_to_moderator(format!(
            "{PUMP_ERROR_HEADER}Error while getting all chat_ids: {:?}",
            e
        )),
        Ok(all_chat_ids) => {
            if let Err(e) = broadcast_message(all_chat_ids, alert_message).await {
                send_error_to_moderator(format!("Error while getting all chat_ids: {:?}", e))
            } else {
                info!("Pump message broadcasted")
            }
        }
    }
}

// Helper functions remain unchanged
pub fn diff_in_percent(now: f64, previous: f64) -> f64 {
    if previous == 0.0 || previous.is_nan() {
        return 0.0;
    }
    ((((now - previous) / previous * 1e4) as i32) / 100) as f64
}

pub fn check_pump(t: &TokenInfo) -> f64 {
    debug!("checking pump for {:?}", t.full_name);
    let mut ret = diff_in_percent(t.price, t.price_prev_24h);
    if ret < SPECIAL_PERCENTAGE || t.market_cap < MIN_MARKET_CAP {
        ret = 0.0;
    }
    ret
}
pub fn check_over_pump(now: f64, then: f64) -> bool {
    (now - then) / then * 100.0 > OVER_SPECIAL_PERCENTAGE
}

// pub fn check_pump_level(increase: f64) -> u8 {
//     (increase / SPECIAL_PERCENTAGE) as u8
// }
