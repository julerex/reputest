//! Configuration module for the reputest service.
//!
//! This module contains configuration structures and environment variable handling
//! for the Twitter/X API integration.

use log::{debug, error, info, warn};
use std::env;

/// Configuration struct for Twitter/X API credentials.
///
/// This struct holds the credentials required to authenticate with the Twitter/X API v2 endpoints.
/// It uses OAuth 2.0 User Context (Access Token) for all operations including posting tweets
/// and searching tweets. It also includes refresh token for automatic token renewal.
#[derive(Debug)]
pub struct TwitterConfig {
    /// The Access Token for OAuth 2.0 User Context authentication (all operations)
    pub access_token: String,
    /// The Refresh Token for automatically refreshing expired access tokens
    pub refresh_token: Option<String>,
    /// The Client ID for OAuth 2.0 operations
    pub client_id: Option<String>,
    /// The Client Secret for OAuth 2.0 operations
    pub client_secret: Option<String>,
}

impl TwitterConfig {
    /// Creates a new `TwitterConfig` instance by loading credentials from environment variables.
    ///
    /// # Required Environment Variables
    ///
    /// - `xapi_access_token`: Twitter API Access Token (OAuth 2.0 User Context for all operations)
    ///
    /// # Optional Environment Variables (for automatic token refresh)
    ///
    /// - `xapi_refresh_token`: Refresh Token for automatically refreshing expired access tokens
    /// - `xapi_client_id`: Client ID for OAuth 2.0 operations
    /// - `xapi_client_secret`: Client Secret for OAuth 2.0 operations
    ///
    /// # Returns
    ///
    /// - `Ok(TwitterConfig)`: If the required environment variable is present
    /// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the environment variable is missing
    ///
    /// # Example
    ///
    /// ```rust
    /// use reputest::TwitterConfig;
    ///
    /// // Set environment variables before calling
    /// std::env::set_var("xapi_access_token", "your_access_token");
    /// std::env::set_var("xapi_refresh_token", "your_refresh_token");
    /// std::env::set_var("xapi_client_id", "your_client_id");
    /// std::env::set_var("xapi_client_secret", "your_client_secret");
    ///
    /// let config = TwitterConfig::from_env().unwrap();
    /// ```
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Loading Twitter configuration from environment variables");

        // Load required access token
        let access_token = match env::var("xapi_access_token") {
            Ok(token) => {
                let token_length = token.len();
                info!(
                    "Found xapi_access_token environment variable with length: {}",
                    token_length
                );

                // Log token info (masked for security)
                let token_prefix = if token_length > 8 {
                    &token[..8]
                } else {
                    &token
                };
                let token_suffix = if token_length > 16 {
                    &token[token_length - 8..]
                } else if token_length > 8 {
                    &token[8..]
                } else {
                    ""
                };

                let masked_token = if token_length > 16 {
                    format!("{}...{}", token_prefix, token_suffix)
                } else if token_length > 8 {
                    format!("{}...", token_prefix)
                } else {
                    format!("{}...", token_prefix)
                };

                debug!("Access token (masked): {}", masked_token);

                // Validate token format (basic checks)
                if token.is_empty() {
                    error!("Access token is empty");
                    return Err("Access token cannot be empty".into());
                }

                if token_length < 10 {
                    warn!(
                        "Access token seems unusually short ({} characters)",
                        token_length
                    );
                }

                token
            }
            Err(e) => {
                error!("Failed to load xapi_access_token from environment: {}", e);
                error!("Make sure xapi_access_token environment variable is set");
                return Err(
                    format!("Missing xapi_access_token environment variable: {}", e).into(),
                );
            }
        };

        // Load optional refresh token
        let refresh_token = match env::var("xapi_refresh_token") {
            Ok(token) => {
                let token_length = token.len();
                info!(
                    "Found xapi_refresh_token environment variable with length: {}",
                    token_length
                );

                // Log token info (masked for security)
                let token_prefix = if token_length > 8 {
                    &token[..8]
                } else {
                    &token
                };
                let token_suffix = if token_length > 16 {
                    &token[token_length - 8..]
                } else if token_length > 8 {
                    &token[8..]
                } else {
                    ""
                };

                let masked_token = if token_length > 16 {
                    format!("{}...{}", token_prefix, token_suffix)
                } else if token_length > 8 {
                    format!("{}...", token_prefix)
                } else {
                    format!("{}...", token_prefix)
                };

                debug!("Refresh token (masked): {}", masked_token);

                if token.is_empty() {
                    warn!("Refresh token is empty, automatic token refresh will be disabled");
                    None
                } else {
                    Some(token)
                }
            }
            Err(_) => {
                info!("No xapi_refresh_token found in environment variables - automatic token refresh will be disabled");
                None
            }
        };

