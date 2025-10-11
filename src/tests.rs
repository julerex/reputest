use crate::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use std::collections::BTreeMap;
use tower::ServiceExt;
use http_body_util::BodyExt;

// Helper function to create the app for testing
fn create_test_app() -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .route("/tweet", post(handle_tweet))
}

#[tokio::test]
async fn test_handle_root() {
    let response = handle_root().await;
    assert_eq!(response, "A new reputest is in the house!");
}

#[tokio::test]
async fn test_handle_reputest_get() {
    let response = handle_reputest_get().await;
    assert_eq!(response, "Reputesting!");
}

#[tokio::test]
async fn test_handle_reputest_post() {
    let response = handle_reputest_post().await;
    assert_eq!(response, "Reputesting!");
}

#[tokio::test]
async fn test_handle_health() {
    let response = handle_health().await;
    let Json(json_response): Json<Value> = response;
    
    assert_eq!(json_response["status"], "healthy");
    assert_eq!(json_response["service"], "reputest");
}

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
    assert!(json_response["message"].as_str().unwrap().contains("Failed to post tweet"));
}

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

#[test]
fn test_generate_oauth_signature() {
    let mut params = BTreeMap::new();
    params.insert("oauth_consumer_key".to_string(), "test_consumer_key".to_string());
    params.insert("oauth_nonce".to_string(), "test_nonce".to_string());
    params.insert("oauth_signature_method".to_string(), "HMAC-SHA1".to_string());
    params.insert("oauth_timestamp".to_string(), "1234567890".to_string());
    params.insert("oauth_token".to_string(), "test_token".to_string());
    params.insert("oauth_version".to_string(), "1.0".to_string());
    
    let signature = generate_oauth_signature(
        "POST",
        "https://api.x.com/2/tweets",
        &params,
        "test_consumer_secret",
        "test_token_secret"
    );
    
    // Signature should be a valid base64 string
    assert!(!signature.is_empty());
    assert!(signature.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    
    // Same parameters should generate same signature
    let signature2 = generate_oauth_signature(
        "POST",
        "https://api.x.com/2/tweets",
        &params,
        "test_consumer_secret",
        "test_token_secret"
    );
    assert_eq!(signature, signature2);
}

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
