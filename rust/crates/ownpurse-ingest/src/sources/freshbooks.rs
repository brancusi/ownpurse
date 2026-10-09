//! FreshBooks API responses (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct FreshBooks;

impl Projector for FreshBooks {
    fn id(&self) -> &'static str {
        "freshbooks.json@1"
    }

    fn reads(&self) -> &'static str {
        "FreshBooks API responses"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("freshbooks.json@1 is not built yet".into()))
    }
}
