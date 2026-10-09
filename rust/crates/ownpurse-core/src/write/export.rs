//! The review export for the accountant (PM addition 3): the sealed package as a PDF (to read) and a CSV (to
//! sort and tick), both carrying the package hash, so what the accountant reviews is provably what gets
//! approved. Refuses an unsealed or edited package. Deterministic: the same package gives the same bytes.

use super::{
    JournalRef, Links, Op, Operation, Package, Precondition, Section, WriteError, op_url, pdf, seal,
};
use rust_decimal::Decimal;
use std::collections::BTreeMap;

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn dr_cr(a: Decimal) -> (String, String) {
    if a > Decimal::ZERO {
        (super::money_text(a), String::new())
    } else if a < Decimal::ZERO {
        (String::new(), super::money_text(-a))
    } else {
        (String::new(), String::new())
    }
}

fn jref(r: &JournalRef) -> String {
    match r {
        JournalRef::Id(id) => id.clone(),
        JournalRef::CreatedBy(op) => format!("the journal {op} creates"),
    }
}

/// The operation's date: the journal's, or the period it touches.
fn date_of(o: &Operation) -> String {
    super::ops::effect_date(o).unwrap_or("").to_string()
}

/// One line describing an operation.
pub fn summary(o: &Operation) -> String {
    match &o.op {
        Op::CreateJournal(j) => {
            let total: Decimal = j
                .lines
                .iter()
                .filter(|l| l.amount > Decimal::ZERO)
                .map(|l| l.amount)
                .sum();
            format!(
                "create a journal dated {} · {} lines · {} · {}{}",
                j.date,
                j.lines.len(),
                super::money_text(total),
                j.narration,
                if j.show_on_cash_basis_reports {
                    ""
                } else {
                    " · not on cash-basis reports"
                }
            )
        }
        Op::RecodeBankLine(r) => {
            let mut what = vec![];
            if let Some(c) = &r.to_account_code {
                what.push(format!("account {} → {c}", r.from_account_code));
            }
            if let Some(d) = &r.to_description {
                what.push(format!("description {:?} → {d:?}", r.from_description));
            }
            format!(
                "recode bank transaction {} line {}: {}",
                r.bank_transaction_id,
                r.line_item_id,
                what.join("; ")
            )
        }
        Op::VoidJournal { journal } => format!("void journal {}", jref(journal)),
        Op::CreateBankTransaction(b) => format!(
            "create a {} bank transaction dated {} on {} · {} line{} · {} · {}",
            b.kind,
            b.date,
            b.bank_account,
            b.lines.len(),
            if b.lines.len() == 1 { "" } else { "s" },
            super::money_text(b.total()),
            b.reference
        ),
        Op::CreateBankTransfer(t) => format!(
            "transfer {} from {} to {} dated {} · {}",
            super::money_text(t.amount),
            t.from_account,
            t.to_account,
            t.date,
            t.reference
        ),
        Op::CreateContact(c) => format!(
            "create the contact {:?}{}",
            c.name,
            c.email
                .as_ref()
                .map(|e| format!(" <{e}>"))
                .unwrap_or_default()
        ),
        Op::ArchiveContact { contact } => format!(
            "archive the contact {}",
            match contact {
                JournalRef::Id(id) => id.clone(),
                JournalRef::CreatedBy(op) => format!("{op} creates"),
            }
        ),
        Op::VoidBankTransaction { bank_transaction } => format!(
            "void bank transaction {} (Status DELETED)",
            match bank_transaction {
                JournalRef::Id(id) => id.clone(),
                JournalRef::CreatedBy(op) => format!("the transaction {op} creates"),
            }
        ),
        Op::AddNote(n) => format!(
            "note on {} {}: {:?}",
            n.entity,
            match &n.target {
                JournalRef::Id(id) => id.clone(),
                JournalRef::CreatedBy(op) => format!("the document {op} creates"),
            },
            n.details
        ),
        Op::Attach(a) => format!(
            "attach {} ({}, {} bytes, sha256 {}) to {}",
            a.file_name,
            a.mime,
            a.bytes,
            &a.sha256[..12.min(a.sha256.len())],
            jref(&a.journal)
        ),
    }
}

