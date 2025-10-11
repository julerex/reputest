use log::{info, error};
use std::env;
use std::net::SocketAddr;
use warp::{http::Response, Filter};
use reqwest::Client;
use serde_json::json;
use sha1::Sha1;
use rand::Rng;
use std::collections::BTreeMap;
use hmac::{Hmac, Mac};
use base64::Engine;

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
        param_string.push_str(&format!("{}={}", percent_encode(key), percent_encode(value)));
    }

    // Create signature base string
    let signature_base = format!(
        "{}&{}&{}",
        method,
        percent_encode(url),
        percent_encode(&param_string)
    );

    // Create signing key
    let signing_key = format!("{}&{}", percent_encode(consumer_secret), percent_encode(token_secret));

    // Generate HMAC-SHA1 signature
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
        .expect("HMAC can take key of any size");
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

async fn post_tweet(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let consumer_key = env::var("xapi_consumer_key")?;
    let consumer_secret = env::var("xapi_consumer_secret")?;
    let access_token = env::var("xapi_access_token")?;
    let access_token_secret = env::var("xapi_access_token_secret")?;

    let client = Client::new();
    let url = "https://api.x.com/2/tweets";
    
    let payload = json!({
        "text": text
    });

    // Generate OAuth parameters
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();
    let nonce = generate_nonce();

    let mut oauth_params = BTreeMap::new();
    oauth_params.insert("oauth_consumer_key".to_string(), consumer_key.clone());
    oauth_params.insert("oauth_nonce".to_string(), nonce);
    oauth_params.insert("oauth_signature_method".to_string(), "HMAC-SHA1".to_string());
    oauth_params.insert("oauth_timestamp".to_string(), timestamp);
    oauth_params.insert("oauth_token".to_string(), access_token.clone());
    oauth_params.insert("oauth_version".to_string(), "1.0".to_string());

    let signature = generate_oauth_signature("POST", url, &oauth_params, &consumer_secret, &access_token_secret);
    oauth_params.insert("oauth_signature".to_string(), signature);

    // Build Authorization header
    let mut auth_header = String::from("OAuth ");
    for (i, (key, value)) in oauth_params.iter().enumerate() {
        if i > 0 {
            auth_header.push_str(", ");
        }
        auth_header.push_str(&format!("{}=\"{}\"", key, percent_encode(value)));
    }

    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

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

#[tokio::main]
async fn main() {
    env_logger::init();

    // Define routes
    let reputest_get = warp::path("reputest")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| {
            info!("Reputesting!");
            Response::builder()
                .header("Content-Type", "text/plain")
                .body("Reputesting!")
        });

    let reputest_post = warp::path("reputest")
        .and(warp::path::end())
        .and(warp::post())
        .map(|| {
            info!("Reputesting!");
            Response::builder()
                .header("Content-Type", "text/plain")
                .body("Reputesting!")
        });

    let health = warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .map(|| {
            Response::builder()
                .header("Content-Type", "application/json")
                .body(r#"{"status":"healthy","service":"reputest"}"#)
        });

    let tweet = warp::path("tweet")
        .and(warp::path::end())
        .and(warp::post())
        .and_then(|| async {
            match post_tweet("Hello world").await {
                Ok(response) => {
                    info!("Tweet posted successfully");
                    Ok::<_, warp::Rejection>(
                        Response::builder()
                            .header("Content-Type", "application/json")
                            .body(format!(r#"{{"status":"success","message":"Tweet posted","response":"{}"}}"#, response))
                    )
                }
                Err(e) => {
                    error!("Failed to post tweet: {}", e);
                    Ok::<_, warp::Rejection>(
                        Response::builder()
                            .status(500)
                            .header("Content-Type", "application/json")
                            .body(format!(r#"{{"status":"error","message":"Failed to post tweet","error":"{}"}}"#, e))
                    )
                }
            }
        });

    let root = warp::path::end().and(warp::get()).map(|| {
        Response::builder()
            .header("Content-Type", "text/plain")
            .body("A new reputest is in the house!")
    });

    // Combine all routes
    let routes = reputest_get.or(reputest_post).or(health).or(tweet).or(root);

    // Get port from environment variable, default to 3000
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    info!("Starting reputest server on {}", addr);
    warp::serve(routes).run(addr).await
}
