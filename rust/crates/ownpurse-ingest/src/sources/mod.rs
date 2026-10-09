//! One module per source format. Each exports a unit struct implementing `Projector`, registered in `all()`.

pub mod bank_csv;
pub mod eth;
pub mod extraction;
pub mod freshbooks;
pub mod ofx;
pub mod qbo;
pub mod sheets;
pub mod xero;
pub mod xlsx;

use crate::Projector;

pub fn all() -> Vec<&'static dyn Projector> {
    vec![
        &xero::Xero,
        &qbo::Qbo,
        &freshbooks::FreshBooks,
        &bank_csv::BankCsv,
        &ofx::Ofx,
        &extraction::Extraction,
        &sheets::Sheets,
        &xlsx::Xlsx,
        &eth::Eth,
    ]
}
