//! Cronjob module for scheduled tasks.
//!
//! This module searches Twitter for @reputest mentions and processes vibe-related
//! commands, megajoule transfers, and good-vibes attestations.

use crate::config::TwitterConfig;
use crate::db::{
    get_good_vibes_count, get_user_id_by_username, get_vibe_score_five, get_vibe_score_four,
    get_vibe_score_one, get_vibe_score_six, get_vibe_score_three, get_vibe_score_two,
    has_vibe_request, increment_follower_count, refresh_materialized_views, save_following,
    save_user, save_vibe_request,
};
use crate::twitter::lookup_user_by_username;
use crate::twitter::{
    extract_megajoule_transfer, extract_mention_with_following, extract_mention_with_question,
    extract_vibe_emitter, fetch_user_following, record_good_vibes, record_megajoule_transfer,
    reply_to_tweet, sanitize_for_logging, search_mentions, tweet_text_mentions_reputest,
};
use log::{debug, error, info};
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Starts the cronjob scheduler for @reputest mentions every 30 seconds.
///
/// Each tick:
/// 1. Searches for mentions of @reputest from the past 24 hours and handles:
///    - Following queries (`@reputest @username following?`)
///    - Megajoule transfers (`@reputest 20 megajoules to @alice`, `#` optional)
///    - Good vibes (`@reputest @alice gmgv`, `#` optional)
///    - Vibe score queries (`@reputest @username?`)
///    - Total vibes count (messages containing `vibecount`)
/// 2. Refreshes all materialized views and records timing metrics
///
/// # Returns
///
/// - `Ok(JobScheduler)`: The configured job scheduler
/// - `Err(Box<dyn std::error::Error + Send + Sync>)`: If there's an error creating or configuring the scheduler
///
/// # Job Schedule
///
/// The job runs every 30 seconds using the cron expression "0/30 * * * * * *"
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
/// Processes scheduled checks for @reputest mentions
async fn process_mentions() {
    debug!("Starting scheduled check for @reputest mentions");
    match search_mentions().await {
        Ok(mentions) => {
            if mentions.is_empty() {
                info!("No mentions found to reply to");
                return;
            }

            info!("Found {} mentions to reply to", mentions.len());

            let pool = match crate::db::get_db_pool().await {
                Ok(pool) => pool,
                Err(e) => {
                    error!("Failed to get database pool for mentions processing: {}", e);
                    return;
                }
            };

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

            for mention in mentions {
                if !tweet_text_mentions_reputest(&mention.tweet_text) {
                    info!(
                        "Skipping tweet {} from @{} - text does not mention @reputest",
                        mention.tweet_id, mention.author_username
                    );
                    continue;
                }

                let created_at = mention.created_at.to_rfc3339();
                if let Some(mentioned_username) =
                    extract_mention_with_following(&mention.tweet_text)
                {
                    process_following_query(
                        &pool,
                        &mut config,
                        &mention.tweet_id,
                        &mention.tweet_text,
                        &mention.author_username,
                        &mentioned_username,
                        &created_at,
                    )
                    .await;
                } else if let Some((amount, receiver_username)) =
                    extract_megajoule_transfer(&mention.tweet_text)
                {
                    record_megajoule_transfer(
                        &pool,
                        &mut config,
                        &mention,
                        amount,
                        &receiver_username,
                    )
                    .await;
                } else if let Some(vibe_emitter_username) = extract_vibe_emitter(
                    &mention.tweet_text,
                    mention.reply_target_username.as_deref(),
                ) {
                    record_good_vibes(&pool, &mut config, &mention, &vibe_emitter_username).await;
                } else if let Some(mentioned_username) =
                    extract_mention_with_question(&mention.tweet_text)
                {
                    process_vibe_query(
                        &pool,
                        &mut config,
                        &mention.tweet_id,
                        &mention.tweet_text,
                        &mention.author_username,
                        &mentioned_username,
                        &created_at,
                    )
                    .await;
                } else if mention.tweet_text.to_lowercase().contains("vibecount") {
                    process_vibecount_request(
                        &pool,
                        &mention.tweet_id,
                        &mention.tweet_text,
                        &mention.author_username,
                        &created_at,
                    )
                    .await;
                } else {
                    info!(
                        "Skipping general mention from @{} at {} - no vibecount request or specific vibe/following query",
                        mention.author_username, created_at
                    );
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
                if let Err(e) = save_user(
                    pool,
                    &user_id,
                    author_username,
                    &name,
                    created_at_utc,
                    follower_count,
                )
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
                if let Err(e) = save_user(
                    pool,
                    &user_id,
                    mentioned_username,
                    &name,
                    created_at_utc,
                    follower_count,
                )
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

    // Create a job that runs every 30 seconds
    sched
        .add(Job::new_async("0/30 * * * * * *", |_uuid, _l| {
            Box::pin(async {
                process_mentions().await;
                process_materialized_view_refresh().await;
            })
        })?)
        .await?;

    info!("Cronjob scheduler configured to search @reputest mentions, process commands, and refresh materialized views every 30 seconds");
    Ok(sched)
}

/// Starts the cronjob scheduler and keeps it running.
///
/// This is a convenience function that starts the @reputest mention cronjob and keeps
/// the scheduler running indefinitely. It handles graceful shutdown when receiving a
/// Ctrl+C signal.
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
