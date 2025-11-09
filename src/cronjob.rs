//! Cronjob module for scheduled tasks.
//!
//! This module contains functionality for running scheduled tasks, specifically
//! for searching Twitter for tweets with specific hashtags and checking for mentions.

use crate::db::{get_good_vibes_count, get_user_id_by_username, has_good_vibes_record};
use crate::twitter::{reply_to_tweet, search_mentions, search_tweets_with_hashtag};
use log::{error, info};
use tokio_cron_scheduler::{Job, JobScheduler};

/// Starts the cronjob scheduler for searching tweets with hashtag "gmgv" and checking mentions every hour.
///
/// This function creates a new job scheduler and adds a job that runs every hour
/// to perform two tasks:
/// 1. Search for tweets containing the hashtag "gmgv" from the past 7 days
/// 2. Check for mentions of @reputest from the past hour and reply with good vibes count
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
/// The job runs every hour using the cron expression "0 0 * * * * *"
/// which means:
/// - 0 seconds
/// - 0 minutes
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
pub async fn start_gmgv_cronjob() -> Result<JobScheduler, Box<dyn std::error::Error + Send + Sync>>
{
    let sched = JobScheduler::new().await?;

    // Create a job that runs every hour
    sched
        .add(Job::new_async("0 0 * * * * *", |_uuid, _l| {
            Box::pin(async {
                // Task 1: Search for #gmgv tweets
                info!("Starting scheduled search for #gmgv tweets");
                match search_tweets_with_hashtag("gmgv").await {
                    Ok(_) => {
                        info!("Scheduled search for #gmgv tweets completed successfully");
                    }
                    Err(e) => {
                        error!("Scheduled search for #gmgv tweets failed: {}", e);
                    }
                }

                // Task 2: Check for mentions and reply with good vibes count
                info!("Starting scheduled check for @reputest mentions");
                match search_mentions().await {
                    Ok(mentions) => {
                        if mentions.is_empty() {
                            info!("No mentions found to reply to");
                        } else {
                            info!("Found {} mentions to reply to", mentions.len());

                            // Get the database pool for user lookups and vibe checks
                            let pool = match crate::db::get_db_pool().await {
                                Ok(pool) => pool,
                                Err(e) => {
                                    error!(
                                        "Failed to get database pool for mentions processing: {}",
                                        e
                                    );
                                    return;
                                }
                            };

                            // Reply to each mention
                            for (tweet_id, _tweet_text, author_username, mentioned_user) in mentions {
                                if let Some(mentioned_username) = mentioned_user {
                                    // This is a vibe score query - check if the author has good vibes from the mentioned user

                                    // First, get the user IDs for both the tweet author (sensor) and mentioned user (emitter)
                                    // Look up the mentioned user's ID from database (should already exist from previous searches)
                                    match get_user_id_by_username(&pool, &mentioned_username).await {
                                        Ok(Some(mentioned_user_id)) => {
                                            // Now check if there's a good vibes record between author (sensor) and mentioned user (emitter)
                                            // We need the author's user ID too
                                            let author_user_id = match crate::db::get_user_id_by_username(&pool, &author_username).await {
                                                Ok(Some(user_id)) => user_id,
                                                Ok(None) => {
                                                    error!("Could not find user ID for author @{}", author_username);
                                                    continue;
                                                }
                                                Err(e) => {
                                                    error!("Failed to get user ID for @{}: {}", author_username, e);
                                                    continue;
                                                }
                                            };

                                            // Check if there's a vibe record
                                            match has_good_vibes_record(&pool, &author_user_id, &mentioned_user_id).await {
                                                Ok(has_record) => {
                                                    let score = if has_record { 1 } else { 0 };
                                                    let reply_text = format!("Your vibe score for @{} is {}", mentioned_username, score);
                                                    info!("Replying to vibe query tweet {} with: {}", tweet_id, reply_text);

                                                    match reply_to_tweet(&reply_text, &tweet_id).await {
                                                        Ok(_) => {
                                                            info!("Successfully replied to vibe query from @{}", author_username);
                                                        }
                                                        Err(e) => {
                                                            error!("Failed to reply to vibe query from @{}: {}", author_username, e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Failed to check vibe record for @{} -> @{}: {}", author_username, mentioned_username, e);
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            error!("Could not find mentioned user @{}", mentioned_username);
                                            // Reply with score 0 since user doesn't exist
                                            let reply_text = format!("Your vibe score for @{} is 0", mentioned_username);
                                            info!("Replying to vibe query tweet {} with: {} (user not found)", tweet_id, reply_text);

                                            match reply_to_tweet(&reply_text, &tweet_id).await {
                                                Ok(_) => {
                                                    info!("Successfully replied to vibe query from @{} (user not found)", author_username);
                                                }
                                                Err(e) => {
                                                    error!("Failed to reply to vibe query from @{}: {}", author_username, e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to lookup mentioned user @{}: {}", mentioned_username, e);
                                        }
                                    }
                                } else {
                                    // This is just a regular mention without a vibe query - reply with total count
                                    match get_good_vibes_count(&pool).await {
                                        Ok(vibes_count) => {
                                            let reply_text = format!("Hello @{}! The current good vibes count is: {}", author_username, vibes_count);
                                            info!("Replying to general mention tweet {} with: {}", tweet_id, reply_text);

                                            match reply_to_tweet(&reply_text, &tweet_id).await {
                                                Ok(_) => {
                                                    info!("Successfully replied to general mention from @{}", author_username);
                                                }
                                                Err(e) => {
                                                    error!("Failed to reply to general mention from @{}: {}", author_username, e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to get good vibes count for general mention: {}", e);
                                        }
                                    }
                                }
                            }

                            info!("Scheduled check for mentions completed successfully");
                        }
                    }
                    Err(e) => {
                        error!("Scheduled check for mentions failed: {}", e);
                    }
                }
            })
        })?)
        .await?;

    info!("Cronjob scheduler configured to search for #gmgv tweets and check mentions every hour");
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
