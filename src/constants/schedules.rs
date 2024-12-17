pub const INTERVAL_15MIN: &str = "15min";
pub const INTERVAL_HOURLY: &str = "1h";
pub const INTERVAL_6HOUR: &str = "6h";
pub const INTERVAL_24HOUR: &str = "24h";
pub const INTERVAL_WEDNESDAY: &str = "wed";
pub const INTERVAL_FRIDAY: &str = "fri";
pub const INTERVAL_MONDAY: &str = "mon";
pub const INTERVAL_SATURDAY: &str = "sat";

// Cron expressions
pub const CRON_15MIN: &str = "0 */15 * * * *"; // Every 15 minutes
pub const CRON_HOURLY: &str = "0 0 * * * *"; // Every hour
pub const CRON_6HOUR: &str = "0 0 */6 * * *"; // Every 6 hours
pub const CRON_24HOUR: &str = "0 0 15 * * *"; // Every 6 hours
pub const CRON_WEDNESDAY: &str = "0 0 12 * * Wed"; // Wednesday at noon
pub const CRON_FRIDAY: &str = "0 0 12 * * Fri"; // Friday at noon
pub const CRON_MONDAY: &str = "0 0 12 * * Mon"; // Monday at noon
pub const CRON_SATURDAY: &str = "0 0 12 * * Sat"; // Saturday at noon

// All valid intervals with their cron expressions
pub static INTERVALS: &[(&str, &str)] = &[
    (INTERVAL_15MIN, CRON_15MIN),
    (INTERVAL_HOURLY, CRON_HOURLY),
    (INTERVAL_6HOUR, CRON_6HOUR),
    (INTERVAL_24HOUR, CRON_24HOUR),
    (INTERVAL_WEDNESDAY, CRON_WEDNESDAY),
    (INTERVAL_FRIDAY, CRON_FRIDAY),
    (INTERVAL_MONDAY, CRON_MONDAY),
    (INTERVAL_SATURDAY, CRON_SATURDAY),
];
// const _CRON_15SEC_NAME: &str = "15sec";
// const _CRON_30SEC_NAME: &str = "30sec";
// const _CRON_1MIN_NAME: &str = "1min";
// const _CRON_15SEC_EXPR: &str = "*/15 * * * * *"; // Every 15 seconds
// const _CRON_30SEC_EXPR: &str = "*/30 * * * * *"; // Every 30 seconds
// const _CRON_1MIN_EXPR: &str = "0 * * * * *"; // Every minute
/// Parse interval string to standardized format
pub fn parse_interval(input: &str) -> Option<&'static str> {
    match input.trim().to_lowercase().as_str() {
        "15m" | "15min" | "quarter" => Some(INTERVAL_15MIN),
        "1h" | "hourly" | "hour" => Some(INTERVAL_HOURLY),
        "6h" | "6hour" => Some(INTERVAL_6HOUR),
        "24h" | "24hour" | "daily" => Some(INTERVAL_24HOUR),
        "wed" | "wednesday" => Some(INTERVAL_WEDNESDAY),
        "fri" | "friday" => Some(INTERVAL_FRIDAY),
        "mon" | "monday" => Some(INTERVAL_MONDAY),
        "sat" | "saturday" => Some(INTERVAL_SATURDAY),
        _ => None,
    }
}
