//! Token Encryption Utility
//!
//! This script encrypts tokens using AES-256-GCM for secure database storage.
//! Requires TOKEN_ENCRYPTION_KEY environment variable to be set.

use std::io::{self, Write};

// Re-use the crypto module from the main crate
use reputest::crypto::encrypt_token;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔐 Token Encryption Utility");
    println!("===========================");
    println!();

    // Check if encryption key is configured
    if std::env::var("TOKEN_ENCRYPTION_KEY").is_err() {
        eprintln!("❌ Error: TOKEN_ENCRYPTION_KEY environment variable is not set.");
        eprintln!();
        eprintln!("Generate a key with:");
        eprintln!("  openssl rand -hex 32");
        eprintln!();
        eprintln!("Then set it:");
        eprintln!("  export TOKEN_ENCRYPTION_KEY=\"your_64_char_hex_key\"");
        std::process::exit(1);
    }

    // Get the token to encrypt
    print!("Enter the token to encrypt: ");
    io::stdout().flush()?;
    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim();

    if token.is_empty() {
        eprintln!("❌ Error: Token cannot be empty");
        std::process::exit(1);
    }

    // Encrypt the token
    match encrypt_token(token) {
        Ok(encrypted) => {
            println!();
            println!("✅ Token encrypted successfully!");
            println!();
            println!("Encrypted value (hex):");
            println!("{}", encrypted);
            println!();
            println!("📝 Use this value in your database INSERT statement.");
        }
        Err(e) => {
            eprintln!("❌ Encryption failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
