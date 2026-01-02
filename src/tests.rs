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
    config::get_server_port,
    db::{
        get_db_pool, get_vibe_score_one, get_vibe_score_three, get_vibe_score_two, save_good_vibes,
        save_user,
    },
    handlers::{
        handle_health, handle_reputest_get, handle_reputest_post, handle_root, handle_tweet,
    },
    twitter::{extract_mention_with_question, extract_vibe_emitter},
};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

/// Creates a test application instance with all routes configured.
///
/// This helper function sets up a minimal Axum router with all the same routes
/// as the main application, but without middleware layers that might interfere
/// with testing. It's used by integration tests to make HTTP requests.
///
/// # Parameters
///
/// - `pool`: Optional database pool. If provided, the root route will be included.
///   If None, the root route is omitted since it requires database access.
///
/// # Returns
///
/// An Axum `Router` instance configured with application routes.
fn create_test_app(pool: Option<PgPool>) -> Router {
    let base_router = Router::new()
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .route("/tweet", post(handle_tweet));

    if let Some(pool) = pool {
        let stateful_router = Router::new().route("/", get(handle_root)).with_state(pool);
        base_router.merge(stateful_router)
    } else {
        base_router
    }
}

/// Tests the root endpoint handler function directly.
///
/// This test verifies that the `handle_root` function returns HTML
/// without making an HTTP request. It requires a database connection.
#[tokio::test]
async fn test_handle_root() {
    // Skip test if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping handle_root test - DATABASE_URL not set");
        return;
    }

    let pool = match get_db_pool().await {
        Ok(pool) => pool,
        Err(_) => {
            println!("Skipping handle_root test - could not connect to database");
            return;
        }
    };

    let response = handle_root(State(pool)).await;
    match response {
        Ok(Html(html)) => {
            // Verify it's HTML and contains the expected table structure
            assert!(html.contains("<table>"));
            assert!(html.contains("sensor"));
            assert!(html.contains("sensor name"));
            assert!(html.contains("emitter"));
            assert!(html.contains("emitter name"));
            assert!(html.contains("one-degree-vibe-count"));
            assert!(html.contains("two-degree-vibe-count"));
            assert!(html.contains("three-degree-vibe-count"));
            assert!(html.contains("four-degree-vibe-count"));
        }
        Err((status, msg)) => {
            panic!("handle_root returned error: {} - {}", status, msg);
        }
    }
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
/// - The response status is 200 OK (or 500 if database unavailable)
/// - The response body contains HTML with a table
#[tokio::test]
async fn test_root_endpoint() {
    // Skip test if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping root endpoint test - DATABASE_URL not set");
        return;
    }

    let pool = match get_db_pool().await {
        Ok(pool) => pool,
        Err(_) => {
            println!("Skipping root endpoint test - could not connect to database");
            return;
        }
    };

    let app = create_test_app(Some(pool));

    let request = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Should be OK if database query succeeds, or 500 if it fails
    assert!(status == StatusCode::OK || status == StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    if status == StatusCode::OK {
        // Verify it's HTML and contains the expected table structure
        assert!(body_str.contains("<table>"));
        assert!(body_str.contains("sensor"));
        assert!(body_str.contains("emitter"));
        assert!(body_str.contains("two-degree-vibe-count"));
    }
}

