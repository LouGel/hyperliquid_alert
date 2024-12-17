use crate::bot::send_error_to_moderator;
use crate::constants::schedules::INTERVALS;
// use crate::db::diesel::tokens_at::timestamp_in_min;
use crate::db::services::tokens::TokensAt;
use crate::global_data::{get_last_token_map, update_token_data};
use crate::procedures::fill_demands::execute_demands;
use crate::procedures::pump_alert::check_and_send_pump;
use chrono::prelude::*;
use cron_clock::Schedule;
use tokio::time::{sleep, Duration};

use std::str::FromStr;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn add_main_sequence(scheduler: &JobScheduler) {
    let (_, schedule) = INTERVALS.first().expect("Couldn gain cron expression");

    // Schedule the job
    scheduler
        .add(
            Job::new_async(schedule, move |_uuid, _l| {
                Box::pin(async move { execute_sequence().await })
            })
            .unwrap(),
        )
        .await
        .unwrap();
}

async fn execute_sequence() {
    let now = Utc::now();
    let timestamp_in_min = (now.timestamp() / 60) as i32;

    let mut times: Vec<String> = Vec::new();

    for (name, cron_str) in INTERVALS {
        if is_time_matching(cron_str, now.clone()) {
            times.push(name.to_string());
        }
    }

    info!("Fetching Datas for :Handeling {:?}", times);
    if let Err(e) = update_token_data().await {
        send_error_to_moderator(format!("Error during fetch {:?}", e));
        return;
    }

    info!("Executing check pump");
    check_and_send_pump().await;

    let tokens = get_last_token_map().await;

    let tokens_at = TokensAt {
        tokens,
        times,
        timestamp_in_min,
    };

    info!("Executing regular demand");
    execute_demands(tokens_at.clone()).await;

    info!("Updating database");
    if let Err(_) = tokens_at.insert().await {
        sleep(Duration::from_secs(1)).await;
        if let Err(e) = tokens_at.insert().await {
            send_error_to_moderator(format!("Error pushing in database 2 times{:?}", e));
        }
    }
    info!("SUCCESS");
}

fn is_time_matching(cron_str: &str, now: DateTime<Utc>) -> bool {
    let schedule = match Schedule::from_str(cron_str) {
        Ok(s) => s,
        Err(e) => {
            error!("Invalid cron expression '{}': {}", cron_str, e);
            return false;
        }
    };

    schedule.includes(now)
}
