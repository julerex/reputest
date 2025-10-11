//! # Reputest
//!
//! A Rust web service that provides HTTP endpoints for testing and demonstration purposes.
//! The service includes functionality to post tweets via the Twitter/X API using OAuth 1.0a authentication.
//!
//! ## Features
//!
//! - HTTP server with multiple endpoints (`/`, `/reputest`, `/health`, `/tweet`)
//! - Twitter/X API integration with OAuth 1.0a authentication
//! - Comprehensive test suite
//! - Structured logging
//! - Health check endpoint
//!
//! ## Environment Variables
//!
//! The following environment variables are required for Twitter API functionality:
//! - `xapi_consumer_key`: Twitter API consumer key
//! - `xapi_consumer_secret`: Twitter API consumer secret  
//! - `xapi_access_token`: Twitter API access token
//! - `xapi_access_token_secret`: Twitter API access token secret
//! - `PORT`: Server port (defaults to 3000)
//!
//! ## API Endpoints
//!
//! - `GET /`: Returns a welcome message
//! - `GET /reputest`: Returns "Reputesting!" message
//! - `POST /reputest`: Returns "Reputesting!" message
//! - `GET /health`: Returns service health status
//! - `POST /tweet`: Posts a tweet to Twitter/X (requires API credentials)

use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use base64::Engine;
use hmac::{Hmac, Mac};
use log::{error, info};
use rand::Rng;
use reqwest::Client;
use serde_json::{json, Value};
use sha1::Sha1;
use std::collections::BTreeMap;
use std::env;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

