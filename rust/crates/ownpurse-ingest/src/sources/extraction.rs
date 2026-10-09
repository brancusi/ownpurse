//! An extractor's reading of a PDF or image: statements, receipts, checks (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct Extraction;

impl Projector for Extraction {
    fn id(&self) -> &'static str {
        "extraction@1"
    }

    fn reads(&self) -> &'static str {
        "An extractor's reading of a PDF or image: statements, receipts, checks"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("extraction@1 is not built yet".into()))
    }
}
