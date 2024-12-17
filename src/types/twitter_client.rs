// use anyhow::{Context, Result};
// use base64::encode;
// use reqwest::{header, Client};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize)]
// struct TweetRequest {
//     text: String,
// }

// #[derive(Debug, Deserialize)]
// pub struct TweetResponse {
//     pub data: TweetData,
// }

// #[derive(Debug, Deserialize)]
// struct TweetData {
//     pub id: String,
//     pub text: String,
// }

// #[derive(Debug, Deserialize)]
// struct OAuth2Token {
//     pub access_token: String,
//     pub token_type: String,
//     pub expires_in: i32,
// }

// #[derive(Debug)]
// pub struct TwitterBot {
//     client: Client,
//     client_id: String,
//     client_secret: String,
//     access_token: Option<String>,
// }

// impl TwitterBot {
//     pub fn new(client_id: String, client_secret: String) -> Self {
//         let client = Client::new();
//         Self {
//             client,
//             client_id,
//             client_secret,
//             access_token: None,
//         }
//     }

//     /// Get OAuth 2.0 Bearer Token
//     pub async fn authenticate(&mut self) -> Result<()> {
//         let url = "https://api.twitter.com/2/oauth2/token";

//         // Create Basic Auth header from client_id and client_secret
//         let auth_str = format!("{}:{}", self.client_id, self.client_secret);
//         let auth_header = format!("Basic {}", encode(auth_str.as_bytes()));

//         let mut headers = header::HeaderMap::new();
//         headers.insert(
//             header::AUTHORIZATION,
//             header::HeaderValue::from_str(&auth_header)?,
//         );
//         headers.insert(
//             header::CONTENT_TYPE,
//             header::HeaderValue::from_static("application/x-www-form-urlencoded"),
//         );

//         // Request body parameters
//         let params = [("grant_type", "client_credentials")];

//         let response = self
//             .client
//             .post(url)
//             .headers(headers)
//             .form(&params)
//             .send()
//             .await
//             .context("Failed to send authentication request")?;

//         if !response.status().is_success() {
//             let error_text = response.text().await?;
//             anyhow::bail!("Twitter OAuth2 error: {}", error_text);
//         }

//         let token = response
//             .json::<OAuth2Token>()
//             .await
//             .context("Failed to parse OAuth2 token response")?;

//         self.access_token = Some(token.access_token);
//         Ok(())
//     }

//     /// Post a tweet using Twitter API v2
//     pub async fn post_tweet(&self, message: &str) -> Result<TweetResponse> {
//         let access_token = self
//             .access_token
//             .as_ref()
//             .ok_or_else(|| anyhow::anyhow!("Not authenticated. Call authenticate() first"))?;

//         let url = "https://api.twitter.com/2/tweets";

//         let mut headers = header::HeaderMap::new();
//         headers.insert(
//             header::AUTHORIZATION,
//             header::HeaderValue::from_str(&format!("Bearer {}", access_token))?,
//         );
//         headers.insert(
//             header::CONTENT_TYPE,
//             header::HeaderValue::from_static("application/json"),
//         );

//         let tweet = TweetRequest {
//             text: message.to_string(),
//         };

//         let response = self
//             .client
//             .post(url)
//             .headers(headers)
//             .json(&tweet)
//             .send()
//             .await
//             .context("Failed to send tweet request")?;

//         if !response.status().is_success() {
//             let error_text = response.text().await?;
//             anyhow::bail!("Twitter API error: {}", error_text);
//         }

//         let tweet_response = response
//             .json::<TweetResponse>()
//             .await
//             .context("Failed to parse tweet response")?;

//         Ok(tweet_response)
//     }

//     /// Check if the bot is authenticated
//     pub fn is_authenticated(&self) -> bool {
//         self.access_token.is_some()
//     }
// }

// /// Example of rate limiting and retry logic
// pub mod utils {
//     use super::TwitterBot;
//     use anyhow::Result;
//     use std::{thread, time::Duration};

//     pub async fn post_tweet_with_retry(
//         bot: &TwitterBot,
//         message: &str,
//         max_retries: u32,
//         delay_ms: u64,
//     ) -> Result<()> {
//         let mut attempts = 0;

//         while attempts < max_retries {
//             match bot.post_tweet(message).await {
//                 Ok(response) => {
//                     println!("Tweet posted successfully! ID: {}", response.data.id);
//                     return Ok(());
//                 }
//                 Err(e) => {
//                     attempts += 1;
//                     if attempts == max_retries {
//                         return Err(e);
//                     }
//                     eprintln!("Attempt {} failed: {}. Retrying...", attempts, e);
//                     thread::sleep(Duration::from_millis(delay_ms));
//                 }
//             }
//         }

//         Ok(())
//     }
// }

// // Tests
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_authentication() {
//         let client_id = env::var("TWITTER_CLIENT_ID").unwrap();
//         let client_secret = env::var("TWITTER_CLIENT_SECRET").unwrap();

//         let mut bot = TwitterBot::new(client_id, client_secret);
//         assert!(!bot.is_authenticated());

//         bot.authenticate().await.unwrap();
//         assert!(bot.is_authenticated());
//     }
// }
