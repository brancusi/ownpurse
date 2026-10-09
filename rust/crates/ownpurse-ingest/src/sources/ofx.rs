//! OFX / QFX bank statements (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct Ofx;

impl Projector for Ofx {
    fn id(&self) -> &'static str {
        "bank.ofx@1"
    }

    fn reads(&self) -> &'static str {
        "OFX / QFX bank statements"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("bank.ofx@1 is not built yet".into()))
    }
}
