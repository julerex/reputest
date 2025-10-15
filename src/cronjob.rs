//! Cronjob module for scheduled tasks.
//!
//! This module contains functionality for running scheduled tasks, specifically
//! for searching Twitter for tweets with specific hashtags at regular intervals.

use crate::twitter::search_tweets_with_hashtag;
use log::{error, info};
use tokio_cron_scheduler::{Job, JobScheduler};

/// Starts the cronjob scheduler for searching tweets with hashtag "gmgv" every 10 minutes.
///
/// This function creates a new job scheduler and adds a job that runs every 10 minutes
/// to search for tweets containing the hashtag "gmgv" from the past hour. The job
/// will log all found tweets to the application logs.
///
/// # Returns
///
/// - `Ok(JobScheduler)`: The configured job scheduler
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If there's an error creating or configuring the scheduler
///
/// # Job Schedule
///
/// The job runs every 10 minutes using the cron expression "0 */10 * * * * *"
/// which means:
/// - 0 seconds
/// - Every 10 minutes
/// - Every hour
/// - Every day
/// - Every month
/// - Every day of the week
///
/// # Example
///
/// ```rust
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

    // Create a job that runs every 10 minutes
    sched
        .add(Job::new_async("0 */10 * * * * *", |_uuid, _l| {
            Box::pin(async {
                info!("Starting scheduled search for #gmgv tweets");
                match search_tweets_with_hashtag("gmgv").await {
                    Ok(_) => {
                        info!("Scheduled search for #gmgv tweets completed successfully");
                    }
                    Err(e) => {
                        error!("Scheduled search for #gmgv tweets failed: {}", e);
                    }
                }
            })
        })?)
        .await?;

    info!("Cronjob scheduler configured to search for #gmgv tweets every 10 minutes");
    Ok(sched)
}

/// Starts the cronjob scheduler and keeps it running.
///
/// This is a convenience function that starts the GMGV hashtag search cronjob
/// and keeps the scheduler running indefinitely. It handles graceful shutdown
/// when receiving a Ctrl+C signal.
///
/// # Returns
///
/// - `Ok(())`: If the scheduler runs successfully until shutdown
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If there's an error starting or running the scheduler
///
/// # Example
///
/// ```rust
/// use reputest::run_gmgv_cronjob;
///
/// #[tokio::main]
/// async fn main() {
///     if let Err(e) = run_gmgv_cronjob().await {
///         eprintln!("Cronjob failed: {}", e);
///     }
/// }
/// ```
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
