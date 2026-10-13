//! Copies existing PostgreSQL megajoule rows into TigerBeetle.
//!
//! Transfer ids are tweet ids, so a second run is a no-op for rows already
//! stored. Requires DATABASE_URL, TB_CLUSTER_ID, and TB_ADDRESS.

use log::{error, info, warn};
use reputest::db;
use reputest::ledger::{self, TransferOutcome};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    ledger::init()?;
    let pool = db::get_db_pool().await?;
    let rows = db::list_megajoule_transfers(&pool).await?;
    info!("Importing {} megajoule rows into TigerBeetle", rows.len());

    let mut created = 0usize;
    let mut already_existed = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for row in rows {
        if row.amount <= 0 {
            warn!(
                "Skipping tweet {} amount {} (must be positive)",
                row.tweet_id, row.amount
            );
            skipped += 1;
            continue;
        }

        let sender_id = match ledger::parse_snowflake_id(&row.sender_id) {
            Ok(id) => id,
            Err(e) => {
                warn!("Skipping tweet {}: invalid sender id: {}", row.tweet_id, e);
                skipped += 1;
                continue;
            }
        };
        let receiver_id = match ledger::parse_snowflake_id(&row.receiver_id) {
            Ok(id) => id,
            Err(e) => {
                warn!(
                    "Skipping tweet {}: invalid receiver id: {}",
                    row.tweet_id, e
                );
                skipped += 1;
                continue;
            }
        };
        let transfer_id = match ledger::parse_snowflake_id(&row.tweet_id) {
            Ok(id) => id,
            Err(e) => {
                warn!("Skipping tweet {}: invalid tweet id: {}", row.tweet_id, e);
                skipped += 1;
                continue;
            }
        };

        match ledger::record_transfer(sender_id, receiver_id, row.amount as u128, transfer_id).await
        {
            Ok(TransferOutcome::Created) => {
                info!(
                    "Imported tweet {} {} -> {} amount {}",
                    row.tweet_id, row.sender_id, row.receiver_id, row.amount
                );
                created += 1;
            }
            Ok(TransferOutcome::AlreadyExists) => {
                already_existed += 1;
            }
            Err(e) => {
                error!("Failed to import tweet {}: {}", row.tweet_id, e);
                failed += 1;
            }
        }
    }

    info!(
        "Megajoule import finished: created={} already_existed={} skipped={} failed={}",
        created, already_existed, skipped, failed
    );

    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
