//! HTTP route handlers for the reputest service.
//!
//! This module contains all the HTTP route handler functions that process
//! incoming requests and return appropriate responses.

use axum::{http::StatusCode, response::Json};
use log::{error, info};
use serde_json::{json, Value};

use crate::twitter::post_tweet;

/// Handles GET requests to the `/reputest` endpoint.
///
/// This endpoint returns a simple "Reputesting!" message and logs the request.
/// It's primarily used for testing and demonstration purposes.
///
/// # Returns
///
/// A static string "Reputesting!".
pub async fn handle_reputest_get() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

/// Handles POST requests to the `/reputest` endpoint.
///
/// This endpoint returns a simple "Reputesting!" message and logs the request.
/// It's primarily used for testing and demonstration purposes.
///
/// # Returns
///
/// A static string "Reputesting!".
pub async fn handle_reputest_post() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

/// Handles GET requests to the `/health` endpoint.
///
/// This endpoint provides a health check for the service, returning the current
/// status and service name. It's commonly used by load balancers and monitoring
/// systems to verify that the service is running and responsive.
///
/// # Returns
///
/// A JSON response containing:
/// - `status`: Always "healthy" when the service is running
/// - `service`: The service name "reputest"
///
/// # Example Response
///
/// ```json
/// {
///   "status": "healthy",
///   "service": "reputest"
/// }
/// ```
pub async fn handle_health() -> Json<Value> {
    Json(json!({"status": "healthy", "service": "reputest"}))
}

/// Handles POST requests to the `/tweet` endpoint.
///
/// This endpoint posts a tweet to Twitter/X with the message "Hello world".
/// It demonstrates the OAuth 2.0 Bearer token authentication and Twitter API v2 integration.
///
/// # Returns
///
/// - `Ok(Json<Value>)`: Success response with tweet details
/// - `Err((StatusCode, Json<Value>))`: Error response with status code and error details
///
/// # Success Response
///
/// ```json
/// {
///   "status": "success",
///   "message": "Tweet posted",
///   "response": "<Twitter API response>"
/// }
/// ```
///
/// # Error Response
///
/// ```json
/// {
///   "status": "error",
///   "message": "Failed to post tweet",
///   "error": "<error details>"
/// }
/// ```
///
/// # Requirements
///
/// Requires Twitter API Bearer token to be set in environment variables.
pub async fn handle_tweet() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match post_tweet("Hello world").await {
        Ok(response) => {
            info!("Tweet posted successfully");
            Ok(Json(
                json!({"status": "success", "message": "Tweet posted", "response": response}),
            ))
        }
        Err(e) => {
            error!("Failed to post tweet: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({"status": "error", "message": "Failed to post tweet", "error": e.to_string()}),
                ),
            ))
        }
    }
}

/// Handles GET requests to the root `/` endpoint.
///
/// This endpoint returns a welcome message for the reputest service.
/// It serves as the main entry point for the service.
///
/// # Returns
///
/// A static string "A new reputest is in the house!".
pub async fn handle_root() -> &'static str {
    "A new reputest is in the house!"
}
