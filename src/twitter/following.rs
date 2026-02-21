//! Twitter/X API integration for fetching user following lists.

use log::{error, info, warn};
use reqwest::Client;
use sqlx::PgPool;

use super::api::make_authenticated_request;
use crate::config::TwitterConfig;
use crate::db::save_user;
use crate::oauth::build_oauth2_user_context_header;

/// Represents a user from the Twitter following API response.
#[derive(Debug)]
#[allow(dead_code)]
pub struct FollowedUser {
    pub id: String,
    pub username: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Fetches all accounts that the given user follows, with pagination.
///
/// For each followed user, upserts into the users table and returns the list
/// of (follower_id, followed_id) pairs for storage in the following table.
///
/// # Parameters
///
/// - `config`: Mutable reference to TwitterConfig (may be updated with new token)
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `follower_user_id`: The Twitter user ID whose following list to fetch
///
/// # Returns
///
/// - `Ok(Vec<FollowedUser>)`: All followed users
/// - `Err(...)`: If the API request fails (e.g. 403 for protected account)
pub async fn fetch_user_following(
    config: &mut TwitterConfig,
    pool: &PgPool,
    follower_user_id: &str,
) -> Result<Vec<FollowedUser>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Fetching following list for user {}", follower_user_id);

    let client = Client::new();
    let base_url = format!(
        "https://api.x.com/2/users/{}/following?max_results=1000&user.fields=id,username,name,created_at,public_metrics",
        follower_user_id
    );

    let mut all_followed = Vec::new();
    let mut pagination_token: Option<String> = None;
    let mut page_count: u32 = 0;
    const MAX_PAGES: u32 = 15; // Rate limit: 15 requests per 15 minutes; cap to avoid hammering

    loop {
        let url = match &pagination_token {
            Some(token) => format!("{}&pagination_token={}", base_url, token),
            None => base_url.clone(),
        };

        let auth_header = build_oauth2_user_context_header(&config.access_token);
        let request_builder = client
            .get(&url)
            .header("Authorization", auth_header.clone());

        let response_text =
            make_authenticated_request(config, pool, request_builder, "fetch_user_following")
                .await?;
        let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

        // Check for API errors
        if let Some(errors) = json_response.get("errors").and_then(|e| e.as_array()) {
            for err in errors {
                if let Some(title) = err.get("title").and_then(|v| v.as_str()) {
                    error!("Twitter API error: {}", title);
                    if title.contains("Forbidden") || title.contains("403") {
                        return Err(
                            "User's following list is not accessible (protected or suspended)"
                                .into(),
                        );
                    }
                }
            }
        }

        // Parse data array
        if let Some(data) = json_response.get("data").and_then(|d| d.as_array()) {
            for user in data {
                let followers_count = user
                    .get("public_metrics")
                    .and_then(|pm| pm.get("followers_count"))
                    .and_then(|v| v.as_i64())
                    .map(|n| n as i32);

                if let (Some(id), Some(username), Some(name), Some(created_at_str)) = (
                    user.get("id").and_then(|v| v.as_str()),
                    user.get("username").and_then(|v| v.as_str()),
                    user.get("name").and_then(|v| v.as_str()),
                    user.get("created_at").and_then(|v| v.as_str()),
                ) {
                    let created_at_utc = match chrono::DateTime::parse_from_rfc3339(created_at_str)
                    {
                        Ok(dt) => dt.with_timezone(&chrono::Utc),
                        Err(e) => {
                            warn!("Failed to parse created_at for user {}: {}", id, e);
                            chrono::Utc::now()
                        }
                    };

                    // Upsert user into database (with follower_count from API when available)
                    if let Err(e) =
                        save_user(pool, id, username, name, created_at_utc, followers_count).await
                    {
                        warn!("Failed to save user {} (@{}): {}", id, username, e);
                    }

                    all_followed.push(FollowedUser {
                        id: id.to_string(),
                        username: username.to_string(),
                        name: name.to_string(),
                        created_at: created_at_utc,
                    });
                }
            }
        }

        page_count += 1;

        // Check for next page
        pagination_token = json_response
            .get("meta")
            .and_then(|m| m.get("next_token"))
            .and_then(|t| t.as_str())
            .map(String::from);

        if pagination_token.is_none() {
            info!(
                "Fetched {} accounts that user {} follows ({} pages)",
                all_followed.len(),
                follower_user_id,
                page_count
            );
            break;
        }

        if page_count >= MAX_PAGES {
            warn!(
                "Reached max page limit ({}), stopping. Fetched {} accounts so far.",
                MAX_PAGES,
                all_followed.len()
            );
            break;
        }

        // Brief delay to respect rate limits
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(all_followed)
}
