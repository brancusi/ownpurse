//! Phase 8, the run side of the write path (CLI only; the TUI never links this crate): the guarded write
//! transport, the attempt log and approvals in the record, run / verify / reconcile / recover, the loopback
//! fake Xero server the tests use, and the demo probe kit. The shared types are `ownpurse_core::write`.

pub mod fake;
pub mod log;
pub mod run;
pub mod transport;

pub use log::{OpLog, RecordApprovals, op_key};
pub use transport::{UreqWire, Wire, WireError, WriteSettings, XeroWriter};
