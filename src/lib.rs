//! # Reputest Library
//!
//! A Rust web service library that provides HTTP endpoints for testing and demonstration purposes.
//! The service includes functionality to post tweets and search tweets via the Twitter/X API using
//! OAuth 2.0 User Context Access Token authentication for v2 endpoints.
//!
//! ## Features
//!
//! - HTTP server with multiple endpoints (`/`, `/reputest`, `/health`)
//! - Twitter/X API integration with OAuth 2.0 User Context Access Token authentication
//! - Comprehensive test suite
//! - Structured logging
//! - Health check endpoint
//!
//! ## Configuration
//!
//! The following configuration is required:
//! - Database with access tokens stored in the `access_tokens` table
//! - `DATABASE_URL`: PostgreSQL connection string
//! - `PORT`: Server port (defaults to 3000)
//!
//!
//! ## API Endpoints
//!
//! - `GET /`: Returns a welcome message
//! - `GET /reputest`: Returns "Reputesting!" message
//! - `POST /reputest`: Returns "Reputesting!" message
//! - `GET /health`: Returns service health status

pub mod config;
pub mod cronjob;
pub mod crypto;
pub mod db;
pub mod handlers;
pub mod oauth;
pub mod twitter;

// Re-export commonly used types and functions
pub use config::{get_server_port, TwitterConfig};
pub use cronjob::{run_gmgv_cronjob, start_gmgv_cronjob};
pub use crypto::validate_encryption_config;
pub use handlers::{handle_health, handle_reputest_get, handle_reputest_post, handle_root};
pub use oauth::build_oauth2_user_context_header;
pub use twitter::search_tweets_with_hashtag;

#[cfg(test)]
mod tests;
