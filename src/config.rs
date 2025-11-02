//! Configuration module for the reputest service.
//!
//! This module contains configuration structures and environment variable handling
//! for the Twitter/X API integration.

use crate::db;
use log::{debug, error, info, warn};
use sqlx::PgPool;
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
    /// Saves a refresh token to the database.
    ///
    /// # Parameters
    ///
    /// - `pool`: A reference to the PostgreSQL connection pool
    /// - `token`: The refresh token to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the token was successfully saved
    /// - `Err(...)`: If saving failed
    async fn save_refresh_token_to_db(
        pool: &PgPool,
        token: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Attempting to save refresh token to database");

        // Ensure the table exists
        if let Err(e) = db::create_refresh_tokens_table(pool).await {
            warn!("Failed to ensure refresh_tokens table exists: {}", e);
        }

        match db::save_refresh_token(pool, token).await {
            Ok(_) => {
                info!("Successfully saved refresh token to database");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to save refresh token to database: {}", e);
                Err(e)
            }
        }
    }

    /// Creates a new `TwitterConfig` instance by loading credentials from the database and environment variables.
    ///
    /// # Required Parameters
    ///
    /// - `pool`: A reference to the PostgreSQL connection pool to fetch tokens from
    ///
    /// # Required Database Tables
    ///
    /// - `access_tokens`: Must contain at least one access token
    ///
    /// # Optional Environment Variables (for automatic token refresh)
    ///
    /// - `xapi_client_id`: Client ID for OAuth 2.0 operations
    /// - `xapi_client_secret`: Client Secret for OAuth 2.0 operations
    ///
    /// # Token Loading
    ///
    /// - Access token: Loaded from the `access_tokens` table (required)
    /// - Refresh token: Loaded from the `refresh_tokens` table (optional)
    /// - Client credentials: Loaded from environment variables (optional)
    ///
    /// # Returns
    ///
    /// - `Ok(TwitterConfig)`: If the required access token is found in the database
    /// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the access token is missing from the database
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use reputest::{TwitterConfig, db};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let pool = db::get_db_pool().await.unwrap();
    ///     // Optionally set environment variables
    ///     std::env::set_var("xapi_client_id", "your_client_id");
    ///     std::env::set_var("xapi_client_secret", "your_client_secret");
    ///
    ///     let config = TwitterConfig::from_env(&pool).await.unwrap();
    /// }
    /// ```
    pub async fn from_env(pool: &PgPool) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Loading Twitter configuration from database");

        // Load required access token from database
        let access_token = match db::get_latest_access_token(pool).await {
            Ok(Some(token)) => {
                let token_length = token.len();
                info!(
                    "Found access token in database with length: {}",
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
            Ok(None) => {
                error!("No access token found in database");
                return Err("No access token found in database".into());
            }
            Err(e) => {
                error!("Failed to load access token from database: {}", e);
                return Err(format!("Failed to load access token from database: {}", e).into());
            }
        };

        // Load optional refresh token from database
        let refresh_token = match db::get_latest_refresh_token(pool).await {
            Ok(Some(token)) => {
                info!("Successfully loaded refresh token from database");
                Some(token)
            }
            Ok(None) => {
                info!("No refresh token found in database");
                None
            }
            Err(e) => {
                warn!("Failed to load refresh token from database: {}", e);
                None
            }
        };

        // Load optional client credentials
        let client_id = match env::var("xapi_client_id") {
            Ok(id) => {
                info!("Found xapi_client_id environment variable");
                debug!(
                    "Client ID (masked): {}...",
                    &id[..std::cmp::min(id.len(), 8)]
                );
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
                debug!(
                    "Client secret (masked): {}...",
                    &secret[..std::cmp::min(secret.len(), 8)]
                );
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
        if config.refresh_token.is_some()
            && config.client_id.is_some()
            && config.client_secret.is_some()
        {
            info!("Automatic token refresh is enabled");
        } else {
            info!("Automatic token refresh is disabled - manual token refresh required");
        }

        Ok(config)
    }

    /// Attempts to refresh the access token using the stored refresh token and client credentials.
    ///
    /// This method automatically refreshes an expired access token if all required credentials
    /// are available. It updates the access token in the config and saves both tokens to the database.
    ///
    /// # Parameters
    ///
    /// - `pool`: A reference to the PostgreSQL connection pool to save tokens to
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the token was successfully refreshed
    /// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If refresh failed or credentials are missing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use reputest::{TwitterConfig, db};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let pool = db::get_db_pool().await.unwrap();
    ///     let mut config = TwitterConfig::from_env(&pool).await.unwrap();
    ///     match config.refresh_access_token(&pool).await {
    ///         Ok(_) => println!("Token refreshed successfully"),
    ///         Err(e) => eprintln!("Token refresh failed: {}", e),
    ///     }
    /// }
    /// ```
    pub async fn refresh_access_token(
        &mut self,
        pool: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                    error!("Missing refresh token (should be loaded from database)");
                }
                return Err("Missing required credentials for token refresh".into());
            }
        };

        info!("All required credentials available for token refresh");

        // Import the refresh function from oauth module
        use crate::oauth::refresh_access_token;

        // Attempt to refresh the token
        match refresh_access_token(client_id, client_secret, refresh_token).await {
            Ok((new_access_token, new_refresh_token)) => {
                info!("Access token refreshed successfully");

                // Update the access token in the config
                let old_token_length = self.access_token.len();
                self.access_token = new_access_token.clone();
                let new_token_length = self.access_token.len();

                // Save new access token to database
                if let Err(e) = db::save_access_token(pool, &new_access_token).await {
                    warn!("Failed to save access token to database: {}", e);
                    warn!("Access token updated in memory only");
                } else {
                    info!("Access token successfully saved to database");
                }

                // Update refresh token if a new one was provided
                if let Some(new_refresh) = new_refresh_token {
                    info!("Updating refresh token with new token from Twitter");
                    self.refresh_token = Some(new_refresh.clone());

                    // Try to save to database
                    if let Err(e) = Self::save_refresh_token_to_db(pool, &new_refresh).await {
                        warn!("Failed to save refresh token to database: {}", e);
                        warn!("Refresh token updated in memory only");
                    } else {
                        info!("Refresh token successfully saved to database");
                    }
                }

                info!(
                    "Access token updated: old length {}, new length {}",
                    old_token_length, new_token_length
                );

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
                } else {
                    format!("{}...", token_prefix)
                };

                debug!("Updated access token (masked): {}", masked_token);

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
