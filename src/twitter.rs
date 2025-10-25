//! Twitter/X API integration module.
//!
//! This module contains functions for interacting with the Twitter/X API,
//! including posting tweets using OAuth 2.0 User Context authentication.

use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::TwitterConfig;
use crate::oauth::build_oauth2_user_context_header;

/// Makes an authenticated request to the Twitter API with automatic token refresh on 401 errors.
///
/// This helper function handles the common pattern of making authenticated requests to the Twitter API
/// and automatically refreshing the access token if a 401 Unauthorized response is received.
///
/// # Parameters
///
/// - `config`: Mutable reference to TwitterConfig (may be updated with new token)
/// - `request_builder`: A configured reqwest::RequestBuilder ready to send
/// - `operation_name`: Human-readable name for the operation (for logging)
///
/// # Returns
///
/// - `Ok(String)`: The API response body on success
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the request fails or token refresh fails
async fn make_authenticated_request(
    config: &mut TwitterConfig,
    request_builder: reqwest::RequestBuilder,
    operation_name: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Making authenticated request for operation: {}", operation_name);
    
    // First attempt with current token
    let response = request_builder.try_clone()
        .ok_or("Failed to clone request builder")?
        .send()
        .await?;
    
    let status = response.status();
    info!("Received response with status: {} for operation: {}", status, operation_name);
    
    if status.is_success() {
        let response_text = response.text().await?;
        info!("Operation '{}' completed successfully", operation_name);
        debug!("Response body for '{}': {}", operation_name, response_text);
        return Ok(response_text);
    }
    
    // Handle 401 Unauthorized - token might be expired
    if status == 401 {
        warn!("Received 401 Unauthorized for operation '{}' - access token may be expired", operation_name);
        
        if config.can_refresh_token() {
            info!("Attempting automatic token refresh for operation '{}'", operation_name);
            
            match config.refresh_access_token().await {
                Ok(_) => {
                    info!("Token refreshed successfully, retrying operation '{}'", operation_name);
                    
                    // Retry the request with the new token
                    let new_auth_header = build_oauth2_user_context_header(&config.access_token);
                    
                    // Rebuild the request with the new authorization header
                    let retry_response = request_builder
                        .header("Authorization", new_auth_header)
                        .send()
                        .await?;
                    
                    let retry_status = retry_response.status();
                    info!("Retry response status: {} for operation '{}'", retry_status, operation_name);
                    
                    if retry_status.is_success() {
                        let response_text = retry_response.text().await?;
                        info!("Operation '{}' completed successfully after token refresh", operation_name);
                        debug!("Response body for '{}' (after refresh): {}", operation_name, response_text);
                        return Ok(response_text);
                    } else {
                        let error_text = retry_response.text().await?;
                        error!("Operation '{}' failed after token refresh - Status: {}, Response: {}", operation_name, retry_status, error_text);
                        return Err(format!("Twitter API error after token refresh ({}): {}", retry_status, error_text).into());
                    }
                }
                Err(e) => {
                    error!("Token refresh failed for operation '{}': {}", operation_name, e);
                    return Err(format!("Token refresh failed for operation '{}': {}", operation_name, e).into());
                }
            }
        } else {
            error!("Cannot refresh token for operation '{}' - missing refresh credentials", operation_name);
            let error_text = response.text().await?;
            return Err(format!("Twitter API error (401) for operation '{}' and token refresh not available: {}", operation_name, error_text).into());
        }
    }
    
    // Handle other error status codes
    let error_text = response.text().await?;
    error!("Operation '{}' failed - Status: {}, Response: {}", operation_name, status, error_text);
    Err(format!("Twitter API error for operation '{}' ({}): {}", operation_name, status, error_text).into())
}

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
/// The following environment variables must be set:
/// - `xapi_access_token` (OAuth 2.0 User Context Access Token for posting tweets)
///
/// # Example
///
/// ```rust
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

    // Load Twitter API credentials from environment variables
    info!("Loading Twitter configuration from environment variables");
    let mut config = TwitterConfig::from_env()?;
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
    info!("Building OAuth 2.0 User Context authorization header");
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
    make_authenticated_request(&mut config, request_builder, "post_tweet").await
}

