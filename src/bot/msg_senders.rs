use futures::stream::{FuturesUnordered, StreamExt};
use teloxide::prelude::*;
use teloxide::types::*;

use crate::bot::utils::parse_msg_for_tg;
use crate::global_data::get_bot;

pub fn send_message_with_button(
    chat_id: ChatId,
    msg_to_send: &str,
    thread_id: Option<ThreadId>,
    keyboard: InlineKeyboardMarkup,
) {
    let msg_to_send = parse_msg_for_tg(msg_to_send.to_owned());
    let bot = get_bot();
    if let Some(id) = thread_id {
        tokio::spawn(async move {
            let _ = bot
                .send_message(chat_id, msg_to_send)
                .parse_mode(ParseMode::MarkdownV2)
                .message_thread_id(id)
                .reply_markup(keyboard)
                .await
                .map_err(|e| error!("Error sending thread message {}", e));
        });
    } else {
        tokio::spawn(async move {
            let _ = bot
                .send_message(chat_id, msg_to_send)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await
                .map_err(|e| error!("Error sending message {}", e));
        });
    }
}

pub fn send_message(chat_id: ChatId, msg_to_send: &str, thread_id: Option<ThreadId>) {
    let msg_to_send = parse_msg_for_tg(msg_to_send.to_owned());
    let bot = get_bot();
    if let Some(id) = thread_id {
        tokio::spawn(async move {
            let _ = bot
                .send_message(chat_id, msg_to_send)
                .parse_mode(ParseMode::MarkdownV2)
                .message_thread_id(id)
                .await
                .map_err(|e| error!("Error sending thread message {}", e));
        });
    } else {
        tokio::spawn(async move {
            let _ = bot
                .send_message(chat_id, msg_to_send)
                .parse_mode(ParseMode::MarkdownV2)
                .await
                .map_err(|e| error!("Error sending message {}", e));
        });
    }
}

pub async fn broadcast_message(chat_ids: Vec<i64>, message: String) -> anyhow::Result<()> {
    let msg_to_send = parse_msg_for_tg(message);
    const MAX_CONCURRENT: usize = 5;
    const RATE_LIMIT_MS: u64 = 50;

    let bot = get_bot();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(RATE_LIMIT_MS));
    let mut tasks = FuturesUnordered::new();

    for chat_id in chat_ids {
        interval.tick().await;

        while tasks.len() >= MAX_CONCURRENT {
            if let Some(result) = tasks.next().await {
                if let Err(e) = result {
                    error!("Message send error: {}", e);
                }
            }
        }

        let bot = bot.clone();
        let message = msg_to_send.clone();
        tasks.push(tokio::spawn(async move {
            bot.send_message(ChatId(chat_id), message)
                .parse_mode(ParseMode::MarkdownV2)
                .await
        }));
    }

    // Wait for remaining tasks
    let mut error = false;
    while let Some(result) = tasks.next().await {
        if let Err(e) = result {
            error = true;
            error!("Message send error: {}", e);
        }
    }
    if !error {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Error broadcasting message"))
    }
}
