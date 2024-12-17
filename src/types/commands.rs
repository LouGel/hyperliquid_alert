use anyhow::anyhow;
use teloxide::utils::command::BotCommands;
pub const SPECIAL: &str = "pumpcheck";
pub const ALERT: &str = "alert";

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    // #[command(description = "Start receiving alerts.")]
    // Start,
    #[command(description = "Free receiving alerts.")]
    Free,

    #[command(description = "Get all our alerts")]
    Demands,

    #[command(description = "Set an alert.", parse_with = "default")]
    SetAlert { str: String },

    #[command(description = "Start or free the pump check.", parse_with = "default")]
    Special { switch: String },

    // #[command(description = "Delete all your alerts.")]
    // DeleteAlerts,
    #[command(description = "Sow explanation")]
    Help,
}

pub fn switch_type(switch: String) -> anyhow::Result<bool> {
    match switch.to_lowercase().as_ref() {
        "off" | "stop" => Ok(false),
        "on" | "start" => Ok(true),
        _ => Err(anyhow!("/special on or off")),
    }
}

const SPECIAL_PARSE_ERR:&str = "- `\n/setalert \\[TOKEN\\] \\[INTERVAL\\] Otional<\\PERCENTAGE\\>`\n → '/help for interval list'\n";
pub fn parse_alert(input: String) -> anyhow::Result<(String, String, i16)> {
    let opts: Vec<&str> = input.split_ascii_whitespace().collect();
    if opts.len() > 3 || opts.len() == 0 {
        return Err(anyhow!(SPECIAL_PARSE_ERR));
    }
    let token = opts.get(0).ok_or(anyhow!(SPECIAL_PARSE_ERR))?.to_owned();
    let interval = opts.get(1).ok_or(anyhow!(SPECIAL_PARSE_ERR))?.to_owned();
    let percentage_str = opts.get(2).cloned().unwrap_or("0");
    debug!("{percentage_str}");
    let percentage: i16 = percentage_str
        .parse()
        .map_err(|_| anyhow!("Percentage must be an int"))?;
    Ok((token.to_owned(), interval.to_owned(), percentage))
}
