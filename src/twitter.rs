//! Twitter/X API integration module.
//!
//! This module contains functions for interacting with the Twitter/X API,
//! including posting tweets using OAuth 2.0 User Context authentication.

use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::TwitterConfig;
use crate::db;
use crate::oauth::build_oauth2_user_context_header;
use sqlx::PgPool;

/// Extracts the first @mention username from a tweet text.
///
/// This function looks for @username patterns in the tweet text and returns
/// the first username found (without the @ symbol). If no mentions are found,
/// it returns None.
///
/// # Parameters
///
/// - `text`: The tweet text to search for mentions
///
/// # Returns
///
/// - `Some(username)`: The first mentioned username if found
/// - `None`: If no mentions are found
fn extract_first_mention(text: &str) -> Option<String> {
    // Use regex to find @mentions (word characters after @)
    let re = regex::Regex::new(r"@(\w+)").ok()?;
    re.find(text)
        .and_then(|mat| mat.as_str().strip_prefix('@'))
        .map(|s| s.to_string())
}

/// Extracts a username mentioned followed by a question mark from tweet text.
///
/// This function looks for patterns like "@username?", "@username ?", "username?", or "username ?" in the tweet text and returns
/// the username (without the @ symbol or ?). If no such pattern is found,
/// it returns None.
///
/// # Parameters
///
/// - `text`: The tweet text to search for usernames followed by ?
///
/// # Returns
///
/// - `Some(username)`: The username if found followed by ?
/// - `None`: If no username followed by ? is found
pub fn extract_mention_with_question(text: &str) -> Option<String> {
    // First try to match @username? or @username ? patterns (with @ symbol)
    let re_with_at = regex::Regex::new(r"@(\w+)\s*\?").ok()?;
    if let Some(mat) = re_with_at.find(text) {
        return mat.as_str()
            .strip_prefix('@')
            .and_then(|s| s.strip_suffix('?'))
            .map(|s| s.trim().to_string());
    }

    // If no @ pattern found, try to match username? or username ? patterns (without @ symbol)
    // But exclude common words and the bot's username to avoid false positives
    let re_without_at = regex::Regex::new(r"([a-zA-Z0-9_]{1,15})\s*\?").ok()?;
    // Find all matches and take the last valid one (to prefer later usernames over excluded words like "reputest")
    let mut valid_username = None;
    for mat in re_without_at.find_iter(text) {
        let match_end = mat.end();

        // Check that the ? is not followed by a word character (to avoid matching "what?" in "what?ever")
        if match_end < text.len() && text.chars().nth(match_end).unwrap().is_alphanumeric() {
            continue;
        }

        let username = mat.as_str()
            .strip_suffix('?')
            .map(|s| s.trim().to_string())?;

        // Exclude common words that might be followed by ? to avoid false positives
        let excluded_words = ["what", "when", "where", "how", "why", "who", "which", "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should", "can", "may", "might", "must", "shall", "reputest"];
        if !excluded_words.contains(&username.to_lowercase().as_str()) {
            valid_username = Some(username);
        }
    }

    valid_username
}

/// Looks up a user by username using the Twitter API v2.
///
/// This function makes a request to the Twitter API to get user information
/// by username, including their ID and other details.
///
/// # Parameters
///
/// - `config`: Mutable reference to TwitterConfig (may be updated with new token)
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `username`: The Twitter username to look up
///
/// # Returns
///
/// - `Ok(Some((user_id, name, created_at)))`: User information if found
/// - `Ok(None)`: If user not found
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the API request fails
async fn lookup_user_by_username(
    config: &mut TwitterConfig,
    pool: &PgPool,
    username: &str,
) -> Result<
    Option<(String, String, chrono::DateTime<chrono::Utc>)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    info!("Looking up user by username: {}", username);

    let client = Client::new();
    let url = format!(
        "https://api.x.com/2/users/by/username/{}?user.fields=id,name,username,created_at",
        username
    );

    let auth_header = build_oauth2_user_context_header(&config.access_token);
    let request_builder = client.get(&url).header("Authorization", auth_header);

    let response_text =
        make_authenticated_request(config, pool, request_builder, "lookup_user").await?;
    let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

    if let Some(data) = json_response.get("data") {
        if let (Some(id), Some(name), Some(created_at_str)) = (
            data.get("id").and_then(|v| v.as_str()),
            data.get("name").and_then(|v| v.as_str()),
            data.get("created_at").and_then(|v| v.as_str()),
        ) {
            match chrono::DateTime::parse_from_rfc3339(created_at_str) {
                Ok(dt) => {
                    let created_at_utc = dt.with_timezone(&chrono::Utc);
                    info!("Found user {}: {} (@{})", id, name, username);
                    return Ok(Some((id.to_string(), name.to_string(), created_at_utc)));
                }
                Err(e) => {
                    error!(
                        "Failed to parse user created_at '{}': {}",
                        created_at_str, e
                    );
                }
            }
        }
    }

    warn!("User {} not found", username);
    Ok(None)
}

