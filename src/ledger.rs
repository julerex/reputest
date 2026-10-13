//! Megajoule ledger backed by a TigerBeetle cluster.
//!
//! Twitter user ids are account ids. Tweet ids are transfer ids, so replaying a
//! mention is idempotent. Ledger 1 and code 1 mark megajoules. A transfer debits
//! the sender and credits the receiver. Balance is credits posted minus debits
//! posted.

use log::info;
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::OnceLock;
use tigerbeetle as tb;

/// Megajoule ledger number in TigerBeetle.
pub const MEGAJOULE_LEDGER: u32 = 1;
/// Megajoule transfer and account code.
pub const MEGAJOULE_CODE: u16 = 1;

const CLUSTER_ID_ENV: &str = "TB_CLUSTER_ID";
const ADDRESS_ENV: &str = "TB_ADDRESS";

static CLIENT: OnceLock<tb::Client> = OnceLock::new();

/// Outcome of recording a megajoule transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOutcome {
    /// The transfer was written for the first time.
    Created,
    /// A transfer with this tweet id already exists.
    AlreadyExists,
}

/// Connects to the TigerBeetle cluster from `TB_CLUSTER_ID` and `TB_ADDRESS`.
///
/// Cluster id 0 is reserved for tests and is rejected. Hostnames in
/// `TB_ADDRESS` are resolved to IP addresses before connecting.
pub fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cluster_id = cluster_id_from_env()?;
    let raw_address = env::var(ADDRESS_ENV)
        .map_err(|_| format!("{ADDRESS_ENV} is not set (host:port, IP:port, or a port number)"))?;
    let address = resolve_tb_address(&raw_address)?;
    info!(
        "Connecting to TigerBeetle cluster {} at {}",
        cluster_id, address
    );
    let client = tb::Client::new(cluster_id, &address)
        .map_err(|status| format!("TigerBeetle client init failed: {status:?}"))?;
    CLIENT
        .set(client)
        .map_err(|_| "TigerBeetle client was already initialized".to_string())?;
    Ok(())
}

/// Returns the process-wide TigerBeetle client.
pub fn client() -> Result<&'static tb::Client, Box<dyn std::error::Error + Send + Sync>> {
    CLIENT
        .get()
        .ok_or_else(|| "TigerBeetle client is not initialized".into())
}

/// Parses `TB_CLUSTER_ID`, rejecting the test-reserved id 0.
pub fn cluster_id_from_env() -> Result<u128, Box<dyn std::error::Error + Send + Sync>> {
    let raw = env::var(CLUSTER_ID_ENV)
        .map_err(|_| format!("{CLUSTER_ID_ENV} is not set (128-bit decimal integer)"))?;
    let cluster_id = parse_cluster_id(&raw)?;
    if cluster_id == 0 {
        return Err(format!("{CLUSTER_ID_ENV} 0 is reserved for tests").into());
    }
    Ok(cluster_id)
}

/// Parses a 128-bit decimal cluster id.
pub fn parse_cluster_id(value: &str) -> Result<u128, Box<dyn std::error::Error + Send + Sync>> {
    value
        .parse::<u128>()
        .map_err(|e| format!("invalid cluster id '{value}': {e}").into())
}

/// Parses a Twitter snowflake id into a TigerBeetle account or transfer id.
pub fn parse_snowflake_id(value: &str) -> Result<u128, Box<dyn std::error::Error + Send + Sync>> {
    let id = value
        .parse::<u128>()
        .map_err(|e| format!("invalid snowflake id '{value}': {e}"))?;
    if id == 0 {
        return Err("snowflake id must not be zero".into());
    }
    Ok(id)
}

/// Posted megajoule balance: credits minus debits.
///
/// The result saturates at the i64 bounds. A missing account is represented by
/// zeros and therefore a balance of 0.
pub fn account_balance_i64(credits_posted: u128, debits_posted: u128) -> i64 {
    let credits = i128::try_from(credits_posted).unwrap_or(i128::MAX);
    let debits = i128::try_from(debits_posted).unwrap_or(i128::MAX);
    (credits - debits).clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

/// Rejects amount 0 and transfers to oneself, which TigerBeetle will not store.
pub fn validate_transfer(
    sender: u128,
    receiver: u128,
    amount: u128,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if amount == 0 {
        return Err("megajoule amount must not be zero".into());
    }
    if sender == receiver {
        return Err("sender and receiver must be different".into());
    }
    Ok(())
}

/// True when TigerBeetle can use `address` without DNS (port-only, IP, or IP:port).
pub fn is_literal_tb_address(address: &str) -> bool {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    trimmed.parse::<SocketAddr>().is_ok() || trimmed.parse::<std::net::IpAddr>().is_ok()
}

/// Resolves a TigerBeetle address to `host:port` or `[ipv6]:port`.
pub fn resolve_tb_address(
    address: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let trimmed = address.trim();
    if is_literal_tb_address(trimmed) {
        return Ok(trimmed.to_string());
    }
    let resolved = trimmed
        .to_socket_addrs()
        .map_err(|e| format!("failed to resolve TigerBeetle address '{trimmed}': {e}"))?
        .next()
        .ok_or_else(|| format!("no addresses resolved for '{trimmed}'"))?;
    Ok(resolved.to_string())
}

/// Ensures sender and receiver accounts exist, then records the transfer.
pub async fn record_transfer(
    sender_id: u128,
    receiver_id: u128,
    amount: u128,
    transfer_id: u128,
) -> Result<TransferOutcome, Box<dyn std::error::Error + Send + Sync>> {
    validate_transfer(sender_id, receiver_id, amount)?;
    ensure_accounts(&[sender_id, receiver_id]).await?;
    let client = client()?;
    let results = client
        .create_transfers(&[tb::Transfer {
            id: transfer_id,
            debit_account_id: sender_id,
            credit_account_id: receiver_id,
            amount,
            ledger: MEGAJOULE_LEDGER,
            code: MEGAJOULE_CODE,
            ..Default::default()
        }])
        .map_err(|e| format!("TigerBeetle create_transfers closed: {e:?}"))?
        .await
        .map_err(|e| format!("TigerBeetle create_transfers failed: {e:?}"))?;

    match results.first().map(|r| r.status) {
        None | Some(tb::CreateTransferStatus::Created) => Ok(TransferOutcome::Created),
        Some(tb::CreateTransferStatus::Exists) => Ok(TransferOutcome::AlreadyExists),
        Some(status) => Err(format!("TigerBeetle transfer {transfer_id} failed: {status}").into()),
    }
}

/// Looks up a user's posted megajoule balance. Missing accounts are 0.
pub async fn get_balance(user_id: u128) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let client = client()?;
    let accounts = client
        .lookup_accounts(&[user_id])
        .map_err(|e| format!("TigerBeetle lookup_accounts closed: {e:?}"))?
        .await
        .map_err(|e| format!("TigerBeetle lookup_accounts failed: {e:?}"))?;
    let Some(account) = accounts.first() else {
        return Ok(0);
    };
    Ok(account_balance_i64(
        account.credits_posted,
        account.debits_posted,
    ))
}

