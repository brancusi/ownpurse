//! QuickBooks Online API query responses (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct Qbo;

impl Projector for Qbo {
    fn id(&self) -> &'static str {
        "qbo.json@1"
    }

    fn reads(&self) -> &'static str {
        "QuickBooks Online API query responses"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("qbo.json@1 is not built yet".into()))
    }
}