/// Makes an authenticated request to the Twitter API with automatic token refresh on 401 errors.
///
/// This helper function handles the common pattern of making authenticated requests to the Twitter API
/// and automatically refreshing the access token if a 401 Unauthorized response is received.
///
/// # Parameters
///
/// - `config`: Mutable reference to TwitterConfig (may be updated with new token)
/// - `pool`: A reference to the PostgreSQL connection pool for saving refreshed tokens
/// - `request_builder`: A configured reqwest::RequestBuilder ready to send
/// - `operation_name`: Human-readable name for the operation (for logging)
///
/// # Returns
///
/// - `Ok(String)`: The API response body on success
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the request fails or token refresh fails
async fn make_authenticated_request(
    config: &mut TwitterConfig,
    pool: &PgPool,
    request_builder: reqwest::RequestBuilder,
    operation_name: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Making authenticated request for operation: {}",
        operation_name
    );

    // First attempt with current token
    let response = request_builder
        .try_clone()
        .ok_or("Failed to clone request builder")?
        .send()
        .await?;

    let status = response.status();
    info!(
        "Received response with status: {} for operation: {}",
        status, operation_name
    );

    if status.is_success() {
        let response_text = response.text().await?;
        info!("Operation '{}' completed successfully", operation_name);
        debug!("Response body for '{}': {}", operation_name, response_text);
        return Ok(response_text);
    }

    // Handle 401 Unauthorized - token might be expired
    if status == 401 {
        warn!(
            "Received 401 Unauthorized for operation '{}' - access token may be expired",
            operation_name
        );

        if config.can_refresh_token() {
            info!(
                "Attempting automatic token refresh for operation '{}'",
                operation_name
            );

            match config.refresh_access_token(pool).await {
                Ok(_) => {
                    info!(
                        "Token refreshed successfully, retrying operation '{}'",
                        operation_name
                    );

                    // Retry the request with the new token
                    let new_auth_header = build_oauth2_user_context_header(&config.access_token);

                    // Rebuild the request with the new authorization header
                    let retry_response = request_builder
                        .header("Authorization", new_auth_header)
                        .send()
                        .await?;

                    let retry_status = retry_response.status();
                    info!(
                        "Retry response status: {} for operation '{}'",
                        retry_status, operation_name
                    );

                    if retry_status.is_success() {
                        let response_text = retry_response.text().await?;
                        info!(
                            "Operation '{}' completed successfully after token refresh",
                            operation_name
                        );
                        debug!(
                            "Response body for '{}' (after refresh): {}",
                            operation_name, response_text
                        );
                        return Ok(response_text);
                    } else {
                        let error_text = retry_response.text().await?;
                        error!(
                            "Operation '{}' failed after token refresh - Status: {}, Response: {}",
                            operation_name, retry_status, error_text
                        );
                        return Err(format!(
                            "Twitter API error after token refresh ({}): {}",
                            retry_status, error_text
                        )
                        .into());
                    }
                }
                Err(e) => {
                    error!(
                        "Token refresh failed for operation '{}': {}",
                        operation_name, e
                    );
                    return Err(format!(
                        "Token refresh failed for operation '{}': {}",
                        operation_name, e
                    )
                    .into());
                }
            }
        } else {
            error!(
                "Cannot refresh token for operation '{}' - missing refresh credentials",
                operation_name
            );
            let error_text = response.text().await?;
            return Err(format!(
                "Twitter API error (401) for operation '{}' and token refresh not available: {}",
                operation_name, error_text
            )
            .into());
        }
    }

    // Handle other error status codes
    let error_text = response.text().await?;
    error!(
        "Operation '{}' failed - Status: {}, Response: {}",
        operation_name, status, error_text
    );
    Err(format!(
        "Twitter API error for operation '{}' ({}): {}",
        operation_name, status, error_text
    )
    .into())
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
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for posting tweets)
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
    info!("Building OAuth 2.0 User Context authorization header");
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