/// Configuration struct for Twitter/X API credentials.
///
/// This struct holds all the necessary OAuth 1.0a credentials required to authenticate
/// with the Twitter/X API. All fields are loaded from environment variables.
#[derive(Debug)]
struct TwitterConfig {
    /// The consumer key (API key) from the Twitter Developer Portal
    consumer_key: String,
    /// The consumer secret from the Twitter Developer Portal
    consumer_secret: String,
    /// The access token for the authenticated user
    access_token: String,
    /// The access token secret for the authenticated user
    access_token_secret: String,
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
    /// // Set environment variables before calling
    /// std::env::set_var("xapi_consumer_key", "your_key");
    /// std::env::set_var("xapi_consumer_secret", "your_secret");
    /// std::env::set_var("xapi_access_token", "your_token");
    /// std::env::set_var("xapi_access_token_secret", "your_token_secret");
    ///
    /// let config = TwitterConfig::from_env()?;
    /// ```
    fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TwitterConfig {
            consumer_key: env::var("xapi_consumer_key")?,
            consumer_secret: env::var("xapi_consumer_secret")?,
            access_token: env::var("xapi_access_token")?,
            access_token_secret: env::var("xapi_access_token_secret")?,
        })
    }
}

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
fn generate_oauth_signature(
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
/// assert_eq!(percent_encode("hello world"), "hello%20world");
/// assert_eq!(percent_encode("test@example.com"), "test%40example.com");
/// assert_eq!(percent_encode("abc123"), "abc123");
/// ```
fn percent_encode(s: &str) -> String {
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
/// let nonce1 = generate_nonce();
/// let nonce2 = generate_nonce();
/// assert_ne!(nonce1, nonce2); // Nonces should be different
/// assert_eq!(nonce1.len(), 32); // Should be 32 characters
/// ```
fn generate_nonce() -> String {
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
/// let timestamp = get_current_timestamp()?;
/// let ts: u64 = timestamp.parse().unwrap();
/// assert!(ts > 1600000000); // Should be a reasonable timestamp
/// ```
fn get_current_timestamp() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
/// let config = TwitterConfig::from_env()?;
/// let oauth_params = build_oauth_params(&config)?;
/// assert!(oauth_params.contains_key("oauth_consumer_key"));
/// assert!(oauth_params.contains_key("oauth_nonce"));
/// ```
fn build_oauth_params(
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
/// ```
/// OAuth oauth_consumer_key="value", oauth_nonce="value", ...
/// ```
///
/// # Example
///
/// ```rust
/// let mut params = BTreeMap::new();
/// params.insert("oauth_consumer_key".to_string(), "key".to_string());
/// params.insert("oauth_nonce".to_string(), "nonce".to_string());
///
/// let header = build_auth_header(&params);
/// assert!(header.starts_with("OAuth "));
/// assert!(header.contains("oauth_consumer_key=\"key\""));
/// ```
fn build_auth_header(oauth_params: &BTreeMap<String, String>) -> String {
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

/// Posts a tweet to Twitter/X using the API v2 endpoint.
///
/// This function handles the complete OAuth 1.0a authentication flow and posts
/// a tweet to the Twitter/X API. It generates the necessary OAuth signature,
/// builds the authorization header, and sends the request.
///
/// # Parameters
///
/// - `text`: The text content of the tweet to post
///
/// # Returns
///
/// - `Ok(String)`: The API response body on successful tweet posting
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
///
/// # Requirements
///
/// The following environment variables must be set:
/// - `xapi_consumer_key`
/// - `xapi_consumer_secret`
/// - `xapi_access_token`
/// - `xapi_access_token_secret`
///
/// # Example
///
/// ```rust
/// let result = post_tweet("Hello from Rust!").await;
/// match result {
///     Ok(response) => println!("Tweet posted: {}", response),
///     Err(e) => eprintln!("Failed to post tweet: {}", e),
/// }
/// ```
///
/// # Errors
///
/// This function can fail for several reasons:
/// - Missing or invalid Twitter API credentials
/// - Network connectivity issues
/// - Twitter API rate limiting or other API errors
/// - Invalid tweet content (too long, etc.)
async fn post_tweet(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Load Twitter API credentials from environment variables
    let config = TwitterConfig::from_env()?;
    let client = Client::new();
    let url = "https://api.x.com/2/tweets";

    // Create the tweet payload
    let payload = json!({
        "text": text
    });

    // Build OAuth parameters and generate signature
    let mut oauth_params = build_oauth_params(&config)?;
    let signature = generate_oauth_signature(
        "POST",
        url,
        &oauth_params,
        &config.consumer_secret,
        &config.access_token_secret,
    );
    oauth_params.insert("oauth_signature".to_string(), signature);

    // Build the Authorization header with OAuth parameters
    let auth_header = build_auth_header(&oauth_params);

    // Send the authenticated request to Twitter API
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    // Handle the API response
    if response.status().is_success() {
        let response_text = response.text().await?;
        info!("Tweet posted successfully: {}", response_text);
        Ok(response_text)
    } else {
        let error_text = response.text().await?;
        error!("Failed to post tweet: {}", error_text);
        Err(format!("Twitter API error: {}", error_text).into())
    }
}

// Route handler functions

/// Handles GET requests to the `/reputest` endpoint.
///
/// This endpoint returns a simple "Reputesting!" message and logs the request.
/// It's primarily used for testing and demonstration purposes.
///
/// # Returns
///
/// A static string "Reputesting!".
async fn handle_reputest_get() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

/// Handles POST requests to the `/reputest` endpoint.
///
/// This endpoint returns a simple "Reputesting!" message and logs the request.
/// It's primarily used for testing and demonstration purposes.
///
/// # Returns
///
/// A static string "Reputesting!".
async fn handle_reputest_post() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

/// Handles GET requests to the `/health` endpoint.
///
/// This endpoint provides a health check for the service, returning the current
/// status and service name. It's commonly used by load balancers and monitoring
/// systems to verify that the service is running and responsive.
///
/// # Returns
///
/// A JSON response containing:
/// - `status`: Always "healthy" when the service is running
/// - `service`: The service name "reputest"
///
/// # Example Response
///
/// ```json
/// {
///   "status": "healthy",
///   "service": "reputest"
/// }
/// ```
async fn handle_health() -> Json<Value> {
    Json(json!({"status": "healthy", "service": "reputest"}))
}

/// Handles POST requests to the `/tweet` endpoint.
///
/// This endpoint posts a tweet to Twitter/X with the message "Hello world".
/// It demonstrates the OAuth 1.0a authentication flow and Twitter API integration.
///
/// # Returns
///
/// - `Ok(Json<Value>)`: Success response with tweet details
/// - `Err((StatusCode, Json<Value>))`: Error response with status code and error details
///
/// # Success Response
///
/// ```json
/// {
///   "status": "success",
///   "message": "Tweet posted",
///   "response": "<Twitter API response>"
/// }
/// ```
///
/// # Error Response
///
/// ```json
/// {
///   "status": "error",
///   "message": "Failed to post tweet",
///   "error": "<error details>"
/// }
/// ```
///
/// # Requirements
///
/// Requires Twitter API credentials to be set in environment variables.
async fn handle_tweet() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match post_tweet("Hello world").await {
        Ok(response) => {
            info!("Tweet posted successfully");
            Ok(Json(
                json!({"status": "success", "message": "Tweet posted", "response": response}),
            ))
        }
        Err(e) => {
            error!("Failed to post tweet: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({"status": "error", "message": "Failed to post tweet", "error": e.to_string()}),
                ),
            ))
        }
    }
}

/// Handles GET requests to the root `/` endpoint.
///
/// This endpoint returns a welcome message for the reputest service.
/// It serves as the main entry point for the service.
///
/// # Returns
///
/// A static string "A new reputest is in the house!".
async fn handle_root() -> &'static str {
    "A new reputest is in the house!"
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
/// // With PORT=8080 set in environment
/// let port = get_server_port(); // Returns 8080
///
/// // With no PORT set
/// let port = get_server_port(); // Returns 3000
/// ```
fn get_server_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number")
}

/// Main entry point for the reputest web service.
///
/// This function initializes the logging system, sets up the HTTP server with all routes,
/// and starts listening for incoming requests. The server runs indefinitely until terminated.
///
/// # Server Configuration
///
/// The server is configured with the following routes:
/// - `GET /`: Root endpoint with welcome message
/// - `GET /reputest`: Test endpoint returning "Reputesting!"
/// - `POST /reputest`: Test endpoint returning "Reputesting!"
/// - `GET /health`: Health check endpoint
/// - `POST /tweet`: Twitter API integration endpoint
///
/// # Middleware
///
/// The server includes HTTP request tracing middleware for logging and debugging.
///
/// # Port Configuration
///
/// The server port is determined by the `PORT` environment variable, defaulting to 3000.
///
/// # Logging
///
/// The application uses the `env_logger` crate for structured logging. Log levels
/// can be controlled via the `RUST_LOG` environment variable.
///
/// # Example Usage
///
/// ```bash
/// # Run with default port 3000
/// cargo run
///
/// # Run on custom port
/// PORT=8080 cargo run
///
/// # Run with debug logging
/// RUST_LOG=debug cargo run
/// ```
///
/// # Panics
///
/// This function will panic if:
/// - The server port cannot be bound (e.g., port already in use)
/// - There's an error starting the HTTP server
#[tokio::main]
async fn main() {
    // Initialize the logging system
    env_logger::init();

    // Build the HTTP application with all routes and middleware
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .route("/tweet", post(handle_tweet))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Get the server port and bind address
    let port = get_server_port();
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Starting reputest server on {}", addr);

    // Bind to the address and start serving requests
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests;