fn note_of(o: &Operation) -> String {
    let mut v: Vec<String> = vec![];
    if let Some(g) = &o.gated {
        v.push(format!("accountant gate: {g}"));
    }
    v.extend(o.flags.iter().cloned());
    v.join(" | ")
}

const CSV_HEAD: [&str; 14] = [
    "package_hash",
    "package_id",
    "row_type",
    "op",
    "kind",
    "org",
    "date",
    "account_code",
    "account_name",
    "debit",
    "credit",
    "description",
    "note",
    "xero_url",
];

/// The CSV: one `op` row per operation, its `line` rows (journal lines), `expected` rows (account
/// movements), a `recovery` row; then `excluded`, `checklist` and `accountant` rows.
pub fn csv(p: &Package) -> Result<String, WriteError> {
    csv_with(p, &Links::new())
}

/// [`csv`] with "Open in Xero" links (`xero_url` on each `op` row whose target exists before the run).
pub fn csv_with(p: &Package, links: &Links) -> Result<String, WriteError> {
    let hash = seal::verify(p)?;
    let mut rows: Vec<Vec<String>> = vec![CSV_HEAD.iter().map(|s| s.to_string()).collect()];
    let row = |t: &str, o: Option<&Operation>, rest: [String; 6]| -> Vec<String> {
        let mut r = vec![hash.clone(), p.id.clone(), t.into()];
        match o {
            Some(o) => r.extend([o.id.clone(), o.op.kind().into(), o.org.clone(), date_of(o)]),
            None => r.extend([String::new(), String::new(), String::new(), String::new()]),
        }
        r.extend(rest);
        r.push(String::new());
        r
    };
    let e = String::new;
    if let Some(a) = &p.accountant_decision {
        rows.push(row(
            "accountant",
            None,
            [e(), e(), e(), e(), a.clone(), e()],
        ));
    }
    for o in &p.ops {
        let mut r = row("op", Some(o), [e(), e(), e(), e(), summary(o), note_of(o)]);
        if let Some(u) = op_url(o, links, None) {
            *r.last_mut().expect("columns") = u;
        }
        rows.push(r);
        if let Op::CreateJournal(j) = &o.op {
            for l in &j.lines {
                let (dr, cr) = dr_cr(l.amount);
                rows.push(row(
                    "line",
                    Some(o),
                    [
                        l.account_code.clone(),
                        e(),
                        dr,
                        cr,
                        l.description.clone(),
                        e(),
                    ],
                ));
            }
        }
        for m in &o.expected {
            let (dr, cr) = dr_cr(m.amount);
            rows.push(row(
                "expected",
                Some(o),
                [
                    m.code.clone(),
                    m.name.clone(),
                    dr,
                    cr,
                    match m.section {
                        Section::Bs => "balance sheet".into(),
                        Section::Pl => "profit and loss".into(),
                    },
                    e(),
                ],
            ));
        }
        if let Some(r) = &o.recovery {
            rows.push(row(
                "recovery",
                Some(o),
                [e(), e(), e(), e(), r.note.clone(), e()],
            ));
        }
    }
    for x in &p.excluded {
        rows.push(row(
            "excluded",
            None,
            [e(), e(), e(), e(), x.what.clone(), x.why.clone()],
        ));
    }
    for c in &p.checklist {
        rows.push(row("checklist", None, [e(), e(), e(), e(), c.clone(), e()]));
    }
    let mut out = String::new();
    for r in rows {
        out.push_str(&r.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    Ok(out)
}

fn cols(a: Decimal) -> String {
    let (dr, cr) = dr_cr(a);
    format!("{dr:>14} {cr:>14}")
}

fn cut(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// The review text: the PDF's lines, and what `write seal` prints.
pub fn lines(p: &Package) -> Result<Vec<String>, WriteError> {
    lines_with(p, &Links::new())
}

/// [`lines`] with an "open in Xero" line under each operation whose target exists before the run.
pub fn lines_with(p: &Package, links: &Links) -> Result<Vec<String>, WriteError> {
    let hash = seal::verify(p)?;
    let mut l: Vec<String> = vec![];
    l.push(format!("Xero change package: {}", p.title));
    l.push(format!(
        "id {} · profile {} · planned {}",
        p.id, p.profile, p.created_at
    ));
    l.push(format!("sha256 {hash}"));
    for s in &p.supersedes {
        l.push(format!("supersedes {s}"));
    }
    let gated = p.ops.iter().filter(|o| o.gated.is_some()).count();
    l.push(format!(
        "{} operation{} · {gated} under the accountant gate · {} excluded",
        p.ops.len(),
        if p.ops.len() == 1 { "" } else { "s" },
        p.excluded.len()
    ));
    if let Some(a) = &p.accountant_decision {
        l.push(format!("accountant's decision: {a}"));
    }
    l.push(String::new());
    l.push(
        "Nothing in this package has been sent to Xero. An approval binds to the hash above: any change makes"
            .into(),
    );
    l.push("a new hash, which needs a new approval.".into());
    let mut totals: BTreeMap<(String, String, String), Decimal> = BTreeMap::new();
    for o in &p.ops {
        for m in &o.expected {
            *totals
                .entry((o.org.clone(), m.code.clone(), m.name.clone()))
                .or_default() += m.amount;
        }
    }
    l.push(String::new());
    l.push(
        "EXPECTED TRIAL BALANCE MOVEMENT (all operations; every other account must not move)"
            .into(),
    );
    l.push(format!(
        "{:<8} {:<8} {:<38} {:>14} {:>14}",
        "org", "code", "account", "debit", "credit"
    ));
    let mut any = false;
    for ((org, code, name), a) in &totals {
        if a.is_zero() {
            continue;
        }
        any = true;
        l.push(format!(
            "{org:<8} {code:<8} {:<38} {}",
            cut(name, 38),
            cols(*a)
        ));
    }
    if !any {
        l.push("(no account moves)".into());
    }
    for o in &p.ops {
        l.push(String::new());
        l.push(format!(
            "[{}] {} · {} · {}",
            o.id,
            o.op.kind(),
            o.org,
            date_of(o)
        ));
        l.push(format!("  {}", summary(o)));
        if let Some(u) = op_url(o, links, None) {
            l.push(format!("  open in Xero: {u}"));
        }
        if let Some(g) = &o.gated {
            l.push(format!("  ACCOUNTANT GATE: {g}"));
        }
        for f in &o.flags {
            l.push(format!("  flag: {f}"));
        }
        for d in &o.decision_refs {
            l.push(format!(
                "  decision {} ({})",
                d.id,
                cut(&d.content_hash, 12)
            ));
        }
        for d in &o.source_refs {
            l.push(format!("  source {} ({})", d.id, cut(&d.content_hash, 12)));
        }
        if !o.depends_on.is_empty() {
            l.push(format!("  after {}", o.depends_on.join(", ")));
        }
        l.push(
            format!("  request: {:?} {}", o.request.method, o.request.path)
                .replace("Put", "PUT")
                .replace("Post", "POST"),
        );
        if let Op::CreateJournal(j) = &o.op {
            l.push(format!(
                "  {:<8} {:<40} {:>14} {:>14}",
                "code", "description", "debit", "credit"
            ));
            for x in &j.lines {
                l.push(format!(
                    "  {:<8} {:<40} {}",
                    x.account_code,
                    cut(&x.description, 40),
                    cols(x.amount)
                ));
            }
        }
        if let Op::CreateBankTransaction(b) = &o.op {
            l.push(format!(
                "  {:<8} {:<40} {:>14}",
                "code", "description", "amount"
            ));
            for x in &b.lines {
                l.push(format!(
                    "  {:<8} {:<40} {:>14}",
                    x.account_code,
                    cut(&x.description, 40),
                    super::money_text(x.amount)
                ));
            }
            l.push(format!(
                "  contact {}",
                match &b.contact {
                    JournalRef::Id(id) => id.clone(),
                    JournalRef::CreatedBy(op) => format!("the contact {op} creates"),
                }
            ));
        }
        l.push("  checked live before sending:".into());
        for pc in &o.preconditions {
            l.push(format!(
                "    {}",
                match pc {
                    Precondition::Target {
                        entity,
                        id,
                        fingerprint,
                        version,
                        status,
                        ..
                    } => format!(
                        "{entity} {id} unchanged ({status}{}, fingerprint {})",
                        version.map(|v| format!(", v{v}")).unwrap_or_default(),
                        cut(fingerprint, 12)
                    ),
                    Precondition::MarkerAbsent { marker } =>
                        format!("no journal already carries {marker}"),
                    Precondition::AccountsActive { codes } =>
                        format!("accounts {} exist once and are ACTIVE", codes.join(", ")),
                    Precondition::PeriodOpen { date } => format!("{date} is after the lock dates"),
                    Precondition::AttachmentAbsent { file_name, .. } =>
                        format!("{file_name} is not attached yet"),
                    Precondition::Exists { entity, id } => format!("{entity} {id} exists"),
                    Precondition::ReferenceAbsent { marker, entity } => format!(
                        "no {} already carries {marker}",
                        if entity.as_deref() == Some("BankTransfers") {
                            "bank transfer"
                        } else {
                            "bank transaction"
                        }
                    ),
                    Precondition::ContactNameAbsent { name } =>
                        format!("no contact is named {name:?}"),
                }
            ));
        }
        if o.expected.is_empty() {
            l.push("  expected movement: none".into());
        } else {
            l.push("  expected movement:".into());
            for m in &o.expected {
                l.push(format!(
                    "    {:<8} {:<38} {} {}",
                    m.code,
                    cut(&m.name, 38),
                    cols(m.amount),
                    match m.section {
                        Section::Bs => "BS",
                        Section::Pl => "P&L",
                    }
                ));
            }
            let (_, pl) = super::delta::sections(&o.expected);
            l.push(format!("    profit effect {}", super::money_text(-pl)));
        }
        if let Some(r) = &o.recovery {
            l.push(format!("  recovery: {}", r.note));
        }
        if let Some(w) = &o.history_note {
            l.push(format!(
                "  history note after it lands: \"ownpurse package <hash12> op {}: {w}\"",
                o.id
            ));
        }
    }
    if !p.excluded.is_empty() {
        l.push(String::new());
        l.push("EXCLUDED (listed, never executable)".into());
        for x in &p.excluded {
            l.push(format!("  {} - {}", x.what, x.why));
        }
    }
    if !p.checklist.is_empty() {
        l.push(String::new());
        l.push("AFTER THE RUN (a person does these)".into());
        for c in &p.checklist {
            l.push(format!("  [ ] {c}"));
        }
    }
    l.push(String::new());
    l.push("To approve, the owner records an approval quoting the full hash:".into());
    l.push(format!("  {hash}"));
    l.push(
        "(`ownpurse write approve-record` at his own terminal, or a chat message he types, imported by hash)."
            .into(),
    );
    Ok(l)
}

pub fn pdf(p: &Package) -> Result<Vec<u8>, WriteError> {
    pdf_with(p, &Links::new())
}

/// [`pdf`] with "open in Xero" lines.
pub fn pdf_with(p: &Package, links: &Links) -> Result<Vec<u8>, WriteError> {
    let hash = seal::verify(p)?;
    Ok(pdf::render(
        &format!("package {} · sha256 {hash}", p.id),
        &lines_with(p, links)?,
    ))
}
