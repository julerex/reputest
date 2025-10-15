//! # Tests Module
//!
//! This module contains comprehensive tests for the reputest web service.
//! It includes unit tests for individual functions and integration tests for HTTP endpoints.
//!
//! ## Test Categories
//!
//! ### Unit Tests
//! - OAuth utility functions (`percent_encode`, `generate_nonce`, `get_current_timestamp`)
//! - OAuth parameter building (`build_oauth_params`, `build_auth_header`)
//! - OAuth signature generation (`generate_oauth_signature`)
//! - Configuration loading (`TwitterConfig::from_env`)
//! - Server configuration (`get_server_port`)
//!
//! ### Integration Tests
//! - HTTP endpoint testing for all routes
//! - Request/response validation
//! - Error handling verification
//!
//! ## Test Environment
//!
//! Tests run in isolation and clean up environment variables after execution.
//! The Twitter API integration tests expect missing credentials and verify proper error handling.

use crate::{
    config::{get_server_port, TwitterConfig},
    handlers::{
        handle_health, handle_reputest_get, handle_reputest_post, handle_root, handle_tweet,
    },
    oauth::{
        build_auth_header, build_oauth_params, generate_nonce, generate_oauth_signature,
        get_current_timestamp, percent_encode,
    },
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use std::collections::BTreeMap;
use tower::ServiceExt;

/// Creates a test application instance with all routes configured.
///
/// This helper function sets up a minimal Axum router with all the same routes
/// as the main application, but without middleware layers that might interfere
/// with testing. It's used by integration tests to make HTTP requests.
///
/// # Returns
///
/// An Axum `Router` instance configured with all application routes.
fn create_test_app() -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .route("/tweet", post(handle_tweet))
}

/// Tests the root endpoint handler function directly.
///
/// This test verifies that the `handle_root` function returns the expected
/// welcome message without making an HTTP request.
#[tokio::test]
async fn test_handle_root() {
    let response = handle_root().await;
    assert_eq!(response, "A new reputest is in the house!");
}

/// Tests the GET /reputest endpoint handler function directly.
///
/// This test verifies that the `handle_reputest_get` function returns the
/// expected "Reputesting!" message without making an HTTP request.
#[tokio::test]
async fn test_handle_reputest_get() {
    let response = handle_reputest_get().await;
    assert_eq!(response, "Reputesting!");
}

/// Tests the POST /reputest endpoint handler function directly.
///
/// This test verifies that the `handle_reputest_post` function returns the
/// expected "Reputesting!" message without making an HTTP request.
#[tokio::test]
async fn test_handle_reputest_post() {
    let response = handle_reputest_post().await;
    assert_eq!(response, "Reputesting!");
}

/// Tests the health endpoint handler function directly.
///
/// This test verifies that the `handle_health` function returns a properly
/// formatted JSON response with the correct status and service name.
#[tokio::test]
async fn test_handle_health() {
    let response = handle_health().await;
    let Json(json_response): Json<Value> = response;

    assert_eq!(json_response["status"], "healthy");
    assert_eq!(json_response["service"], "reputest");
}

/// Integration test for the root endpoint (GET /).
///
/// This test makes an actual HTTP request to the root endpoint and verifies:
/// - The response status is 200 OK
/// - The response body contains the expected welcome message
#[tokio::test]
async fn test_root_endpoint() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "A new reputest is in the house!");
}

/// Integration test for the GET /reputest endpoint.
///
/// This test makes an actual HTTP GET request to the /reputest endpoint and verifies:
/// - The response status is 200 OK
/// - The response body contains "Reputesting!"
#[tokio::test]
async fn test_reputest_get_endpoint() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/reputest")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "Reputesting!");
}

/// Integration test for the POST /reputest endpoint.
///
/// This test makes an actual HTTP POST request to the /reputest endpoint and verifies:
/// - The response status is 200 OK
/// - The response body contains "Reputesting!"
#[tokio::test]
async fn test_reputest_post_endpoint() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/reputest")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "Reputesting!");
}

/// Integration test for the health endpoint (GET /health).
///
/// This test makes an actual HTTP request to the health endpoint and verifies:
/// - The response status is 200 OK
/// - The response is valid JSON
/// - The JSON contains the expected status and service fields
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let json_response: Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json_response["status"], "healthy");
    assert_eq!(json_response["service"], "reputest");
}

