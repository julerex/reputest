//! OAuth 2.0 Bearer Token authentication module for Twitter/X API integration.
//!
//! This module contains functions for implementing OAuth 2.0 Bearer Token
//! authentication as required by the Twitter/X API v2 endpoints.

/// Builds the Authorization header for OAuth 2.0 Bearer Token authentication.
///
/// This function creates the proper Authorization header for OAuth 2.0 Bearer Token
/// authentication, which is required for Twitter API v2 endpoints like search/recent
/// and posting tweets.
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
