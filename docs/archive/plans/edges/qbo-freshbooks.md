# Edges: QuickBooks Online and FreshBooks

What decomposing two more accounting systems into facts taught us about the model. The samples are the supplier
side of two Xero bills in the March 2026 world (`samples/README.md`):

- `samples/qbo/`: Quillfeather Design Consultancy LLP's QuickBooks Online, source `qbo:quillfeather`. Invoice
  1031 (2,400.00, 2026-03-04) to Tallowmere, its payment on 2026-03-18, a 49.00 card purchase, a journal entry,
  and a second read of the invoice two weeks later with a new `SyncToken`, `PrivateNote` and `CustomerMemo`.
- `samples/freshbooks/`: Saltmarsh Cupping Lab LLC's FreshBooks, source `freshbooks:saltmarsh`. Invoice 57
  (362.18, 2026-03-11) read while unpaid and again once paid, its payment on 2026-03-24, a client, an expense,
  and the `users/me` identity that names the business.

Projectors: `rust/crates/ownpurse-ingest/src/sources/qbo.rs` (`qbo.json@1`) and `freshbooks.rs`
(`freshbooks.json@1`). Both run a small normalising pass over the JSON, then the generic mirror
(`jsonmirror.rs`), then an interpretation into the shared vocabulary. Tests: `tests/qbo.rs` (14) and
`tests/freshbooks.rs` (14).

## Summary

The model held. Every field of both sources lands in the log; re-reading a file is an empty transaction; the
second QuickBooks read asserts exactly the four fields that changed (plus the new file); every QuickBooks mirror
entry balances to zero; money as `(amount, commodity)` (D13) took FreshBooks' money objects without strain.
Where it strained, the strain was almost all in the generic mirror's configuration language, not in the fact
model: `JsonSource` cannot say "this field is a reference" except by an id field inside an object, and its
position fallback for list items collides with real ids. One real model-level gap showed up: a snapshot read has
no notion of order, so an older file read late rolls a document back.

## QuickBooks Online

### References named by the field, not by an id field

**Source.** Every reference is `{"value": "58", "name": "Tallowmere Coffee Roasting Co."}`. What it points at
is only in the field name: `CustomerRef` is a Customer, `DepositToAccountRef`, `ItemAccountRef` and
`IncomeAccountRef` are Accounts, `SalesTermRef` is a Term. **Model.** `JsonSource.ref_ids` recognises a
reference by an id field the object holds (`ContactID` → Contact). Every QuickBooks ref holds the same field,
`value`, so `ref_ids` cannot tell them apart; left alone, the mirror would make each ref a component entity with
a `value` string, and no ref would ever join to the Customer or Account it names. **Workaround.** The projector
rewrites each `…Ref` object to `{"@Customer": "58", "name": …}` before the mirror sees it, and lists
`("@Customer", "Customer")` and friends in `ref_ids`. The ref is now a real ref (`qbo.Invoice/CustomerRef` →
the Customer document), and `name` is reported as an embedded copy, as Xero's contacts are. **Lost.** Nothing:
the kind is in the target's doc id. **Options.** Keep normalising per source; or give `JsonSource` a hook that
names a reference from the field and its value. **Recommendation.** Add the hook (below). Three sources now need
it (QuickBooks by field name, QuickBooks `EntityRef`/`LinkedTxn` by a sibling value, FreshBooks by plain-number
foreign keys), and a hook covers all three where a table covers one.

### References whose kind is in a sibling value

