//! Tweet search functionality for Twitter API.
//!
//! This module searches for @reputest mentions via Twitter API v2 and records
//! megajoule transfers and good-vibes attestations from those tweets.

use log::{debug, error, info, warn};
use reqwest::Client;
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::TwitterConfig;
use crate::db;
use crate::oauth::build_oauth2_user_context_header;

use super::api::{lookup_user_by_username, make_authenticated_request};
use super::tweets::reply_to_tweet;

/// A tweet that mentioned @reputest, ready for command dispatch.
pub struct MentionedTweet {
    pub tweet_id: String,
    pub tweet_text: String,
    pub author_id: String,
    pub author_username: String,
    pub author_display_name: String,
    pub reply_target_username: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Saves included user objects from a search response and returns lookup maps.
async fn index_included_users(
    json_response: &serde_json::Value,
    pool: &PgPool,
) -> (
    std::collections::HashMap<String, String>,
    std::collections::HashMap<String, String>,
) {
    let mut users_username_map = std::collections::HashMap::new();
    let mut users_name_map = std::collections::HashMap::new();
    let Some(users) = json_response
        .get("includes")
        .and_then(|includes| includes.get("users"))
        .and_then(|users| users.as_array())
    else {
        return (users_username_map, users_name_map);
    };

    for user in users {
        let Some(id_str) = user.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(username_str) = user.get("username").and_then(|v| v.as_str()) else {
            continue;
        };
        let name_str = user
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(username_str);

        users_username_map.insert(id_str.to_string(), username_str.to_string());
        users_name_map.insert(id_str.to_string(), name_str.to_string());

        if let Some(created_at_str) = user.get("created_at").and_then(|v| v.as_str()) {
            match chrono::DateTime::parse_from_rfc3339(created_at_str) {
                Ok(dt) => {
                    let created_at_utc = dt.with_timezone(&chrono::Utc);
                    if let Err(e) = crate::db::save_user(
                        pool,
                        id_str,
                        username_str,
                        name_str,
                        created_at_utc,
                        None,
                    )
                    .await
                    {
                        error!("Failed to save user data for {}: {}", username_str, e);
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

    (users_username_map, users_name_map)
}

/// Records a megajoule transfer from a mention tweet, looking up the receiver and replying.
pub(crate) async fn record_megajoule_transfer(
    pool: &PgPool,
    config: &mut TwitterConfig,
    mention: &MentionedTweet,
    amount: i32,
    receiver_username: &str,
) {
    let tweet_id = mention.tweet_id.as_str();
    let poster_id = mention.author_id.as_str();
    let poster_username = mention.author_username.as_str();
    let poster_display_name = mention.author_display_name.as_str();
    let created_at = mention.created_at;
    info!(
        "  Poster (megajoule sender): {} (@{})",
        poster_display_name, poster_username
    );
    info!("  Receiver: @{}", receiver_username);
    info!("  Amount: {}", amount);

    let receiver_user_info = match crate::db::get_user_info_by_username(pool, receiver_username)
        .await
    {
        Ok(Some((user_id, name, created_at))) => {
            info!(
                "Using cached user info for @{} from database",
                receiver_username
            );
            Some((user_id, name, created_at))
        }
        Ok(None) => {
            info!(
                "User @{} not found in database, looking up via Twitter API",
                receiver_username
            );
            match lookup_user_by_username(config, pool, receiver_username).await {
                Ok(Some((user_id, name, created_at, follower_count))) => {
                    if let Err(e) = crate::db::save_user(
                        pool,
                        &user_id,
                        receiver_username,
                        &name,
                        created_at,
                        follower_count,
                    )
                    .await
                    {
                        error!("Failed to save receiver user data: {}", e);
                    }
                    Some((user_id, name, created_at))
                }
                Ok(None) => {
                    warn!(
                        "Receiver user {} not found via Twitter API",
                        receiver_username
                    );
                    let reply_text = format!(
                        "I couldn't find a Twitter user with the handle '{}'. Please check the spelling and try again.",
                        receiver_username
                    );
                    info!(
                        "Replying to tweet {} with user not found message: {}",
                        tweet_id, reply_text
                    );
                    match reply_to_tweet(&reply_text, tweet_id).await {
                        Ok(response) => {
                            info!("Successfully replied to tweet {}: {}", tweet_id, response);
                        }
                        Err(e) => {
                            warn!("Failed to reply to tweet {}: {}", tweet_id, e);
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

    let Some((receiver_user_id, _, _)) = receiver_user_info else {
        return;
    };

    let sender_id = match crate::ledger::parse_snowflake_id(poster_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid sender id for megajoule transfer: {}", e);
            return;
        }
    };
    let receiver_id = match crate::ledger::parse_snowflake_id(&receiver_user_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid receiver id for megajoule transfer: {}", e);
            return;
        }
    };
    let transfer_id = match crate::ledger::parse_snowflake_id(tweet_id) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid tweet id for megajoule transfer: {}", e);
            return;
        }
    };
    if amount <= 0 {
        warn!(
            "Skipping tweet {} from @{} sending {} megajoules to @{} - amount must be positive",
            tweet_id, poster_username, amount, receiver_username
        );
        return;
    }

    match crate::ledger::record_transfer(sender_id, receiver_id, amount as u128, transfer_id).await
    {
        Ok(crate::ledger::TransferOutcome::AlreadyExists) => {
            info!(
                "Skipping tweet {} from @{} sending {} megajoules to @{} (posted at {}) - already processed",
                tweet_id, poster_username, amount, receiver_username, created_at
            );
        }
        Ok(crate::ledger::TransferOutcome::Created) => {
            info!(
                "Megajoule transfer recorded from mention tweet {} (posted {}): sender @{} ({}) → receiver @{} ({}) amount {}",
                tweet_id,
                created_at,
                poster_username,
                poster_id,
                receiver_username,
                receiver_user_id,
                amount
            );
            let reply_text = format!(
                "Your {} megajoules to {} have been noted.",
                amount, receiver_username
            );
            info!(
                "Replying to tweet {} with confirmation: {}",
                tweet_id, reply_text
            );
            match reply_to_tweet(&reply_text, tweet_id).await {
                Ok(response) => {
                    info!("Successfully replied to tweet {}: {}", tweet_id, response);
                }
                Err(e) => {
                    warn!("Failed to reply to tweet {}: {}", tweet_id, e);
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to record megajoule transfer from tweet {}: {}",
                tweet_id, e
            );
        }
    }
}

/// Records a good-vibes attestation from a mention tweet, looking up the emitter and replying.
pub(crate) async fn record_good_vibes(
    pool: &PgPool,
    config: &mut TwitterConfig,
    mention: &MentionedTweet,
    vibe_emitter_username: &str,
) {
    let tweet_id = mention.tweet_id.as_str();
    let poster_id = mention.author_id.as_str();
    let poster_username = mention.author_username.as_str();
    let poster_display_name = mention.author_display_name.as_str();
    let created_at = mention.created_at;
    info!(
        "  Poster (vibe receiver): {} (@{})",
        poster_display_name, poster_username
    );
    info!("  Vibe emitter: {}", vibe_emitter_username);

    let user_info = match crate::db::get_user_info_by_username(pool, vibe_emitter_username).await {
        Ok(Some((user_id, name, created_at))) => {
            info!(
                "Using cached user info for @{} from database",
                vibe_emitter_username
            );
            Some((user_id, name, created_at))
        }
        Ok(None) => {
            info!(
                "User @{} not found in database, looking up via Twitter API",
                vibe_emitter_username
            );
            match lookup_user_by_username(config, pool, vibe_emitter_username).await {
                Ok(Some((user_id, name, created_at, follower_count))) => {
                    if let Err(e) = crate::db::save_user(
                        pool,
                        &user_id,
                        vibe_emitter_username,
                        &name,
                        created_at,
                        follower_count,
                    )
                    .await
                    {
                        error!("Failed to save emitter user data: {}", e);
                    }
                    Some((user_id, name, created_at))
                }
                Ok(None) => {
                    warn!(
                        "Emitter user {} not found via Twitter API",
                        vibe_emitter_username
                    );
                    let reply_text = format!(
                        "I couldn't find a Twitter user with the handle '{}'. Please check the spelling and try again.",
                        vibe_emitter_username
                    );
                    info!(
                        "Replying to tweet {} with user not found message: {}",
                        tweet_id, reply_text
                    );
                    match reply_to_tweet(&reply_text, tweet_id).await {
                        Ok(response) => {
                            info!("Successfully replied to tweet {}: {}", tweet_id, response);
                        }
                        Err(e) => {
                            warn!("Failed to reply to tweet {}: {}", tweet_id, e);
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

    let Some((emitter_user_id, _, _)) = user_info else {
        return;
    };

    match crate::db::has_good_vibes_tweet(pool, tweet_id).await {
        Ok(true) => {
            info!(
                "Skipping tweet {} from @{} mentioning @{} (posted at {}) - already processed for good vibes",
                tweet_id, poster_username, vibe_emitter_username, created_at
            );
        }
        Ok(false) => {
            match crate::db::has_good_vibes_record(pool, poster_id, &emitter_user_id).await {
                Ok(true) => {
                    match crate::db::get_good_vibes_tweet_id(pool, &emitter_user_id, poster_id)
                        .await
                    {
                        Ok(Some(original_tweet_id)) => {
                            let tweet_url =
                                format!("https://twitter.com/i/status/{}", original_tweet_id);
                            let reply_text = format!(
                                "You've already declared these vibes! See your previous tweet: {}",
                                tweet_url
                            );
                            info!(
                                "Replying to tweet {} with duplicate vibes message: {}",
                                tweet_id, reply_text
                            );
                            match reply_to_tweet(&reply_text, tweet_id).await {
                                Ok(response) => {
                                    info!(
                                        "Successfully replied to tweet {}: {}",
                                        tweet_id, response
                                    );
                                }
                                Err(e) => {
                                    warn!("Failed to reply to tweet {}: {}", tweet_id, e);
                                }
                            }
                        }
                        Ok(None) => {
                            warn!(
                                "Good vibes record exists but no tweet_id found for emitter {} and sensor {}",
                                emitter_user_id, poster_id
                            );
                        }
                        Err(e) => {
                            error!(
                                "Failed to get original tweet ID for duplicate vibes check: {}",
                                e
                            );
                        }
                    }
                }
                Ok(false) => {
                    if let Err(e) = crate::db::save_good_vibes(
                        pool,
                        tweet_id,
                        &emitter_user_id,
                        poster_id,
                        created_at,
                    )
                    .await
                    {
                        error!(
                            "Failed to save good vibes data (non-constraint error): {}",
                            e
                        );
                    } else {
                        let reply_text = format!(
                            "Your good vibes from {} have been noted.",
                            vibe_emitter_username
                        );
                        info!(
                            "Replying to tweet {} with confirmation: {}",
                            tweet_id, reply_text
                        );
                        match reply_to_tweet(&reply_text, tweet_id).await {
                            Ok(response) => {
                                info!("Successfully replied to tweet {}: {}", tweet_id, response);
                            }
                            Err(e) => {
                                warn!("Failed to reply to tweet {}: {}", tweet_id, e);
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
            error!(
                "Failed to check if tweet {} has been processed: {}",
                tweet_id, e
            );
        }
    }
}

/// Searches for mentions of @reputest in the past 24 hours.
///
/// Uses the Twitter API v2 recent-search endpoint with author and reply-target
/// expansions so good-vibes reply exclusion still works. Paginates up to 10 pages.
///
/// # Returns
///
/// - `Ok(Vec<MentionedTweet>)`: Mentions found in the past 24 hours
/// - `Err(...)`: If authentication fails, network error, or API error
pub async fn search_mentions(
) -> Result<Vec<MentionedTweet>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting search for @reputest mentions in the past 24 hours");

    info!("Loading Twitter configuration from database for mentions search");
    let pool = db::get_db_pool().await?;
    let mut config = TwitterConfig::from_env(&pool).await?;
    debug!("Twitter config loaded successfully for mentions search");

    let client = Client::new();

    let twenty_four_hours_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 86400;

    let query = "@reputest";
    let start_time = chrono::DateTime::from_timestamp(twenty_four_hours_ago as i64, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S.000Z");

    debug!("Building OAuth 2.0 User Context authorization header for mentions search");
    let auth_header = build_oauth2_user_context_header(&config.access_token);

    let mut mentions = Vec::new();
    let mut next_token: Option<String> = None;
    let mut page_count = 0;

    loop {
        page_count += 1;
        info!("Fetching page {} of @reputest mention search", page_count);

        let url = if let Some(token) = &next_token {
            format!(
                "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id,in_reply_to_user_id,referenced_tweets.id&user.fields=id,username,name,created_at&tweet.fields=created_at,author_id,conversation_id,in_reply_to_user_id&next_token={}",
                urlencoding::encode(query),
                start_time,
                token
            )
        } else {
            format!(
                "https://api.x.com/2/tweets/search/recent?query={}&start_time={}&max_results=100&expansions=author_id,in_reply_to_user_id,referenced_tweets.id&user.fields=id,username,name,created_at&tweet.fields=created_at,author_id,conversation_id,in_reply_to_user_id",
                urlencoding::encode(query),
                start_time
            )
        };

        info!("Mentions search URL: {}", url);
        debug!("Search query: {}", query);
        debug!("Start time (24 hours ago): {}", start_time);
        info!(
            "Sending GET request to Twitter API v2 search endpoint for mentions (page {})",
            page_count
        );

        let request_builder = client
            .get(&url)
            .header("Authorization", auth_header.clone());

        let response_text = make_authenticated_request(
            &mut config,
            &pool,
            request_builder,
            &format!("search_mentions_page_{}", page_count),
        )
        .await?;

        debug!(
            "Mentions search response: {} bytes received",
            response_text.len()
        );
        let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

        let (users_username_map, users_name_map) =
            index_included_users(&json_response, &pool).await;

        if let Some(tweets) = json_response.get("data").and_then(|data| data.as_array()) {
            if tweets.is_empty() {
                info!("No mentions of @reputest found on this page");
            } else {
                info!(
                    "Found {} mentions of @reputest on page {}",
                    tweets.len(),
                    page_count
                );
                for (i, tweet) in tweets.iter().enumerate() {
                    let Some(text) = tweet.get("text").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(id) = tweet.get("id").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(author_id) = tweet.get("author_id").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(created_at_str) = tweet.get("created_at").and_then(|v| v.as_str())
                    else {
                        error!("Tweet {} missing created_at field", id);
                        continue;
                    };
                    let created_at = match chrono::DateTime::parse_from_rfc3339(created_at_str) {
                        Ok(dt) => dt.with_timezone(&chrono::Utc),
                        Err(e) => {
                            error!("Failed to parse created_at '{}': {}", created_at_str, e);
                            continue;
                        }
                    };

                    let author_username = users_username_map
                        .get(author_id)
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    let author_display_name = users_name_map
                        .get(author_id)
                        .map(|s| s.as_str())
                        .unwrap_or(author_username);

                    let reply_target_username = tweet
                        .get("in_reply_to_user_id")
                        .and_then(|v| v.as_str())
                        .and_then(|user_id| users_username_map.get(user_id))
                        .cloned();

                    info!(
                        "Mention {} (ID: {}): {} by @{}",
                        i + 1,
                        id,
                        text,
                        author_username
                    );

                    mentions.push(MentionedTweet {
                        tweet_id: id.to_string(),
                        tweet_text: text.to_string(),
                        author_id: author_id.to_string(),
                        author_username: author_username.to_string(),
                        author_display_name: author_display_name.to_string(),
                        reply_target_username,
                        created_at,
                    });
                }
            }
        } else {
            info!("No mentions of @reputest found on this page");
        }

        next_token = json_response
            .get("meta")
            .and_then(|meta| meta.get("next_token"))
            .and_then(|token| token.as_str())
            .map(|s| s.to_string());

        if next_token.is_none() {
            info!("No more mention pages to fetch");
            break;
        }

        if page_count >= 10 {
            warn!("Reached maximum page limit (10), stopping mention pagination");
            break;
        }
    }

    info!(
        "Completed @reputest mention search - processed {} pages, {} tweets",
        page_count,
        mentions.len()
    );
    Ok(mentions)
}
