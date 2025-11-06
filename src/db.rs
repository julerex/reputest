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

/// Stores good vibes data in the database.
///
/// This function inserts information about good vibes between users into the
/// good_vibes table. It includes the tweet ID, emitter user ID, sensor user ID,
/// and the timestamp when the good vibes were created.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `tweet_id`: The ID of the tweet that contains the good vibes
/// - `emitter_id`: The user ID of the person sending good vibes (emitter)
/// - `sensor_id`: The user ID of the person receiving good vibes (sensor)
/// - `created_at`: The timestamp when the tweet was created
///
/// # Returns
///
/// - `Ok(())`: If the vibes data was successfully stored
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the insert fails
pub async fn save_good_vibes(
    pool: &PgPool,
    tweet_id: &str,
    emitter_id: &str,
    sensor_id: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Storing good vibes data in database: tweet {} from {} to {} at {}",
        tweet_id, emitter_id, sensor_id, created_at
    );

    sqlx::query(
        r#"
        INSERT INTO good_vibes (tweet_id, emitter_id, sensor_id, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(tweet_id)
    .bind(emitter_id)
    .bind(sensor_id)
    .bind(created_at)
    .execute(pool)
    .await?;

    info!("Successfully stored good vibes data in database");
    Ok(())
}

/// Stores user data in the database.
///
/// This function inserts or updates user information in the users table.
/// It uses ON CONFLICT to handle cases where the user already exists.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `user_id`: The Twitter user ID
/// - `username`: The Twitter username
/// - `name`: The Twitter display name
/// - `created_at`: The timestamp when the user account was created
///
/// # Returns
///
/// - `Ok(())`: If the user data was successfully stored
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the insert/update fails
pub async fn save_user(
    pool: &PgPool,
    user_id: &str,
    username: &str,
    name: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Storing user data in database: {} (@{}) created at {}",
        name, username, created_at
    );

    sqlx::query(
        r#"
        INSERT INTO users (id, username, name, created_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE SET
            username = EXCLUDED.username,
            name = EXCLUDED.name,
            created_at = EXCLUDED.created_at
        "#,
    )
    .bind(user_id)
    .bind(username)
    .bind(name)
    .bind(created_at)
    .execute(pool)
    .await?;

    info!("Successfully stored user data in database");
    Ok(())
}

/// Retrieves the count of good vibes records from the database.
///
/// This function queries the good_vibes table and returns the total count of records.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
///
/// # Returns
///
/// - `Ok(i64)`: The count of good vibes records
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_good_vibes_count(
    pool: &PgPool,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    info!("Querying database for good vibes count");

    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) as count
        FROM good_vibes
        "#,
    )
    .fetch_one(pool)
    .await?;

    info!("Found {} good vibes records in database", count);
    Ok(count)
}
