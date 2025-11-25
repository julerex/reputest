//! Text parsing and extraction utilities for Twitter content.
//!
//! This module contains functions for parsing tweet text to extract mentions,
//! hashtags, and other structured content.

/// Extracts the first @mention from tweet text where #gmgv directly follows the mention (with optional spaces),
/// excluding the specified username if provided.
/// This ensures vibes are only recorded when #gmgv immediately follows a username mention.
/// Examples: "@alice #gmgv" ✓, "@alice has #gmgv" ✗
///
/// # Parameters
///
/// - `text`: The tweet text to search for mentions
/// - `exclude_username`: Optional username to exclude from matching
///
/// # Returns
///
/// - `Some(username)`: The first mentioned username if found
/// - `None`: If no mentions are found
pub(crate) fn extract_vibe_mention(text: &str, exclude_username: Option<&str>) -> Option<String> {
    // Use regex to find @mentions followed directly by #gmgv (with optional spaces)
    // Pattern: @username\s*#gmgv
    let re = regex::Regex::new(r"@(\w+)\s*#gmgv").ok()?;

    // Find captures and check each one
    for cap in re.captures_iter(text) {
        if let Some(username_match) = cap.get(1) {
            let username = username_match.as_str();
            if Some(username) != exclude_username {
                return Some(username.to_string());
            }
        }
    }

    None
}

/// Extracts a username from a tweet that specifically queries the bot in the format "@reputest username ?" or "@reputest @username ?".
///
/// This function only matches the exact patterns where a tweet starts with "@reputest"
/// followed by a username (with or without @) and ends with a question mark.
/// This is much more restrictive than the previous implementation to avoid false positives.
/// Common words and the bot's username are excluded to prevent false matches.
///
/// # Parameters
///
/// - `text`: The tweet text to analyze
///
/// # Returns
///
/// - `Some(username)`: The username if found in the specific query format
/// - `None`: If the tweet doesn't match the required format
pub fn extract_mention_with_question(text: &str) -> Option<String> {
    // Use regex to match only the specific patterns: "@reputest username ?" or "@reputest @username ?"
    // The pattern ensures the tweet starts with "@reputest" followed by whitespace, then username, optional whitespace, then "?"
    let re = regex::Regex::new(r"^@reputest\s+(@?[a-zA-Z0-9_]{1,15})\s*\?$").ok()?;

    if let Some(captures) = re.captures(text) {
        if let Some(username_match) = captures.get(1) {
            let username = username_match.as_str();
            // Remove @ prefix if present
            let clean_username = username.strip_prefix('@').unwrap_or(username);

            // Exclude common words that might be followed by ? to avoid false positives
            let excluded_words = [
                "what", "when", "where", "how", "why", "who", "which", "the", "a", "an", "is",
                "are", "was", "were", "be", "been", "being", "have", "has", "had", "do", "does",
                "did", "will", "would", "could", "should", "can", "may", "might", "must", "shall",
                "reputest",
            ];
            if !excluded_words.contains(&clean_username.to_lowercase().as_str()) {
                return Some(clean_username.to_string());
            }
        }
    }

    None
}
