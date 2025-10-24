//! OAuth authentication module for Twitter/X API integration.
//!
//! This module contains functions for implementing both OAuth 2.0 Bearer Token
//! authentication (for read-only operations) and OAuth 2.0 User Context
//! authentication (for user-specific operations like posting tweets).

/// Builds the Authorization header for OAuth 2.0 Bearer Token authentication.
///
/// This function creates the proper Authorization header for OAuth 2.0 Bearer Token
/// authentication, which is required for Twitter API v2 endpoints like search/recent
/// (read-only operations).
///
/// # Parameters
///
/// - `bearer_token`: The Bearer Token from the Twitter Developer Portal
///
/// # Returns
///
/// A properly formatted Authorization header string for Bearer Token authentication.
///
/// # Format
///
/// The header follows this format:
/// ```text
/// Bearer YOUR_BEARER_TOKEN_HERE
/// ```
///
/// # Example
///
/// ```rust
/// use reputest::build_bearer_auth_header;
///
/// let header = build_bearer_auth_header("your_bearer_token");
/// assert_eq!(header, "Bearer your_bearer_token");
/// ```
pub fn build_bearer_auth_header(bearer_token: &str) -> String {
    format!("Bearer {}", bearer_token)
}

/// Builds the Authorization header for OAuth 2.0 User Context authentication.
///
/// This function creates the proper Authorization header for OAuth 2.0 User Context
/// authentication, which is required for Twitter API v2 endpoints that perform
/// user-specific operations like posting tweets.
///
/// # Parameters
///
/// - `access_token`: The Access Token obtained through OAuth 2.0 Authorization Code Flow
///
/// # Returns
///
/// A properly formatted Authorization header string for OAuth 2.0 User Context authentication.
///
/// # Format
///
/// The header follows this format:
/// ```text
/// Bearer YOUR_ACCESS_TOKEN_HERE
/// ```
///
/// # Example
///
/// ```rust
/// use reputest::build_oauth2_user_context_header;
///
/// let header = build_oauth2_user_context_header("your_access_token");
/// assert_eq!(header, "Bearer your_access_token");
/// ```
pub fn build_oauth2_user_context_header(access_token: &str) -> String {
    format!("Bearer {}", access_token)
}
