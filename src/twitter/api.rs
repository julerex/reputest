//! Core Twitter API utilities.
//!
//! This module contains low-level API utilities for making authenticated requests
//! to the Twitter API, including automatic token refresh on 401 errors.

use log::{debug, error, info, warn};
use reqwest::Client;
use sqlx::PgPool;

use crate::config::TwitterConfig;
use crate::oauth::build_oauth2_user_context_header;

/// Sanitizes API response text for safe logging.
///
/// This function truncates long responses to prevent accidental exposure
/// of sensitive data in logs.
fn sanitize_response_for_logging(response: &str, max_len: usize) -> String {
    if response.len() > max_len {
        format!("{}... [truncated, {} total bytes]", &response[..max_len], response.len())
    } else {
        response.to_string()
    }
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
pub(crate) async fn make_authenticated_request(
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
        debug!(
            "Response summary for '{}': {} bytes received",
            operation_name,
            response_text.len()
        );
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
                            "Response summary for '{}' (after refresh): {} bytes received",
                            operation_name,
                            response_text.len()
                        );
                        return Ok(response_text);
                    } else {
                        let error_text = retry_response.text().await?;
                        error!(
                            "Operation '{}' failed after token refresh - Status: {}",
                            operation_name, retry_status
                        );
                        debug!(
                            "Error response for '{}': {}",
                            operation_name,
                            sanitize_response_for_logging(&error_text, 200)
                        );
                        return Err(format!(
                            "Twitter API error after token refresh ({})",
                            retry_status
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
        "Operation '{}' failed - Status: {}",
        operation_name, status
    );
    debug!(
        "Error response for '{}': {}",
        operation_name,
        sanitize_response_for_logging(&error_text, 200)
    );
    Err(format!(
        "Twitter API error for operation '{}' ({})",
        operation_name, status
    )
    .into())
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
pub(crate) async fn lookup_user_by_username(
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
