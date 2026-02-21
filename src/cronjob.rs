//! Cronjob module for scheduled tasks.
//!
//! This module contains functionality for running scheduled tasks, specifically
//! for searching Twitter for tweets with specific hashtags and processing vibe-related queries.

use crate::config::TwitterConfig;
use crate::db::{
    get_good_vibes_count, get_user_id_by_username, get_vibe_score_five, get_vibe_score_four,
    get_vibe_score_one, get_vibe_score_six, get_vibe_score_three, get_vibe_score_two,
    has_vibe_request, increment_follower_count, refresh_materialized_views, save_following,
    save_user, save_vibe_request,
};
use crate::twitter::lookup_user_by_username;
use crate::twitter::{
    extract_mention_with_following, fetch_user_following, reply_to_tweet, sanitize_for_logging,
    search_mentions, search_tweets_with_hashtag,
};
use log::{debug, error, info};
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Starts the cronjob scheduler for searching tweets with hashtag "gmgv" and processing vibe queries every 5 minutes.
///
/// This function creates a new job scheduler and adds a job that runs every 5 minutes
/// to perform three tasks:
/// 1. Search for tweets containing the hashtag "gmgv" from the past 24 hours
/// 2. Check for mentions of @reputest from the past 24 hours and reply to:
///    - Specific vibe score queries (e.g., "@reputest @username?")
///    - General requests for the total vibes count (messages containing "vibecount")
/// 3. Refresh all materialized views (degree 1-4 and combined view) and record timing metrics
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

/// Processes the scheduled search for #megajoules tweets
async fn process_megajoule_search() {
    info!("Starting scheduled search for #megajoules tweets");
    match search_tweets_with_hashtag("megajoules").await {
        Ok(_) => {
            info!("Scheduled search for #megajoules tweets completed successfully");
        }
        Err(e) => {
            error!("Scheduled search for #megajoules tweets failed: {}", e);
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

            // Twitter config for looking up users when the query author is not yet in the DB
            let mut config = match TwitterConfig::from_env(&pool).await {
                Ok(c) => c,
                Err(e) => {
                    error!(
                        "Failed to load Twitter config for mentions processing: {}",
                        e
                    );
                    return;
                }
            };

            // Process each mention (following query takes precedence over vibe query)
            for (tweet_id, tweet_text, author_username, mentioned_user, created_at) in mentions {
                if let Some(mentioned_username) = extract_mention_with_following(&tweet_text) {
                    process_following_query(
                        &pool,
                        &mut config,
                        &tweet_id,
                        &tweet_text,
                        &author_username,
                        &mentioned_username,
                        &created_at,
                    )
                    .await;
                } else if let Some(mentioned_username) = mentioned_user {
                    process_vibe_query(
                        &pool,
                        &mut config,
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
                    info!("Skipping general mention from @{} at {} - no vibecount request or specific vibe/following query", author_username, created_at);
                }
            }

            info!("Scheduled check for mentions completed successfully");
        }
        Err(e) => {
            error!("Scheduled check for mentions failed: {}", e);
        }
    }
}

/// Processes materialized view refresh as the last step of the cronjob
async fn process_materialized_view_refresh() {
    info!("Starting materialized view refresh");
    let pool = match crate::db::get_db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            error!(
                "Failed to get database pool for materialized view refresh: {}",
                e
            );
            return;
        }
    };

    match refresh_materialized_views(&pool).await {
        Ok(_) => {
            info!("Materialized view refresh completed successfully");
        }
        Err(e) => {
            error!("Materialized view refresh failed: {}", e);
        }
    }
}

