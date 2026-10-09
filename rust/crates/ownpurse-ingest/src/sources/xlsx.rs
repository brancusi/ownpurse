//! Excel workbooks with a mapping (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct Xlsx;

impl Projector for Xlsx {
    fn id(&self) -> &'static str {
        "xlsx@1"
    }

    fn reads(&self) -> &'static str {
        "Excel workbooks with a mapping"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("xlsx@1 is not built yet".into()))
    }
}
