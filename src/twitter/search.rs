//! Tweet search functionality for Twitter API.
//!
//! This module contains functions for searching tweets by hashtags and mentions
//! using the Twitter API v2.

use log::{debug, error, info, warn};
use reqwest::Client;
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::TwitterConfig;
use crate::db;
use crate::oauth::build_oauth2_user_context_header;

use super::api::{lookup_user_by_username, make_authenticated_request};
use super::parsing::{
    extract_megajoule_transfer, extract_mention_with_question, extract_vibe_emitter,
};
use super::tweets::reply_to_tweet;

/// Processes a single page of tweet search results and saves good vibes data.
///
/// This helper function processes the JSON response from the Twitter API search endpoint,
/// extracts user and tweet information, and saves good vibes data for tweets containing
/// user mentions. It handles pagination by returning the next_token if available.
///
/// # Parameters
///
/// - `json_response`: The JSON response from the Twitter API
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `config`: Mutable reference to TwitterConfig (may be updated with new token)
///
/// # Returns
///
/// - `Ok(Some(next_token))`: If there are more pages, returns the next_token
/// - `Ok(None)`: If this is the last page
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If processing fails
async fn process_search_results(
    json_response: &serde_json::Value,
    pool: &PgPool,
    config: &mut TwitterConfig,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
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
                                        pool,
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
                info!("No tweets found in this page");
            } else {
                info!("Found {} tweets in this page", tweets.len());
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

                            // Check if this is a reply and get the username to exclude
                            let reply_target_username = tweet
                                .get("in_reply_to_user_id")
                                .and_then(|v| v.as_str())
                                .and_then(|user_id| users_username_map.get(user_id))
                                .map(|s| s.as_str());

                            // Extract vibe_emitter from tweet text, excluding reply target if applicable
                            // Handles both "@username #gmgv" and "username #gmgv" formats
                            let tweet_text = text.as_str().unwrap_or("");

                            // Check for megajoule transfer first
                            if let Some((amount, receiver_username)) =
                                extract_megajoule_transfer(tweet_text)
                            {
                                // Process megajoule transfer
                                if let (
                                    Some(poster_id),
                                    Some(poster_username),
                                    Some(poster_display_name),
                                ) = (poster_user_id, poster_username, poster_name)
                                {
                                    info!(
                                        "  Poster (megajoule sender): {} (@{})",
                                        poster_display_name, poster_username
                                    );
                                    info!("  Receiver: @{}", receiver_username);
                                    info!("  Amount: {}", amount);

                                    // Look up receiver user ID (similar to how good vibes handles emitter lookup)
                                    let receiver_user_info =
                                        match crate::db::get_user_info_by_username(
                                            pool,
                                            &receiver_username,
                                        )
                                        .await
                                        {
                                            Ok(Some((user_id, name, created_at))) => {
                                                // User found in database, use cached info
                                                info!(
                                                    "Using cached user info for @{} from database",
                                                    receiver_username
                                                );
                                                Some((user_id, name, created_at))
                                            }
                                            Ok(None) => {
                                                // User not in database, look up via Twitter API
                                                info!("User @{} not found in database, looking up via Twitter API", receiver_username);
                                                match lookup_user_by_username(
                                                    config,
                                                    pool,
                                                    &receiver_username,
                                                )
                                                .await
                                                {
                                                    Ok(Some((user_id, name, created_at))) => {
                                                        // Save the user data for future use
                                                        if let Err(e) = crate::db::save_user(
                                                            pool,
                                                            &user_id,
                                                            &receiver_username,
                                                            &name,
                                                            created_at,
                                                        )
                                                        .await
                                                        {
                                                            error!(
                                                            "Failed to save receiver user data: {}",
                                                            e
                                                        );
                                                        }
                                                        Some((user_id, name, created_at))
                                                    }
                                                    Ok(None) => {
                                                        warn!(
                                                        "Receiver user {} not found via Twitter API",
                                                        receiver_username
                                                    );
                                                        // Reply to let them know the user wasn't found
                                                        let tweet_id = id.as_str().unwrap();
                                                        let reply_text = format!(
                                                        "I couldn't find a Twitter user with the handle '{}'. Please check the spelling and try again.",
                                                        receiver_username
                                                    );
                                                        info!("Replying to tweet {} with user not found message: {}", tweet_id, reply_text);
                                                        match reply_to_tweet(&reply_text, tweet_id)
                                                            .await
                                                        {
                                                            Ok(response) => {
                                                                info!("Successfully replied to tweet {}: {}", tweet_id, response);
                                                            }
                                                            Err(e) => {
                                                                warn!(
                                                                "Failed to reply to tweet {}: {}",
                                                                tweet_id, e
                                                            );
                                                            }
                                                        }
                                                        None
                                                    }
                                                    Err(e) => {
                                                        error!(
                                                        "Failed to lookup receiver user {} via Twitter API: {}",
                                                        receiver_username, e
                                                    );
                                                        None
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Failed to check database for user @{}: {}",
                                                    receiver_username, e
                                                );
                                                None
                                            }
                                        };

                                    // If we have receiver user info, save the megajoule transfer
                                    if let Some((receiver_user_id, _, _)) = receiver_user_info {
                                        // Check if this tweet has already been processed
                                        match crate::db::has_megajoule_tweet(
                                            pool,
                                            id.as_str().unwrap(),
                                        )
                                        .await
                                        {
                                            Ok(true) => {
                                                info!(
                                                    "Skipping tweet {} from @{} sending {} megajoules to @{} (posted at {}) - already processed",
                                                    id.as_str().unwrap(),
                                                    poster_username,
                                                    amount,
                                                    receiver_username,
                                                    created_at
                                                );
                                            }
                                            Ok(false) => {
                                                // Tweet not processed yet, save the megajoule transfer
                                                if let Err(e) = crate::db::save_megajoule(
                                                    pool,
                                                    id.as_str().unwrap(), // tweet_id
                                                    poster_id,            // sender_id
                                                    &receiver_user_id,    // receiver_id
                                                    amount,               // amount
                                                    created_at,           // created_at from tweet
                                                )
                                                .await
                                                {
                                                    error!("Failed to save megajoule transfer (non-constraint error): {}", e);
                                                } else {
                                                    // Successfully saved megajoule transfer, now reply to the tweet confirming transfer was recorded
                                                    let tweet_id = id.as_str().unwrap();
                                                    let reply_text = format!(
                                                        "Your {} megajoules to {} have been noted.",
                                                        amount, receiver_username
                                                    );
                                                    info!("Replying to tweet {} with confirmation: {}", tweet_id, reply_text);
                                                    match reply_to_tweet(&reply_text, tweet_id)
                                                        .await
                                                    {
                                                        Ok(response) => {
                                                            info!("Successfully replied to tweet {}: {}", tweet_id, response);
                                                        }
                                                        Err(e) => {
                                                            warn!(
                                                                "Failed to reply to tweet {}: {}",
                                                                tweet_id, e
                                                            );
                                                            // Don't fail the entire process if replying fails - it's not critical
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("Failed to check if tweet {} has been processed: {}", id.as_str().unwrap(), e);
                                            }
                                        }
                                    }
                                }
                                continue; // Skip good vibes processing for megajoule tweets
                            }

                            let vibe_emitter_username =
                                extract_vibe_emitter(tweet_text, reply_target_username);

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

                                    // First check if the emitter user exists in the database
                                    let user_info = match crate::db::get_user_info_by_username(
                                        pool,
                                        vibe_emitter_username,
                                    )
                                    .await
                                    {
                                        Ok(Some((user_id, name, created_at))) => {
                                            // User found in database, use cached info
                                            info!(
                                                "Using cached user info for @{} from database",
                                                vibe_emitter_username
                                            );
                                            Some((user_id, name, created_at))
                                        }
                                        Ok(None) => {
                                            // User not in database, look up via Twitter API
                                            info!("User @{} not found in database, looking up via Twitter API", vibe_emitter_username);
                                            match lookup_user_by_username(
                                                config,
                                                pool,
                                                vibe_emitter_username,
                                            )
                                            .await
                                            {
                                                Ok(Some((user_id, name, created_at))) => {
                                                    // Save the user data for future use
                                                    if let Err(e) = crate::db::save_user(
                                                        pool,
                                                        &user_id,
                                                        vibe_emitter_username,
                                                        &name,
                                                        created_at,
                                                    )
                                                    .await
                                                    {
                                                        error!(
                                                            "Failed to save emitter user data: {}",
                                                            e
                                                        );
                                                    }
                                                    Some((user_id, name, created_at))
                                                }
                                                Ok(None) => {
                                                    warn!(
                                                        "Emitter user {} not found via Twitter API",
                                                        vibe_emitter_username
                                                    );
                                                    // Reply to let them know the user wasn't found
                                                    let tweet_id = id.as_str().unwrap();
                                                    let reply_text = format!(
                                                        "I couldn't find a Twitter user with the handle '{}'. Please check the spelling and try again.",
                                                        vibe_emitter_username
                                                    );
                                                    info!("Replying to tweet {} with user not found message: {}", tweet_id, reply_text);
                                                    match reply_to_tweet(&reply_text, tweet_id)
                                                        .await
                                                    {
                                                        Ok(response) => {
                                                            info!("Successfully replied to tweet {}: {}", tweet_id, response);
                                                        }
                                                        Err(e) => {
                                                            warn!(
                                                                "Failed to reply to tweet {}: {}",
                                                                tweet_id, e
                                                            );
                                                        }
                                                    }
                                                    None
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "Failed to lookup emitter user {} via Twitter API: {}",
                                                        vibe_emitter_username, e
                                                    );
                                                    None
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!(
                                                "Failed to check database for user @{}: {}",
                                                vibe_emitter_username, e
                                            );
                                            None
                                        }
                                    };

                                    // If we have user info (either from cache or API), save the good vibes data
                                    if let Some((emitter_user_id, _, _)) = user_info {
                                        // First check if this tweet has already been processed
                                        match crate::db::has_good_vibes_tweet(
                                            pool,
                                            id.as_str().unwrap(),
                                        )
                                        .await
                                        {
                                            Ok(true) => {
                                                info!(
                                                    "Skipping tweet {} from @{} mentioning @{} (posted at {}) - already processed for good vibes",
                                                    id.as_str().unwrap(),
                                                    poster_username,
                                                    vibe_emitter_username,
                                                    created_at
                                                );
                                            }
                                            Ok(false) => {
                                                // Tweet not processed yet, check if emitter has already given vibes to this sensor
                                                match crate::db::has_good_vibes_record(
                                                    pool,
                                                    poster_id,
                                                    &emitter_user_id,
                                                )
                                                .await
                                                {
                                                    Ok(true) => {
                                                        // Good vibes already exist, get the original tweet ID and reply with duplicate message
                                                        match crate::db::get_good_vibes_tweet_id(
                                                            pool,
                                                            &emitter_user_id,
                                                            poster_id,
                                                        )
                                                        .await
                                                        {
                                                            Ok(Some(original_tweet_id)) => {
                                                                let tweet_id = id.as_str().unwrap();
                                                                let tweet_url = format!("https://twitter.com/i/status/{}", original_tweet_id);
                                                                let reply_text = format!(
                                                                    "You've already declared these vibes! See your previous tweet: {}",
                                                                    tweet_url
                                                                );
                                                                info!("Replying to tweet {} with duplicate vibes message: {}", tweet_id, reply_text);
                                                                match reply_to_tweet(
                                                                    &reply_text,
                                                                    tweet_id,
                                                                )
                                                                .await
                                                                {
                                                                    Ok(response) => {
                                                                        info!("Successfully replied to tweet {}: {}", tweet_id, response);
                                                                    }
                                                                    Err(e) => {
                                                                        warn!("Failed to reply to tweet {}: {}", tweet_id, e);
                                                                        // Don't fail the entire process if replying fails - it's not critical
                                                                    }
                                                                }
                                                            }
                                                            Ok(None) => {
                                                                warn!("Good vibes record exists but no tweet_id found for emitter {} and sensor {}", emitter_user_id, poster_id);
                                                            }
                                                            Err(e) => {
                                                                error!("Failed to get original tweet ID for duplicate vibes check: {}", e);
                                                            }
                                                        }
                                                    }
                                                    Ok(false) => {
                                                        // No existing good vibes, save the new good vibes data
                                                        if let Err(e) = crate::db::save_good_vibes(
                                                            pool,
                                                            id.as_str().unwrap(), // tweet_id
                                                            &emitter_user_id, // emitter_id (person who sent good vibes)
                                                            poster_id, // sensor_id (person who received good vibes)
                                                            created_at, // created_at from tweet
                                                        )
                                                        .await
                                                        {
                                                            error!("Failed to save good vibes data (non-constraint error): {}", e);
                                                        } else {
                                                            // Successfully saved good vibes data, now reply to the tweet confirming good vibes were recorded
                                                            let tweet_id = id.as_str().unwrap();
                                                            let reply_text = format!(
                                                                "Your good vibes from {} have been noted.",
                                                                vibe_emitter_username
                                                            );
                                                            info!("Replying to tweet {} with confirmation: {}", tweet_id, reply_text);
                                                            match reply_to_tweet(
                                                                &reply_text,
                                                                tweet_id,
                                                            )
                                                            .await
                                                            {
                                                                Ok(response) => {
                                                                    info!("Successfully replied to tweet {}: {}", tweet_id, response);
                                                                }
                                                                Err(e) => {
                                                                    warn!("Failed to reply to tweet {}: {}", tweet_id, e);
                                                                    // Don't fail the entire process if replying fails - it's not critical
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to check for existing good vibes record: {}", e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("Failed to check if tweet {} has been processed: {}", id.as_str().unwrap(), e);
                                            }
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
        info!("No tweets found in this page");
    }

    // Check for next_token for pagination
    let next_token = json_response
        .get("meta")
        .and_then(|meta| meta.get("next_token"))
        .and_then(|token| token.as_str())
        .map(|s| s.to_string());

    Ok(next_token)
}

/// Searches for tweets with a specific hashtag in the past 6 hours and saves good vibes data.
///
/// This function uses the Twitter API v2 search endpoint to find tweets containing
/// the specified hashtag that were posted within the past 6 hours. It extracts vibe
/// emitter (poster) and vibe receiver (mentioned user) information and saves it
/// to the database. It uses OAuth 2.0 User Context Access Token authentication for v2 endpoints.
///
/// The function uses pagination to ensure all tweets with the hashtag are processed,
/// including replies that might appear on later pages of results.
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
/// ```rust,no_run
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

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database for search");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully for search");
    let client = Client::new();

    // Calculate the timestamp for 6 hours ago
    let six_hours_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 21600; // 21600 seconds = 6 hours

    // Build the search query with hashtag and time filter
    let query = format!("#{}", hashtag);
    let start_time = chrono::DateTime::from_timestamp(six_hours_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    debug!("Building OAuth 2.0 User Context authorization header for search");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    let mut next_token: Option<String> = None;
    let mut page_count = 0;

    loop {
        page_count += 1;
        info!("Fetching page {} of search results", page_count);

        // Build URL with pagination token if available
        let url = if let Some(token) = &next_token {
            format!(
                "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id,referenced_tweets.id&user.fields=id,username,name,created_at&tweet.fields=created_at,conversation_id,in_reply_to_user_id&next_token={}",
                urlencoding::encode(&query),
                start_time,
                token
            )
        } else {
            format!(
                "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id,referenced_tweets.id&user.fields=id,username,name,created_at&tweet.fields=created_at,conversation_id,in_reply_to_user_id",
                urlencoding::encode(&query),
                start_time
            )
        };

        info!("Search URL: {}", url);
        debug!("Search query: {}", query);
        debug!("Start time (6 hours ago): {}", start_time);

        // Log request details
        info!(
            "Sending GET request to Twitter API v2 search endpoint (page {})",
            page_count
        );
        debug!("Request URL: {}", url);
        debug!("Request headers: Authorization: Bearer [REDACTED]");

        // Create the request builder
        let request_builder = client
            .get(&url)
            .header("Authorization", auth_header.clone());

        // Use the authenticated request helper with automatic token refresh
        let response_text = make_authenticated_request(
            &mut config,
            &pool,
            request_builder,
            &format!("search_tweets_page_{}", page_count),
        )
        .await?;

        debug!("Search response: {} bytes received", response_text.len());
        let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

        // Process this page of results
        next_token = process_search_results(&json_response, &pool, &mut config).await?;

        // Break if no more pages
        if next_token.is_none() {
            info!("No more pages to fetch");
            break;
        }

        // Safety check to prevent infinite loops (limit to 10 pages)
        if page_count >= 10 {
            warn!("Reached maximum page limit (10), stopping pagination");
            break;
        }
    }

    info!(
        "Completed search for hashtag #{} - processed {} pages",
        hashtag, page_count
    );
    Ok(())
}

/// Searches for mentions of the reputest user in the past 6 hours and returns tweet information.
///
/// This function uses the Twitter API v2 search endpoint to find tweets that mention
/// @reputest and were posted within the past 6 hours. It returns a vector of tuples containing
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
pub async fn search_mentions() -> Result<
    Vec<(String, String, String, Option<String>, String)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    info!("Starting search for @reputest mentions in the past hour");

    // Get database pool and load Twitter API credentials from database
    info!("Loading Twitter configuration from database for mentions search");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully for mentions search");

    let client = Client::new();

    // Calculate the timestamp for 6 hours ago
    let six_hours_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 21600; // 21600 seconds = 6 hours

    // Build the search query for mentions of @reputest
    let query = "@reputest";
    let start_time = chrono::DateTime::from_timestamp(six_hours_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");
    let url = format!(
        "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id&user.fields=id,username,name&tweet.fields=created_at,author_id",
        urlencoding::encode(query),
        start_time
    );

    info!("Mentions search URL: {}", url);
    debug!("Search query: {}", query);
    debug!("Start time (6 hours ago): {}", start_time);

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    debug!("Building OAuth 2.0 User Context authorization header for mentions search");
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

    debug!(
        "Mentions search response: {} bytes received",
        response_text.len()
    );
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
                info!("No mentions of @reputest found in the past 6 hours");
            } else {
                info!(
                    "Found {} mentions of @reputest in the past 6 hours:",
                    tweets.len()
                );
                for (i, tweet) in tweets.iter().enumerate() {
                    if let (Some(text), Some(id), Some(author_id), Some(created_at)) = (
                        tweet.get("text").and_then(|v| v.as_str()),
                        tweet.get("id").and_then(|v| v.as_str()),
                        tweet.get("author_id").and_then(|v| v.as_str()),
                        tweet.get("created_at").and_then(|v| v.as_str()),
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
                            created_at.to_string(),
                        ));
                    }
                }
            }
        } else {
            info!("No mentions of @reputest found in the past 6 hours");
        }
    } else {
        info!("No mentions of @reputest found in the past 6 hours");
    }

    Ok(mentions)
}
