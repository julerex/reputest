//! Direct message functionality for Twitter API.
//!
//! This module contains functions for searching and replying to direct messages
//! using the Twitter API v2.

use log::{debug, info};
use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::TwitterConfig;
use crate::db;
use crate::oauth::build_oauth2_user_context_header;

use super::api::make_authenticated_request;

/// Searches for direct messages sent to the reputest user in the past 6 hours.
///
/// This function uses the Twitter API v2 DM events endpoint to find DMs that were sent
/// to the @reputest account within the past 6 hours. It returns a vector of tuples containing
/// DM ID, DM text, sender username, and timestamp.
///
/// # Returns
///
/// - `Ok(Vec<(String, String, String, String)>)`: Vector of (dm_id, dm_text, sender_username, created_at) tuples
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for reading DMs)
pub async fn search_direct_messages() -> Result<
    Vec<(String, String, String, String)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    info!("Starting search for direct messages to @reputest in the past hour");

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database for DM search");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully for DM search");

    let client = Client::new();

    // Calculate the timestamp for 6 hours ago
    let six_hours_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 21600; // 21600 seconds = 6 hours

    // Build the search query for DM events
    let start_time = chrono::DateTime::from_timestamp(six_hours_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");

    let url = format!(
        "https://api.x.com/2/dm_events?max_results=100&event_types=MessageCreate&dm_event.fields=id,text,event_type,created_at,sender_id&user.fields=id,username,name&expansions=sender_id&start_time={}",
        start_time
    );

    info!("DM search URL: {}", url);
    debug!("Start time (6 hours ago): {}", start_time);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    debug!("Building OAuth 2.0 User Context authorization header for DM search");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending GET request to Twitter API v2 DM events endpoint");
    debug!("Request URL: {}", url);
    debug!("Request headers: Authorization: Bearer [REDACTED]");

    // Create the request builder
    let request_builder = client.get(&url).header("Authorization", auth_header);

    // Use the authenticated request helper with automatic token refresh
    let response_text =
        make_authenticated_request(&mut config, &pool, request_builder, "search_direct_messages").await?;

    debug!("DM search response body: {}", response_text);
    let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

    // Create a map of user ID to username for quick lookup
    let mut users_username_map = std::collections::HashMap::new();
    if let Some(includes) = json_response.get("includes") {
        if let Some(users) = includes.get("users") {
            if let Some(users_array) = users.as_array() {
                for user in users_array {
                    if let (Some(id), Some(username)) = (
                        user.get("id").and_then(|v| v.as_str()),
                        user.get("username").and_then(|v| v.as_str()),
                    ) {
                        users_username_map.insert(id.to_string(), username.to_string());
                    }
                }
            }
        }
    }

    // Extract DM events from the response
    let mut dms = Vec::new();
    if let Some(data) = json_response.get("data") {
        if let Some(events) = data.as_array() {
            if events.is_empty() {
                info!("No direct messages found in the past 6 hours");
            } else {
                info!(
                    "Found {} direct messages in the past 6 hours:",
                    events.len()
                );
                for (i, event) in events.iter().enumerate() {
                    if let (Some(id), Some(text), Some(created_at), Some(sender_id)) = (
                        event.get("id").and_then(|v| v.as_str()),
                        event.get("text").and_then(|v| v.as_str()),
                        event.get("created_at").and_then(|v| v.as_str()),
                        event.get("sender_id").and_then(|v| v.as_str()),
                    ) {
                        let sender_username = users_username_map
                            .get(sender_id)
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");

                        info!(
                            "DM {} (ID: {}): {} by @{}",
                            i + 1,
                            id,
                            text,
                            sender_username
                        );
                        dms.push((
                            id.to_string(),
                            text.to_string(),
                            sender_username.to_string(),
                            created_at.to_string(),
                        ));
                    }
                }
            }
        } else {
            info!("No direct messages found in the past 6 hours");
        }
    } else {
        info!("No direct messages found in the past 6 hours");
    }

    Ok(dms)
}

/// Replies to a direct message using the Twitter/X API v2 endpoint.
///
/// This function sends a reply to an existing DM conversation by creating a new
/// DM event in the same conversation. It uses OAuth 2.0 User Context authentication.
///
/// # Parameters
///
/// - `text`: The text content of the DM reply
/// - `recipient_id`: The user ID of the recipient to reply to
///
/// # Returns
///
/// - `Ok(String)`: The API response body on successful DM posting
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for sending DMs)
pub async fn reply_to_dm(
    text: &str,
    recipient_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Starting DM reply operation to user {} with text: '{}'",
        recipient_id, text
    );

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully");

    let client = Client::new();
    let url = "https://api.x.com/2/dm_conversations/with/:participant_id/messages";
    let conversation_url = url.replace(":participant_id", recipient_id);
    info!("Target URL: {}", conversation_url);

    // Create the DM payload
    let payload = json!({
        "text": text
    });
    debug!("DM payload: {}", serde_json::to_string_pretty(&payload)?);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    debug!("Building OAuth 2.0 User Context authorization header");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending POST request to Twitter API v2 DM endpoint");
    debug!("Request URL: {}", conversation_url);
    debug!("Request headers: Authorization: Bearer [REDACTED], Content-Type: application/json");
    debug!(
        "Request payload: {}",
        serde_json::to_string_pretty(&payload)?
    );

    // Create the request builder
    let request_builder = client
        .post(conversation_url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload);

    // Use the authenticated request helper with automatic token refresh
    make_authenticated_request(&mut config, &pool, request_builder, "reply_to_dm").await
}

