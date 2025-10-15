//! Configuration module for the reputest service.
//!
//! This module contains configuration structures and environment variable handling
//! for the Twitter/X API integration.

use std::env;

/// Configuration struct for Twitter/X API credentials.
///
/// This struct holds all the necessary OAuth 1.0a credentials required to authenticate
/// with the Twitter/X API. All fields are loaded from environment variables.
#[derive(Debug)]
pub struct TwitterConfig {
    /// The consumer key (API key) from the Twitter Developer Portal
    pub consumer_key: String,
    /// The consumer secret from the Twitter Developer Portal
    pub consumer_secret: String,
    /// The access token for the authenticated user
    pub access_token: String,
    /// The access token secret for the authenticated user
    pub access_token_secret: String,
}

impl TwitterConfig {
    /// Creates a new `TwitterConfig` instance by loading credentials from environment variables.
    ///
    /// # Required Environment Variables
    ///
    /// - `xapi_consumer_key`: Twitter API consumer key
    /// - `xapi_consumer_secret`: Twitter API consumer secret
    /// - `xapi_access_token`: Twitter API access token
    /// - `xapi_access_token_secret`: Twitter API access token secret
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
    /// std::env::set_var("xapi_consumer_key", "your_key");
    /// std::env::set_var("xapi_consumer_secret", "your_secret");
    /// std::env::set_var("xapi_access_token", "your_token");
    /// std::env::set_var("xapi_access_token_secret", "your_token_secret");
    ///
    /// let config = TwitterConfig::from_env().unwrap();
    /// ```
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TwitterConfig {
            consumer_key: env::var("xapi_consumer_key")?,
            consumer_secret: env::var("xapi_consumer_secret")?,
            access_token: env::var("xapi_access_token")?,
            access_token_secret: env::var("xapi_access_token_secret")?,
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
