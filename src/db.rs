//! Database module for storing and retrieving tokens.
//!
//! This module provides functionality to store and retrieve OAuth 2.0 refresh tokens
//! and access tokens in a PostgreSQL database. It manages the refresh_tokens and
//! access_tokens tables which store tokens along with their creation timestamps.

use log::{debug, info, warn};
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet, VecDeque};
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

/// Checks if there is a good vibes record between a specific sensor and emitter.
///
/// This function queries the good_vibes table to see if there is a record where
/// the sensor_id matches the provided sensor and emitter_id matches the provided emitter.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `sensor_id`: The user ID of the person who received good vibes (sensor)
/// - `emitter_id`: The user ID of the person who sent good vibes (emitter)
///
/// # Returns
///
/// - `Ok(true)`: If a good vibes record exists between the sensor and emitter
/// - `Ok(false)`: If no good vibes record exists between the sensor and emitter
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn has_good_vibes_record(
    pool: &PgPool,
    sensor_id: &str,
    emitter_id: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Checking for good vibes record between sensor {} and emitter {}",
        sensor_id, emitter_id
    );

    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM good_vibes
            WHERE sensor_id = $1 AND emitter_id = $2
        ) as exists
        "#,
    )
    .bind(sensor_id)
    .bind(emitter_id)
    .fetch_one(pool)
    .await?;

    info!(
        "Good vibes record check result: {} (sensor: {}, emitter: {})",
        exists, sensor_id, emitter_id
    );
    Ok(exists)
}

/// Retrieves a user ID by username from the database.
///
/// This function queries the users table to find the user ID for a given username.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `username`: The Twitter username to look up
///
/// # Returns
///
/// - `Ok(Some(user_id))`: The user ID if the username exists
/// - `Ok(None)`: If the username is not found
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_user_id_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Looking up user ID for username: {}", username);

    let user_id: Option<String> = sqlx::query_scalar(
        r#"
        SELECT id FROM users WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    match &user_id {
        Some(id) => info!("Found user ID {} for username @{}", id, username),
        None => info!("No user found with username @{}", username),
    }

    Ok(user_id)
}

/// Retrieves complete user information by username from the database.
///
/// This function queries the users table to get all stored information
/// for a user by their username.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `username`: The Twitter username to look up
///
/// # Returns
///
/// - `Ok(Some((user_id, name, created_at)))`: Complete user information if found
/// - `Ok(None)`: If the username is not found in the database
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_user_info_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<
    Option<(String, String, chrono::DateTime<chrono::Utc>)>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    info!("Looking up complete user info for username: {}", username);

    let result: Option<(String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT id, name, created_at FROM users WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    match &result {
        Some((id, name, created_at)) => info!(
            "Found user info for @{}: {} (ID: {}) created at {}",
            username, name, id, created_at
        ),
        None => info!("No user info found for username @{}", username),
    }

    Ok(result)
}

/// Checks if a tweet ID exists in the vibe_requests table.
///
/// This function queries the vibe_requests table to see if the given tweet_id
/// has already been processed for a vibe score query.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `tweet_id`: The tweet ID to check
///
/// # Returns
///
/// - `Ok(true)`: If the tweet ID exists in the vibe_requests table
/// - `Ok(false)`: If the tweet ID does not exist in the vibe_requests table
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn has_vibe_request(
    pool: &PgPool,
    tweet_id: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Checking if tweet {} has been processed for vibe request",
        tweet_id
    );

    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM vibe_requests
            WHERE tweet_id = $1
        ) as exists
        "#,
    )
    .bind(tweet_id)
    .fetch_one(pool)
    .await?;

    info!(
        "Vibe request check result: {} (tweet: {})",
        exists, tweet_id
    );
    Ok(exists)
}

/// Checks if a tweet ID exists in the good_vibes table.
///
/// This function queries the good_vibes table to see if the given tweet_id
/// has already been processed for good vibes data.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `tweet_id`: The tweet ID to check
///
/// # Returns
///
/// - `Ok(true)`: If the tweet ID exists in the good_vibes table
/// - `Ok(false)`: If the tweet ID does not exist in the good_vibes table
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn has_good_vibes_tweet(
    pool: &PgPool,
    tweet_id: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Checking if tweet {} has already been processed for good vibes",
        tweet_id
    );

    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM good_vibes
            WHERE tweet_id = $1
        ) as exists
        "#,
    )
    .bind(tweet_id)
    .fetch_one(pool)
    .await?;

    info!(
        "Good vibes tweet check result: {} (tweet: {})",
        exists, tweet_id
    );
    Ok(exists)
}