async fn ensure_accounts(ids: &[u128]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut unique = Vec::new();
    for id in ids {
        if !unique.contains(id) {
            unique.push(*id);
        }
    }
    let accounts: Vec<tb::Account> = unique
        .into_iter()
        .map(|id| tb::Account {
            id,
            ledger: MEGAJOULE_LEDGER,
            code: MEGAJOULE_CODE,
            ..Default::default()
        })
        .collect();
    let client = client()?;
    let results = client
        .create_accounts(&accounts)
        .map_err(|e| format!("TigerBeetle create_accounts closed: {e:?}"))?
        .await
        .map_err(|e| format!("TigerBeetle create_accounts failed: {e:?}"))?;
    for result in results {
        match result.status {
            tb::CreateAccountStatus::Created | tb::CreateAccountStatus::Exists => {}
            status => {
                return Err(format!("TigerBeetle create_accounts failed: {status}").into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn parse_snowflake_id_accepts_twitter_user_id() {
        assert_eq!(parse_snowflake_id("2244994945").unwrap(), 2244994945);
    }

    #[test]
    fn parse_snowflake_id_accepts_tweet_id() {
        assert_eq!(
            parse_snowflake_id("1848339196650332633").unwrap(),
            1_848_339_196_650_332_633
        );
    }

    #[test]
    fn parse_snowflake_id_rejects_non_numeric() {
        assert!(parse_snowflake_id("alice").is_err());
    }

    #[test]
    fn parse_snowflake_id_rejects_zero() {
        assert!(parse_snowflake_id("0").is_err());
    }

    #[test]
    fn parse_cluster_id_accepts_decimal() {
        assert_eq!(parse_cluster_id("1").unwrap(), 1);
    }

    #[test]
    fn account_balance_i64_subtracts_debits_from_credits() {
        assert_eq!(account_balance_i64(50, 20), 30);
    }

    #[test]
    fn account_balance_i64_can_be_negative() {
        assert_eq!(account_balance_i64(10, 40), -30);
    }

    #[test]
    fn account_balance_i64_is_zero_when_account_is_empty() {
        assert_eq!(account_balance_i64(0, 0), 0);
    }

    #[test]
    fn validate_transfer_rejects_zero_amount() {
        assert!(validate_transfer(1, 2, 0).is_err());
    }

    #[test]
    fn validate_transfer_rejects_self_transfer() {
        assert!(validate_transfer(7, 7, 10).is_err());
    }

    #[test]
    fn validate_transfer_accepts_distinct_accounts() {
        assert!(validate_transfer(1, 2, 10).is_ok());
    }

    #[test]
    fn is_literal_tb_address_accepts_port_only() {
        assert!(is_literal_tb_address("3000"));
    }

    #[test]
    fn is_literal_tb_address_accepts_ipv4() {
        assert!(is_literal_tb_address("127.0.0.1:3000"));
    }

    #[test]
    fn is_literal_tb_address_accepts_ipv6() {
        assert!(is_literal_tb_address("[::1]:3000"));
    }

    #[test]
    fn is_literal_tb_address_rejects_hostname() {
        assert!(!is_literal_tb_address("reputest-tigerbeetle.internal:3000"));
    }

    #[test]
    fn resolve_tb_address_leaves_literals_unchanged() {
        assert_eq!(resolve_tb_address("3000").unwrap(), "3000");
        assert_eq!(
            resolve_tb_address("127.0.0.1:3000").unwrap(),
            "127.0.0.1:3000"
        );
    }

    #[test]
    fn socket_addr_display_wraps_ipv6_in_brackets() {
        let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3000);
        assert_eq!(addr.to_string(), "[::1]:3000");
        let v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000);
        assert_eq!(v4.to_string(), "127.0.0.1:3000");
    }
}
