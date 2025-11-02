//! Database module for storing and retrieving tokens.
//!
//! This module provides functionality to store and retrieve OAuth 2.0 refresh tokens
//! and access tokens in a PostgreSQL database. It manages the refresh_tokens and
//! access_tokens tables which store tokens along with their creation timestamps.

use log::{debug, info, warn};
use sqlx::{PgPool, Row};
use std::env;

/// Establishes a connection to the PostgreSQL database using DATABASE_URL.
///
/// # Returns
///
/// - `Ok(PgPool)`: A connection pool to the database
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the connection fails or DATABASE_URL is missing
pub async fn get_db_pool() -> Result<PgPool, Box<dyn std::error::Error + Send + Sync>> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable is not set")?;

    info!("Connecting to PostgreSQL database");
    debug!(
        "Database URL (masked): {}...",
        &database_url[..std::cmp::min(database_url.len(), 20)]
    );

    let pool = PgPool::connect(&database_url).await?;
    info!("Successfully connected to PostgreSQL database");

    Ok(pool)
}

/// Retrieves the most recent refresh token from the database.
///
/// This function queries the refresh_tokens table and returns the token
/// with the latest created_at timestamp.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
///
/// # Returns
///
/// - `Ok(Option<String>)`: The latest refresh token if one exists, None otherwise
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_latest_refresh_token(
    pool: &PgPool,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Querying database for latest refresh token");

    let row = sqlx::query(
        r#"
        SELECT token, created_at
        FROM refresh_tokens
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let token: String = row.get("token");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

            let token_length = token.len();
            let masked_token = if token_length > 16 {
                format!("{}...{}", &token[..8], &token[token_length - 8..])
            } else if token_length > 8 {
                format!("{}...", &token[..8])
            } else {
                format!("{}...", &token[..8])
            };

            info!(
                "Found refresh token created at {} (masked: {})",
                created_at, masked_token
            );
            debug!("Refresh token length: {}", token_length);

            Ok(Some(token))
        }
        None => {
            warn!("No refresh tokens found in database");
            Ok(None)
        }
    }
}

/// Stores a new refresh token in the database.
///
/// This function inserts a new refresh token into the refresh_tokens table
/// with the current timestamp. The old tokens remain in the table for historical
/// purposes, but only the latest one will be retrieved.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `token`: The refresh token to store
///
/// # Returns
///
/// - `Ok(())`: If the token was successfully stored
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the insert fails
pub async fn save_refresh_token(
    pool: &PgPool,
    token: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Storing new refresh token in database");

    let token_length = token.len();
    let masked_token = if token_length > 16 {
        format!("{}...{}", &token[..8], &token[token_length - 8..])
    } else if token_length > 8 {
        format!("{}...", &token[..8])
    } else {
        format!("{}...", &token[..8])
    };

    debug!("Refresh token length: {}", token_length);
    debug!("Refresh token (masked): {}", masked_token);

    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (token, created_at)
        VALUES ($1, NOW())
        "#,
    )
    .bind(token)
    .execute(pool)
    .await?;

    info!("Successfully stored new refresh token in database");
    Ok(())
}

/// Retrieves the most recent access token from the database.
///
/// This function queries the access_tokens table and returns the token
/// with the latest created_at timestamp.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
///
/// # Returns
///
/// - `Ok(Option<String>)`: The latest access token if one exists, None otherwise
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_latest_access_token(
    pool: &PgPool,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Querying database for latest access token");

    let row = sqlx::query(
        r#"
        SELECT token, created_at
        FROM access_tokens
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let token: String = row.get("token");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

            let token_length = token.len();
            let masked_token = if token_length > 16 {
                format!("{}...{}", &token[..8], &token[token_length - 8..])
            } else if token_length > 8 {
                format!("{}...", &token[..8])
            } else {
                format!("{}...", &token[..8])
            };

            info!(
                "Found access token created at {} (masked: {})",
                created_at, masked_token
            );
            debug!("Access token length: {}", token_length);

            Ok(Some(token))
        }
        None => {
            warn!("No access tokens found in database");
            Ok(None)
        }
    }
}

/// Stores a new access token in the database.
///
/// This function inserts a new access token into the access_tokens table
/// with the current timestamp. The old tokens remain in the table for historical
/// purposes, but only the latest one will be retrieved.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `token`: The access token to store
///
/// # Returns
///
/// - `Ok(())`: If the token was successfully stored
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the insert fails
pub async fn save_access_token(
    pool: &PgPool,
    token: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Storing new access token in database");

    let token_length = token.len();
    let masked_token = if token_length > 16 {
        format!("{}...{}", &token[..8], &token[token_length - 8..])
    } else if token_length > 8 {
        format!("{}...", &token[..8])
    } else {
        format!("{}...", &token[..8])
    };

    debug!("Access token length: {}", token_length);
    debug!("Access token (masked): {}", masked_token);

    sqlx::query(
        r#"
        INSERT INTO access_tokens (token, created_at)
        VALUES ($1, NOW())
        "#,
    )
    .bind(token)
    .execute(pool)
    .await?;

    info!("Successfully stored new access token in database");
    Ok(())
}

/// Creates the access_tokens table if it doesn't exist.
///
/// This function is safe to call multiple times as it uses CREATE TABLE IF NOT EXISTS.
/// It's typically called during application initialization or via the setup script.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
///
/// # Returns
///
/// - `Ok(())`: If the table was created or already exists
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the table creation fails
pub async fn create_access_tokens_table(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Creating access_tokens table if it doesn't exist");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS access_tokens (
            id SERIAL PRIMARY KEY,
            token TEXT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("access_tokens table ready");

    // Create an index on created_at for faster queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_access_tokens_created_at
        ON access_tokens (created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    info!("Index on created_at created (if it didn't exist)");

    Ok(())
}

/// Creates the refresh_tokens table if it doesn't exist.
///
/// This function is safe to call multiple times as it uses CREATE TABLE IF NOT EXISTS.
/// It's typically called during application initialization or via the setup script.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
///
/// # Returns
///
/// - `Ok(())`: If the table was created or already exists
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the table creation fails
pub async fn create_refresh_tokens_table(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Creating refresh_tokens table if it doesn't exist");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id SERIAL PRIMARY KEY,
            token TEXT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("refresh_tokens table ready");

    // Create an index on created_at for faster queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_refresh_tokens_created_at
        ON refresh_tokens (created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    info!("Index on created_at created (if it didn't exist)");

    Ok(())
}
