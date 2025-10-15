//! OAuth 1.0a authentication module for Twitter/X API integration.
//!
//! This module contains all the functions necessary for implementing OAuth 1.0a
//! authentication as required by the Twitter/X API.

use base64::Engine;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha1::Sha1;
use std::collections::BTreeMap;

use crate::config::TwitterConfig;

/// Generates an OAuth 1.0a signature for Twitter/X API requests.
///
/// This function implements the OAuth 1.0a signature generation algorithm as specified
/// in RFC 5849. It creates an HMAC-SHA1 signature using the provided parameters and secrets.
///
/// # Parameters
///
/// - `method`: HTTP method (e.g., "GET", "POST")
/// - `url`: The full URL of the API endpoint
/// - `params`: BTreeMap containing all OAuth parameters (sorted by key)
/// - `consumer_secret`: The consumer secret from Twitter Developer Portal
/// - `token_secret`: The access token secret for the authenticated user
///
/// # Returns
///
/// A base64-encoded HMAC-SHA1 signature string.
///
/// # Algorithm Steps
///
/// 1. Create a normalized parameter string from the OAuth parameters
/// 2. Create a signature base string from method, URL, and parameters
/// 3. Create a signing key from consumer secret and token secret
/// 4. Generate HMAC-SHA1 signature using the signing key and base string
/// 5. Base64 encode the resulting signature
///
/// # Example
///
/// ```rust
/// use reputest::generate_oauth_signature;
/// use std::collections::BTreeMap;
///
/// let mut params = BTreeMap::new();
/// params.insert("oauth_consumer_key".to_string(), "key".to_string());
/// params.insert("oauth_nonce".to_string(), "nonce".to_string());
///
/// let signature = generate_oauth_signature(
///     "POST",
///     "https://api.x.com/2/tweets",
///     &params,
///     "consumer_secret",
///     "token_secret"
/// );
/// ```
pub fn generate_oauth_signature(
    method: &str,
    url: &str,
    params: &BTreeMap<String, String>,
    consumer_secret: &str,
    token_secret: &str,
) -> String {
    // Create parameter string by joining all OAuth parameters
    // Parameters are already sorted by BTreeMap, so we just need to join them
    let mut param_string = String::new();
    for (i, (key, value)) in params.iter().enumerate() {
        if i > 0 {
            param_string.push('&');
        }
        param_string.push_str(&format!(
            "{}={}",
            percent_encode(key),
            percent_encode(value)
        ));
    }

    // Create signature base string as per OAuth 1.0a specification
    // Format: METHOD&ENCODED_URL&ENCODED_PARAMETERS
    let signature_base = format!(
        "{}&{}&{}",
        method,
        percent_encode(url),
        percent_encode(&param_string)
    );

    // Create signing key by concatenating consumer secret and token secret
    // Format: ENCODED_CONSUMER_SECRET&ENCODED_TOKEN_SECRET
    let signing_key = format!(
        "{}&{}",
        percent_encode(consumer_secret),
        percent_encode(token_secret)
    );

    // Generate HMAC-SHA1 signature using the signing key and base string
    type HmacSha1 = Hmac<Sha1>;
    let mut mac =
        HmacSha1::new_from_slice(signing_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(signature_base.as_bytes());
    let result = mac.finalize();

    // Return base64-encoded signature
    base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
}

/// Percent-encodes a string according to RFC 3986.
///
/// This function implements the percent-encoding scheme used in OAuth 1.0a.
/// Characters that are unreserved (alphanumeric, hyphen, underscore, period, tilde)
/// are left unchanged, while all other characters are encoded as %XX where XX is
/// the hexadecimal representation of the character's byte value.
///
/// # Parameters
///
/// - `s`: The string to encode
///
/// # Returns
///
/// A percent-encoded string.
///
/// # Example
///
/// ```rust
/// use reputest::percent_encode;
///
/// assert_eq!(percent_encode("hello world"), "hello%20world");
/// assert_eq!(percent_encode("test@example.com"), "test%40example.com");
/// assert_eq!(percent_encode("abc123"), "abc123");
/// ```
pub fn percent_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            // Unreserved characters that don't need encoding
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            // All other characters need percent encoding
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// Generates a cryptographically secure random nonce for OAuth requests.
///
/// A nonce is a random string used to prevent replay attacks in OAuth 1.0a.
/// This function generates a 32-character alphanumeric string using the system's
/// secure random number generator.
///
/// # Returns
///
/// A 32-character random alphanumeric string.
///
/// # Example
///
/// ```rust
/// use reputest::generate_nonce;
///
/// let nonce1 = generate_nonce();
/// let nonce2 = generate_nonce();
/// assert_ne!(nonce1, nonce2); // Nonces should be different
/// assert_eq!(nonce1.len(), 32); // Should be 32 characters
/// ```
pub fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    let nonce: String = (0..32)
        .map(|_| {
            // Use alphanumeric characters for the nonce
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();
    nonce
}

/// Gets the current Unix timestamp as a string.
///
/// This function returns the current time as the number of seconds since the Unix epoch
/// (January 1, 1970 00:00:00 UTC), formatted as a string. This is used for the
/// `oauth_timestamp` parameter in OAuth 1.0a requests.
///
/// # Returns
///
/// - `Ok(String)`: The current Unix timestamp as a string
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If there's an error getting the system time
///
/// # Example
///
/// ```rust
/// use reputest::get_current_timestamp;
///
/// let timestamp = get_current_timestamp().unwrap();
/// let ts: u64 = timestamp.parse().unwrap();
/// assert!(ts > 1600000000); // Should be a reasonable timestamp
/// ```
pub fn get_current_timestamp() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string())
}

