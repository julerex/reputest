//! OAuth authentication module for Twitter/X API integration.
//!
//! This module contains functions for implementing OAuth 2.0 User Context
//! authentication for all Twitter API v2 operations including posting tweets
//! and searching tweets.

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
/// use reputest::oauth::build_oauth2_user_context_header;
///
/// let header = build_oauth2_user_context_header("your_access_token");
/// assert_eq!(header, "Bearer your_access_token");
/// ```
pub fn build_oauth2_user_context_header(access_token: &str) -> String {
    format!("Bearer {}", access_token)
}
