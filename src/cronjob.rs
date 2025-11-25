//! Cronjob module for scheduled tasks.
//!
//! This module contains functionality for running scheduled tasks, specifically
//! for searching Twitter for tweets with specific hashtags and processing vibe-related queries.

use crate::db::{
    get_good_vibes_count, get_user_id_by_username, get_vibe_score_one, get_vibe_score_three,
    get_vibe_score_two, has_vibe_request, save_vibe_request,
};
use crate::twitter::{reply_to_tweet, search_mentions, search_tweets_with_hashtag};
use log::{debug, error, info};
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Starts the cronjob scheduler for searching tweets with hashtag "gmgv" and processing vibe queries every 5 minutes.
///
/// This function creates a new job scheduler and adds a job that runs every 5 minutes
/// to perform two tasks:
/// 1. Search for tweets containing the hashtag "gmgv" from the past 6 hours
/// 2. Check for mentions of @reputest from the past 6 hours and reply to:
///    - Specific vibe score queries (e.g., "@reputest @username?")
///    - General requests for the total vibes count (messages containing "vibecount")
///
/// The job will log all found tweets and mentions to the application logs.
///
/// # Returns
///
/// - `Ok(JobScheduler)`: The configured job scheduler
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If there's an error creating or configuring the scheduler
///
/// # Job Schedule
///
/// The job runs every 5 minutes using the cron expression "0 0/5 * * * * *"
/// which means:
/// - 0 seconds
/// - Every 5 minutes (0/5)
/// - Every hour
/// - Every day
/// - Every month
/// - Every day of the week
///
/// # Example
///
/// ```rust,no_run
/// use reputest::start_gmgv_cronjob;
///
/// #[tokio::main]
/// async fn main() {
///     let scheduler = start_gmgv_cronjob().await.unwrap();
///     scheduler.start().await.unwrap();
///     
///     // Keep the scheduler running
///     tokio::signal::ctrl_c().await.unwrap();
/// }
/// ```
///
/// # Errors
///
/// This function can fail if:
/// - The job scheduler cannot be created
/// - The cron expression is invalid
/// - There's an error adding the job to the scheduler
///
/// Processes the scheduled search for #gmgv tweets
async fn process_hashtag_search() {
    info!("Starting scheduled search for #gmgv tweets");
    match search_tweets_with_hashtag("gmgv").await {
        Ok(_) => {
            info!("Scheduled search for #gmgv tweets completed successfully");
        }
        Err(e) => {
            error!("Scheduled search for #gmgv tweets failed: {}", e);
        }
    }
}

/// Processes scheduled checks for @reputest mentions and replies to vibe queries
async fn process_mentions() {
    debug!("Starting scheduled check for @reputest mentions");
    match search_mentions().await {
        Ok(mentions) => {
            if mentions.is_empty() {
                info!("No mentions found to reply to");
                return;
            }

            info!("Found {} mentions to reply to", mentions.len());

            // Get the database pool for user lookups and vibe checks
            let pool = match crate::db::get_db_pool().await {
                Ok(pool) => pool,
                Err(e) => {
                    error!("Failed to get database pool for mentions processing: {}", e);
                    return;
                }
            };

            // Reply to each mention
            for (tweet_id, tweet_text, author_username, mentioned_user, created_at) in mentions {
                if let Some(mentioned_username) = mentioned_user {
                    process_vibe_query(
                        &pool,
                        &tweet_id,
                        &tweet_text,
                        &author_username,
                        &mentioned_username,
                        &created_at,
                    )
                    .await;
                } else if tweet_text.to_lowercase().contains("vibecount") {
                    process_vibecount_request(
                        &pool,
                        &tweet_id,
                        &tweet_text,
                        &author_username,
                        &created_at,
                    )
                    .await;
                } else {
                    info!("Skipping general mention from @{} at {} - no vibecount request or specific vibe query", author_username, created_at);
                }
            }

            info!("Scheduled check for mentions completed successfully");
        }
        Err(e) => {
            error!("Scheduled check for mentions failed: {}", e);
        }
    }
}

