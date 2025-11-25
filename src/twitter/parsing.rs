//! Text parsing and extraction utilities for Twitter content.
//!
//! This module contains functions for parsing tweet text to extract mentions,
//! hashtags, and other structured content.

/// Extracts the vibe emitter username from tweet text where #gmgv directly follows.
/// The word before #gmgv can optionally start with @ - it will be stripped if present.
/// Examples: "@alice #gmgv" ✓, "alice #gmgv" ✓, "alice has #gmgv" ✗
///
/// # Parameters
///
/// - `text`: The tweet text to search for the vibe emitter
/// - `exclude_username`: Optional username to exclude from matching (e.g., reply target)
///
/// # Returns
///
/// - `Some(username)`: The username before #gmgv (without @ prefix)
/// - `None`: If no valid pattern is found
pub(crate) fn extract_vibe_emitter(text: &str, exclude_username: Option<&str>) -> Option<String> {
    // Common English words that shouldn't be treated as usernames
    let excluded_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "can", "may", "might", "must",
        "shall", "with", "for", "from", "this", "that", "these", "those", "it", "its", "my",
        "your", "his", "her", "their", "our", "all", "any", "some", "no", "not", "but", "and",
        "or", "if", "when", "where", "what", "who", "how", "why", "which", "to", "of", "in", "on",
        "at", "by", "up", "so", "as", "good", "vibes", "great", "awesome", "amazing", "love",
        "like", "really", "very", "much", "more", "just", "also", "too", "here", "there", "now",
        "then", "out", "about",
    ];

    // Match optional @ followed by username, then optional whitespace, then #gmgv
    // The pattern requires either start of string, whitespace, or @ before the username
    let re = regex::Regex::new(r"(?:^|[\s@])@?(\w{1,15})\s*#gmgv").ok()?;

    // Find captures and check each one
    for cap in re.captures_iter(text) {
        if let Some(username_match) = cap.get(1) {
            let username = username_match.as_str();
            // Skip excluded common words (case-insensitive)
            if excluded_words.contains(&username.to_lowercase().as_str()) {
                continue;
            }
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
