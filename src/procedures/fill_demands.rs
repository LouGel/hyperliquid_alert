use teloxide::types::{ChatId, MessageId, ThreadId};

use crate::{
    bot::{send_error, send_error_to_moderator, send_message},
    db::services::{
        demands::{batch_fetch_last_demands_by_time, Demand},
        tokens::{fetch_latest_tokens_at, TokensAt},
    },
    global_data::{TokenMapping, REFERRAL_LINK},
    hyperliquid::fetch_price::TokenInfo,
};
// use anyhow::anyhow;

const DEMAND_ERR_HEADER: &str = "Error satisfying demand for :";

pub async fn execute_demands(tokens_at: TokensAt) {
    match batch_fetch_last_demands_by_time(tokens_at.times.clone()).await {
        Err(e) => {
            error!("ERROOR for batch_fetch_last_demands_by_time");
            return send_error_to_moderator(format!(
                "Error durin getting map demand {}",
                e.to_string()
            ));
        }
        Ok(demand_map) => {
            debug!("Satisfying time");
            let mut err_stack = DEMAND_ERR_HEADER.to_owned();
            for time in tokens_at.times {
                if let Some(demands) = demand_map.get(&time) {
                    if let Err(e) =
                        satisfy_regular_demands_at(demands.to_owned(), &time, &tokens_at.tokens)
                            .await
                    {
                        err_stack.push_str(&format!("{:?}{:?}", time, e));
                    }
                }
            }
            if err_stack != DEMAND_ERR_HEADER {
                send_error_to_moderator(err_stack);
            }
        }
    }
}

pub async fn satisfy_regular_demands_at(
    demands: Vec<Demand>,
    time: &str,
    tokens_now: &TokenMapping,
) -> anyhow::Result<()> {
    debug!("Satisfying demand {:#?}", demands);
    if demands.len() > 0 {
        if let Some(tokens_at) = fetch_latest_tokens_at(time).await? {
            debug!("fetched last token at {time}");
            for demand in demands {
                let token = demand
                    .token
                    .clone()
                    // .expect("No token in demand")
                    .to_uppercase();

                if let Some(new_token) = tokens_now.get(&token) {
                    let latest = tokens_at.tokens.get(&token).unwrap();
                    process_tokens(demand, new_token, latest)?;
                } else {
                    debug!("Nothing for {token}");
                    send_error(
                        ChatId(demand.chat_id),
                        "No pair for {token} ",
                        demand.thread_id.map(|id| ThreadId(MessageId(id))),
                    );
                }
            }
        } else {
            info!("No last token infos for {time}")
        };
    } else {
        info!("No standard alert for now");
    }
    Ok(())
}
fn process_tokens(demand: Demand, new: &TokenInfo, previous: &TokenInfo) -> anyhow::Result<()> {
    let diff_wanted = demand.percentage;
    debug!("Diff wanted {diff_wanted}, for demand {}", demand.chat_id);
    let diff = (new.price - previous.price) / previous.price * 100_f64;
    if diff.abs() < 0.01 || diff.abs() < diff_wanted as f64 {
        // debug!("price token info{:?} no dif", new.full_name);
        Ok(())
    } else {
        let msg = format_dif_message(
            new.price,
            diff,
            new.pair_number.unwrap_or_default(),
            &demand.interval,
            &new.name,
        );
        debug!("Sending for demand{} dif", msg);
        let chat_id = ChatId(demand.chat_id);
        send_message(
            chat_id,
            &msg,
            demand.thread_id.map(|id| ThreadId(MessageId(id))),
        );
        Ok(())
    }
}

//
fn format_dif_message(price: f64, diff: f64, pair_no: u16, time: &str, name: &str) -> String {
    let movement = if diff <= 0.0 { "dropped" } else { "risen" };
    let diff = format!("{:.2}", diff);
    format!(
        "__*📈 WAGMI Alert*__:\n[{}]({}{}) has {} by {}% in the last {} : {}$",
        name, REFERRAL_LINK, pair_no, movement, diff, time, price
    )
}
