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
pub mod cronjob;
pub mod handlers;
pub mod oauth;
pub mod twitter;

// Re-export commonly used types and functions
pub use config::{get_server_port, TwitterConfig};
pub use cronjob::{run_gmgv_cronjob, start_gmgv_cronjob};
pub use handlers::{
    handle_health, handle_reputest_get, handle_reputest_post, handle_root, handle_tweet,
};
pub use oauth::{
    build_auth_header, build_oauth_params, generate_nonce, generate_oauth_signature,
    get_current_timestamp, percent_encode,
};
pub use twitter::{post_tweet, search_tweets_with_hashtag};