/// Builds the OAuth 1.0a parameters required for Twitter/X API authentication.
///
/// This function creates a BTreeMap containing all the required OAuth parameters
/// for authenticating with the Twitter/X API. The parameters are automatically
/// sorted by key (due to BTreeMap) which is required for OAuth signature generation.
///
/// # Parameters
///
/// - `config`: A `TwitterConfig` instance containing the API credentials
///
/// # Returns
///
/// - `Ok(BTreeMap<String, String>)`: A map containing all OAuth parameters
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If timestamp generation fails
///
/// # OAuth Parameters Included
///
/// - `oauth_consumer_key`: The consumer key from the config
/// - `oauth_nonce`: A randomly generated nonce
/// - `oauth_signature_method`: Set to "HMAC-SHA1"
/// - `oauth_timestamp`: Current Unix timestamp
/// - `oauth_token`: The access token from the config
/// - `oauth_version`: Set to "1.0"
///
/// # Example
///
/// ```rust
/// use reputest::{TwitterConfig, build_oauth_params};
///
/// // Set environment variables for the test
/// std::env::set_var("xapi_consumer_key", "test_key");
/// std::env::set_var("xapi_consumer_secret", "test_secret");
/// std::env::set_var("xapi_access_token", "test_token");
/// std::env::set_var("xapi_access_token_secret", "test_token_secret");
///
/// let config = TwitterConfig::from_env().unwrap();
/// let oauth_params = build_oauth_params(&config).unwrap();
/// assert!(oauth_params.contains_key("oauth_consumer_key"));
/// assert!(oauth_params.contains_key("oauth_nonce"));
/// ```
pub fn build_oauth_params(
    config: &TwitterConfig,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut oauth_params = BTreeMap::new();

    // Add all required OAuth 1.0a parameters
    oauth_params.insert(
        "oauth_consumer_key".to_string(),
        config.consumer_key.clone(),
    );
    oauth_params.insert("oauth_nonce".to_string(), generate_nonce());
    oauth_params.insert(
        "oauth_signature_method".to_string(),
        "HMAC-SHA1".to_string(),
    );
    oauth_params.insert("oauth_timestamp".to_string(), get_current_timestamp()?);
    oauth_params.insert("oauth_token".to_string(), config.access_token.clone());
    oauth_params.insert("oauth_version".to_string(), "1.0".to_string());

    Ok(oauth_params)
}

/// Builds the Authorization header string for OAuth 1.0a requests.
///
/// This function takes the OAuth parameters and formats them into the proper
/// Authorization header format required by the Twitter/X API. All parameter
/// values are percent-encoded before being included in the header.
///
/// # Parameters
///
/// - `oauth_params`: A BTreeMap containing the OAuth parameters
///
/// # Returns
///
/// A properly formatted Authorization header string.
///
/// # Format
///
/// The header follows this format:
/// ```text
/// OAuth oauth_consumer_key="value", oauth_nonce="value", ...
/// ```
///
/// # Example
///
/// ```rust
/// use reputest::build_auth_header;
/// use std::collections::BTreeMap;
///
/// let mut params = BTreeMap::new();
/// params.insert("oauth_consumer_key".to_string(), "key".to_string());
/// params.insert("oauth_nonce".to_string(), "nonce".to_string());
///
/// let header = build_auth_header(&params);
/// assert!(header.starts_with("OAuth "));
/// assert!(header.contains("oauth_consumer_key=\"key\""));
/// ```
pub fn build_auth_header(oauth_params: &BTreeMap<String, String>) -> String {
    let mut auth_header = String::from("OAuth ");

    // Join all OAuth parameters with commas
    for (i, (key, value)) in oauth_params.iter().enumerate() {
        if i > 0 {
            auth_header.push_str(", ");
        }
        // Percent-encode the value and wrap in quotes
        auth_header.push_str(&format!("{}=\"{}\"", key, percent_encode(value)));
    }

    auth_header
}
