//! OAuth authentication module for Twitter/X API integration.
//!
//! This module contains functions for implementing OAuth 2.0 User Context
//! authentication for all Twitter API v2 operations including posting tweets
//! and searching tweets. It also includes automatic token refresh functionality.

use log::{debug, error, info, warn};
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
    // Log token information for debugging (masked for security)
    let token_length = access_token.len();
    let token_prefix = if token_length > 8 {
        &access_token[..8]
    } else {
        access_token
    };
    let token_suffix = if token_length > 16 {
        &access_token[token_length - 8..]
    } else if token_length > 8 {
        &access_token[8..]
    } else {
        ""
    };

    let masked_token = if token_length > 16 {
        format!("{}...{}", token_prefix, token_suffix)
    } else {
        format!("{}...", token_prefix)
    };

    info!(
        "Building OAuth 2.0 User Context header with token length: {}",
        token_length
    );
    debug!("OAuth token (masked): {}", masked_token);

    let header = format!("Bearer {}", access_token);
    debug!(
        "Generated Authorization header: Bearer {}...",
        &header[7..std::cmp::min(header.len(), 20)]
    );

    header
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
    info!("Starting OAuth 2.0 access token refresh process");

    // Log token info (masked for security)
    let refresh_token_length = refresh_token.len();
    let refresh_token_prefix = if refresh_token_length > 8 {
        &refresh_token[..8]
    } else {
        refresh_token
    };
    let refresh_token_suffix = if refresh_token_length > 16 {
        &refresh_token[refresh_token_length - 8..]
    } else if refresh_token_length > 8 {
        &refresh_token[8..]
    } else {
        ""
    };

    let masked_refresh_token = if refresh_token_length > 16 {
        format!("{}...{}", refresh_token_prefix, refresh_token_suffix)
    } else {
        format!("{}...", refresh_token_prefix)
    };

    info!("Refresh token length: {}", refresh_token_length);
    debug!("Refresh token (masked): {}", masked_refresh_token);
    debug!(
        "Client ID (masked): {}...",
        &client_id[..std::cmp::min(client_id.len(), 8)]
    );
    debug!(
        "Client secret (masked): {}...",
        &client_secret[..std::cmp::min(client_secret.len(), 8)]
    );

    let client = reqwest::Client::new();
    let url = "https://api.twitter.com/2/oauth2/token";

    info!("Making token refresh request to: {}", url);

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
    info!("Token refresh response status: {}", status);

    if status.is_success() {
        let response_text = response.text().await?;
        info!("Token refresh successful");
        debug!("Token refresh response body: {}", response_text);

        // Parse the JSON response to extract access_token
        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
            let new_token_length = access_token.len();
            let new_token_prefix = if new_token_length > 8 {
                &access_token[..8]
            } else {
                access_token
            };
            let new_token_suffix = if new_token_length > 16 {
                &access_token[new_token_length - 8..]
            } else if new_token_length > 8 {
                &access_token[8..]
            } else {
                ""
            };

            let masked_new_token = if new_token_length > 16 {
                format!("{}...{}", new_token_prefix, new_token_suffix)
            } else {
                format!("{}...", new_token_prefix)
            };

            info!(
                "New access token obtained with length: {}",
                new_token_length
            );
            debug!("New access token (masked): {}", masked_new_token);

            // Check if we also got a new refresh token
            let new_refresh_token = if let Some(new_refresh_token) =
                json.get("refresh_token").and_then(|v| v.as_str())
            {
                let new_refresh_length = new_refresh_token.len();
                let new_refresh_prefix = if new_refresh_length > 8 {
                    &new_refresh_token[..8]
                } else {
                    new_refresh_token
                };
                let new_refresh_suffix = if new_refresh_length > 16 {
                    &new_refresh_token[new_refresh_length - 8..]
                } else if new_refresh_length > 8 {
                    &new_refresh_token[8..]
                } else {
                    ""
                };

                let masked_new_refresh = if new_refresh_length > 16 {
                    format!("{}...{}", new_refresh_prefix, new_refresh_suffix)
                } else {
                    format!("{}...", new_refresh_prefix)
                };

                info!(
                    "New refresh token also provided with length: {}",
                    new_refresh_length
                );
                debug!("New refresh token (masked): {}", masked_new_refresh);
                warn!("New refresh token received - it will be saved to the database");
                Some(new_refresh_token.to_string())
            } else {
                None
            };

            // Check token expiration
            if let Some(expires_in) = json.get("expires_in").and_then(|v| v.as_u64()) {
                info!("New access token expires in {} seconds", expires_in);
                let hours = expires_in / 3600;
                let minutes = (expires_in % 3600) / 60;
                if hours > 0 {
                    info!(
                        "Token will expire in {} hours and {} minutes",
                        hours, minutes
                    );
                } else {
                    info!("Token will expire in {} minutes", minutes);
                }
            }

            Ok((access_token.to_string(), new_refresh_token))
        } else {
            error!("No access_token found in refresh response");
            Err("No access_token in refresh response".into())
        }
    } else {
        let error_text = response.text().await?;
        error!(
            "Token refresh failed with status {}: {}",
            status, error_text
        );
        Err(format!("Token refresh failed ({}): {}", status, error_text).into())
    }
}
