mod bot;
mod constants;
mod db;
mod global_data;
mod handlers;
mod hyperliquid;
mod init;
mod procedures;

mod types;
#[macro_use]
extern crate log;

use std::env;
use std::sync::Arc;

use dotenv::dotenv;
use global_data::{update_demand_data, update_token_data, BOT};
use handlers::callback::callback_handler;
use handlers::commands::commands_handler;
use handlers::invites::handle_new_chat_members;

use init::init_pool;

use procedures::main::add_main_sequence;
use teloxide::utils::command::BotCommands;
use teloxide::{prelude::*, types::ChatKind};
use tokio_cron_scheduler::JobScheduler;
use types::commands::Command;

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::init_timed();

    info!("Bot instanciation");

    let bot = Bot::from_env();

    let db_url = env::var("DATABASE_URL").unwrap();
    init_pool(db_url).await.expect("Could not init pool");
    // init_bdd::check_and_init_db().await.map_err(|e| {
    //     error!("{:#?}", e);
    //     panic!()
    // });

    let commands = Command::bot_commands();
    bot.set_my_commands(commands).await.unwrap(); // Clone bot when calling methods

    update_token_data().await.expect("Cannot budate token data");
    update_demand_data().await.expect("Cannod fetch demand ata");
    update_demand_data().await.expect("Couldnt fetch map");

    // Create the dependency map teloxide handler
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter(|msg: Message| matches!(msg.chat.kind, ChatKind::Public(_)))
                .branch(teloxide::filter_command::<Command, _>().endpoint(commands_handler))
                .branch(dptree::endpoint(message_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    BOT.set(Arc::new(bot.clone()))
        .expect("Bot est déjà initialisé");
    let scheduler = JobScheduler::new().await.unwrap();
    add_main_sequence(&scheduler).await;
    let scheduler_handle = tokio::spawn(async move {
        scheduler.start().await.unwrap();
    });

    // Start the dispatcher
    Dispatcher::builder(bot, handler)
        .default_handler(|upd| async move {
            warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .build()
        .dispatch()
        .await;

    // Wait for the scheduler task (if necessary)
    scheduler_handle.await.unwrap();
}

pub async fn message_handler(bot: Bot, msg: Message) -> anyhow::Result<()> {
    handle_new_chat_members(bot, &msg).await?;
    Ok(())
}
