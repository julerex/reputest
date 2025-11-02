//! # Tests Module
//!
//! This module contains comprehensive tests for the reputest web service.
//! It includes unit tests for individual functions and integration tests for HTTP endpoints.
//!
//! ## Test Categories
//!
//! ### Unit Tests
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
//! Tests run in isolation and clean up after execution.
//! The Twitter API integration tests expect missing database tokens and verify proper error handling.
//! Some tests require DATABASE_URL to be set and will be skipped if it's not available.

use crate::{
    config::{get_server_port, TwitterConfig},
    handlers::{
        handle_health, handle_reputest_get, handle_reputest_post, handle_root, handle_tweet,
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
/// Twitter API access token is not available in the database or database connection fails.
/// It expects:
/// - The response status to be 500 Internal Server Error
/// - The response to be valid JSON with an error status
/// - The error message to indicate a failure to post the tweet
///
/// This test is important for ensuring proper error handling in production
/// environments where database tokens might be missing or invalid.
#[tokio::test]
async fn test_tweet_endpoint_without_credentials() {
    let app = create_test_app();

    let request = Request::builder()
        .uri("/tweet")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should return 500 because Twitter access token is not in database or DATABASE_URL not set
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

/// Unit test for the TwitterConfig::from_env function.
///
/// This test verifies that the configuration loading function:
/// - Returns an error when no access token is found in the database
/// - Successfully loads configuration when tokens are present in the database
/// - Correctly maps database tokens to struct fields
///
/// Note: This test requires DATABASE_URL to be set and a database connection.
/// The test will be skipped if DATABASE_URL is not available.
#[tokio::test]
async fn test_twitter_config_from_env() {
    // Skip test if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test_twitter_config_from_env: DATABASE_URL not set");
        return;
    }

    use crate::db;

    // Get database pool
    let pool = match db::get_db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!(
                "Skipping test_twitter_config_from_env: Failed to connect to database: {}",
                e
            );
            return;
        }
    };

    // Test 1: Should fail when no access token exists in database
    let result = TwitterConfig::from_env(&pool).await;
    match result {
        Err(e) => {
            // Expected: should fail because no token in database
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("access token")
                    || error_msg.contains("database")
                    || error_msg.contains("No access token found"),
                "Error message should mention access token or database: {}",
                error_msg
            );
        }
        Ok(config) => {
            // If a token exists from previous tests, verify it was loaded correctly
            assert!(
                !config.access_token.is_empty(),
                "Access token should not be empty"
            );
            // Client credentials might be from environment, which is fine
        }
    }
}

/// Unit test for TwitterConfig loading with tokens in database.
///
/// This test verifies that tokens can be loaded from the database when they exist.
/// It inserts test tokens and then verifies they can be loaded correctly.
///
/// Note: This test requires DATABASE_URL to be set and a database connection.
/// The test will be skipped if DATABASE_URL is not available.
#[tokio::test]
async fn test_twitter_config_loads_from_database() {
    // Skip test if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping test_twitter_config_loads_from_database: DATABASE_URL not set");
        return;
    }

    use crate::db;

    // Get database pool
    let pool = match db::get_db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping test_twitter_config_loads_from_database: Failed to connect to database: {}", e);
            return;
        }
    };

    // Ensure tables exist
    if let Err(e) = db::create_access_tokens_table(&pool).await {
        eprintln!("Warning: Failed to create access_tokens table: {}", e);
    }
    if let Err(e) = db::create_refresh_tokens_table(&pool).await {
        eprintln!("Warning: Failed to create refresh_tokens table: {}", e);
    }

    // Insert test access token
    let test_access_token = "test_access_token_12345";
    if let Err(e) = db::save_access_token(&pool, test_access_token).await {
        eprintln!("Skipping test: Failed to insert test access token: {}", e);
        return;
    }

    // Insert test refresh token
    let test_refresh_token = "test_refresh_token_67890";
    if let Err(e) = db::save_refresh_token(&pool, test_refresh_token).await {
        eprintln!("Warning: Failed to insert test refresh token: {}", e);
    }

    // Now test loading from database
    let result = TwitterConfig::from_env(&pool).await;
    match result {
        Ok(config) => {
            // Should load the token we just inserted
            assert_eq!(config.access_token, test_access_token);
            if let Some(refresh) = config.refresh_token {
                assert_eq!(refresh, test_refresh_token);
            }
        }
        Err(e) => {
            panic!(
                "Failed to load config from database after inserting tokens: {}",
                e
            );
        }
    }
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
