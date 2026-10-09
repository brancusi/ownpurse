//! ownpurse core: the local record (read-only), exact-decimal money, the ledger rebuilt from documents, trial
//! balances and bases, the intercompany engine and the side-by-side compare. No `f64` reaches an amount
//! (invariant V11; `clippy::float_arithmetic` is denied workspace-wide).

pub mod balances;

pub mod bridge;
pub mod canon;
pub mod classify;
pub mod compare;
pub mod config;
pub mod dates;
pub mod deeplink;
pub mod estimate;
pub mod glossary;
pub mod intercompany;
pub mod ledger;
pub mod links;
pub mod manifest;
pub mod money;
pub mod raster;
pub mod record;
pub mod statements;
pub mod text;
pub mod view;
pub mod viz;
pub mod walkthrough;
pub mod write;

pub use balances::{
    Balances, Basis, BasisKind, Diff, Mode, compare, resolve, trial_balance, xero_tb_balances,
};
pub use ledger::{Account, Chart, Documents, Ledger, LedgerBasis, Posting, Unbalanced};
pub use money::{dr_cr, fmt, money_str};
pub use record::Record;
pub use rust_decimal::Decimal;
