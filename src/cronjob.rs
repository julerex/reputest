//! Cronjob module for scheduled tasks.
//!
//! This module contains functionality for running scheduled tasks, specifically
//! for searching Twitter for tweets with specific hashtags and checking for mentions.

use crate::db::get_good_vibes_count;
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

                            // Get the current good vibes count
                            let pool = match crate::db::get_db_pool().await {
                                Ok(pool) => pool,
                                Err(e) => {
                                    error!(
                                        "Failed to get database pool for good vibes count: {}",
                                        e
                                    );
                                    return;
                                }
                            };

                            let vibes_count = match get_good_vibes_count(&pool).await {
                                Ok(count) => count,
                                Err(e) => {
                                    error!("Failed to get good vibes count: {}", e);
                                    return;
                                }
                            };

                            // Reply to each mention
                            for (tweet_id, _tweet_text, author_username) in mentions {
                                let reply_text = format!(
                                    "Hello @{}! The current good vibes count is: {}",
                                    author_username, vibes_count
                                );
                                info!("Replying to tweet {} with: {}", tweet_id, reply_text);

                                match reply_to_tweet(&reply_text, &tweet_id).await {
                                    Ok(_) => {
                                        info!(
                                            "Successfully replied to mention from @{}",
                                            author_username
                                        );
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to reply to mention from @{}: {}",
                                            author_username, e
                                        );
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
