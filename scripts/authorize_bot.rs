//! Twitter Bot Authorization Script
//!
//! This script helps you obtain OAuth 2.0 User Context tokens for your Twitter bot.
//! Run this script once to get the access token, then use it in your bot.

use std::collections::HashMap;
use std::io::{self, Write};
use url::Url;

/// Generates a cryptographically secure random string for PKCE
fn generate_code_verifier() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..128)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generates code challenge from code verifier using SHA256
fn generate_code_challenge(code_verifier: &str) -> String {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

/// Builds the authorization URL for Twitter OAuth 2.0
fn build_authorization_url(client_id: &str, redirect_uri: &str, code_challenge: &str) -> String {
    let mut url = Url::parse("https://twitter.com/i/oauth2/authorize").unwrap();
    let mut query_params = HashMap::new();

    query_params.insert("response_type", "code");
    query_params.insert("client_id", client_id);
    query_params.insert("redirect_uri", redirect_uri);
    query_params.insert("scope", "tweet.read tweet.write users.read offline.access");
    query_params.insert("state", "state");
    query_params.insert("code_challenge", code_challenge);
    query_params.insert("code_challenge_method", "S256");

    for (key, value) in query_params {
        url.query_pairs_mut().append_pair(key, value);
    }

    url.to_string()
}

/// Exchanges authorization code for access token
async fn exchange_code_for_token(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("code", code);
    params.insert("code_verifier", code_verifier);

    let response = client
        .post("https://api.twitter.com/2/oauth2/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let response_text = response.text().await?;
        println!("Token response: {}", response_text);

        // Parse the JSON response to extract access_token and refresh_token
        let json: serde_json::Value = serde_json::from_str(&response_text)?;
        if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
            // Check if we also got a refresh token
            if let Some(refresh_token) = json.get("refresh_token").and_then(|v| v.as_str()) {
                println!("✅ Refresh token also received!");
                println!("📝 Set these environment variables for automatic token refresh:");
                println!(
                    "   - Fly.io: fly secrets set xapi_refresh_token=\"{}\"",
                    refresh_token
                );
                println!("   - Docker: Use environment variables or Docker secrets");
                println!("   - Local: Store in .env file (never commit to version control)");
                println!("");
                println!("💡 For automatic refresh, also set:");
                println!("   export xapi_client_id=\"your_client_id\"");
                println!("   export xapi_client_secret=\"your_client_secret\"");
                println!("");
                println!("🔄 With all credentials set, your bot will automatically refresh expired tokens!");
            }
            Ok(access_token.to_string())
        } else {
            Err("No access_token in response".into())
        }
    } else {
        let error_text = response.text().await?;
        Err(format!("Token exchange failed: {}", error_text).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🤖 Twitter Bot Authorization Helper");
    println!("=====================================");

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

    print!("Enter your Redirect URI (e.g., http://localhost:8080/callback): ");
    io::stdout().flush()?;
    let mut redirect_uri = String::new();
    io::stdin().read_line(&mut redirect_uri)?;
    let redirect_uri = redirect_uri.trim();

    // Generate PKCE parameters
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    // Build authorization URL
    let auth_url = build_authorization_url(client_id, redirect_uri, &code_challenge);

    println!("\n🔗 Authorization Steps:");
    println!("1. Open this URL in your browser:");
    println!("   {}", auth_url);
    println!("\n2. Authorize the application");
    println!("3. Copy the 'code' parameter from the callback URL");
    println!("4. Paste it below:");

    print!("\nEnter the authorization code: ");
    io::stdout().flush()?;
    let mut auth_code = String::new();
    io::stdin().read_line(&mut auth_code)?;
    let auth_code = auth_code.trim();

    // Exchange code for token
    println!("\n🔄 Exchanging code for access token...");
    let access_token = exchange_code_for_token(
        client_id,
        client_secret,
        redirect_uri,
        auth_code,
        &code_verifier,
    )
    .await?;

    println!("\n✅ Success! Your access token is:");
    println!("{}", access_token);
    println!("\n📝 Add this to your environment variables:");
    println!("export xapi_access_token=\"{}\"", access_token);

    Ok(())
}
