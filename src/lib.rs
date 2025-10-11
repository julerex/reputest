//! # Reputest Library
//!
//! A Rust web service library that provides HTTP endpoints for testing and demonstration purposes.
//! The service includes functionality to post tweets via the Twitter/X API using OAuth 1.0a authentication.
//!
//! ## Features
//!
//! - HTTP server with multiple endpoints (`/`, `/reputest`, `/health`, `/tweet`)
//! - Twitter/X API integration with OAuth 1.0a authentication
//! - Comprehensive test suite
//! - Structured logging
//! - Health check endpoint
//!
//! ## Environment Variables
//!
//! The following environment variables are required for Twitter API functionality:
//! - `xapi_consumer_key`: Twitter API consumer key
//! - `xapi_consumer_secret`: Twitter API consumer secret  
//! - `xapi_access_token`: Twitter API access token
//! - `xapi_access_token_secret`: Twitter API access token secret
//! - `PORT`: Server port (defaults to 3000)
//!
//! ## API Endpoints
//!
//! - `GET /`: Returns a welcome message
//! - `GET /reputest`: Returns "Reputesting!" message
//! - `POST /reputest`: Returns "Reputesting!" message
//! - `GET /health`: Returns service health status
//! - `POST /tweet`: Posts a tweet to Twitter/X (requires API credentials)

pub mod config;
pub mod oauth;
pub mod twitter;
pub mod handlers;

// Re-export commonly used types and functions
pub use config::{TwitterConfig, get_server_port};
pub use oauth::{
    generate_oauth_signature, percent_encode, generate_nonce, get_current_timestamp,
    build_oauth_params, build_auth_header,
};
pub use twitter::post_tweet;
pub use handlers::{
    handle_health, handle_reputest_get, handle_reputest_post, handle_root, handle_tweet,
};
