//! Configuration module for the reputest service.
//!
//! This module contains configuration structures and environment variable handling
//! for the Twitter/X API integration.

use std::env;

/// Configuration struct for Twitter/X API credentials.
///
/// This struct holds the Bearer Token required to authenticate with the Twitter/X API v2 endpoints.
/// The Bearer Token is loaded from environment variables.
#[derive(Debug)]
pub struct TwitterConfig {
    /// The Bearer Token for OAuth 2.0 authentication (v2 endpoints)
    pub bearer_token: String,
}

impl TwitterConfig {
    /// Creates a new `TwitterConfig` instance by loading credentials from environment variables.
    ///
    /// # Required Environment Variables
    ///
    /// - `xapi_bearer_token`: Twitter API Bearer Token (OAuth 2.0 for v2 endpoints)
    ///
    /// # Returns
    ///
    /// - `Ok(TwitterConfig)`: If all required environment variables are present
    /// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If any environment variable is missing
    ///
    /// # Example
    ///
    /// ```rust
    /// use reputest::TwitterConfig;
    ///
    /// // Set environment variables before calling
    /// std::env::set_var("xapi_bearer_token", "your_bearer_token");
    ///
    /// let config = TwitterConfig::from_env().unwrap();
    /// ```
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TwitterConfig {
            bearer_token: env::var("xapi_bearer_token")?,
        })
    }
}

/// Gets the server port from environment variables or returns the default.
///
/// This function reads the `PORT` environment variable and parses it as a u16.
/// If the environment variable is not set or cannot be parsed, it defaults to 3000.
///
/// # Returns
///
/// The port number as a u16.
///
/// # Panics
///
/// This function will panic if the `PORT` environment variable is set to a value
/// that cannot be parsed as a valid port number.
///
/// # Example
///
/// ```rust
/// use reputest::get_server_port;
///
/// // With PORT=8080 set in environment
/// let port = get_server_port(); // Returns 8080
///
/// // With no PORT set
/// let port = get_server_port(); // Returns 3000
/// ```
pub fn get_server_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number")
}
