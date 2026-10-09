//! Google Sheets values responses with a mapping (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct Sheets;

impl Projector for Sheets {
    fn id(&self) -> &'static str {
        "sheet.values@1"
    }

    fn reads(&self) -> &'static str {
        "Google Sheets values responses with a mapping"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("sheet.values@1 is not built yet".into()))
    }
}