/// Stores a vibe request in the database.
///
/// This function inserts a tweet ID into the vibe_requests table to mark
/// that a vibe score query has been processed for this tweet.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `tweet_id`: The tweet ID to store
///
/// # Returns
///
/// - `Ok(())`: If the vibe request was successfully stored
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the insert fails
pub async fn save_vibe_request(
    pool: &PgPool,
    tweet_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Storing vibe request for tweet {} in database", tweet_id);

    sqlx::query(
        r#"
        INSERT INTO vibe_requests (tweet_id)
        VALUES ($1)
        "#,
    )
    .bind(tweet_id)
    .execute(pool)
    .await?;

    info!(
        "Successfully stored vibe request for tweet {} in database",
        tweet_id
    );
    Ok(())
}

/// Retrieves all good vibes relationships as an adjacency list.
///
/// This function queries the good_vibes table and returns a HashMap where
/// keys are emitter user IDs and values are vectors of sensor user IDs.
/// This represents the directed graph where an edge from A to B means
/// A has good vibes for B.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
///
/// # Returns
///
/// - `Ok(HashMap<String, Vec<String>>)`: Adjacency list representation of the graph
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
#[allow(dead_code)]
pub async fn get_good_vibes_graph(
    pool: &PgPool,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Building good vibes graph from database");

    let rows = sqlx::query(
        r#"
        SELECT emitter_id, sensor_id
        FROM good_vibes
        ORDER BY created_at
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for row in rows {
        let emitter_id: String = row.get("emitter_id");
        let sensor_id: String = row.get("sensor_id");

        graph.entry(emitter_id).or_default().push(sensor_id);
    }

    info!("Built good vibes graph with {} nodes", graph.len());
    debug!("Graph structure: {:?}", graph.keys().collect::<Vec<_>>());

    Ok(graph)
}

/// Calculates the shortest path distance (in degrees) between two users in the good vibes graph.
///
/// This function implements a BFS algorithm to find the minimum number of "hops"
/// between a source user and a target user in the directed good vibes graph.
/// A distance of 1 means direct connection, 2 means through one intermediate, etc.
/// Returns None if no path exists between the users.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `source_user_id`: The user ID to start the search from (emitter)
/// - `target_user_id`: The user ID to find the path to (sensor)
/// - `max_depth`: Maximum depth to search (to prevent infinite loops in large graphs)
///
/// # Returns
///
/// - `Ok(Some(distance))`: The shortest path distance if a path exists
/// - `Ok(None)`: If no path exists between the users
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
///
/// # Example
///
/// If Alice has good vibes for Bob, and Bob has good vibes for Charlie:
/// - Distance from Alice to Bob: 1
/// - Distance from Bob to Charlie: 1
/// - Distance from Alice to Charlie: 2
#[allow(dead_code)]
pub async fn get_vibe_distance(
    pool: &PgPool,
    source_user_id: &str,
    target_user_id: &str,
    max_depth: usize,
) -> Result<Option<usize>, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Calculating vibe distance from {} to {} (max depth: {})",
        source_user_id, target_user_id, max_depth
    );

    // If source and target are the same, distance is 0
    if source_user_id == target_user_id {
        return Ok(Some(0));
    }

    let graph = get_good_vibes_graph(pool).await?;

    // BFS to find shortest path
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new(); // (user_id, distance)

    queue.push_back((source_user_id.to_string(), 0));
    visited.insert(source_user_id.to_string());

    while let Some((current_user, distance)) = queue.pop_front() {
        // If we've exceeded max depth, stop searching this path
        if distance >= max_depth {
            continue;
        }

        // Get neighbors (users that current_user has good vibes for)
        if let Some(neighbors) = graph.get(&current_user) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    // Found the target!
                    if neighbor == target_user_id {
                        let final_distance = distance + 1;
                        info!(
                            "Found path from {} to {} with distance {}",
                            source_user_id, target_user_id, final_distance
                        );
                        return Ok(Some(final_distance));
                    }

                    visited.insert(neighbor.clone());
                    queue.push_back((neighbor.clone(), distance + 1));
                }
            }
        }
    }

    info!(
        "No path found from {} to {} within max depth {}",
        source_user_id, target_user_id, max_depth
    );
    Ok(None)
}

