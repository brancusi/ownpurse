//! Ethereum JSON-RPC transactions and receipts for one wallet (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct Eth;

impl Projector for Eth {
    fn id(&self) -> &'static str {
        "eth.rpc@1"
    }

    fn reads(&self) -> &'static str {
        "Ethereum JSON-RPC transactions and receipts for one wallet"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("eth.rpc@1 is not built yet".into()))
    }
}
