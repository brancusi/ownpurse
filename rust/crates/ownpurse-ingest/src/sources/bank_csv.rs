//! Bank activity downloads (CSV) with a column mapping (not built yet).

use crate::{IngestError, Input, Projection, Projector};
use ownpurse_facts::Db;

pub struct BankCsv;

impl Projector for BankCsv {
    fn id(&self) -> &'static str {
        "bank.csv@1"
    }

    fn reads(&self) -> &'static str {
        "Bank activity downloads (CSV) with a column mapping"
    }

    fn project(&self, _db: &Db, _input: &Input<'_>) -> Result<Projection, IngestError> {
        Err(IngestError("bank.csv@1 is not built yet".into()))
    }
}
