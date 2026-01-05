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
//! ## Configuration
//!
//! The following configuration is required:
//! - Database with access tokens stored in the `access_tokens` table
//! - `DATABASE_URL`: PostgreSQL connection string
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
    http::HeaderValue,
    routing::{get, post},
    Router,
};
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{set_header::SetResponseHeaderLayer, trace::TraceLayer};

mod config;
mod cronjob;
mod crypto;
mod db;
mod handlers;
mod oauth;
mod twitter;

use config::get_server_port;
use cronjob::start_gmgv_cronjob;
use handlers::{handle_health, handle_reputest_get, handle_reputest_post, handle_root};

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

    // Validate security configuration at startup
    if let Err(e) = crypto::validate_encryption_config() {
        log::error!(
            "SECURITY ERROR: Token encryption is not properly configured: {}",
            e
        );
        log::error!("Set TOKEN_ENCRYPTION_KEY environment variable with a 32-byte hex key.");
        log::error!("Generate a key with: openssl rand -hex 32");
        log::error!("Refusing to start without encryption configured.");
        std::process::exit(1);
    }
    info!("Security configuration validated successfully");

    // Note: Tokens are now loaded directly from the database when needed
    // No need to pre-load them as environment variables

    // Create database pool
    let db_pool = match db::get_db_pool().await {
        Ok(pool) => {
            info!("Database pool created successfully");
            pool
        }
        Err(e) => {
            log::error!("Failed to create database pool: {}", e);
            log::error!("Server will start but database-dependent endpoints may fail");
            // Create a dummy pool - this won't work but allows server to start
            // In production, you might want to panic or exit here
            return;
        }
    };

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

    // Configure rate limiting: 30 requests per minute per IP
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2) // Refill rate: 2 tokens per second
            .burst_size(30) // Maximum burst: 30 requests
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("Failed to create rate limiter config"),
    );

    let governor_limiter = governor_config.limiter().clone();

    // Spawn a background task to clean up rate limiter storage periodically
    let cleanup_interval = std::time::Duration::from_secs(60);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(cleanup_interval).await;
            governor_limiter.retain_recent();
        }
    });

    // Build the HTTP application with all routes and middleware
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .with_state(db_pool)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(GovernorLayer {
                    config: governor_config,
                })
                // SECURITY: Add security headers to all responses
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_CONTENT_TYPE_OPTIONS,
                    HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_FRAME_OPTIONS,
                    HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::X_XSS_PROTECTION,
                    HeaderValue::from_static("1; mode=block"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static(
                        "default-src 'self'; style-src 'self' 'unsafe-inline'",
                    ),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::REFERRER_POLICY,
                    HeaderValue::from_static("strict-origin-when-cross-origin"),
                )),
        );

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