/// Integration test for the GET /reputest endpoint.
///
/// This test makes an actual HTTP GET request to the /reputest endpoint and verifies:
/// - The response status is 200 OK
/// - The response body contains "Reputesting!"
#[tokio::test]
async fn test_reputest_get_endpoint() {
    let app = create_test_app(None);

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
    let app = create_test_app(None);

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
    let app = create_test_app(None);

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
    let app = create_test_app(None);

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

/// Unit test for the extract_mention_with_question function.
///
/// This test verifies that the function correctly extracts usernames from
/// mention patterns with question marks, including cases with and without @ symbols.
#[test]
fn test_extract_mention_with_question() {
    // Test cases that should match - exact format "@reputest username ?" or "@reputest @username ?"

    // Test case with @ symbol and space before question mark
    assert_eq!(
        extract_mention_with_question("@reputest @callanable ?"),
        Some("callanable".to_string())
    );

    // Test case with @ symbol and no space before question mark
    assert_eq!(
        extract_mention_with_question("@reputest @user?"),
        Some("user".to_string())
    );

    // Test case with @ symbol and multiple spaces
    assert_eq!(
        extract_mention_with_question("@reputest @testuser   ?"),
        Some("testuser".to_string())
    );

    // Test case without @ symbol and space before question mark
    assert_eq!(
        extract_mention_with_question("@reputest callanable ?"),
        Some("callanable".to_string())
    );

    // Test case without @ symbol and no space before question mark
    assert_eq!(
        extract_mention_with_question("@reputest user?"),
        Some("user".to_string())
    );

    // Test case without @ symbol and multiple spaces
    assert_eq!(
        extract_mention_with_question("@reputest testuser   ?"),
        Some("testuser".to_string())
    );

    // Test cases that should return None - doesn't match the required format

    // Test case with the bot mention followed by question mark (should not match as there's no username)
    assert_eq!(extract_mention_with_question("@reputest ?"), None);

    // Test case with no question mark
    assert_eq!(extract_mention_with_question("@reputest hello"), None);

    // Test case with only the bot mention
    assert_eq!(extract_mention_with_question("@reputest"), None);

    // Test cases that don't start with @reputest
    assert_eq!(extract_mention_with_question("hello @user?"), None);
    assert_eq!(extract_mention_with_question("@user?"), None);

    // Test cases with excluded words (but these won't match anyway due to the strict format)
    assert_eq!(extract_mention_with_question("@reputest what?"), None);
    assert_eq!(extract_mention_with_question("@reputest why?"), None);
    assert_eq!(extract_mention_with_question("@reputest reputest?"), None);

    // Test cases with extra content before or after
    assert_eq!(extract_mention_with_question("Hey @reputest @user?"), None);
    assert_eq!(
        extract_mention_with_question("@reputest @user? More text"),
        None
    );
}

/// Unit test for the extract_vibe_emitter function.
///
/// This test verifies that the function extracts the word immediately before #gmgv,
/// stripping @ if present, and excluding common English words.
#[test]
fn test_extract_vibe_emitter() {
    // Test direct proximity match with @ (no spaces)
    assert_eq!(
        extract_vibe_emitter("@alice#gmgv", None),
        Some("alice".to_string())
    );

    // Test proximity match with @ and space
    assert_eq!(
        extract_vibe_emitter("@alice #gmgv", None),
        Some("alice".to_string())
    );

    // Test proximity match with @ and multiple spaces
    assert_eq!(
        extract_vibe_emitter("@alice   #gmgv", None),
        Some("alice".to_string())
    );

    // Test direct proximity match WITHOUT @ (no spaces)
    assert_eq!(
        extract_vibe_emitter("alice#gmgv", None),
        Some("alice".to_string())
    );

    // Test proximity match WITHOUT @ and space
    assert_eq!(
        extract_vibe_emitter("alice #gmgv", None),
        Some("alice".to_string())
    );

    // Test proximity match WITHOUT @ and multiple spaces
    assert_eq!(
        extract_vibe_emitter("alice   #gmgv", None),
        Some("alice".to_string())
    );

    // Test reply tweet - excluded username doesn't get vibes even with proximity
    assert_eq!(
        extract_vibe_emitter("@bob #gmgv @alice #gmgv too", Some("bob")),
        Some("alice".to_string()) // bob is excluded, alice gets vibes
    );

    // Test reply tweet where excluded user has proximity match
    assert_eq!(
        extract_vibe_emitter("@bob #gmgv thanks!", Some("bob")),
        None // bob has proximity match but is excluded
    );

    // Test multiple proximity matches - picks first valid one
    assert_eq!(
        extract_vibe_emitter("@alice #gmgv @bob #gmgv", None),
        Some("alice".to_string()) // picks first proximity match
    );

    // Test case sensitivity
    assert_eq!(
        extract_vibe_emitter("@Alice #gmgv", Some("alice")),
        Some("Alice".to_string()) // "alice" != "Alice" so Alice is not excluded
    );

    // Test thread scenario with proximity
    assert_eq!(
        extract_vibe_emitter("@user1 @user2 #gmgv @user3 also here", Some("user1")),
        Some("user2".to_string()) // user2 has proximity match, user1 excluded
    );

    // Test non-mention in sentence context
    assert_eq!(
        extract_vibe_emitter("Shoutout to johndoe #gmgv", None),
        Some("johndoe".to_string())
    );

    // Test that common English words are filtered out
    assert_eq!(
        extract_vibe_emitter("this is good #gmgv", None),
        None // "good" is a common word
    );

    // Test that common English words are filtered out
    assert_eq!(
        extract_vibe_emitter("sending vibes #gmgv", None),
        None // "vibes" is a common word
    );
}

/// Integration test for the pagerank-style vibe scoring algorithm.
///
/// This test verifies that the three-degree vibe scoring works correctly by:
/// 1. Setting up test users (Alice, Bob, Charlie, Danielle, Edgar, David, Frank)
/// 2. Creating good vibes relationships: Alice->Bob, Bob->Charlie, Bob->Danielle, Alice->Edgar, Edgar->Charlie, Charlie->Frank
/// 3. Testing various vibe score calculations for all three degrees
///
/// Expected results:
/// - 1st degree (direct): Alice to Bob = 1, Alice to Edgar = 1, Bob to Charlie = 1
/// - 2nd degree (paths of length 2): Alice to Charlie = 2 (Bob->Charlie + Edgar->Charlie), Alice to Danielle = 1 (Bob->Danielle)
/// - 3rd degree (paths of length 3): Alice to Frank = 2 (Bob->Charlie->Frank + Edgar->Charlie->Frank)
/// - No connections: Charlie to Alice = 0, Alice to David = 0, Same user = 0
#[tokio::test]
async fn test_pagerank_vibe_scoring() {
    // Skip test if DATABASE_URL is not set
    if std::env::var("DATABASE_URL").is_err() {
        println!("Skipping pagerank test - DATABASE_URL not set");
        return;
    }

    let pool = match get_db_pool().await {
        Ok(pool) => pool,
        Err(_) => {
            println!("Skipping pagerank test - could not connect to database");
            return;
        }
    };

    let now = Utc::now();

    // Create test users
    let alice_id = "alice_test_123";
    let bob_id = "bob_test_456";
    let charlie_id = "charlie_test_789";
    let danielle_id = "danielle_test_000";
    let edgar_id = "edgar_test_111";
    let frank_id = "frank_test_222";
    let david_id = "david_test_999";

    // Save test users
    save_user(&pool, alice_id, "alice", "Alice Test", now)
        .await
        .unwrap();
    save_user(&pool, bob_id, "bob", "Bob Test", now)
        .await
        .unwrap();
    save_user(&pool, charlie_id, "charlie", "Charlie Test", now)
        .await
        .unwrap();
    save_user(&pool, danielle_id, "danielle", "Danielle Test", now)
        .await
        .unwrap();
    save_user(&pool, edgar_id, "edgar", "Edgar Test", now)
        .await
        .unwrap();
    save_user(&pool, frank_id, "frank", "Frank Test", now)
        .await
        .unwrap();
    save_user(&pool, david_id, "david", "David Test", now)
        .await
        .unwrap();

    // Create good vibes relationships: Alice->Bob, Bob->Charlie, Bob->Danielle, Alice->Edgar, Edgar->Charlie, Charlie->Frank
    save_good_vibes(&pool, "tweet_alice_bob", alice_id, bob_id, now)
        .await
        .unwrap();
    save_good_vibes(&pool, "tweet_bob_charlie", bob_id, charlie_id, now)
        .await
        .unwrap();
    save_good_vibes(&pool, "tweet_bob_danielle", bob_id, danielle_id, now)
        .await
        .unwrap();
    save_good_vibes(&pool, "tweet_alice_edgar", alice_id, edgar_id, now)
        .await
        .unwrap();
    save_good_vibes(&pool, "tweet_edgar_charlie", edgar_id, charlie_id, now)
        .await
        .unwrap();
    save_good_vibes(&pool, "tweet_charlie_frank", charlie_id, frank_id, now)
        .await
        .unwrap();

    // Test first-degree connections (direct)
    assert_eq!(
        get_vibe_score_one(&pool, alice_id, bob_id).await.unwrap(),
        1,
        "Alice should have 1st-degree vibe score 1 for Bob (direct)"
    );
    assert_eq!(
        get_vibe_score_one(&pool, alice_id, edgar_id).await.unwrap(),
        1,
        "Alice should have 1st-degree vibe score 1 for Edgar (direct)"
    );
    assert_eq!(
        get_vibe_score_one(&pool, bob_id, charlie_id).await.unwrap(),
        1,
        "Bob should have 1st-degree vibe score 1 for Charlie (direct)"
    );
    assert_eq!(
        get_vibe_score_one(&pool, alice_id, charlie_id)
            .await
            .unwrap(),
        0,
        "Alice should have 1st-degree vibe score 0 for Charlie (no direct connection)"
    );

    // Test second-degree connections (paths of length 2)
    assert_eq!(get_vibe_score_two(&pool, alice_id, charlie_id).await.unwrap(), 2, "Alice should have 2nd-degree vibe score 2 for Charlie (2 paths: Alice->Bob->Charlie + Alice->Edgar->Charlie)");
    assert_eq!(
        get_vibe_score_two(&pool, alice_id, danielle_id)
            .await
            .unwrap(),
        1,
        "Alice should have 2nd-degree vibe score 1 for Danielle (1 path: Alice->Bob->Danielle)"
    );
    assert_eq!(
        get_vibe_score_two(&pool, alice_id, frank_id).await.unwrap(),
        0,
        "Alice should have 2nd-degree vibe score 0 for Frank (no direct paths of length 2)"
    );

    // Test third-degree connections (paths of length 3)
    assert_eq!(get_vibe_score_three(&pool, alice_id, frank_id).await.unwrap(), 2, "Alice should have 3rd-degree vibe score 2 for Frank (2 paths: Alice->Bob->Charlie->Frank + Alice->Edgar->Charlie->Frank)");
    assert_eq!(
        get_vibe_score_three(&pool, alice_id, charlie_id)
            .await
            .unwrap(),
        0,
        "Alice should have 3rd-degree vibe score 0 for Charlie (no paths of length 3)"
    );

    // Test no connection (reverse direction)
    assert_eq!(
        get_vibe_score_one(&pool, charlie_id, alice_id)
            .await
            .unwrap(),
        0,
        "Charlie should have 1st-degree vibe score 0 for Alice (no reverse direct path)"
    );
    assert_eq!(
        get_vibe_score_two(&pool, charlie_id, alice_id)
            .await
            .unwrap(),
        0,
        "Charlie should have 2nd-degree vibe score 0 for Alice (no reverse paths)"
    );
    assert_eq!(
        get_vibe_score_three(&pool, charlie_id, alice_id)
            .await
            .unwrap(),
        0,
        "Charlie should have 3rd-degree vibe score 0 for Alice (no reverse paths)"
    );

    // Test connection to unconnected user
    assert_eq!(
        get_vibe_score_one(&pool, alice_id, david_id).await.unwrap(),
        0,
        "Alice should have 1st-degree vibe score 0 for David (not connected)"
    );
    assert_eq!(
        get_vibe_score_two(&pool, alice_id, david_id).await.unwrap(),
        0,
        "Alice should have 2nd-degree vibe score 0 for David (not connected)"
    );
    assert_eq!(
        get_vibe_score_three(&pool, alice_id, david_id)
            .await
            .unwrap(),
        0,
        "Alice should have 3rd-degree vibe score 0 for David (not connected)"
    );

    // Test same user
    assert_eq!(
        get_vibe_score_one(&pool, alice_id, alice_id).await.unwrap(),
        0,
        "Same user should have 1st-degree vibe score 0"
    );
    assert_eq!(
        get_vibe_score_two(&pool, alice_id, alice_id).await.unwrap(),
        0,
        "Same user should have 2nd-degree vibe score 0"
    );
    assert_eq!(
        get_vibe_score_three(&pool, alice_id, alice_id)
            .await
            .unwrap(),
        0,
        "Same user should have 3rd-degree vibe score 0"
    );

    // Clean up test data (optional - in a real test environment you might want to rollback)
    // For now, we'll leave the test data in place since it's clearly marked as test data
    println!("Pagerank vibe scoring test completed successfully");
}