/// Processes a specific vibe score query (e.g., "@reputest @username?")
async fn process_vibe_query(
    pool: &PgPool,
    config: &mut TwitterConfig,
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

    // Get the author's user ID first; if not in DB, look up via API, add them, then reply that they have no good vibes yet
    let author_user_id = match crate::db::get_user_id_by_username(pool, author_username).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            if let Ok(Some((user_id, name, created_at_utc, follower_count))) =
                lookup_user_by_username(config, pool, author_username).await
            {
                if let Err(e) =
                    save_user(pool, &user_id, author_username, &name, created_at_utc, follower_count)
                        .await
                {
                    error!(
                        "Failed to save author @{} to users table: {}",
                        author_username, e
                    );
                }
            }
            reply_author_no_good_vibes(pool, tweet_id, author_username).await;
            return;
        }
        Err(e) => {
            error!("Failed to get user ID for @{}: {}", author_username, e);
            return;
        }
    };

    // Look up the mentioned user's ID from database; if not in DB, reply with same format and all zeros (no path)
    let mentioned_user_id = match get_user_id_by_username(pool, mentioned_username).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            info!(
                "Mentioned user @{} not in database, replying with all-zero vibe scores",
                mentioned_username
            );
            let reply_text = format!(
                "Your vibes for {} are:\n1st degree: 0\n2nd degree: 0\n3rd degree: 0\n4th degree: 0\n5th degree: 0\n6th degree: 0",
                mentioned_username
            );
            send_reply_and_mark_processed(pool, &reply_text, tweet_id, author_username).await;
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

    // Calculate the vibe scores (degrees 1, 2, 3, 4, 5, and 6)
    match tokio::try_join!(
        get_vibe_score_one(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_two(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_three(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_four(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_five(pool, &author_user_id, &mentioned_user_id),
        get_vibe_score_six(pool, &author_user_id, &mentioned_user_id)
    ) {
        Ok((score_one, score_two, score_three, score_four, score_five, score_six)) => {
            let reply_text = format!(
                "Your vibes for {} are:\n1st degree: {}\n2nd degree: {}\n3rd degree: {}\n4th degree: {}\n5th degree: {}\n6th degree: {}",
                mentioned_username, score_one, score_two, score_three, score_four, score_five, score_six
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

/// Replies when the query author is not in the good vibes graph (no #gmgv declarations).
async fn reply_author_no_good_vibes(pool: &PgPool, tweet_id: &str, author_username: &str) {
    info!(
        "Author @{} not in good vibes graph, replying with 'no good vibes yet'",
        author_username
    );
    let reply_text = "You have not declared any good vibes yet.";

    match reply_to_tweet(reply_text, tweet_id).await {
        Ok(_) => {
            info!(
                "Successfully replied to vibe query from @{} (author has no good vibes)",
                author_username
            );
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

/// Processes a following query (e.g., "@reputest @username following?")
async fn process_following_query(
    pool: &PgPool,
    config: &mut TwitterConfig,
    tweet_id: &str,
    _tweet_text: &str,
    author_username: &str,
    mentioned_username: &str,
    _created_at: &str,
) {
    match has_vibe_request(pool, tweet_id).await {
        Ok(true) => {
            info!(
                "Skipping following query tweet {} from @{} for @{} - already processed",
                tweet_id, author_username, mentioned_username
            );
            return;
        }
        Ok(false) => {}
        Err(e) => {
            error!(
                "Failed to check if tweet {} has been processed: {}",
                tweet_id, e
            );
            return;
        }
    }

    // Resolve username -> user_id (DB or API)
    let follower_user_id = match get_user_id_by_username(pool, mentioned_username).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            if let Ok(Some((user_id, name, created_at_utc, follower_count))) =
                lookup_user_by_username(config, pool, mentioned_username).await
            {
                if let Err(e) =
                    save_user(pool, &user_id, mentioned_username, &name, created_at_utc, follower_count)
                        .await
                {
                    error!(
                        "Failed to save @{} to users table: {}",
                        mentioned_username, e
                    );
                }
                user_id
            } else {
                info!(
                    "User @{} not found, skipping following query",
                    mentioned_username
                );
                let reply_text = format!("User @{} not found.", mentioned_username);
                send_reply_and_mark_processed(pool, &reply_text, tweet_id, author_username).await;
                return;
            }
        }
        Err(e) => {
            error!("Failed to get user ID for @{}: {}", mentioned_username, e);
            return;
        }
    };

    // Fetch following list via API
    let followed_users = match fetch_user_following(config, pool, &follower_user_id).await {
        Ok(users) => users,
        Err(e) => {
            error!(
                "Failed to fetch following list for @{}: {}",
                mentioned_username, e
            );
            let reply_text = format!(
                "Could not fetch @{}'s following list (account may be protected or suspended).",
                mentioned_username
            );
            send_reply_and_mark_processed(pool, &reply_text, tweet_id, author_username).await;
            return;
        }
    };

    let now = chrono::Utc::now();
    let mut new_count = 0u32;

    for followed in &followed_users {
        if followed.id == follower_user_id {
            continue; // Skip self-follow
        }
        match save_following(pool, &follower_user_id, &followed.id, now).await {
            Ok(inserted) => {
                if inserted {
                    new_count += 1;
                    if let Err(e) = increment_follower_count(pool, &followed.id).await {
                        error!(
                            "Failed to increment follower_count for {}: {}",
                            followed.id, e
                        );
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to save following {} -> {}: {}",
                    follower_user_id, followed.id, e
                );
            }
        }
    }

    let reply_text = format!(
        "Fetched {} accounts @{} follows. {} new relationships stored.",
        followed_users.len(),
        mentioned_username,
        new_count
    );
    send_reply_and_mark_processed(pool, &reply_text, tweet_id, author_username).await;
}

/// Sends a reply to a tweet and marks it as processed
async fn send_reply_and_mark_processed(
    pool: &PgPool,
    reply_text: &str,
    tweet_id: &str,
    author_username: &str,
) {
    // SECURITY: Sanitize text before logging to prevent log injection
    info!(
        "Replying to tweet {} with: {}",
        tweet_id,
        sanitize_for_logging(reply_text, 150)
    );

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
                process_hashtag_search().await; // #gmgv
                process_megajoule_search().await; // #megajoules
                process_mentions().await;
                process_materialized_view_refresh().await;
            })
        })?)
        .await?;

    info!("Cronjob scheduler configured to search for #gmgv and #megajoules tweets, process vibe queries, and refresh materialized views every 5 minutes");
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
