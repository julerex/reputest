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
pub use parsing::{extract_mention_with_following, extract_mention_with_question};
pub use search::{search_mentions, search_tweets_with_hashtag};
pub use tweets::reply_to_tweet;

// Crate-internal re-exports (used by tests and other modules)
#[allow(unused_imports)]
pub(crate) use api::{lookup_user_by_username, sanitize_for_logging};
#[allow(unused_imports)]
pub(crate) use following::fetch_user_following;
#[allow(unused_imports)]
pub(crate) use parsing::extract_vibe_emitter;
