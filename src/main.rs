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

// Configuration struct for Twitter API credentials
#[derive(Debug)]
struct TwitterConfig {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_token_secret: String,
}

impl TwitterConfig {
    fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TwitterConfig {
            consumer_key: env::var("xapi_consumer_key")?,
            consumer_secret: env::var("xapi_consumer_secret")?,
            access_token: env::var("xapi_access_token")?,
            access_token_secret: env::var("xapi_access_token_secret")?,
        })
    }
}

fn generate_oauth_signature(
    method: &str,
    url: &str,
    params: &BTreeMap<String, String>,
    consumer_secret: &str,
    token_secret: &str,
) -> String {
    // Create parameter string
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

    // Create signature base string
    let signature_base = format!(
        "{}&{}&{}",
        method,
        percent_encode(url),
        percent_encode(&param_string)
    );

    // Create signing key
    let signing_key = format!(
        "{}&{}",
        percent_encode(consumer_secret),
        percent_encode(token_secret)
    );

    // Generate HMAC-SHA1 signature
    type HmacSha1 = Hmac<Sha1>;
    let mut mac =
        HmacSha1::new_from_slice(signing_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(signature_base.as_bytes());
    let result = mac.finalize();

    base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
}

fn percent_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    let nonce: String = (0..32)
        .map(|_| {
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();
    nonce
}

fn get_current_timestamp() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string())
}

fn build_oauth_params(
    config: &TwitterConfig,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut oauth_params = BTreeMap::new();
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

fn build_auth_header(oauth_params: &BTreeMap<String, String>) -> String {
    let mut auth_header = String::from("OAuth ");
    for (i, (key, value)) in oauth_params.iter().enumerate() {
        if i > 0 {
            auth_header.push_str(", ");
        }
        auth_header.push_str(&format!("{}=\"{}\"", key, percent_encode(value)));
    }
    auth_header
}

async fn post_tweet(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let config = TwitterConfig::from_env()?;
    let client = Client::new();
    let url = "https://api.x.com/2/tweets";

    let payload = json!({
        "text": text
    });

    // Build OAuth parameters and signature
    let mut oauth_params = build_oauth_params(&config)?;
    let signature = generate_oauth_signature(
        "POST",
        url,
        &oauth_params,
        &config.consumer_secret,
        &config.access_token_secret,
    );
    oauth_params.insert("oauth_signature".to_string(), signature);

    // Build Authorization header
    let auth_header = build_auth_header(&oauth_params);

    // Send the request
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    // Handle response
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
async fn handle_reputest_get() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

async fn handle_reputest_post() -> &'static str {
    info!("Reputesting!");
    "Reputesting!"
}

async fn handle_health() -> Json<Value> {
    Json(json!({"status": "healthy", "service": "reputest"}))
}

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

async fn handle_root() -> &'static str {
    "A new reputest is in the house!"
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number")
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Build our application with a route
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/reputest", get(handle_reputest_get))
        .route("/reputest", post(handle_reputest_post))
        .route("/health", get(handle_health))
        .route("/tweet", post(handle_tweet))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Start server
    let port = get_server_port();
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Starting reputest server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests;
