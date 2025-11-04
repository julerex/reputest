//! Twitter Tweet Posting Script
//!
//! This script allows you to post a tweet to Twitter/X by providing
//! your access token and the message you want to tweet.

use std::io::{self, Write};

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
fn build_oauth2_user_context_header(access_token: &str) -> String {
    let header = format!("Bearer {}", access_token);
    println!("🔑 Authorization header built successfully");
    header
}

/// Posts a tweet to Twitter/X using the API v2 endpoint.
///
/// This function uses OAuth 2.0 User Context authentication to post a tweet
/// to the Twitter/X API v2 endpoint. It builds the proper authorization header
/// and sends the request with the tweet content.
///
/// # Parameters
///
/// - `access_token`: The OAuth 2.0 User Context Access Token
/// - `text`: The text content of the tweet to post
///
/// # Returns
///
/// - `Ok(String)`: The API response body on successful tweet posting
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If authentication fails, network error, or API error
async fn post_tweet(
    access_token: &str,
    text: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting tweet post operation for text: '{}'", text);

    let client = reqwest::Client::new();
    let url = "https://api.x.com/2/tweets";
    println!("📍 Target URL: {}", url);

    // Create the tweet payload
    let payload = serde_json::json!({
        "text": text
    });
    println!("📝 Tweet payload created");

    // Build the Authorization header with OAuth 2.0 User Context Access Token
    println!("🔐 Building OAuth 2.0 User Context authorization header");
    let auth_header = build_oauth2_user_context_header(access_token);

    // Log request details
    println!("📤 Sending POST request to Twitter API v2");
    println!("🔗 Request URL: {}", url);
    println!(
        "📋 Request headers: Authorization: Bearer [REDACTED], Content-Type: application/json"
    );

    // Create the request builder
    let request_builder = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(&payload);

    // Send the request
    let response = request_builder.send().await?;
    let status = response.status();
    println!("📊 Received response with status: {}", status);

    if status.is_success() {
        let response_text = response.text().await?;
        println!("✅ Tweet posted successfully!");
        println!("📄 Response: {}", response_text);
        Ok(response_text)
    } else {
        let error_text = response.text().await?;
        println!("❌ Tweet posting failed!");
        println!("🚨 Status: {}, Error: {}", status, error_text);
        Err(format!("Twitter API error ({}): {}", status, error_text).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🐦 Twitter Tweet Posting Tool");
    println!("==============================");

    // Get access token from user
    print!("🔑 Enter your Twitter Access Token: ");
    io::stdout().flush()?;
    let mut access_token = String::new();
    io::stdin().read_line(&mut access_token)?;
    let access_token = access_token.trim();

    if access_token.is_empty() {
        println!("❌ Access token cannot be empty!");
        return Err("Access token is required".into());
    }

    // Get tweet message from user
    print!("📝 Enter your tweet message: ");
    io::stdout().flush()?;
    let mut tweet_text = String::new();
    io::stdin().read_line(&mut tweet_text)?;
    let tweet_text = tweet_text.trim();

    if tweet_text.is_empty() {
        println!("❌ Tweet message cannot be empty!");
        return Err("Tweet message is required".into());
    }

    // Validate tweet length (Twitter's limit is 280 characters)
    if tweet_text.chars().count() > 280 {
        println!(
            "❌ Tweet is too long! {} characters (max 280)",
            tweet_text.chars().count()
        );
        return Err("Tweet exceeds 280 character limit".into());
    }

    println!("📏 Tweet length: {} characters", tweet_text.chars().count());

    // Post the tweet
    println!("\n🚀 Posting your tweet...");
    match post_tweet(access_token, tweet_text).await {
        Ok(response) => {
            println!("\n🎉 Success! Your tweet has been posted.");
            println!("📄 Full response: {}", response);
        }
        Err(e) => {
            println!("\n💥 Failed to post tweet: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
