//! ownpurse TUI. Read-only by construction: this crate depends only on ownpurse-core (V1), never on a
//! write or API crate. Sync and login run the installed `ownpurse` CLI as a child process after a
//! confirm (spec P6 network); the other network actions keep their placeholders. Nothing here reaches
//! Xero itself.

pub mod activity;
pub mod agent;
pub mod anchors;
pub mod app;
pub mod books;
pub mod bridge;
pub mod canonical;
pub mod classify;
pub mod clock;
pub mod compare;
pub mod external;
pub mod filter;
pub mod glyphs;
pub mod history;
pub mod hit;
pub mod ic_nav;
pub mod keymap;
pub mod keymap_table;
pub mod layout;
pub mod ledger_nav;
pub mod ledgers;
pub mod manifest;
pub mod manifest_tab;
pub mod model;
pub mod net;
pub mod overlay;
pub mod probe;
pub mod project_config;
pub mod quiet;
pub mod render;
pub mod run;
pub mod runtime;
pub mod snapshot;
pub mod state;
pub mod system;
pub mod theme;
/// The token table (moved into `theme/`; this path is kept for the renderers).
pub use theme::tokens as theme_tokens;
pub mod ui;
pub mod upgrade;
pub mod user_config;
pub mod views;
pub mod walk;
pub mod widgets;