/// Calculates the first-degree vibe score (direct connections) between two users.
///
/// This function returns 1 if there's a direct connection from emitter to sensor,
/// and 0 otherwise.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `sensor_user_id`: The user ID of the person receiving good vibes (sensor)
/// - `emitter_user_id`: The user ID of the person giving good vibes (emitter)
///
/// # Returns
///
/// - `Ok(1)`: Direct connection exists
/// - `Ok(0)`: No direct connection
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_vibe_score_one(
    pool: &PgPool,
    sensor_user_id: &str,
    emitter_user_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Calculating first-degree vibe score for sensor {} from emitter {}",
        sensor_user_id, emitter_user_id
    );

    let has_direct = has_good_vibes_record(pool, sensor_user_id, emitter_user_id).await?;
    let score = if has_direct { 1 } else { 0 };

    info!(
        "First-degree vibe score from {} to {}: {}",
        emitter_user_id, sensor_user_id, score
    );

    Ok(score)
}

/// Calculates the second-degree vibe score (paths of length 2) between two users.
///
/// This function counts the number of paths of length exactly 2 from emitter to sensor
/// in the good vibes graph (emitter -> X -> sensor).
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `sensor_user_id`: The user ID of the person receiving good vibes (sensor)
/// - `emitter_user_id`: The user ID of the person giving good vibes (emitter)
///
/// # Returns
///
/// - `Ok(count)`: Number of paths of length 2
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_vibe_score_two(
    pool: &PgPool,
    sensor_user_id: &str,
    emitter_user_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Calculating second-degree vibe score for sensor {} from emitter {}",
        sensor_user_id, emitter_user_id
    );

    let path_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) as path_count
        FROM good_vibes g1
        JOIN good_vibes g2 ON g1.sensor_id = g2.emitter_id
        WHERE g1.emitter_id = $1 AND g2.sensor_id = $2
        "#,
    )
    .bind(emitter_user_id)
    .bind(sensor_user_id)
    .fetch_one(pool)
    .await?;

    let score = path_count as usize;
    info!(
        "Found {} paths of length 2 from {} to {} - second-degree score: {}",
        path_count, emitter_user_id, sensor_user_id, score
    );

    Ok(score)
}

/// Calculates the third-degree vibe score (paths of length 3) between two users.
///
/// This function counts the number of paths of length exactly 3 from emitter to sensor
/// in the good vibes graph (emitter -> X -> Y -> sensor).
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `sensor_user_id`: The user ID of the person receiving good vibes (sensor)
/// - `emitter_user_id`: The user ID of the person giving good vibes (emitter)
///
/// # Returns
///
/// - `Ok(count)`: Number of paths of length 3
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the query fails
pub async fn get_vibe_score_three(
    pool: &PgPool,
    sensor_user_id: &str,
    emitter_user_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Calculating third-degree vibe score for sensor {} from emitter {}",
        sensor_user_id, emitter_user_id
    );

    let path_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) as path_count
        FROM good_vibes g1
        JOIN good_vibes g2 ON g1.sensor_id = g2.emitter_id
        JOIN good_vibes g3 ON g2.sensor_id = g3.emitter_id
        WHERE g1.emitter_id = $1 AND g3.sensor_id = $2
        "#,
    )
    .bind(emitter_user_id)
    .bind(sensor_user_id)
    .fetch_one(pool)
    .await?;

    let score = path_count as usize;
    info!(
        "Found {} paths of length 3 from {} to {} - third-degree score: {}",
        path_count, emitter_user_id, sensor_user_id, score
    );

    Ok(score)
}

/// Calculates the combined vibe score between two users (deprecated - use individual degree functions).
///
/// This function is kept for backward compatibility but now delegates to the individual
/// degree functions. For new code, use get_vibe_score_one, get_vibe_score_two, and get_vibe_score_three.
///
/// # Parameters
///
/// - `pool`: A reference to the PostgreSQL connection pool
/// - `sensor_user_id`: The user ID of the person receiving good vibes (sensor)
/// - `emitter_user_id`: The user ID of the person giving good vibes (emitter)
/// - `max_depth`: Maximum depth to search for connections (unused)
///
/// # Returns
///
/// - `Ok(score)`: The second-degree vibe score (for backward compatibility)
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If the calculation fails
#[allow(dead_code)]
#[deprecated(note = "Use get_vibe_score_one, get_vibe_score_two, and get_vibe_score_three instead")]
pub async fn get_vibe_score(
    pool: &PgPool,
    sensor_user_id: &str,
    emitter_user_id: &str,
    _max_depth: usize,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    get_vibe_score_two(pool, sensor_user_id, emitter_user_id).await
}
