use crate::{db::services::chat::insert_chat, global_data::MY_ID};
use teloxide::prelude::*;

pub async fn handle_new_chat_members(bot: Bot, msg: &Message) -> anyhow::Result<Option<()>> {
    if let Some(new_members) = msg.new_chat_members() {
        // Check if the bot is among the new members
        let bot_was_added = new_members.iter().any(|user| user.id == *MY_ID);

        if bot_was_added {
            // The bot was added to a group
            let chat_id = msg.chat.id;
            insert_chat(chat_id.0).await?;

            // Send a welcome message to the group
            bot.send_message(chat_id, INVITED_MESSAGE).await?;

            // Perform any additional initialization here
            info!("Bot added to group: {:?}", chat_id);
            return Ok(Some(()));
        }
    }
    return Ok(None);
}

pub const INVITED_MESSAGE: &str =
    "Hello everyone! I'm your friendly bot 🤖. Thanks for adding me to the group!";
