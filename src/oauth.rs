//! OAuth authentication module for Twitter/X API integration.
//!
//! This module contains functions for implementing OAuth 2.0 User Context
//! authentication for all Twitter API v2 operations including posting tweets
//! and searching tweets. It also includes automatic token refresh functionality.

use log::{debug, error};
use std::collections::HashMap;

/// Builds the Authorization header for OAuth 2.0 User Context authentication.
///
/// This function creates the proper Authorization header for OAuth 2.0 User Context
/// authentication, which is required for Twitter API v2 endpoints that perform
/// user-specific operations like posting tweets.
///
/// # Parameters
///
/// - `access_token`: The Access Token obtained through OAuth 2.0 Authorization Code Flow
///
/// # Returns
///
/// A properly formatted Authorization header string for OAuth 2.0 User Context authentication.
///
/// # Format
///
/// The header follows this format:
/// ```text
/// Bearer YOUR_ACCESS_TOKEN_HERE
/// ```
///
/// # Example
///
/// ```rust
/// use reputest::oauth::build_oauth2_user_context_header;
///
/// let header = build_oauth2_user_context_header("your_access_token");
/// assert_eq!(header, "Bearer your_access_token");
/// ```
pub fn build_oauth2_user_context_header(access_token: &str) -> String {
    debug!("Building OAuth 2.0 User Context header");
    format!("Bearer {}", access_token)
}

/// Refreshes an OAuth 2.0 User Context access token using a refresh token.
///
/// This function refreshes an expired access token using the provided
/// refresh token and client credentials. It follows the OAuth 2.0 token refresh flow
/// and returns a new access token. This function is used by the refresh token utility script.
///
/// # Parameters
///
/// - `client_id`: The OAuth 2.0 client ID
/// - `client_secret`: The OAuth 2.0 client secret
/// - `refresh_token`: The refresh token obtained during initial authorization
///
/// # Returns
///
/// - `Ok((String, Option<String>))`: The new access token and optionally a new refresh token on successful refresh
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the refresh fails
///
/// # Example
///
/// ```rust
/// use reputest::oauth::refresh_access_token;
///
/// #[tokio::main]
/// async fn main() {
///     let result = refresh_access_token(
///         "your_client_id",
///         "your_client_secret",
///         "your_refresh_token"
///     ).await;
///     match result {
///         Ok((new_token, new_refresh)) => {
///             println!("New access token: {}", new_token);
///             if let Some(refresh) = new_refresh {
///                 println!("New refresh token: {}", refresh);
///             }
///         },
///         Err(e) => eprintln!("Token refresh failed: {}", e),
///     }
/// }
/// ```
pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<(String, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting OAuth 2.0 access token refresh process");

    let client = reqwest::Client::new();
    let url = "https://api.twitter.com/2/oauth2/token";

    debug!("Making token refresh request to: {}", url);

    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);

    debug!("Token refresh request parameters: grant_type=refresh_token, refresh_token=[REDACTED]");

    let response = client
        .post(url)
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;

    let status = response.status();
    debug!("Token refresh response status: {}", status);

    if status.is_success() {
        let response_text = response.text().await?;
        debug!("Token refresh successful");

        // Parse the JSON response to extract access_token
        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
            debug!("New access token obtained successfully");

            // Check if we also got a new refresh token
            let new_refresh_token = json.get("refresh_token").and_then(|v| v.as_str()).map(|s| {
                debug!("New refresh token also received");
                s.to_string()
            });

            // Log token expiration info (safe - no sensitive data)
            if let Some(expires_in) = json.get("expires_in").and_then(|v| v.as_u64()) {
                let hours = expires_in / 3600;
                let minutes = (expires_in % 3600) / 60;
                if hours > 0 {
                    debug!(
                        "Token will expire in {} hours and {} minutes",
                        hours, minutes
                    );
                } else {
                    debug!("Token will expire in {} minutes", minutes);
                }
            }

            Ok((access_token.to_string(), new_refresh_token))
        } else {
            error!("No access_token found in refresh response");
            Err("No access_token in refresh response".into())
        }
    } else {
        // Don't log the full error response as it might contain sensitive info
        error!("Token refresh failed with status {}", status);
        Err(format!("Token refresh failed with status {}", status).into())
    }
}
