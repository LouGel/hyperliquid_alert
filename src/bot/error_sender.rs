use rand::Rng;
use teloxide::prelude::*;
use teloxide::types::*;

use crate::constants::moderators::MODERATOR_ID;
use crate::global_data::get_bot;

use super::send_message;

pub fn send_error(chat_id: ChatId, err_msg: &str, thread_id: Option<ThreadId>) {
    let format_err = format!("⚠️ Error: {err_msg}⚠️");

    info!("Handeled Err {format_err}");
    send_message(chat_id, &format_err, thread_id);
}
// pub fn send_alert(callback_id: String, err_msg: &str) {
//     let bot = get_bot();
//     let format_err = format!("⚠️{err_msg}⚠️");
//     tokio::spawn(async move {
//         let _ = bot
//             .answer_callback_query(callback_id)
//             .show_alert(true)
//             .text(format_err)
//             .send()
//             .await;
//     });
// }

pub fn send_unexpected_error(user: &UserId, error: String) {
    let mut rng = rand::thread_rng();
    let aleatory: u64 = rng.gen();
    let format_err = format!(
        "⚠️ Unexpected Error: Retry or ask suppor with ref : {}:{:X}⚠️",
        user, aleatory
    );

    error!("Error no: {:X} for {user}. Value : \n {error}", aleatory);
    let bot = get_bot();
    let user_id = user.clone();

    tokio::spawn(async move {
        let _ = bot
            .send_message(user_id, format_err)
            .parse_mode(ParseMode::Html)
            .await
            .map_err(|e| error!("Error {}", e));
    });
}
pub fn send_error_to_moderator(error: String) {
    send_unexpected_error(&MODERATOR_ID, error);
}
// pub fn send_unexpected_callback_function_error(user: &User, callback: &str) {
//     let user_id = user.id;
//     let mut rng = rand::thread_rng();
//     let aleatory: u64 = rng.gen();
//     let format_err = format!(
//         "⚠️ Unexpected Error: Might be due to update, free using  older messages and relaunch with /start if it persists , here the ref to send to the support: {}:{:X}⚠️",
//         user_id, aleatory
//     );
//     error!(
//         "Error no: {:X} for {user_id}.  Callback = {callback}",
//         aleatory
//     );
//     let bot = get_bot();
//     let user_id = user.id.clone();

//     tokio::spawn(async move {
//         let _ = bot
//             .send_message(user_id, format_err)
//             .parse_mode(ParseMode::Html)
//             .await
//             .map_err(|e| error!("Error {}", e));
//     });
// }
