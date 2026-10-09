//! The rules table (design §4): the shape of an entry's patch → the Xero write operation, or "not writable".
//! The rules are data plus small predicates; a new write type is one row in [`RULES`].

use super::Entry;
use serde_json::Value;

/// The facts a rule looks at.
pub struct Shape<'a> {
    pub entity: &'a str,
    /// A new document (`before` is None).
    pub new: bool,
    /// `(op, path, value)` of every patch operation.
    pub ops: Vec<(&'static str, String, Value)>,
    pub before: Option<&'a Value>,
    pub after: &'a Value,
    /// The entry's resolution for a reconciled bank line (`reclass` / `browser`), if chosen.
    pub resolution: Option<&'a str>,
}

impl<'a> Shape<'a> {
    pub fn of(entry: &'a Entry, before: Option<&'a Value>, after: &'a Value) -> Shape<'a> {
        use json_patch::PatchOperation as P;
        let ops = entry
            .ops()
            .into_iter()
            .map(|o| match o {
                P::Add(a) => ("add", a.path.to_string(), a.value),
                P::Remove(r) => ("remove", r.path.to_string(), Value::Null),
                P::Replace(r) => ("replace", r.path.to_string(), r.value),
                P::Move(m) => ("move", m.path.to_string(), Value::Null),
                P::Copy(c) => ("copy", c.path.to_string(), Value::Null),
                P::Test(t) => ("test", t.path.to_string(), t.value),
            })
            .collect();
        Shape {
            entity: entry.entity.as_str(),
            new: before.is_none(),
            ops,
            before,
            after,
            resolution: entry.resolution.as_deref(),
        }
    }

    fn segs(path: &str) -> Vec<String> {
        path.trim_start_matches('/')
            .split('/')
            .map(|s| s.replace("~1", "/").replace("~0", "~"))
            .collect()
    }

    /// The line ids whose `AccountCode` / `Description` the patch changes, if those are ALL it changes.
    pub fn recoded_lines(&self) -> Option<Vec<String>> {
        let mut ids: Vec<String> = vec![];
        for (op, path, _) in &self.ops {
            let s = Shape::segs(path);
            let ok = matches!(*op, "replace" | "add" | "remove")
                && s.len() == 3
                && s[0] == "LineItems"
                && matches!(s[2].as_str(), "AccountCode" | "Description");
            if !ok {
                return None;
            }
            if !ids.contains(&s[1]) {
                ids.push(s[1].clone());
            }
        }
        (!ids.is_empty()).then_some(ids)
    }

    /// The attachment names the patch adds, if adding attachments is ALL it does.
    pub fn added_attachments(&self) -> Option<Vec<String>> {
        let mut names = vec![];
        for (op, path, value) in &self.ops {
            let s = Shape::segs(path);
            match (*op, s.len()) {
                ("add", 1) if s[0] == "Attachments" => {
                    names.extend(value.as_object()?.keys().cloned());
                }
                ("add", 2) if s[0] == "Attachments" => names.push(s[1].clone()),
                _ => return None,
            }
        }
        (!names.is_empty()).then_some(names)
    }

    /// The notes the patch adds to the document's history (`add /History/-` with `{"Details": …}`, or
    /// `add /History` with a list of them), if adding notes is ALL it does.
    pub fn added_notes(&self) -> Option<Vec<String>> {
        let mut notes = vec![];
        let one = |v: &Value| v.get("Details").and_then(Value::as_str).map(str::to_string);
        for (op, path, value) in &self.ops {
            let s = Shape::segs(path);
            match (*op, s.len()) {
                ("add", 1) if s[0] == "History" => {
                    for n in value.as_array()? {
                        notes.push(one(n)?);
                    }
                }
                ("add", 2) if s[0] == "History" => notes.push(one(value)?),
                _ => return None,
            }
        }
        (!notes.is_empty()).then_some(notes)
    }

    fn status(v: Option<&Value>) -> &str {
        v.and_then(|v| v.get("Status"))
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    /// The target is a reconciled bank transaction (Xero refuses API edits of it).
    pub fn reconciled(&self) -> bool {
        self.before
            .and_then(|b| b.get("IsReconciled"))
            .and_then(Value::as_bool)
            == Some(true)
    }

    /// The recode changes only account codes (a reclass journal can't carry a description change).
    pub fn account_only(&self) -> bool {
        self.ops.iter().all(|(_, p, _)| p.ends_with("/AccountCode"))
    }

    /// A recode of exactly one line.
    pub fn one_line_recode(&self) -> bool {
        !self.new && self.recoded_lines().is_some_and(|l| l.len() == 1)
    }

    /// Only `/Status` POSTED → VOIDED.
    pub fn is_void(&self) -> bool {
        !self.ops.is_empty()
            && self.ops.iter().all(|(op, path, v)| {
                *op == "replace" && path == "/Status" && v.as_str() == Some("VOIDED")
            })
            && Shape::status(self.before) == "POSTED"
    }
}

/// One row: the entity, the operation it maps to, and when it applies.
pub struct Rule {
    pub entity: &'static str,
    /// The write op kind (`create_journal`, `void_journal`, `attach`, `recode_bank_line`).
    pub op: &'static str,
    /// What the rule matches, for docs and the CLI.
    pub shape: &'static str,
    pub when: fn(&Shape) -> bool,
    /// A warning every match carries (None: none).
    pub warns: Option<&'static str>,
}

pub const RULES: [Rule; 14] = [
    Rule {
        entity: "ManualJournals",
        op: "create_journal",
        shape: "a new document",
        when: |s| s.new,
        warns: None,
    },
    Rule {
        entity: "ManualJournals",
        op: "void_journal",
        shape: "only /Status POSTED → VOIDED",
        when: |s| !s.new && s.is_void(),
        warns: None,
    },
    // a posted journal can't be edited in place: an explicit `void-recreate` voids it and creates the corrected
    // journal (never the default: voiding is visible in Xero's history)
    Rule {
        entity: "ManualJournals",
        op: "void_recreate",
        shape: "lines, amounts, accounts, narration or date of a POSTED journal, resolution void-recreate",
        when: |s| {
            !s.new
                && s.resolution == Some("void-recreate")
                && Shape::status(s.before) == "POSTED"
                && Shape::status(Some(s.after)) == "POSTED"
        },
        warns: None,
    },
    Rule {
        entity: "ManualJournals",
        op: "attach",
        shape: "only /Attachments/<name> added",
        when: |s| !s.new && s.added_attachments().is_some(),
        warns: Some("attachment"),
    },
    Rule {
        entity: "BankTransactions",
        op: "attach",
        shape: "only /Attachments/<name> added (a receipt on a bank transaction)",
        when: |s| !s.new && s.added_attachments().is_some(),
        warns: Some("attachment"),
    },
    // cash and petty cash with no bank feed: a new SPEND or RECEIVE (its recovery: the void below)
    Rule {
        entity: "BankTransactions",
        op: "create_bank_transaction",
        shape: "a new document, Type SPEND or RECEIVE (AUTHORISED, the marker in its Reference)",
        when: |s| {
            s.new
                && matches!(
                    s.after.get("Type").and_then(Value::as_str),
                    Some("SPEND" | "RECEIVE")
                )
        },
        warns: None,
    },
    Rule {
        entity: "BankTransactions",
        op: "void_bank_transaction",
        shape: "only /Status AUTHORISED → DELETED, on an UNRECONCILED transaction (how the API voids one)",
        when: |s| {
            !s.new
                && !s.reconciled()
                && !s.ops.is_empty()
                && s.ops.iter().all(|(op, path, v)| {
                    *op == "replace" && path == "/Status" && v.as_str() == Some("DELETED")
                })
                && Shape::status(s.before) == "AUTHORISED"
        },
        warns: None,
    },
    // petty cash funded from the bank: a new transfer (permanent history: the API can't void a transfer)
    Rule {
        entity: "BankTransfers",
        op: "create_bank_transfer",
        shape: "a new document (from and to bank accounts, amount, date; the marker in its Reference)",
        when: |s| s.new,
        warns: Some("transfer"),
    },
    // the contact a cash entry needs: a new contact by name (refused when the exact name exists)
    Rule {
        entity: "Contacts",
        op: "create_contact",
        shape: "a new document (Name, optional EmailAddress)",
        when: |s| s.new,
        warns: None,
    },
    // a note in the document's history (also the audit note every landed write adds): permanent
    Rule {
        entity: "ManualJournals",
        op: "add_note",
        shape: "only /History/- added ({\"Details\": …}): a note in the journal's history",
        when: |s| !s.new && s.added_notes().is_some(),
        warns: Some("note"),
    },
    Rule {
        entity: "BankTransactions",
        op: "add_note",
        shape: "only /History/- added ({\"Details\": …}): a note in the transaction's history",
        when: |s| !s.new && s.added_notes().is_some(),
        warns: Some("note"),
    },
    Rule {
        entity: "BankTransactions",
        op: "recode_bank_line",
        shape: "only /LineItems/<id>/AccountCode and/or /Description, on one UNRECONCILED line",
        when: |s| s.one_line_recode() && !s.reconciled(),
        warns: None,
    },
    // Xero refuses API edits of reconciled bank transactions (HTTP 400, verified on the Demo Company): the
    // change is carried out by a reclass journal (the default) or in the browser
    Rule {
        entity: "BankTransactions",
        op: "reclass_journal",
        shape: "an account recode of a RECONCILED line, resolution reclass: Dr new / Cr old in the org's monthly reclass journal",
        when: |s| {
            s.one_line_recode()
                && s.reconciled()
                && s.resolution == Some("reclass")
                && s.account_only()
        },
        warns: None,
    },
    Rule {
        entity: "BankTransactions",
        op: "browser_recode",
        shape: "a recode of a RECONCILED line, resolution browser: unreconcile → edit → re-reconcile (a checklist step)",
        when: |s| s.one_line_recode() && s.reconciled() && s.resolution == Some("browser"),
        warns: None,
    },
];

/// What an entry compiles to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Matched {
    /// The row of [`RULES`] it matches.
    Op(&'static str),
    /// No row matches: an error issue with a message and the supported path.
    NotWritable { message: String, hint: String },
    /// A recode of a RECONCILED bank line with no resolution chosen (or one the chosen resolution can't carry):
    /// an error that offers the two resolutions.
    Unresolved { message: String, hint: String },
}

/// The rule an entry matches (first match wins), or why it's not writable.
pub fn classify(s: &Shape) -> Matched {
    if s.ops.is_empty() {
        return Matched::NotWritable {
            message: "the entry changes nothing".into(),
            hint: "undo it (u)".into(),
        };
    }
    if let Some(r) = RULES.iter().find(|r| r.entity == s.entity && (r.when)(s)) {
        return Matched::Op(r.op);
    }
    if s.entity == "BankTransactions" && s.one_line_recode() && s.reconciled() {
        return Matched::Unresolved {
            message: if s.resolution == Some("reclass") {
                "reconciled: Xero refuses edits, and a reclass journal carries only an account change".into()
            } else {
                "reconciled: Xero refuses edits of a reconciled bank transaction".into()
            },
            hint: "resolve it: a reclass journal (the default: `manifest resolve <n> reclass`), or the browser path (unreconcile, edit, re-reconcile: `manifest resolve <n> browser`)".into(),
        };
    }
    let (message, hint) = match s.entity {
        "ManualJournals" if Shape::status(s.before) == "VOIDED" => (
            "the journal is voided in Xero; a voided journal can't change".to_string(),
            "create a new journal instead".to_string(),
        ),
        "ManualJournals" => (
            format!(
                "Xero can't edit a posted journal in place ({})",
                s.ops
                    .iter()
                    .map(|(_, p, _)| p.as_str())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "resolve it as void + re-create (`manifest resolve <n> void-recreate`): the original is voided and the corrected journal created; voiding shows in Xero's history"
                .to_string(),
        ),
        "BankTransactions" if s.new => (
            "creating bank transactions is not a v1 write".to_string(),
            "bank lines come from the feed or a statement import; post a manual journal instead"
                .to_string(),
        ),
        "BankTransactions" if s.recoded_lines().is_some_and(|l| l.len() > 1) => (
            "the entry recodes more than one line".to_string(),
            "make one entry per line (each is its own recode)".to_string(),
        ),
        "BankTransactions" => (
            "only a line's account and description can be recoded on a bank transaction"
                .to_string(),
            "for amounts, dates or contacts, post a correcting manual journal".to_string(),
        ),
        other => (
            format!("{other} is not written in v1"),
            "v1 writes manual journals (create, void, attach) and bank-line recodes".to_string(),
        ),
    };
    Matched::NotWritable { message, hint }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Refs, entry_payload};
    use serde_json::json;

    fn entry(entity: &str, before: Option<&Value>, after: &Value) -> Entry {
        let p = entry_payload(
            entity,
            "o",
            Some("x"),
            before,
            after,
            "",
            &Refs::default(),
            None,
        );
        Entry {
            seq: 1,
            at: String::new(),
            actor: "human".into(),
            entity: entity.into(),
            org: "o".into(),
            id: "x".into(),
            before_fp: p["before_fp"].as_str().map(str::to_string),
            after_fp: p["after_fp"].as_str().map(str::to_string),
            after_bfp: None,
            patch: p["patch"].clone(),
            why: String::new(),
            refs: Refs::default(),
            note: None,
            replaces: None,
            replaced_by: None,
            undone: false,
            resolution: None,
        }
    }

    fn matched(entity: &str, before: Option<Value>, after: Value) -> Matched {
        let b = before.as_ref().map(crate::manifest::normalise::normalise);
        let a = crate::manifest::normalise::normalise(&after);
        let e = entry(entity, before.as_ref(), &after);
        classify(&Shape::of(&e, b.as_ref(), &a))
    }

    #[test]
    fn the_table() {
        let mj = json!({"ManualJournalID": "m", "Status": "POSTED", "Narration": "x",
            "JournalLines": [{"AccountCode": "6420", "LineAmount": 10}, {"AccountCode": "2420", "LineAmount": -10}]});
        assert_eq!(
            matched("ManualJournals", None, mj.clone()),
            Matched::Op("create_journal")
        );
        let mut v = mj.clone();
        v["Status"] = json!("VOIDED");
        assert_eq!(
            matched("ManualJournals", Some(mj.clone()), v),
            Matched::Op("void_journal")
        );
        let mut a = mj.clone();
        a["Attachments"] = json!([{"FileName": "s.csv"}]);
        assert_eq!(
            matched("ManualJournals", Some(mj.clone()), a),
            Matched::Op("attach")
        );
        let mut amt = mj.clone();
        amt["JournalLines"][0]["LineAmount"] = json!(11);
        amt["JournalLines"][1]["LineAmount"] = json!(-11);
        let Matched::NotWritable { hint, .. } = matched("ManualJournals", Some(mj), amt) else {
            panic!()
        };
        assert!(hint.contains("void-recreate"));
        let bt = json!({"Status": "AUTHORISED", "LineItems": [
            {"LineItemID": "l1", "AccountCode": "5510", "Description": "a", "LineAmount": 1},
            {"LineItemID": "l2", "AccountCode": "5510", "Description": "b", "LineAmount": 2}]});
        let mut r = bt.clone();
        r["LineItems"][1]["AccountCode"] = json!("2440");
        r["LineItems"][1]["Description"] = json!("bb");
        assert_eq!(
            matched("BankTransactions", Some(bt.clone()), r.clone()),
            Matched::Op("recode_bank_line")
        );
        r["LineItems"][0]["AccountCode"] = json!("2440");
        assert!(
            matches!(matched("BankTransactions", Some(bt.clone()), r), Matched::NotWritable { hint, .. } if hint.contains("one entry per line"))
        );
        let mut amount = bt.clone();
        amount["LineItems"][0]["LineAmount"] = json!(5);
        assert!(matches!(
            matched("BankTransactions", Some(bt), amount),
            Matched::NotWritable { .. }
        ));
        assert!(
            matches!(matched("Items", None, json!({"Name": "x"})), Matched::NotWritable { message, .. } if message.contains("Items"))
        );
    }
}
