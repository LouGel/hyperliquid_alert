// src/hyperliquid/fetch_price.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::global_data::TokenMapping;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub name: String,
    pub full_name: Option<String>,
    pub price: f64,
    pub price_prev_24h: f64,
    pub pair_number: Option<u16>,
    pub market_cap: u32,
}

pub async fn fetch_token_data() -> anyhow::Result<(TokenMapping, Vec<String>)> {
    // Initialize the HTTP client
    let client = Client::new();

    // Send the POST request
    let (meta, market_data_array) = client
        .post("https://api.hyperliquid.xyz/info")
        .json(&serde_json::json!({"type": "spotMetaAndAssetCtxs"}))
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;

    // Parse 'tokens' array
    let tokens_array = meta.tokens;
    let universe = meta.universe;

    // Map from index to Token

    let mut token_array: Vec<String> = Vec::new();
    let mut index_to_token_item: HashMap<usize, Token> = HashMap::new();
    for token in tokens_array {
        token_array.push(token.name.to_uppercase());
        index_to_token_item.insert(token.index, token);
    }

    // Build the mapping and the array
    let mut token_mapping: TokenMapping = HashMap::new();

    for market_data_value in market_data_array.iter() {
        let market_data_item: MarketDataItem = market_data_value.to_owned();

        if let Some(pair) = universe.iter().find(|x| x.name == market_data_item.coin) {
            let index = pair
                .tokens
                .iter()
                .find(|&&x| x != 0)
                .expect("Canno0t find this issue");
            let index = index.to_owned() as usize;

            if let Some(token_item) = index_to_token_item.get(&index) {
                let name_upper = token_item.name.to_uppercase();
                let full_name = token_item.full_name.clone();
                let price: f64 = market_data_item.mark_px.parse()?;
                let price_prev_24h: f64 = market_data_item.prev_day_px.parse()?;
                let pair_number = transform_coin_to_pair_no(&market_data_item.coin);
                let market_cap =
                    (price * market_data_item.circulating_supply.parse::<f64>()?).round() as u32;

                // Create the TokenInfo object
                let token_info = TokenInfo {
                    name: token_item.name.clone(),
                    full_name,
                    price,
                    price_prev_24h,
                    pair_number,
                    market_cap,
                };

                // Insert into the mapping
                token_mapping.insert(name_upper, token_info.clone());
            }
        } else {
            error!("Pair not found foe {:?}", market_data_value)
        }
    }

    // Return the mapping and array as a tuple
    Ok((token_mapping, token_array))
}

fn transform_coin_to_pair_no(input: &str) -> Option<u16> {
    match input {
        "PURR/USDC" => Some(10_000),
        _ => Some(
            10_000
                + input[1..]
                    .parse::<u16>()
                    .expect(&format!("ERROR IN INPUT transform_coin_to_pair_no{input}")),
        ),
    }
}

type ApiResponse = (ResponseMeta, Vec<MarketDataItem>);
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarketDataItem {
    pub prev_day_px: String,
    pub day_ntl_vlm: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub circulating_supply: String,
    pub coin: String,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    pub universe: Vec<UniverseItem>,
    pub tokens: Vec<Token>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub name: String,
    pub sz_decimals: u32,
    pub wei_decimals: u32,
    pub index: usize,
    pub token_id: String,
    pub is_canonical: bool,
    pub evm_contract: Option<String>,
    pub full_name: Option<String>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseItem {
    pub tokens: Vec<i32>,
    pub name: String,
    pub index: i32,
    pub is_canonical: bool,
}
