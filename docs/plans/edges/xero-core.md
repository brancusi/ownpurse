# Edges: the core and Xero

What building the log, the store, the ingest API and the Xero projector showed about the model. Each finding
says what happened, what we did, and what is still open. Three of them were found by tests, not by reading:
that is the case for writing the laws first.

## Found by the laws

**References are declared by field name, not guessed from an id.** The first mirror treated any object holding
a known id field (`AccountID`, `ContactID`) as a reference to that entity and kept only the id. A Xero journal
line holds an `AccountID` and is still a line: its amount and description were dropped. The round-trip law
(rebuild every document from its facts and compare with the source) caught it on the depreciation journal. The
`JsonSource` config now names reference fields (`("Contact", "Contact", "ContactID")`); an object anywhere else
is a component, so nothing is lost by guessing. QuickBooks needs the same rule for a different reason: all its
refs carry the id in a field called `value`, so only the field name (`CustomerRef`) says what they point at.

**Owned lists lose their order unless it is a fact.** A tracking category's options are entities of their own
(they have ids and other documents point at them), held in an ordered list. Facts are a set, so the order went
with them. The law caught it; the mirror now writes `<ns>/index` on every list item, owned or component. The
general rule: **order is data**. Any list whose order matters (lines, legs, options) carries its position as an
attribute.

**An older read must never overwrite a newer one.** Replaying the whole manifest on a log that already held the
second pull re-read the first pull's invoices and reverted the edited invoice to its old quantity. A snapshot
source's read is only as good as its moment. Fixed: before diffing a document, the mirror finds when it was last
read (the latest `tx/observed-at` of the transactions that cited a file carrying it) and skips it, with a
`stale-read` pressure, if this read is older. This makes the order of ingest irrelevant for snapshot sources,
which is what replay needs. Open: two reads with the same `observed_at` but different content (clock skew, two
connectors); we take the later transaction today.

## Found by building

**A transaction cannot use an attribute it installs.** The log's second transaction installs the core
vocabulary, so it cannot say who wrote it (`tx/actor` is in that vocabulary). Datomic has the same rule. Every
projector that needs new attributes therefore writes two transactions: the install (now stamped with the
projector and the file that needed it) and the facts. The ingest plan is made twice, once before and once after
the install, so lookups see the new attributes. Acceptable; it also makes schema growth easy to audit
(`samples/queries/08-schema-growth.sql`).

**Document and observation are roles, not types.** A Xero bank transaction is a document (Xero's record, with
its own id and versions) and an observation (money moved). One entity carries both `doc/id` and `obs/id`, two
identities with the same value. That works: the entity is an open map, Clojure style, and a "kind" is just which
attributes are present. The cost is in reading: SQL views need one name per entity, so the `label` view ranks
identities. Recommendation: keep one entity; consider making `obs/id` optional (an observation is anything with
`obs/kind`) so the duplicate identity goes away.

**Embedded copies are dropped, on purpose.** A Xero invoice carries `Contact: {ContactID, Name}`: a copy of the
contact as of the invoice. Storing the copy's `Name` on the contact entity would flip it back and forth between
what the invoice says and what the Contacts collection says, on every read. The mirror stores the reference
only and reports `embedded-copy`; the bytes stay in the drawer. The interpretation does use the embedded name,
as `obs/counterparty` on the invoice ("who the invoice was to, as written on it"), which is the right place for
a copy.

**Null, empty and absent read the same.** Facts have no null: a field that is `null` or `[]` is simply not
asserted. For accounting data we found no case where the difference matters, but the round-trip law has to
normalise it away, and a source that means something by an empty list (a cleared set of tracking options) would
need an explicit fact for "cleared".

**Positions are weak identities.** Journal lines, tax components and addresses have no ids, so the mirror names
them by position (`…/JournalLines/0`). Inserting a line renames every line after it, and the snapshot diff then
retracts and re-asserts more than changed. Reported as `position-identity`. Options: a content hash as identity
(stable under reordering, but an edited line becomes a new line), or accept it. Recommendation: accept for
mirrored sources (the history is still exact); revisit if a source edits lines in place often.

**Re-reading writes a transaction even when nothing changed.** The new file is new evidence: each document gains
the file in `doc/file`. So "Xero said the same thing on 2 April" is in the log, which is what `stale-read` relies
on. The cost is one fact per document per read. Recommendation: keep it; it is the "last confirmed" record an
auditor asks for.

**Reference by a code that is unique only within one org.** Xero lines name their account by `AccountCode`, not
`AccountID`. The projector resolves the code at projection time among the accounts this source already
described, so **Accounts must be ingested first**; otherwise it reports `unresolved-ref` and the mirror entry
lacks that posting's account. A projection that depends on what else is in the database breaks the promise that
replay gives the same log in any order. Recommendation: move code resolution into a derivation (a rule over the
log, like the link rules) so projectors only depend on their own file. The same applies to the sales-tax account
(found by `SystemAccount = GST`).

**One event, two accounts.** A bank transfer moves money between two accounts, but `obs/account` holds one. It
became a parent observation with two legs (`obs.kind/transfer-leg`), each on its own account. The same shape
fits a crypto swap and a card payment with a fee. Recommendation: make "an event with legs" the general rule
and say so in the vocabulary.

**One event, two documents, in one source.** Xero writes a bank transaction for each side of a transfer as well
as the transfer itself. The link rule's notion of a *stream* (source and document kind) handles it: the same
event shows up once per stream, so the transfer legs link to the transfer's legs, and two identical coffees in
one stream stay two events.

**The currency is often implied.** Xero payments and transfers name no currency; it is the bank account's. The
link rule falls back to `account/commodity`. Recommendation: the interpretation should write `obs/commodity`
from the account so every money observation has one.

**Business dates arrive as midnight instants, twice.** Xero sends `Date: /Date(1773273600000+0000)/` and
`DateString: 2026-03-12T00:00:00` for the same date. The mirror keeps both (they are what Xero said) and stores
a midnight business date as a date, not an instant. The interpretation reads `DateString`.

**Money in SQL needs care.** SQLite's `sum()` turns decimal text into floating point. The store registers
`dsum()` for exact sums; the `sqlite3` shell does not have it. Recommendation: views that total money should be
computed in Rust, and the hand-written queries in `samples/queries` use `dsum`.

## Still open

- Mirror entries are claims stamped by the projector, but they depend on other documents (account codes, the
  tax account). See the code-resolution recommendation above; it touches decision D2.
- The `as_of` used by the CLI replays the log up to a transaction; fine at sample scale, a persisted index is
  phase M6.
- Projector versions: a fix to the mirror (like the two above) changes what a replay produces. The log keeps the
  old projector's facts; a reprocess under the new version is a new transaction with `tx/projector` saying so.
  We have not yet written the reprocess command.
