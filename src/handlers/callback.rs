use anyhow::anyhow;
use teloxide::{types::CallbackQuery, Bot};

use crate::{
    bot::{msg_delete::delete_message, send_error, send_message},
    db::services::demands::{delete_demand_by_composite_id, Demand},
};

use super::commands::check_if_from_admin;

pub async fn callback_handler(_: Bot, q: CallbackQuery) -> anyhow::Result<()> {
    if let Some(maybe_message) = q.message.clone() {
        if let Some(message) = maybe_message.regular_message() {
            let chat_id = message.chat.id;
            let thread_id = message.thread_id;

            let callback_data = q
                .data
                .as_deref()
                .ok_or_else(|| anyhow!("Callback data is empty"))?;

            let opts: Vec<&str> = callback_data.split('_').collect();
            debug!("Callback data {}", callback_data);
            let (chat_id_bis, composite_id) = match (opts.get(0), opts.get(1)) {
                (Some(chat_id_str), Some(composite_id)) => match chat_id_str.parse::<i64>() {
                    Ok(chat_id) => Ok((chat_id, composite_id.to_string())),
                    Err(_) => {
                        let err = format!("Invalid chat_id format: {}", chat_id_str);
                        send_error(chat_id, &err, thread_id);
                        Err(anyhow!(err))
                    }
                },
                _ => {
                    let err = format!("Invalid callback data format: {}", callback_data);
                    send_error(chat_id, &err, thread_id);
                    Err(anyhow!(err))
                }
            }?;

            let (d_chat_id, _, _, _, _) = match Demand::parse_composite_id(&composite_id) {
                Some(parsed) => parsed,
                None => {
                    let err = format!("Failed to parse demand from composite_id: {}", composite_id);
                    send_error(chat_id, &err, thread_id);
                    return Err(anyhow!(err));
                }
            };

            if chat_id.0 != d_chat_id || d_chat_id != chat_id_bis {
                let err = format!("Chat ID mismatch: {} ! {}", chat_id, d_chat_id);
                send_error(chat_id, &err, thread_id);
                return Err(anyhow!(err));
            }
            match check_if_from_admin(message.clone(), Some(q.from)).await {
                Ok(None) => {
                    return Ok(());
                }
                Err(e) => error!("{}", e),
                _ => info!("ok"),
            }
            if let Err(e) = delete_demand_by_composite_id(&composite_id).await {
                let err = format!("Failed to delete demand: {}", e);
                error!("{}", err);
                send_error(chat_id, &err, thread_id);
                return Err(anyhow!(err));
            }
            send_message(chat_id, "Demand erased", thread_id);
            delete_message(message, chat_id);
        } else {
            send_message(maybe_message.chat().id, "Mesage too old", None);
        }
    }
    Ok(())
}
