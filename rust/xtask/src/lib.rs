//! Repo tasks (`cargo run -p xtask -- <task>`).

/// The demo and engine fixture generator (`xtask demo fixtures | states | check`).
pub mod demo;

/// The live agent-drive suite (`xtask agent-drive`).
pub mod live;