        // Load optional client credentials
        let client_id = match env::var("xapi_client_id") {
            Ok(id) => {
                info!("Found xapi_client_id environment variable");
                debug!("Client ID (masked): {}...", &id[..std::cmp::min(id.len(), 8)]);
                Some(id)
            }
            Err(_) => {
                info!("No xapi_client_id found in environment variables");
                None
            }
        };

        let client_secret = match env::var("xapi_client_secret") {
            Ok(secret) => {
                info!("Found xapi_client_secret environment variable");
                debug!("Client secret (masked): {}...", &secret[..std::cmp::min(secret.len(), 8)]);
                Some(secret)
            }
            Err(_) => {
                info!("No xapi_client_secret found in environment variables");
                None
            }
        };

        // Check if we have all required credentials for automatic refresh
        if refresh_token.is_some() && (client_id.is_none() || client_secret.is_none()) {
            warn!("Refresh token is provided but client credentials are missing - automatic token refresh will be disabled");
        }

        let config = TwitterConfig {
            access_token,
            refresh_token,
            client_id,
            client_secret,
        };

        info!("Twitter configuration loaded successfully");
        if config.refresh_token.is_some() && config.client_id.is_some() && config.client_secret.is_some() {
            info!("Automatic token refresh is enabled");
        } else {
            info!("Automatic token refresh is disabled - manual token refresh required");
        }

        Ok(config)
    }

    /// Attempts to refresh the access token using the stored refresh token and client credentials.
    ///
    /// This method automatically refreshes an expired access token if all required credentials
    /// are available. It updates the access token in the config and logs the process.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the token was successfully refreshed
    /// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If refresh failed or credentials are missing
    ///
    /// # Example
    ///
    /// ```rust
    /// use reputest::TwitterConfig;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut config = TwitterConfig::from_env().unwrap();
    ///     match config.refresh_access_token().await {
    ///         Ok(_) => println!("Token refreshed successfully"),
    ///         Err(e) => eprintln!("Token refresh failed: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn refresh_access_token(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Attempting to refresh access token");
        
        // Check if we have all required credentials for refresh
        let (client_id, client_secret, refresh_token) = match (
            self.client_id.as_ref(),
            self.client_secret.as_ref(),
            self.refresh_token.as_ref(),
        ) {
            (Some(id), Some(secret), Some(token)) => (id, secret, token),
            _ => {
                error!("Cannot refresh token: missing required credentials");
                if self.client_id.is_none() {
                    error!("Missing xapi_client_id");
                }
                if self.client_secret.is_none() {
                    error!("Missing xapi_client_secret");
                }
                if self.refresh_token.is_none() {
                    error!("Missing xapi_refresh_token");
                }
                return Err("Missing required credentials for token refresh".into());
            }
        };

        info!("All required credentials available for token refresh");
        
        // Import the refresh function from oauth module
        use crate::oauth::refresh_access_token;
        
        // Attempt to refresh the token
        match refresh_access_token(client_id, client_secret, refresh_token).await {
            Ok(new_access_token) => {
                info!("Access token refreshed successfully");
                
                // Update the access token in the config
                let old_token_length = self.access_token.len();
                self.access_token = new_access_token;
                let new_token_length = self.access_token.len();
                
                info!("Access token updated: old length {}, new length {}", old_token_length, new_token_length);
                
                // Log the updated token info (masked)
                let token_prefix = if new_token_length > 8 {
                    &self.access_token[..8]
                } else {
                    &self.access_token
                };
                let token_suffix = if new_token_length > 16 {
                    &self.access_token[new_token_length - 8..]
                } else if new_token_length > 8 {
                    &self.access_token[8..]
                } else {
                    ""
                };
                
                let masked_token = if new_token_length > 16 {
                    format!("{}...{}", token_prefix, token_suffix)
                } else if new_token_length > 8 {
                    format!("{}...", token_prefix)
                } else {
                    format!("{}...", token_prefix)
                };
                
                debug!("Updated access token (masked): {}", masked_token);
                warn!("Access token has been refreshed - consider updating your xapi_access_token environment variable");
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to refresh access token: {}", e);
                Err(e)
            }
        }
    }

    /// Checks if automatic token refresh is available.
    ///
    /// Returns true if all required credentials (client_id, client_secret, refresh_token)
    /// are available for automatic token refresh.
    ///
    /// # Returns
    ///
    /// `true` if automatic refresh is available, `false` otherwise.
    pub fn can_refresh_token(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some() && self.refresh_token.is_some()
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