/// Processes a specific vibe score query (e.g., "@reputest @username?")
async fn process_vibe_query(
    pool: &PgPool,
    tweet_id: &str,
    _tweet_text: &str,
    author_username: &str,
    mentioned_username: &str,
    created_at: &str,
) {
    // First, check if this tweet has already been processed
    match has_vibe_request(pool, tweet_id).await {
        Ok(true) => {
            info!(
                "Skipping vibe query tweet {} from @{} asking about @{} (posted at {}) - already processed",
                tweet_id, author_username, mentioned_username, created_at
            );
            return;
        }
        Ok(false) => {
            // Tweet not processed yet, proceed with normal logic
        }
        Err(e) => {
            error!(
                "Failed to check if tweet {} has been processed: {}",
                tweet_id, e
            );
            return;
        }
    }

    // Look up the mentioned user's ID from database
    let mentioned_user_id = match get_user_id_by_username(pool, mentioned_username).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            reply_with_zero_score(pool, tweet_id, author_username, mentioned_username).await;
            return;
        }
        Err(e) => {
            error!(
                "Failed to lookup mentioned user @{}: {}",
                mentioned_username, e
            );
            return;
        }
    };

    // Get the author's user ID
    let author_user_id = match crate::db::get_user_id_by_username(pool, author_username).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            error!("Could not find user ID for author @{}", author_username);
            return;
        }
        Err(e) => {
            error!("Failed to get user ID for @{}: {}", author_username, e);
            return;
        }
    };

    // Calculate the three-degree vibe scores
    match tokio::try_join!(
        get_vibe_score_one(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_two(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_three(pool, &author_user_id, &mentioned_user_id)
    ) {
        Ok((score_one, score_two, score_three)) => {
            let reply_text = format!(
                "Your vibes for @{} are:\n1st degree: {}\n2nd degree: {}\n3rd degree: {}",
                mentioned_username, score_one, score_two, score_three
            );
            send_reply_and_mark_processed(pool, &reply_text, tweet_id, author_username).await;
        }
        Err(e) => {
            error!(
                "Failed to calculate vibe scores for @{} -> @{}: {}",
                author_username, mentioned_username, e
            );
        }
    }
}

/// Replies with a message for users not found in the database
async fn reply_with_zero_score(
    pool: &PgPool,
    tweet_id: &str,
    author_username: &str,
    mentioned_username: &str,
) {
    info!(
        "Mentioned user @{} not found in database, returning 'no vibes' message",
        mentioned_username
    );
    let reply_text = format!("@{} has no vibes", mentioned_username);
    info!(
        "Replying to vibe query tweet {} with: {} (user not found)",
        tweet_id, reply_text
    );

    match reply_to_tweet(&reply_text, tweet_id).await {
        Ok(_) => {
            info!(
                "Successfully replied to vibe query from @{} (user not found)",
                author_username
            );
            // Mark this tweet as processed
            if let Err(e) = save_vibe_request(pool, tweet_id).await {
                error!("Failed to save vibe request for tweet {}: {}", tweet_id, e);
            }
        }
        Err(e) => {
            error!(
                "Failed to reply to vibe query from @{}: {}",
                author_username, e
            );
        }
    }
}

/// Processes a vibecount request
async fn process_vibecount_request(
    pool: &PgPool,
    tweet_id: &str,
    _tweet_text: &str,
    author_username: &str,
    created_at: &str,
) {
    // First, check if this tweet has already been processed
    match has_vibe_request(pool, tweet_id).await {
        Ok(true) => {
            info!(
                "Skipping vibecount request tweet {} from @{} (posted at {}) - already processed",
                tweet_id, author_username, created_at
            );
            return;
        }
        Ok(false) => {
            // Tweet not processed yet, proceed with normal logic
        }
        Err(e) => {
            error!(
                "Failed to check if vibecount tweet {} has been processed: {}",
                tweet_id, e
            );
            return;
        }
    }

    // Reply with total vibes count
    match get_good_vibes_count(pool).await {
        Ok(vibes_count) => {
            let reply_text = format!(
                "Hello @{}! The current good vibes count is: {}",
                author_username, vibes_count
            );
            send_reply_and_mark_processed(pool, &reply_text, tweet_id, author_username).await;
        }
        Err(e) => {
            error!(
                "Failed to get good vibes count for vibecount request: {}",
                e
            );
        }
    }
}

/// Sends a reply to a tweet and marks it as processed
async fn send_reply_and_mark_processed(
    pool: &PgPool,
    reply_text: &str,
    tweet_id: &str,
    author_username: &str,
) {
    info!("Replying to tweet {} with: {}", tweet_id, reply_text);

    match reply_to_tweet(reply_text, tweet_id).await {
        Ok(_) => {
            info!("Successfully replied to request from @{}", author_username);
            // Mark this tweet as processed
            if let Err(e) = save_vibe_request(pool, tweet_id).await {
                error!("Failed to save vibe request for tweet {}: {}", tweet_id, e);
            }
        }
        Err(e) => {
            error!(
                "Failed to reply to request from @{}: {}",
                author_username, e
            );
        }
    }
}

pub async fn start_gmgv_cronjob() -> Result<JobScheduler, Box<dyn std::error::Error + Send + Sync>>
{
    let sched = JobScheduler::new().await?;

    // Create a job that runs every 5 minutes
    sched
        .add(Job::new_async("0 0/5 * * * * *", |_uuid, _l| {
            Box::pin(async {
                process_hashtag_search().await;
                process_mentions().await;
            })
        })?)
        .await?;

    info!("Cronjob scheduler configured to search for #gmgv tweets and process vibe queries every 5 minutes");
    Ok(sched)
}

/// Starts the cronjob scheduler and keeps it running.
///
/// This is a convenience function that starts the GMGV hashtag search and mentions
/// checking cronjob and keeps the scheduler running indefinitely. It handles graceful shutdown
/// when receiving a Ctrl+C signal.
///
/// # Returns
///
/// - `Ok(())`: If the scheduler runs successfully until shutdown
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If there's an error starting or running the scheduler
///
/// # Example
///
/// ```rust,no_run
/// use reputest::run_gmgv_cronjob;
///
/// #[tokio::main]
/// async fn main() {
///     if let Err(e) = run_gmgv_cronjob().await {
///         eprintln!("Cronjob failed: {}", e);
///     }
/// }
/// ```
#[allow(dead_code)]
pub async fn run_gmgv_cronjob() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut sched = start_gmgv_cronjob().await?;
    sched.start().await?;

    info!("Cronjob scheduler started successfully");

    // Wait for Ctrl+C signal to gracefully shutdown
    tokio::signal::ctrl_c().await?;
    info!("Received shutdown signal, stopping cronjob scheduler");

    sched.shutdown().await?;
    info!("Cronjob scheduler stopped");

    Ok(())
}
