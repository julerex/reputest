//! Text parsing and extraction utilities for Twitter content.
//!
//! This module contains functions for parsing tweet text to extract mentions,
//! hashtags, and other structured content.

/// Maximum allowed length for text input to prevent ReDoS attacks.
/// Twitter's max tweet length is 280 characters, so 500 is generous.
const MAX_INPUT_LENGTH: usize = 500;

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
/// - `None`: If no valid pattern is found or input exceeds maximum length
pub(crate) fn extract_vibe_emitter(text: &str, exclude_username: Option<&str>) -> Option<String> {
    // SECURITY: Limit input length to prevent ReDoS attacks
    if text.len() > MAX_INPUT_LENGTH {
        log::warn!(
            "Input text exceeds maximum length ({} > {}), rejecting",
            text.len(),
            MAX_INPUT_LENGTH
        );
        return None;
    }
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
/// This function matches patterns where "@reputest" appears in the tweet (allowing for replies where
/// other mentions may come first), followed by whitespace, then a username (with or without @),
/// optional whitespace, then "?". Additional content after the "?" is allowed. This is more restrictive
/// than the previous implementation to avoid false positives. Common words and the bot's username are
/// excluded to prevent false matches.
///
/// # Parameters
///
/// - `text`: The tweet text to analyze
///
/// # Returns
///
/// - `Some(username)`: The username if found in the specific query format
/// - `None`: If the tweet doesn't match the required format or input exceeds maximum length
pub fn extract_mention_with_question(text: &str) -> Option<String> {
    // SECURITY: Limit input length to prevent ReDoS attacks
    if text.len() > MAX_INPUT_LENGTH {
        log::warn!(
            "Input text exceeds maximum length ({} > {}), rejecting",
            text.len(),
            MAX_INPUT_LENGTH
        );
        return None;
    }

    // Use regex to match only the specific patterns: "@reputest username ?" or "@reputest @username ?"
    // The pattern allows "@reputest" to appear anywhere in the tweet (for replies), followed by whitespace,
    // then username, optional whitespace, then "?". Note: We allow content after the "?" to handle cases
    // where users add extra text. The pattern requires @reputest to be preceded by start of string or whitespace
    // to avoid matching it as part of another word.
    let re = regex::Regex::new(r"(?:^|\s)@reputest\s+(@?[a-zA-Z0-9_]{1,15})\s*\?").ok()?;

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

/// Extracts a username from a tweet that queries the bot for following list: "@reputest @username following?".
///
/// Matches patterns where "@reputest" appears, followed by whitespace, a username (with or without @),
/// whitespace, then "following?". Precedence: use this before extract_mention_with_question when classifying.
///
/// # Parameters
///
/// - `text`: The tweet text to analyze
///
/// # Returns
///
/// - `Some(username)`: The username if found in the following-query format
/// - `None`: If the tweet doesn't match or input exceeds maximum length
pub fn extract_mention_with_following(text: &str) -> Option<String> {
    if text.len() > MAX_INPUT_LENGTH {
        log::warn!(
            "Input text exceeds maximum length ({} > {}), rejecting",
            text.len(),
            MAX_INPUT_LENGTH
        );
        return None;
    }

    let re = regex::Regex::new(r"(?:^|\s)@reputest\s+(@?[a-zA-Z0-9_]{1,15})\s+following\?").ok()?;

    if let Some(captures) = re.captures(text) {
        if let Some(username_match) = captures.get(1) {
            let username = username_match.as_str();
            let clean_username = username.strip_prefix('@').unwrap_or(username);

            let excluded_words = [
                "what",
                "when",
                "where",
                "how",
                "why",
                "who",
                "which",
                "the",
                "a",
                "an",
                "is",
                "are",
                "was",
                "were",
                "be",
                "been",
                "being",
                "have",
                "has",
                "had",
                "do",
                "does",
                "did",
                "will",
                "would",
                "could",
                "should",
                "can",
                "may",
                "might",
                "must",
                "shall",
                "reputest",
                "following",
            ];
            if !excluded_words.contains(&clean_username.to_lowercase().as_str()) {
                return Some(clean_username.to_string());
            }
        }
    }

    None
}

/// Extracts megajoule transfer information from tweet text.
/// Format: "Send INTEGER #megajoules to @username"
/// Examples: "Send 100 #megajoules to @alice" ✓, "send 50 #megajoules to bob" ✓
///
/// # Parameters
///
/// - `text`: The tweet text to search for megajoule transfer
///
/// # Returns
///
/// - `Some((amount, receiver_username))`: The amount and receiver username if found
/// - `None`: If no valid pattern is found or input exceeds maximum length
pub(crate) fn extract_megajoule_transfer(text: &str) -> Option<(i32, String)> {
    // SECURITY: Limit input length to prevent ReDoS attacks
    if text.len() > MAX_INPUT_LENGTH {
        log::warn!(
            "Input text exceeds maximum length ({} > {}), rejecting",
            text.len(),
            MAX_INPUT_LENGTH
        );
        return None;
    }

    // Case-insensitive regex: "send" + INTEGER + "#megajoules" + "to" + @username
    let re = regex::Regex::new(r"(?i)send\s+(\d+)\s+#megajoules\s+to\s+@?(\w{1,15})").ok()?;

    if let Some(captures) = re.captures(text) {
        if let (Some(amount_match), Some(username_match)) = (captures.get(1), captures.get(2)) {
            if let Ok(amount) = amount_match.as_str().parse::<i32>() {
                let username = username_match.as_str().to_string();
                return Some((amount, username));
            }
        }
    }

    None
}