/// Searches for tweets with a specific hashtag in the past hour.
///
/// This function uses the Twitter API v2 search endpoint to find tweets containing
/// the specified hashtag that were posted within the last hour. It uses OAuth 2.0
/// User Context Access Token authentication for v2 endpoints. It logs all found
/// tweets to the application logs.
///
/// # Parameters
///
/// - `hashtag`: The hashtag to search for (without the # symbol)
///
/// # Returns
///
/// - `Ok(())`: If the search completed successfully (regardless of results)
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following environment variables must be set:
/// - `xapi_access_token` (OAuth 2.0 User Context Access Token for v2 endpoints)
///
/// # Example
///
/// ```rust
/// use reputest::search_tweets_with_hashtag;
///
/// #[tokio::main]
/// async fn main() {
///     let result = search_tweets_with_hashtag("gmgv").await;
///     match result {
///         Ok(_) => println!("Search completed successfully"),
///         Err(e) => eprintln!("Failed to search tweets: {}", e),
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
pub async fn search_tweets_with_hashtag(
    hashtag: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting tweet search operation for hashtag: '{}'", hashtag);

    // Post a tweet with "loggerman"
    info!("Posting test tweet with 'loggerman'");
    match post_tweet("loggerman").await {
        Ok(response) => info!("Posted 'loggerman' tweet successfully: {}", response),
        Err(e) => error!("Failed to post 'loggerman' tweet: {}", e),
    }

    // Load Twitter API credentials from environment variables
    info!("Loading Twitter configuration from environment variables for search");
    let mut config = TwitterConfig::from_env()?;
    debug!("Twitter config loaded successfully for search");
    let client = Client::new();

    // Calculate the timestamp for 1 hour ago
    let one_hour_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 3600; // 3600 seconds = 1 hour

    // Build the search query with hashtag and time filter
    let query = format!("#{}", hashtag);
    let start_time = chrono::DateTime::from_timestamp(one_hour_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");
    let url = format!(
        "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100",
        urlencoding::encode(&query),
        start_time
    );

    info!("Search URL: {}", url);
    debug!("Search query: {}", query);
    debug!("Start time: {}", start_time);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    info!("Building OAuth 2.0 User Context authorization header for search");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending GET request to Twitter API v2 search endpoint");
    debug!("Request URL: {}", url);
    debug!("Request headers: Authorization: Bearer [REDACTED]");

    // Create the request builder
    let request_builder = client
        .get(&url)
        .header("Authorization", auth_header);

    // Use the authenticated request helper with automatic token refresh
    let response_text = make_authenticated_request(&mut config, request_builder, "search_tweets").await?;
    
    debug!("Search response body: {}", response_text);
    let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

    // Extract tweets from the response
    if let Some(data) = json_response.get("data") {
        if let Some(tweets) = data.as_array() {
            if tweets.is_empty() {
                info!("No tweets found with hashtag #{} in the past hour", hashtag);
            } else {
                info!(
                    "Found {} tweets with hashtag #{} in the past hour:",
                    tweets.len(),
                    hashtag
                );
                for (i, tweet) in tweets.iter().enumerate() {
                    if let Some(text) = tweet.get("text") {
                        if let Some(id) = tweet.get("id") {
                            info!("Tweet {} (ID: {}): {}", i + 1, id, text);
                        } else {
                            info!("Tweet {}: {}", i + 1, text);
                        }
                    }
                }
            }
        } else {
            warn!("Unexpected response format: data is not an array");
        }
    } else {
        info!("No tweets found with hashtag #{} in the past hour", hashtag);
    }

    Ok(())
}
