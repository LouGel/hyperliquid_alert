use teloxide::prelude::*;

use crate::global_data::get_bot;

pub fn delete_message(msg: &Message, chat_id: ChatId) {
    let bot = get_bot(); // no arc as per doc
    let msg_id = msg.id.clone();

    tokio::spawn(async move {
        let _ = bot
            .delete_message(chat_id, msg_id)
            .await
            .map_err(|e| error!("Error {}", e));
    });
}
