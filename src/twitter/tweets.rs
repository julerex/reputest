//! Tweet operations for Twitter API.
//!
//! This module contains functions for posting and replying to tweets
//! using the Twitter API v2.

use log::{debug, info};
use reqwest::Client;
use serde_json::json;

use crate::config::TwitterConfig;
use crate::db;
use crate::oauth::build_oauth2_user_context_header;

use super::api::make_authenticated_request;

/// Posts a tweet to Twitter/X using the API v2 endpoint.
///
/// This function uses OAuth 2.0 User Context authentication to post a tweet
/// to the Twitter/X API v2 endpoint. It builds the proper authorization header
/// and sends the request with the tweet content.
///
/// # Parameters
///
/// - `text`: The text content of the tweet to post
///
/// # Returns
///
/// - `Ok(String)`: The API response body on successful tweet posting
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for posting tweets)
///
/// # Example
///
/// ```rust,no_run
/// use reputest::post_tweet;
///
/// #[tokio::main]
/// async fn main() {
///     let result = post_tweet("Hello from Rust!").await;
///     match result {
///         Ok(response) => println!("Tweet posted: {}", response),
///         Err(e) => eprintln!("Failed to post tweet: {}", e),
///     }
/// }
/// ```
///
/// # Errors
///
/// This function can fail for several reasons:
/// - Missing or invalid Twitter API credentials
/// - Network connectivity issues
/// - Twitter API rate limiting or other API errors
/// - Invalid tweet content (too long, etc.)
pub async fn post_tweet(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting tweet post operation for text: '{}'", text);

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully");

    let client = Client::new();
    let url = "https://api.x.com/2/tweets";
    info!("Target URL: {}", url);

    // Create the tweet payload
    let payload = json!({
        "text": text
    });
    debug!("Tweet payload: {}", serde_json::to_string_pretty(&payload)?);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    debug!("Building OAuth 2.0 User Context authorization header");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending POST request to Twitter API v2");
    debug!("Request URL: {}", url);
    debug!("Request headers: Authorization: Bearer [REDACTED], Content-Type: application/json");
    debug!(
        "Request payload: {}",
        serde_json::to_string_pretty(&payload)?
    );

    // Create the request builder
    let request_builder = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload);

    // Use the authenticated request helper with automatic token refresh
    make_authenticated_request(&mut config, &pool, request_builder, "post_tweet").await
}

/// Replies to a tweet using the Twitter/X API v2 endpoint.
///
/// This function posts a reply to an existing tweet by including the `reply` parameter
/// in the tweet payload. It uses OAuth 2.0 User Context authentication.
///
/// # Parameters
///
/// - `text`: The text content of the reply tweet
/// - `reply_to_tweet_id`: The ID of the tweet to reply to
///
/// # Returns
///
/// - `Ok(String)`: The API response body on successful reply posting
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for posting tweets)
pub async fn reply_to_tweet(
    text: &str,
    reply_to_tweet_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Starting reply operation to tweet {} with text: '{}'",
        reply_to_tweet_id, text
    );

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully");

    let client = Client::new();
    let url = "https://api.x.com/2/tweets";
    info!("Target URL: {}", url);

    // Create the reply payload
    let payload = json!({
        "text": text,
        "reply": {
            "in_reply_to_tweet_id": reply_to_tweet_id
        }
    });
    debug!("Reply payload: {}", serde_json::to_string_pretty(&payload)?);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    debug!("Building OAuth 2.0 User Context authorization header");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending POST request to Twitter API v2 for reply");
    debug!("Request URL: {}", url);
    debug!("Request headers: Authorization: Bearer [REDACTED], Content-Type: application/json");
    debug!(
        "Request payload: {}",
        serde_json::to_string_pretty(&payload)?
    );

    // Create the request builder
    let request_builder = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload);

    // Use the authenticated request helper with automatic token refresh
    make_authenticated_request(&mut config, &pool, request_builder, "reply_to_tweet").await
}
