//! Twitter/X API integration module.
//!
//! This module contains functions for interacting with the Twitter/X API,
//! including posting tweets, searching, and direct messages using OAuth 2.0
//! User Context authentication.

mod api;
mod following;
mod parsing;
mod search;
mod tweets;

// Re-export public API
#[allow(unused_imports)]
pub use parsing::{
    extract_mention_with_balance, extract_mention_with_following, extract_mention_with_question,
};
#[allow(unused_imports)]
pub use search::{search_mentions, MentionedTweet};

// Crate-internal re-exports (used by tests and other modules)
#[allow(unused_imports)]
pub(crate) use api::{lookup_user_by_username, sanitize_for_logging};
#[allow(unused_imports)]
pub(crate) use following::fetch_user_following;
#[allow(unused_imports)]
pub(crate) use parsing::{
    extract_megajoule_transfer, extract_vibe_emitter, tweet_text_mentions_reputest,
};
#[allow(unused_imports)]
pub(crate) use search::{record_good_vibes, record_megajoule_transfer};
pub use tweets::reply_to_tweet;
