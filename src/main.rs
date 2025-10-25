//! # Reputest
//!
//! A Rust web service that provides HTTP endpoints for testing and demonstration purposes.
//! The service includes functionality to post tweets via the Twitter/X API using OAuth 2.0 User Context authentication.
//!
//! ## Features
//!
//! - HTTP server with multiple endpoints (`/`, `/reputest`, `/health`, `/tweet`)
//! - Twitter/X API integration with OAuth 2.0 User Context authentication
//! - Comprehensive test suite
//! - Structured logging
//! - Health check endpoint
//!
//! ## Environment Variables
//!
//! The following environment variables are required for Twitter API functionality:
//! - `xapi_access_token`: Twitter API Access token (OAuth 2.0 User Context for v2 endpoints)
//! - `PORT`: Server port (defaults to 3000)
//!
//! ## API Endpoints
//!
//! - `GET /`: Returns a welcome message
//! - `GET /reputest`: Returns "Reputesting!" message
//! - `POST /reputest`: Returns "Reputesting!" message
//! - `GET /health`: Returns service health status
//! - `POST /tweet`: Posts a tweet to Twitter/X (requires API credentials)

use axum::{
    routing::{get, post},
    Router,
};
use log::info;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod config;
mod cronjob;
mod handlers;
mod oauth;
mod twitter;

use config::get_server_port;
use cronjob::start_gmgv_cronjob;
use handlers::{
    handle_health, handle_reputest_get, handle_reputest_post, handle_root, handle_tweet,
};

/// Main entry point for the reputest web service.
///
/// This function initializes the logging system, sets up the HTTP server with all routes,
/// and starts listening for incoming requests. The server runs indefinitely until terminated.
///
/// # Server Configuration
///
/// The server is configured with the following routes:
/// - `GET /`: Root endpoint with welcome message
/// - `GET /reputest`: Test endpoint returning "Reputesting!"
/// - `POST /reputest`: Test endpoint returning "Reputesting!"
/// - `GET /health`: Health check endpoint
/// - `POST /tweet`: Twitter API integration endpoint
///
/// # Middleware
///
/// The server includes HTTP request tracing middleware for logging and debugging.
///
/// # Port Configuration
///
/// The server port is determined by the `PORT` environment variable, defaulting to 3000.
///
/// # Logging
///
/// The application uses the `env_logger` crate for structured logging. Log levels
/// can be controlled via the `RUST_LOG` environment variable.
///
/// # Example Usage
///
/// ```bash
/// # Run with default port 3000
/// cargo run
///
/// # Run on custom port
/// PORT=8080 cargo run
///
/// # Run with debug logging
/// RUST_LOG=debug cargo run
/// ```
///
/// # Panics
///
/// This function will panic if:
/// - The server port cannot be bound (e.g., port already in use)
/// - There's an error starting the HTTP server
#[tokio::main]
async fn main() {
    // Initialize the logging system
    env_logger::init();

    // Start the cronjob scheduler for GMGV hashtag monitoring
    let cronjob_handle = tokio::spawn(async {
        match start_gmgv_cronjob().await {
            Ok(scheduler) => {
                info!("Starting GMGV hashtag monitoring cronjob");
                if let Err(e) = scheduler.start().await {
                    log::error!("Failed to start cronjob scheduler: {}", e);
                    return;
                }
                // Keep the scheduler running indefinitely
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            }
            Err(e) => {
                log::error!("Failed to create cronjob scheduler: {}", e);
            }
        }
    });

    // Build the HTTP application with all routes and middleware
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .route("/tweet", post(handle_tweet))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Get the server port and bind address
    let port = get_server_port();
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Starting reputest server on {}", addr);

    // Bind to the address and start serving requests
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Run both the HTTP server and cronjob concurrently
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                log::error!("HTTP server error: {}", e);
            }
        }
        _ = cronjob_handle => {
            log::info!("Cronjob task completed");
        }
    }
}

#[cfg(test)]
mod tests;