**Source.** `EntityRef` (on a Purchase) carries its kind in its own `type` (`Vendor`, `Customer`, `Employee`);
`LinkedTxn` items are `{"TxnId": "131", "TxnType": "Payment"}`. **Model.** Same gap as above: the kind is
data. **Workaround.** The normaliser reads the kind and rewrites to `{"@Vendor": "64"}` / `{"@Payment": "131"}`,
dropping `type`/`TxnType` (the kind survives in the target's doc id). The payment's `LinkedTxn` then becomes a
`settles` link with `link/amount`. **Recommendation.** The same hook; nothing else needed.

### The subtotal line collides with real line ids (a jsonmirror bug)

**Source.** Invoice lines carry an `Id` ("1"); QuickBooks always appends a subtotal line with no `Id`. **Model.**
`jsonmirror::array` names a list item by its child id if it has one, otherwise by its position, *in the same
namespace*. The sales line (`Id` "1", position 0) and the subtotal (no Id, position 1) both get the key
`…/Line/1`, the walker merges them into one entity, and the transaction is refused: "two values for
`Invoice.Line/DetailType`". This is not hypothetical: it happens on every QuickBooks invoice with one line, and
`line_ids_and_positions_collide_in_the_generic_mirror` reproduces it. Journal entries are worse in a quieter way:
their line ids start at "0", so ids and positions agree by luck. **Workaround.** The projector leaves `Id` out of
`child_ids`, so lines are identified by position (the mirror reports `position-identity`). **Lost.** Line
identity across edits: inserting a line renames the ones after it. **Recommendation.** Fix the mirror: a
position fallback segment should be `@<i>`, as the top-level fallback already is (`format!("@{i}")` in
`mirror`), so it can never equal a real id. Then QuickBooks can list `Id` in `child_ids` again.

### Polymorphic lines

**Source.** A line says what it is in `DetailType` (`SalesItemLineDetail`, `SubTotalLineDetail`,
`DiscountLineDetail`, `AccountBasedExpenseLineDetail`, `JournalEntryLineDetail`, …) and keeps its fields in an
object of that name. **Model.** Fits. The mirror gives each detail kind its own namespace
(`qbo.Invoice.Line.SalesItemLineDetail/ItemRef`), the interpretation switches on `DetailType`, and an
unrecognised kind is mirrored and reported. **Awkward.** A query for "the account every line posts to" has to
know every detail kind's field (`ItemAccountRef`, `AccountRef`, `DiscountAccountRef`); that is what the
interpretation is for. **Recommendation.** Fits; no change.

### Values QuickBooks computes

**Source.** `TotalAmt`, `Balance`, the subtotal line, `Account.CurrentBalance`, `Customer.Balance`. An invoice
has no status at all: open, partly paid and paid are what `Balance` says against `TotalAmt`. **Model.** The
mirror keeps them all as facts; the interpretation skips the subtotal line (it is neither observed nor posted)
and derives `obs/status` from `Balance`. **Awkward.** A payment arriving shows up as an *edit* of the invoice,
the customer and the account, because their computed fields moved. A reviewer reading history sees "invoice
changed" when nothing about the invoice did. **Options.** Drop computed fields from the mirror (loses a free check
that our own arithmetic agrees with the source's); keep them and mark them; keep them silently. **Recommendation.**
Keep them, and let a source mark derived fields (a `derived: &[&str]` list on `JsonSource`, written as the
attribute's `db/doc` or a flag at install) so history views can show those changes as consequences rather than
edits. Low priority.

### Timestamps with an offset

**Source.** `MetaData.CreateTime`, `LastUpdatedTime`, `DeliveryTime` and the response `time` carry the company's
offset, which moves with daylight saving: the 4 March invoice was created at `-08:00`, everything after 8 March
is `-07:00`. **Model.** `Instant::parse_offset` stores UTC and drops the offset (reported as `offset-dropped`).
**Lost.** The local wall-clock time, and so the local date: 19:30 at `-08:00` is the next day in UTC. **Does it
matter here?** No: the business dates (`TxnDate`, `DueDate`) are plain dates and become `obs/date` untouched;
only metadata is affected. **Recommendation.** Fits today. If a source ever gives the business date *only* as an
offset timestamp, take the date from the local text before converting; a small helper beside `parse_offset`
(`local_date(text) -> Date`) would make that the easy path.

### Values wrapped in `{"value": …}` that are not references

**Source.** `CustomerMemo` is `{"value": "Thank you…"}`, the same wrapper as a reference without the `Ref`
suffix. **Model.** The normaliser only treats `…Ref` fields as references; the memo stays a component entity with
one attribute (`qbo.Invoice.CustomerMemo/value`). The second read changes it there, and the diff handles it.
**Awkward.** One extra entity per memo and a longer query path. **Recommendation.** Fits; unwrapping is an
optional normaliser step, not a model change.

### The response envelope

**Source.** `time` (when QuickBooks produced the response) and `startPosition`/`maxResults` (paging). FreshBooks
has the same: `page`, `pages`, `per_page`, `total`. **Model.** The mirror walks only the collections; the rest
is reported as `envelope` and dropped. **Lost.** The source's own read time (more exact than the shell's
`tx/observed-at`, which is when the file reached us), and any statement that the file was the whole list (`total`
against the count read). Without the second, a missing document can never be read as "deleted". **Options.**
Let a projector write transaction metadata (it already can through `TxData::meta`, but core has no attribute for
it). **Recommendation.** Add `tx/source-time` to core (below). Completeness can wait for a source that deletes.

### Order between reads (stale reads)

**Source.** QuickBooks bumps `SyncToken` on every change; FreshBooks bumps `updated`. **Model.** A snapshot
read says "this is the document now". Read an older file after a newer one and the document is rolled back: the
FreshBooks test `an_older_file_read_late_rolls_the_invoice_back` reads the unpaid invoice after the paid one and
the invoice is unpaid again. Nothing in the snapshot knows which version is newer. **Options.** Leave ordering to
the shell (bring files in order); have each projector compare a version field with the stored one and skip
older documents; teach the mirror a version field. **Recommendation.** Give `JsonSource` an optional version
field (`SyncToken`, `updated`, Xero's `UpdatedDateUTC`): when the stored value is newer, the document's facts are
left alone and a `stale-read` pressure is written. This is the one finding that is about the model's meaning of a
snapshot, not about configuration.

### Accounts that live in another file

**Source.** An invoice usually does not name its receivable account (`ARAccountRef` is optional; QuickBooks uses
the one account of type Accounts Receivable), and older API versions do not put the income account on the line
(`ItemAccountRef`), only on the Item. **Model.** The projector finds them in the db: AR by `AccountType`, income
by the Item's `IncomeAccountRef`. Both need Account and Item read first (the sample manifest does). **Lost when
out of order.** The entry is written with an unaccounted posting and `unresolved-ref`, and nothing revisits it
when the Account file arrives. Xero has the same shape (`account_by_code`). **Recommendation.** Fine for this
phase; long term, mirror entries are better computed by a rule pass after ingest than inside one projector run.

### Smaller things that fit

- **Custom fields** (`[{"DefinitionId": "1", "Name": "Project", "Type": "StringType", "StringValue": …}]`): the
  field's name is data. Mirrored as components keyed by `DefinitionId`; finding "Project" is a two-fact join.
  Fits.
- **XML binding leaks** (`LineEx.any[]` with `declaredType`, `scope`, `nil`, `globalScope`): mirrored as is,
  reported as `serialisation-artefact`. Noise, not loss.
- **Pseudo-ids** (`TaxCodeRef: {"value": "NON"}`): a ref to a TaxCode document nothing else describes. Harmless.
- **Book identity** comes from `CompanyInfo` (`book/name`); the home currency is in `Preferences`, which the
  samples do not read, so `book/commodity` is unset (`missing-context`). Observations carry their own commodity.
- **Journal entry sides** are unsigned amounts with `PostingType` Debit/Credit beside them; the projector signs
  them, debit positive (`sign`).

## FreshBooks

### Money as an object (D13)

**Source.** `"amount": {"amount": "362.18", "code": "USD"}`, on the invoice (`amount`, `outstanding`, `paid`,
`discount_total`), each line (`amount`, `unit_cost`), payments and expenses. **Model.** The interpretation maps
it directly to `obs/amount` + `obs/commodity`: this is D13 as written, and it fits exactly. The mirror keeps each
money object as a component entity (`freshbooks.invoice.amount/amount` = 362.18, `…/code` = USD), with the
inner amount parsed as a decimal. **Awkward.** Six entities per two-line invoice just for money, and a paid
invoice's change lands on the `outstanding` and `paid` components rather than the invoice. **Recommendation.**
Fits; keep. A flattening option (`amount` decimal plus `amount.code` on the parent) would be lighter but special-
cases one shape in a generic walker.

### Plain-number foreign keys, two ids, and zero meaning none

**Source.** `customerid: 12`, `invoiceid: 57`, `categoryid: 3` are plain numbers naming other documents. An
invoice carries its own id twice (`id` and `invoiceid`; payments `id` and `logid`; clients `id` and `userid`).
`expenseid: 0`, `clientid: 0`, `projectid: 0` mean "none". **Model.** `ref_ids` only sees ids inside objects, so a
plain number stays a decimal and never joins. **Workaround.** The normaliser rewrites each foreign key to
`{"@client": "12"}` (a real ref), leaves an item's second id for itself as a number (reported `two-ids`), and
leaves 0 as 0 (`zero-as-none`), since a ref to `client/0` would name a document that does not exist. **Lost.**
Nothing. **Recommendation.** The reference hook below covers foreign keys; "0 means none" is source knowledge
that belongs in the same hook (return no reference).

### Integer enums

**Source.** `status: 2` (sent) becomes `4` (paid); `vis_state: 0`. The invoice also sends `v3_status: "sent"`.
**Model.** Mirrored as numbers (`enum-int`); the meaning is only in FreshBooks' documentation. The
interpretation uses the text field `v3_status` for `obs/status`. **Recommendation.** Fits, because FreshBooks
sends a text twin. A source that sends only integers needs an enum table in its config; not needed yet.

### Times without a zone, numbers as text

**Source.** `updated: "2026-03-11 14:05:12"`, `created_at`, `signup_date`: wall-clock times with no zone
(FreshBooks documents them per field). Quantities and tax amounts are text (`"qty": "3"`), and so is
`invoice_number: "0057"`, which must not become 57. **Model.** Wall-clock times stay text (`no-zone`): they are
never used for ordering. Business dates (`create_date`, `due_date`, `date`) become dates. Numeric text is parsed
only for named fields (`amount`, `qty`, `taxAmount1`, …) through `JsonSource.when`. **Lost.** Exact ordering of
FreshBooks events against other sources' instants. **Recommendation.** Fits. `when` is really "parse a typed
value from text"; renaming it `parse` would say so.

### Values that move when a payment arrives

**Source.** When invoice 57 is paid: `status`, `v3_status`, `display_status`, `payment_status`, `outstanding`,
`paid`, `updated` change and `date_paid` appears. **Model.** The snapshot diff asserts exactly those: 9
assertions and 8 retractions on the invoice and its two money components, plus the new file
(`the_paid_invoice_asserts_only_what_the_payment_changed`). Correct, and the same "edit that is not an edit" as
QuickBooks' `Balance`. **Recommendation.** As for QuickBooks: mark derived fields.

### Copies of the client on the invoice

**Source.** The invoice carries the client's `fname`, `lname`, `organization`, `current_organization`,
`street`, `city`, … as plain fields beside `customerid`. **Model.** Mirrored as the invoice's own fields
(`flat-copy`); unlike an embedded object there is nothing to recognise as a copy. `obs/counterparty` takes
`current_organization`. **Recommendation.** Fits. It is what the invoice printed, which is what a counterparty
name on an observation should be.

### No ledger

**Source.** FreshBooks' invoices, payments and expenses name no ledger account; its chart and journal sit behind
its accounting reports. Expenses name a category, not the account the money left. **Model.** The book exists
(`book/id` = `freshbooks:saltmarsh`, `book/name` from `users/me`, `book/authority` the source) but has no
entries; observations only (`no-ledger`). The expense has no `obs/account`. **Options.** Read FreshBooks' general
ledger report as a second document per period; or leave Saltmarsh's books as a witness only. **Recommendation.**
Leave it. Saltmarsh's side matters to Tallowmere as evidence for bill SC-0057, and that needs observations, not
entries.

## Same event, both sides

No links to other sources are written; this is what a rule would match on:

| | QuickBooks invoice 1031 | Xero bill QF-1031 | FreshBooks invoice 57 | Xero bill SC-0057 |
|---|---|---|---|---|
| `obs/amount` | +2400.00 | −2400.00 | +362.18 | −362.18 |
| `obs/date` | 2026-03-04 | 2026-03-04 | 2026-03-11 | 2026-03-11 |
| `obs/reference` | `1031` | `QF-1031` | `0057` | `SC-0057` (Reference "Invoice 57") |
| `obs/counterparty` | Tallowmere Coffee Roasting Co. | Quillfeather Design Consultancy LLP | Tallowmere Coffee Roasting Co. | Saltmarsh Cupping Lab LLC |
| book name (via `doc/source` ← `book/authority`) | Quillfeather Design Consultancy LLP | Tallowmere Coffee Roasting Co. | Saltmarsh Cupping Lab LLC | Tallowmere Coffee Roasting Co. |

So: equal magnitude and opposite sign (each side signs from its own book), the same date, references equal once
reduced to their digits without leading zeros, and each side's counterparty equal to the other side's book name.
The book is reachable by refs alone (observation → `doc/source` → the book whose `book/authority` is that
source). The payments match the same way (QuickBooks 2026-03-18 +2,400.00 against Xero's ACCPAY payment;
FreshBooks 2026-03-24 +362.18 against Xero's pull-2 payment and bank line 9). The reference rule is the only
fuzzy part: Xero's supplier prefix and FreshBooks' zero padding mean exact text never matches.

## Proposed changes to shared files

1. **`jsonmirror.rs`, `Walker::array`** (bug fix): a list item without its own id is named `@<position>`, so it
   cannot collide with a real id.
   ```rust
   // was: let seg = local.unwrap_or_else(|| i.to_string());
   let seg = local.unwrap_or_else(|| format!("@{i}"));
   ```
   `xero.rs` `Interp::lines` builds the same key and changes with it
   (`map_or_else(|| format!("@{i}"), str::to_string)`). Component paths of id-less items change once.
2. **`jsonmirror.rs`, `JsonSource`**: a reference hook, consulted before `ref_ids` for objects and for scalar
   fields.
   ```rust
   /// Name the document a field refers to: (kind, id), or None when it is not a reference (or means none).
   /// Sees the field name and its value, so it can read the kind from the name (`CustomerRef`), from a
   /// sibling (`EntityRef.type`, `LinkedTxn.TxnType`) or from a table (`customerid`).
   pub ref_of: Option<fn(field: &str, v: &Json) -> Option<(String, String)>>,
   ```
   In `walk`, a scalar whose `ref_of` returns `Some` becomes a ref; in `reference`, an object whose `ref_of`
   returns `Some` becomes a ref and its other keys are reported as an embedded copy (the keys that named the
   kind and id excluded). Xero sets `None`. The QuickBooks and FreshBooks normalisers then go away.
3. **`jsonmirror.rs`, `JsonSource`**: `pub version: Option<&'static str>` (`SyncToken`, `updated`,
   `UpdatedDateUTC`). When the stored value on a root document is newer (numeric compare when both parse as
   integers, text otherwise), the mirror skips that document and reports `stale-read`.
4. **`core.json`**: one attribute.
   ```json
   {"ident": "tx/source-time", "type": "instant", "doc": "When the source says it produced the file (a response's own timestamp), beside tx/observed-at"}
   ```
5. **Crate layout**: move `sources/qbo/kit.rs` (document envelope, refer, commodity, book, observation, line,
   mirror entry, in-source link, `owns`) to `src/interp.rs` and use it from `xero.rs` too; today it is a copy of
   Xero's private helpers.
6. **Optional, `jsonmirror.rs`**: rename `JsonSource.when` to `parse`; add `derived: &'static [&'static str]`
   so computed fields are marked at install.