/// Integration test for the tweet endpoint (POST /tweet) without credentials.
///
/// This test verifies that the tweet endpoint properly handles the case where
/// Twitter API credentials are not configured. It expects:
/// - The response status to be 500 Internal Server Error
/// - The response to be valid JSON with an error status
/// - The error message to indicate a failure to post the tweet
///
/// This test is important for ensuring proper error handling in production
/// environments where credentials might be missing or invalid.
#[tokio::test]
async fn test_tweet_endpoint_without_credentials() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/tweet")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should return 500 because Twitter credentials are not set in test environment
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    let json_response: Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json_response["status"], "error");
    assert!(json_response["message"]
        .as_str()
        .unwrap()
        .contains("Failed to post tweet"));
}

/// Unit test for the percent_encode function.
///
/// This test verifies that the percent encoding function correctly handles:
/// - Unreserved characters (alphanumeric, hyphen, underscore, period, tilde) - no encoding
/// - Reserved characters (space, @, +, =) - proper percent encoding
/// - Empty strings - no change
#[test]
fn test_percent_encode() {
    // Test basic characters that don't need encoding
    assert_eq!(percent_encode("abc123"), "abc123");
    assert_eq!(percent_encode("ABC-_.~"), "ABC-_.~");

    // Test characters that need encoding
    assert_eq!(percent_encode("hello world"), "hello%20world");
    assert_eq!(percent_encode("test@example.com"), "test%40example.com");
    assert_eq!(percent_encode("a+b=c"), "a%2Bb%3Dc");

    // Test empty string
    assert_eq!(percent_encode(""), "");
}

/// Unit test for the generate_nonce function.
///
/// This test verifies that the nonce generation function produces:
/// - Unique nonces on each call (no collisions)
/// - Nonces of exactly 32 characters
/// - Nonces containing only alphanumeric characters
#[test]
fn test_generate_nonce() {
    let nonce1 = generate_nonce();
    let nonce2 = generate_nonce();

    // Nonces should be different
    assert_ne!(nonce1, nonce2);

    // Nonces should be 32 characters long
    assert_eq!(nonce1.len(), 32);
    assert_eq!(nonce2.len(), 32);

    // Nonces should only contain alphanumeric characters
    assert!(nonce1.chars().all(|c| c.is_ascii_alphanumeric()));
    assert!(nonce2.chars().all(|c| c.is_ascii_alphanumeric()));
}

/// Unit test for the get_current_timestamp function.
///
/// This test verifies that the timestamp function:
/// - Returns valid Unix timestamps as strings
/// - Produces monotonically increasing timestamps
/// - Returns reasonable timestamps (after 2020)
#[test]
fn test_get_current_timestamp() {
    let timestamp1 = get_current_timestamp().unwrap();
    let timestamp2 = get_current_timestamp().unwrap();

    // Timestamps should be valid numbers
    let ts1: u64 = timestamp1.parse().unwrap();
    let ts2: u64 = timestamp2.parse().unwrap();

    // Second timestamp should be >= first timestamp
    assert!(ts2 >= ts1);

    // Should be reasonable Unix timestamp (after 2020)
    assert!(ts1 > 1577836800); // 2020-01-01
}

/// Unit test for the build_oauth_params function.
///
/// This test verifies that the OAuth parameter building function:
/// - Includes all required OAuth 1.0a parameters
/// - Uses the correct values from the configuration
/// - Generates valid nonces and timestamps
/// - Returns parameters in the correct format
#[test]
fn test_build_oauth_params() {
    let config = TwitterConfig {
        consumer_key: "test_consumer_key".to_string(),
        consumer_secret: "test_consumer_secret".to_string(),
        access_token: "test_access_token".to_string(),
        access_token_secret: "test_access_token_secret".to_string(),
    };

    let oauth_params = build_oauth_params(&config).unwrap();

    // Check required OAuth parameters
    assert_eq!(oauth_params["oauth_consumer_key"], "test_consumer_key");
    assert_eq!(oauth_params["oauth_token"], "test_access_token");
    assert_eq!(oauth_params["oauth_signature_method"], "HMAC-SHA1");
    assert_eq!(oauth_params["oauth_version"], "1.0");

    // Check that nonce and timestamp are present
    assert!(oauth_params.contains_key("oauth_nonce"));
    assert!(oauth_params.contains_key("oauth_timestamp"));

    // Nonce should be 32 characters
    assert_eq!(oauth_params["oauth_nonce"].len(), 32);

    // Timestamp should be a valid number
    let _: u64 = oauth_params["oauth_timestamp"].parse().unwrap();
}

