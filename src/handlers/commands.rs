use crate::{
    bot::{send_error, send_error_to_moderator, send_message},
    constants::schedules::parse_interval,
    db::services::demands::{
        delete_demands_for_chat, get_demands_by_chat_id, send_demands_for, Demand,
    },
    global_data::{get_amount_from_map_for_chat_id, get_bot, get_token_array},
    types::commands::{parse_alert, switch_type, Command, ALERT, SPECIAL},
};
use anyhow::anyhow;
use log::{debug, error, info};
use teloxide::{
    prelude::*,
    types::{ThreadId, User},
};

const ADMIN_CHAT_ID: i64 = 2171722969;

pub async fn commands_handler(_: Bot, message: Message, command: Command) -> anyhow::Result<()> {
    // Early returns for auth checks
    if check_if_from_admin(message.clone(), None).await?.is_none() {
        return Ok(());
    }
    debug!("Asked");
    let _ = verify_user(&message)?;
    let chat_id = message.chat.id;
    let thread_id = message.thread_id;

    let result = match command {
        Command::Free => handle_free_command(chat_id).await,
        Command::Demands => handle_demands_command(chat_id, thread_id).await,
        Command::SetAlert { str } => handle_set_alert(chat_id, thread_id, str).await,
        Command::Special { switch } => handle_special_command(chat_id, thread_id, switch).await,
        Command::Help => Ok(HELP_MESSAGE.to_string()),
    };

    match result {
        Ok(message) if !message.is_empty() => {
            send_message(chat_id, &message, thread_id);
            Ok(())
        }
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Command error: {:?}", err);
            send_error(chat_id, &err.to_string(), thread_id);
            Err(err)
        }
    }
}

fn verify_user(message: &Message) -> anyhow::Result<()> {
    if let Some(user) = &message.from {
        if user.is_bot {
            return Err(anyhow!("DANGER: Bot infiltration"));
        }
        debug!(
            "User: {} {} {} {}",
            user.first_name,
            user.last_name.as_ref().unwrap_or(&"".to_string()),
            user.username.as_ref().unwrap_or(&"".to_string()),
            user.id
        );
        Ok(())
    } else {
        Ok(())
    }
}

async fn handle_free_command(chat_id: ChatId) -> anyhow::Result<String> {
    delete_demands_for_chat(chat_id.0)
        .await
        .map(|_| {
            info!("Deleted alerts for chat {}", chat_id.0);
            "All your alerts have been deleted".to_string()
        })
        .map_err(|e| {
            error!("Failed to delete alerts: {:?}", e);
            anyhow!("Failed to delete alerts. Please try again later")
        })
}

async fn handle_demands_command(
    chat_id: ChatId,
    thread_id: Option<ThreadId>,
) -> anyhow::Result<String> {
    match get_demands_by_chat_id(chat_id.0).await {
        Ok(demands) => {
            send_demands_for(chat_id, thread_id, demands);
            Ok("".to_string())
        }
        Err(e) => {
            send_error_to_moderator(e.to_string());
            Err(anyhow!("Failed to fetch demands. Please try again later"))
        }
    }
}

async fn handle_set_alert(
    chat_id: ChatId,
    thread_id: Option<ThreadId>,
    alert: String,
) -> anyhow::Result<String> {
    check_demand(&chat_id).await?;
    let (token, interval, percentage) = parse_alert(alert)?;

    let token = token.to_ascii_uppercase();
    let token_array = get_token_array().await;

    if !token_array.contains(&token) {
        return Err(anyhow!("Token '{}' doesn't exist", token));
    }

    let standardized_interval =
        parse_interval(&interval).ok_or_else(|| anyhow!("Invalid interval: {}", interval))?;

    let demand = Demand {
        chat_id: chat_id.0,
        thread_id: thread_id.map(|id| id.0 .0),
        type_of: ALERT.to_owned(),
        token: token.clone(),
        percentage,
        interval: standardized_interval.to_string(),
    };

    demand.insert_to_db().await?;

    let mut message = format!(
        "Alert set for token {} at interval {}",
        token, standardized_interval
    );
    if percentage != 0 {
        message += &format!(" for percentage {}", percentage);
    }
    Ok(message)
}

async fn handle_special_command(
    chat_id: ChatId,
    thread_id: Option<ThreadId>,
    switch: String,
) -> anyhow::Result<String> {
    match switch_type(switch) {
        Ok(push) => {
            let demand = Demand::new(chat_id.0, SPECIAL.to_owned(), thread_id.map(|id| id.0 .0));
            if push {
                check_demand(&chat_id).await?;
                demand.insert_to_db().await?;
                Ok("Pump alert set for this channel".to_string())
            } else {
                demand.delete_demand().await?;
                Ok("Pump alert  suppressed for this channel".to_string())
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn check_if_from_admin(
    message: Message,
    compare_id: Option<User>,
) -> anyhow::Result<Option<()>> {
    let user = match compare_id {
        Some(user) => Some(user),
        None => message.from,
    };

    if let Some(user) = user {
        let bot = get_bot();
        let admins = bot.get_chat_administrators(message.chat.id).await?;
        debug!("Admins: {:?}", admins);

        if admins.iter().any(|admin| admin.user.id == user.id) {
            return Ok(Some(()));
        }
    }
    Ok(None)
}

pub async fn check_demand(chat_id: &ChatId) -> anyhow::Result<()> {
    let current_count = get_amount_from_map_for_chat_id(chat_id.0).await;
    debug!(
        "Checking demand for chat {}: count = {}",
        chat_id.0, current_count
    );

    if chat_id.0 != ADMIN_CHAT_ID && current_count >= 3 {
        Err(anyhow!("Max demand reached. Free the demands or erase one"))
    } else {
        Ok(())
    }
}

pub const HELP_MESSAGE: &str = "🤖 __*Wagmi Alert Bot*__\n\
\n\
__*Commands:*__\n\
- `/free` → Delete all alerts\n\
- `/special` → \\(on/start\\)/\\(off/stop\\)  erase or activate pump alert\n\
- `/demands` → Show all our alerts/special. Click to erase one\n\
- `/setalert \\[TOKEN\\] \\[INTERVAL\\] Optional<PERCENTAGE>` → Set alert for token  \n\
\n\
__*Intervals:*__\n\
- 15min/15m → Every 15 minutes\n\
- 1h/hourly → Every hour\n\
- 6h → Every 6 hours\n\
- 24h/daily → Every afternoon \\(15utc\\) \n\
- mon/wed/fri/sat → Respective day at 12:00 UTC\n\
\n\
__*Examples:*__\n\
`/setalert WAGMI 15min 3` → Alert on 3% WAGMI changes\n\
`/setalert WAGMI 1h` → Track all WAGMI price updates in 1H\n\
\n\
__*Note:*__ Only admins can use commands. Set percentage to 0 or omit for all price updates.";
