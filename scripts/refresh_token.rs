//! Twitter Bot Token Refresh Utility
//!
//! This script helps you refresh your OAuth 2.0 User Context access token
//! when it expires.

use std::io::{self, Write};

/// Refreshes an OAuth 2.0 User Context access token using a refresh token
async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<(String, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let mut params = std::collections::HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);

    let response = client
        .post("https://api.twitter.com/2/oauth2/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let response_text = response.text().await?;
        println!("Token refresh response: {}", response_text);

        // Parse the JSON response to extract access_token and potentially new refresh_token
        let json: serde_json::Value = serde_json::from_str(&response_text)?;
        if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
            // Check if we also got a new refresh token
            let new_refresh_token = if let Some(new_refresh_token) =
                json.get("refresh_token").and_then(|v| v.as_str())
            {
                println!("✅ New refresh token also received!");
                println!("📝 Update your refresh token in your secure storage:");
                println!(
                    "   - Fly.io: fly secrets set xapi_refresh_token=\"{}\"",
                    new_refresh_token
                );
                println!("   - Docker: Update your environment variables or Docker secrets");
                println!("   - Local: Update your .env file");
                Some(new_refresh_token.to_string())
            } else {
                None
            };
            Ok((access_token.to_string(), new_refresh_token))
        } else {
            Err("No access_token in response".into())
        }
    } else {
        let error_text = response.text().await?;
        Err(format!("Token refresh failed: {}", error_text).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔄 Twitter Bot Token Refresh Utility");
    println!("====================================");

    // Get credentials from user
    print!("Enter your Twitter App Client ID: ");
    io::stdout().flush()?;
    let mut client_id = String::new();
    io::stdin().read_line(&mut client_id)?;
    let client_id = client_id.trim();

    print!("Enter your Twitter App Client Secret: ");
    io::stdout().flush()?;
    let mut client_secret = String::new();
    io::stdin().read_line(&mut client_secret)?;
    let client_secret = client_secret.trim();

    print!("Enter your refresh token: ");
    io::stdout().flush()?;
    let mut refresh_token = String::new();
    io::stdin().read_line(&mut refresh_token)?;
    let refresh_token = refresh_token.trim();

    // Refresh the token
    println!("\n🔄 Refreshing access token...");
    let (access_token, new_refresh_token) =
        refresh_access_token(client_id, client_secret, refresh_token).await?;

    println!("\n✅ Success! Your new access token is:");
    println!("{}", access_token);
    println!("\n📝 Update your access token:");
    println!(
        "   - Fly.io: fly secrets set xapi_access_token=\"{}\"",
        access_token
    );
    println!("   - Docker: Update your environment variables");
    println!("   - Local: export xapi_access_token=\"{}\"", access_token);

    // If we got a new refresh token, remind user to update it
    if new_refresh_token.is_some() {
        println!("\n⚠️  IMPORTANT: Your old refresh token is now invalid!");
        println!("   You must update your refresh token to continue automatic refresh.");
    }

    Ok(())
}