/// Searches for tweets with a specific hashtag in the past 24 hours and saves good vibes data.
///
/// This function uses the Twitter API v2 search endpoint to find tweets containing
/// the specified hashtag that were posted within the past 24 hours. It extracts vibe
/// emitter (poster) and vibe receiver (mentioned user) information and saves it
/// to the database. It uses OAuth 2.0 User Context Access Token authentication for v2 endpoints.
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
/// # Database Operations
///
/// This function saves good vibes data to the `good_vibes` table for each tweet that contains
/// a user mention. The mentioned user becomes the vibe_emitter and the poster becomes the vibe_receiver.
///
/// # Requirements
///
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for v2 endpoints)
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

    // Post a tweet with "loggerman <TIME>"
    let current_time = chrono::Utc::now().format("%H:%M:%S");
    let tweet_text = format!("loggerman {}", current_time);
    info!("Posting test tweet with '{}'", tweet_text);
    match post_tweet(&tweet_text).await {
        Ok(response) => info!("Posted 'loggerman' tweet successfully: {}", response),
        Err(e) => error!("Failed to post 'loggerman' tweet: {}", e),
    }

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database for search");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully for search");
    let client = Client::new();

    // Calculate the timestamp for 24 hours ago
    let twenty_four_hours_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 86400; // 86400 seconds = 24 hours

    // Build the search query with hashtag and time filter
    let query = format!("#{}", hashtag);
    let start_time = chrono::DateTime::from_timestamp(twenty_four_hours_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");
    let url = format!(
        "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id&user.fields=id,username,name,created_at&tweet.fields=created_at",
        urlencoding::encode(&query),
        start_time
    );

    info!("Search URL: {}", url);
    debug!("Search query: {}", query);
    debug!("Start time (24 hours ago): {}", start_time);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    info!("Building OAuth 2.0 User Context authorization header for search");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending GET request to Twitter API v2 search endpoint");
    debug!("Request URL: {}", url);
    debug!("Request headers: Authorization: Bearer [REDACTED]");

    // Create the request builder
    let request_builder = client.get(&url).header("Authorization", auth_header);

    // Use the authenticated request helper with automatic token refresh
    let response_text =
        make_authenticated_request(&mut config, &pool, request_builder, "search_tweets").await?;

    debug!("Search response body: {}", response_text);
    let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

    // Create maps of user ID to user info for quick lookup
    let mut users_username_map = std::collections::HashMap::new();
    let mut users_name_map = std::collections::HashMap::new();
    let mut users_created_at_map = std::collections::HashMap::new();
    if let Some(includes) = json_response.get("includes") {
        if let Some(users) = includes.get("users") {
            if let Some(users_array) = users.as_array() {
                for user in users_array {
                    if let (Some(id), Some(username), Some(name), Some(created_at_str)) = (
                        user.get("id"),
                        user.get("username"),
                        user.get("name"),
                        user.get("created_at"),
                    ) {
                        if let (
                            Some(id_str),
                            Some(username_str),
                            Some(name_str),
                            Some(created_at_str),
                        ) = (
                            id.as_str(),
                            username.as_str(),
                            name.as_str(),
                            created_at_str.as_str(),
                        ) {
                            users_username_map.insert(id_str.to_string(), username_str.to_string());
                            users_name_map.insert(id_str.to_string(), name_str.to_string());

                            // Parse and store created_at timestamp
                            match chrono::DateTime::parse_from_rfc3339(created_at_str) {
                                Ok(dt) => {
                                    let created_at_utc = dt.with_timezone(&chrono::Utc);
                                    users_created_at_map.insert(id_str.to_string(), created_at_utc);

                                    // Save user data to database
                                    if let Err(e) = crate::db::save_user(
                                        &pool,
                                        id_str,
                                        username_str,
                                        name_str,
                                        created_at_utc,
                                    )
                                    .await
                                    {
                                        error!(
                                            "Failed to save user data for {}: {}",
                                            username_str, e
                                        );
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to parse user created_at '{}': {}",
                                        created_at_str, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract tweets from the response
    if let Some(data) = json_response.get("data") {
        if let Some(tweets) = data.as_array() {
            if tweets.is_empty() {
                info!(
                    "No tweets found with hashtag #{} in the past 24 hours",
                    hashtag
                );
            } else {
                info!(
                    "Found {} tweets with hashtag #{} in the past 24 hours:",
                    tweets.len(),
                    hashtag
                );
                for (i, tweet) in tweets.iter().enumerate() {
                    if let Some(text) = tweet.get("text") {
                        if let Some(id) = tweet.get("id") {
                            // Extract created_at timestamp
                            let created_at_str = tweet.get("created_at").and_then(|v| v.as_str());
                            let created_at = if let Some(created_at_str) = created_at_str {
                                // Parse ISO 8601 timestamp from Twitter API
                                match chrono::DateTime::parse_from_rfc3339(created_at_str) {
                                    Ok(dt) => dt.with_timezone(&chrono::Utc),
                                    Err(e) => {
                                        error!(
                                            "Failed to parse created_at '{}': {}",
                                            created_at_str, e
                                        );
                                        continue;
                                    }
                                }
                            } else {
                                error!("Tweet {} missing created_at field", id);
                                continue;
                            };

                            info!("Tweet {} (ID: {}): {}", i + 1, id, text);

                            // Extract poster information
                            let poster_user_id = tweet.get("author_id").and_then(|v| v.as_str());
                            let poster_username =
                                poster_user_id.and_then(|user_id| users_username_map.get(user_id));
                            let poster_name =
                                poster_user_id.and_then(|user_id| users_name_map.get(user_id));

                            // Extract vibe_emitter from @mentions in tweet text
                            let vibe_emitter_username =
                                extract_first_mention(text.as_str().unwrap_or(""));

                            if let (
                                Some(poster_id),
                                Some(poster_username),
                                Some(poster_display_name),
                            ) = (poster_user_id, poster_username, poster_name)
                            {
                                if let Some(vibe_emitter_username) = &vibe_emitter_username {
                                    info!(
                                        "  Poster (vibe receiver): {} (@{})",
                                        poster_display_name, poster_username
                                    );
                                    info!("  Vibe emitter: {}", vibe_emitter_username);

                                    // Look up the emitter user by username
                                    match lookup_user_by_username(
                                        &mut config,
                                        &pool,
                                        vibe_emitter_username,
                                    )
                                    .await
                                    {
                                        Ok(Some((
                                            emitter_user_id,
                                            emitter_name,
                                            emitter_created_at,
                                        ))) => {
                                            // Save emitter user data
                                            if let Err(e) = crate::db::save_user(
                                                &pool,
                                                &emitter_user_id,
                                                vibe_emitter_username,
                                                &emitter_name,
                                                emitter_created_at,
                                            )
                                            .await
                                            {
                                                error!("Failed to save emitter user data: {}", e);
                                            }

                                            // Save good vibes data
                                            if let Err(e) = crate::db::save_good_vibes(
                                                &pool,
                                                id.as_str().unwrap(), // tweet_id
                                                &emitter_user_id, // emitter_id (person who sent good vibes)
                                                poster_id, // sensor_id (person who received good vibes)
                                                created_at, // created_at from tweet
                                            )
                                            .await
                                            {
                                                error!("Failed to save good vibes data: {}", e);
                                            }
                                        }
                                        Ok(None) => {
                                            warn!("Emitter user {} not found, skipping good vibes save", vibe_emitter_username);
                                        }
                                        Err(e) => {
                                            error!(
                                                "Failed to lookup emitter user {}: {}",
                                                vibe_emitter_username, e
                                            );
                                        }
                                    }
                                }
                            }
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
        info!(
            "No tweets found with hashtag #{} in the past 24 hours",
            hashtag
        );
    }

    Ok(())
}

/// Searches for mentions of the reputest user in the past 24 hours and returns tweet information.
///
/// This function uses the Twitter API v2 search endpoint to find tweets that mention
/// @reputest and were posted within the past 24 hours. It returns a vector of tuples containing
/// tweet ID, tweet text, author username, and optionally a mentioned user followed by "?".
///
/// # Returns
///
/// - `Ok(Vec<(String, String, String, Option<String>)>)`: Vector of (tweet_id, tweet_text, author_username, mentioned_user) tuples
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following must be available:
/// - Database connection (DATABASE_URL environment variable)
/// - Access token in the `access_tokens` table (OAuth 2.0 User Context Access Token for searching tweets)
pub async fn search_mentions(
) -> Result<Vec<(String, String, String, Option<String>)>, Box<dyn std::error::Error + Send + Sync>>
{
    info!("Starting search for @reputest mentions in the past hour");

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database for mentions search");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully for mentions search");

    let client = Client::new();

    // Calculate the timestamp for 24 hours ago
    let twenty_four_hours_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 86400; // 86400 seconds = 24 hours

    // Build the search query for mentions of @reputest
    let query = "@reputest";
    let start_time = chrono::DateTime::from_timestamp(twenty_four_hours_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");
    let url = format!(
        "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id&user.fields=id,username,name&tweet.fields=created_at,author_id",
        urlencoding::encode(query),
        start_time
    );

    info!("Mentions search URL: {}", url);
    debug!("Search query: {}", query);
    debug!("Start time (24 hours ago): {}", start_time);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    info!("Building OAuth 2.0 User Context authorization header for mentions search");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    // Log request details
    info!("Sending GET request to Twitter API v2 search endpoint for mentions");
    debug!("Request URL: {}", url);
    debug!("Request headers: Authorization: Bearer [REDACTED]");

    // Create the request builder
    let request_builder = client.get(&url).header("Authorization", auth_header);

    // Use the authenticated request helper with automatic token refresh
    let response_text =
        make_authenticated_request(&mut config, &pool, request_builder, "search_mentions").await?;

    debug!("Mentions search response body: {}", response_text);
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

    // Extract tweets from the response
    let mut mentions = Vec::new();
    if let Some(data) = json_response.get("data") {
        if let Some(tweets) = data.as_array() {
            if tweets.is_empty() {
                info!("No mentions of @reputest found in the past 24 hours");
            } else {
                info!(
                    "Found {} mentions of @reputest in the past 24 hours:",
                    tweets.len()
                );
                for (i, tweet) in tweets.iter().enumerate() {
                    if let (Some(text), Some(id), Some(author_id)) = (
                        tweet.get("text").and_then(|v| v.as_str()),
                        tweet.get("id").and_then(|v| v.as_str()),
                        tweet.get("author_id").and_then(|v| v.as_str()),
                    ) {
                        let author_username = users_username_map
                            .get(author_id)
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");

                        // Check if the tweet mentions another user followed by ?
                        let mentioned_user = extract_mention_with_question(text);

                        info!(
                            "Mention {} (ID: {}): {} by @{} (querying: {})",
                            i + 1,
                            id,
                            text,
                            author_username,
                            mentioned_user.as_deref().unwrap_or("none")
                        );
                        mentions.push((
                            id.to_string(),
                            text.to_string(),
                            author_username.to_string(),
                            mentioned_user,
                        ));
                    }
                }
            }
        } else {
            info!("No mentions of @reputest found in the past 24 hours");
        }
    } else {
        info!("No mentions of @reputest found in the past 24 hours");
    }

    Ok(mentions)
}
