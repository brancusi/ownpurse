//! The API layer: the token store shared by every process, the token-endpoint client, and (later steps)
//! login, HTTP with rate limits, and sync. Rules:
//! - the only non-GET requests are the OAuth token endpoint's POST and the Phase 8 write transport
//!   (`write::transport`: four allow-listed shapes, behind the profile, tenant and approval locks);
//! - login requests the read scopes plus any scope already granted, never a new write scope;
//! - tests never reach the network: every URL is injected and loopback.
//!
//! The TUI never depends on this crate.

pub mod attachments;
pub mod auth;
pub mod error;
pub mod form;
pub mod history;
pub mod http;
pub mod reports;
pub mod store;
pub mod sync;
pub mod tokens;
pub mod write;

pub use error::ApiError;
pub use tokens::TokenStore;