/// Unit test for the build_auth_header function.
///
/// This test verifies that the authorization header building function:
/// - Starts with "OAuth " prefix
/// - Includes all OAuth parameters in the correct format
/// - Properly formats parameter values with quotes
/// - Handles multiple parameters correctly
#[test]
fn test_build_auth_header() {
    let mut oauth_params = BTreeMap::new();
    oauth_params.insert("oauth_consumer_key".to_string(), "test_key".to_string());
    oauth_params.insert("oauth_token".to_string(), "test_token".to_string());
    oauth_params.insert("oauth_signature".to_string(), "test_signature".to_string());

    let auth_header = build_auth_header(&oauth_params);

    assert!(auth_header.starts_with("OAuth "));
    assert!(auth_header.contains("oauth_consumer_key=\"test_key\""));
    assert!(auth_header.contains("oauth_token=\"test_token\""));
    assert!(auth_header.contains("oauth_signature=\"test_signature\""));
}

/// Unit test for the generate_oauth_signature function.
///
/// This test verifies that the OAuth signature generation function:
/// - Produces valid base64-encoded signatures
/// - Generates deterministic signatures for the same inputs
/// - Handles all required OAuth parameters correctly
/// - Follows the OAuth 1.0a specification
#[test]
fn test_generate_oauth_signature() {
    let mut params = BTreeMap::new();
    params.insert(
        "oauth_consumer_key".to_string(),
        "test_consumer_key".to_string(),
    );
    params.insert("oauth_nonce".to_string(), "test_nonce".to_string());
    params.insert(
        "oauth_signature_method".to_string(),
        "HMAC-SHA1".to_string(),
    );
    params.insert("oauth_timestamp".to_string(), "1234567890".to_string());
    params.insert("oauth_token".to_string(), "test_token".to_string());
    params.insert("oauth_version".to_string(), "1.0".to_string());

    let signature = generate_oauth_signature(
        "POST",
        "https://api.x.com/2/tweets",
        &params,
        "test_consumer_secret",
        "test_token_secret",
    );

    // Signature should be a valid base64 string
    assert!(!signature.is_empty());
    assert!(signature
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));

    // Same parameters should generate same signature
    let signature2 = generate_oauth_signature(
        "POST",
        "https://api.x.com/2/tweets",
        &params,
        "test_consumer_secret",
        "test_token_secret",
    );
    assert_eq!(signature, signature2);
}

/// Unit test for the TwitterConfig::from_env function.
///
/// This test verifies that the configuration loading function:
/// - Returns an error when required environment variables are missing
/// - Successfully loads configuration when all variables are present
/// - Correctly maps environment variables to struct fields
/// - Properly cleans up environment variables after testing
#[test]
fn test_twitter_config_from_env() {
    // Test with missing environment variables
    std::env::remove_var("xapi_consumer_key");
    std::env::remove_var("xapi_consumer_secret");
    std::env::remove_var("xapi_access_token");
    std::env::remove_var("xapi_access_token_secret");

    let result = TwitterConfig::from_env();
    assert!(result.is_err());

    // Test with all environment variables set
    std::env::set_var("xapi_consumer_key", "test_key");
    std::env::set_var("xapi_consumer_secret", "test_secret");
    std::env::set_var("xapi_access_token", "test_token");
    std::env::set_var("xapi_access_token_secret", "test_token_secret");

    let config = TwitterConfig::from_env().unwrap();
    assert_eq!(config.consumer_key, "test_key");
    assert_eq!(config.consumer_secret, "test_secret");
    assert_eq!(config.access_token, "test_token");
    assert_eq!(config.access_token_secret, "test_token_secret");

    // Clean up environment variables
    std::env::remove_var("xapi_consumer_key");
    std::env::remove_var("xapi_consumer_secret");
    std::env::remove_var("xapi_access_token");
    std::env::remove_var("xapi_access_token_secret");
}

/// Unit test for the get_server_port function.
///
/// This test verifies that the server port configuration function:
/// - Returns the default port (3000) when PORT environment variable is not set
/// - Correctly parses and returns custom port values from environment
/// - Properly cleans up environment variables after testing
#[test]
fn test_get_server_port() {
    // Test default port
    std::env::remove_var("PORT");
    let port = get_server_port();
    assert_eq!(port, 3000);

    // Test custom port
    std::env::set_var("PORT", "8080");
    let port = get_server_port();
    assert_eq!(port, 8080);

    // Clean up
    std::env::remove_var("PORT");
}
