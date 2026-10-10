# The landscape, October 2026

Who else is trying to get accounting data out of the systems that hold it, how they do it, and what that means for
ownpurse's connectors. Four research passes on 2026-10-10: unified accounting APIs; open-source, local-first and
connector frameworks; AI bookkeeping, close and capture; Xero's API and the hard problems of sync. Vendor figures
are the vendors' own. Anything marked **[unverified]** was not confirmed against a primary source. This is the
summary and the shortlist for the dossiers. The main claims are linked in the post
[A run on your data](https://ownpurse.com/blog/a-run-on-your-data/). The full map, 135 dossiers with their own
sources, the history and the synthesis are in [The landscape](https://ownpurse.com/blog/the-landscape/); the dossiers are
`site/src/content/landscape/`.

## The short version

- No one found combines four things: the record is **local**, it sits **beside** the ledger across several
  providers, the history is **tamper-evident and held by the owner**, and **writes go out only after a person
  approves them**. That is the space ownpurse is built for.
- The money went the other way. About half a billion dollars went into AI ledgers that replace QuickBooks or
  NetSuite (Digits, Rillet, Campfire, DualEntry, Light) in about a year. Almost none went into the owner holding a
  copy.
- The source owners are tightening access to bulk reads. Xero: usage fees on data egress from 2 March 2026, a ban
  on training AI with API data, a ban on bots that simulate users, and the Journals feed only on its Advanced tier
  with a security assessment. Intuit has metered QuickBooks reads since 28 July 2025. Banks now charge aggregators
  (JPMorgan and Plaid). GoCardless closed new sign-ups for bank data in July 2025.
- Vendors that held the books failed with the books inside: Bench (shut without notice, 27 December 2024),
  Botkeeper (wind-down, February 2026), Maybe Finance (archived, July 2025).
- None of the unified APIs (Codat, Merge, Rutter, Apideck and the rest) keeps history. They hold current state or
  nothing, and a delete, a correction or a pending-to-posted swap destroys what was there before.

## Who is doing what

| Segment | Who | What they keep | Writes | Relation to ownpurse |
|---|---|---|---|---|
| AI-native ledgers and ERPs | Digits, Puzzle, Rillet, Campfire, DualEntry, Light, Pennylane, Layer | Their own ledger, in their cloud | Autonomous | Ask you to move your books in; ownpurse leaves them where they are |
| Bookkeeping as a service | Pilot, Zeni, Finta; Bench and Botkeeper gone | Their copy, their log | Autonomous ("zero human intervention", Pilot) | The opposite pole |
| Agents beside the ledger | Basis, Booke, Docyt, Truewind, Double (Keeper), Ramp's accounting agent | The provider's ledger plus the vendor's change log | Direct, behind confidence thresholds | Same pattern, but closed and hosted, with no owner-held record |
| Close and analytics | Numeric, Fathom, Syft (Xero) | A synced copy, checked | Sign-off | Nearest in architecture; their copy is theirs |
| Capture | Dext, Hubdoc (Xero), AutoEntry (Sage) | Documents, pushed into the ledger | Rules | Sources for ownpurse, not rivals |
| Unified accounting APIs | Codat (now pivoting to banks), Merge, Rutter, Apideck, Unified.to, Chift, Maesn, Validis, FIS (Railz) | A mutable cache or nothing | Asynchronous, fire-and-forget | Not a competitor and not worth using as a source: a third party, lower fidelity, a cost |
| Bank aggregators | Plaid, MX, Finicity (Mastercard), Yodlee (now STG), SimpleFIN, GoCardless | Current transactions | n/a | Possible bank sources; SimpleFIN is the one a person can hold |
| Plain-text and local-first | Beancount and beangulp, hledger, Ledger, Firefly III, Actual, GnuCash, Paisa, Sure (Maybe's fork) | A journal file or a local database | Manual | Kin; they keep normalized entries and discard the raw record |
| Connector frameworks | Airbyte, Singer and Meltano, dlt, Fivetran, Nango, Steampipe | A history table and a final table, or a records cache | Varies | Their state and dedupe patterns are worth borrowing |
| Fact and event ledgers | TigerBeetle, Formance, Blnk, Medici, Datomic (Nubank), XTDB | Immutable transfers or datoms | Programmatic | Design cousins |
| Agent-native, 2025–26 | Bitwave Agentic (July 2026, local CLI and MCP, crypto first, feeds a hosted product), Digits MCP (read-only), Xero and Intuit MCP servers (direct writes), Meridian (Pilot), AccountsOS | Mostly nothing local | Read-only, or direct | Nearest neighbours; none has an approval gate or an owner-held log |

## How the segment works

- **A normalized common model is the product.** Raw source data is an optional side channel (`remote_data`, `_raw`),
  often for debugging only. ownpurse inverts this: the source's own attributes are facts, and a common model is a
  projection.
- **Poll underneath, webhooks on top.** Every vendor tells customers to poll as well, because webhooks get dropped.
  Xero's webhooks cover only contacts, invoices, credit notes, overpayments and prepayments, and need a public
  HTTPS endpoint, so for a local app polling is the backbone.
- **Two ids per record:** the vendor's and the source's (`remote_id`, `platform_id`).
- **Deletes are the weak point.** Rutter's deleted rows simply vanish; Merge sells full delete detection as a paid
  add-on (a full sync every 3 days); Nango infers deletes by diffing a full sync and only if the run completed.

## The hard problems, and where ownpurse stands

| Problem | How others handle it | ownpurse today | Open |
|---|---|---|---|
| Same movement seen by several sources | Fuzzy match on date, amount and account, marked for review (beangulp, Actual, GnuCash) | Sources never merged; link rules with a rule and a confidence | Reviewing links at scale; FITIDs a bank reissues |
| Importing twice | Source id; else hash of the raw line (Ledger); hledger's date high-water mark is fragile | Files by SHA-256 in the drawer; identity ladder of source id, natural key, fingerprint, position; reloading changes nothing | Two reads with the same `observed_at` and different content |
| Incremental sync | Per-stream cursor checkpointed per page (Airbyte, tap-xero); dedupe at the boundary by key (dlt) | `UpdatedDateUTC` high-water mark with a 5 s overlap, a full sweep every 7 days | Store the cursor as facts; checkpoint per page |
| Change detection lies | Fivetran: Xero does not bump `UpdatedDateUTC` for some edits, so it re-syncs fully | The weekly full sweep | Make the sweep cadence a per-connector setting |
| Deletes and voids | Soft-delete flags; Nango only trusts a completed sweep | Status change; old values retracted; `vanished` after a full sweep | `includeDeleted=true` on Xero bank transfers; the DELETED/VOIDED pass on incremental syncs too |
| History | Fivetran's history mode (SCD2) | Native: `as_of` and `history` on any database | Xero's own History records as facts, not a payload |
| Rounding and currency | Xero rounds tax per line | `rust_decimal` at the source's scale | Request `unitdp=4` **[unverified on a live org]** |
| Business dates | Organisation `Timezone` in Xero | Business date and raw text kept, instants in UTC | Derive `book/time-zone` from the organisation |
| Schema drift | Airbyte pauses on breaking changes | The schema only grows; raw bytes replay through a fixed projector | Report a vanished or retyped field as pressure |
| Partial failure | Save the cursor only after every page is applied (Plaid) | An entity in one store transaction; pages already fetched are lost on failure | Write each page to the drawer as it arrives, project, then advance the cursor |
| Write conflicts | Xero's Idempotency-Key lasts 6 minutes | Sealed packages; a readback marker as the real guard; `before_fp` drift check | No version precondition in Xero, so a race window remains |

## Xero, specifically

- **Ids** are GUIDs, unique within an organisation (not promised globally). Identity is (tenant, entity, id), as
  the record already keys it. Line item ids are not stable: an update that omits them recreates the lines. Manual
  journal lines have no ids.
- **Bank statement lines are not available.** Xero does not expose unreconciled statement data through its public
  APIs, and reconciling through the API is not supported. Only the Finance API returns statement lines, and it is
  open only to lending partners. Each transaction carries `IsReconciled`. The statement side must come from the
  bank directly (CSV, OFX, PDF, SimpleFIN), linked to Xero's bank transactions. Those sources are the independent
  witness, so we pull them as well as Xero, not instead of it.
- **Journals** (`offset` by JournalNumber, the nearest thing to an append-only feed) need the Advanced tier
  (A$1,445 a month), a security assessment and use-case approval. A person with their own app is on Starter
  (5 organisations, 1,000 calls a day): no Journals.
- **Attachments:** up to 10 per document, 10 MB each, on 11 kinds of object; `HasAttachments` says which documents
  to ask. The Files API reaches the inbox. Hubdoc has no API; its documents arrive as bills with attachments.
  Every fetch counts against the daily limit and egress.
- **Terms (December 2025):** no training on API data; no passing data to a third party without the user's
  consent; no bots that simulate users; an app needs its own terms and privacy policy. Sending Xero data to a
  hosted model is probably a passthrough that needs the user's consent **[unverified with Xero]**.
- **Already right in the connector:** granular read scopes (`accounting.invoices.read` and the rest), 429
  handling, `pageSize=1000`, the overlap and the weekly sweep.

## Decisions this forces

1. **The app model.** Each person registers their own Xero app (free Starter tier, their own credentials, no
   Journals) or ownpurse runs a shared, certified app (egress paid for every user, PKCE, Xero's approval). The first
   fits local-first and costs users nothing.
2. **Consent for agents.** A stated policy and an explicit consent step before an agent sends Xero data to a hosted
   model.
3. **The connector framework.** A declarative manifest per source (the streams, the identity and cursor of each,
   the raw shape) and a pure projector, as `ownpurse-ingest` already is. The fetch writes pages to the drawer as they
   arrive, the cursor lives in the log as facts, and only a completed, verified sweep may retract.

## Borrow and avoid

- Borrow: beangulp's importer shape (identify, extract, archive the original); "mark, don't drop" for fuzzy
  duplicates; Airbyte's history table and final table (our log and its projections); Nango's rule that only a
  completed sweep may infer a delete; Plaid's added, modified and removed patches (assert and retract);
  TigerBeetle's content-derived ids; XTDB's two times (when it happened, when we learned it); Blnk's
  reconciliation rules as first-class objects; ownpurse as an MCP server over the record.
- Avoid: hashing after mapping (Firefly III), dates as identity (hledger), deleted rows that vanish (Rutter),
  trusting source ids alone (banks reissue FITIDs; Plaid churns ids when a pending item posts).

## Dossier shortlist

Ranked by what they teach the connector and the positioning:

1. **Bitwave Agentic**: the nearest framing (local, agent-native); what is open, what is hosted.
2. **Numeric**: the nearest architecture (a synced copy, checked); how it reconciles.
3. **Basis**: agents beside the ledger with approval, sold to firms.
4. **Digits**: the AI ledger with a read-only MCP; why read-only.
5. **Nango**: syncs, deletion detection, records cache.
6. **Airbyte and tap-xero**: the Xero streams, cursors and state in practice.
7. **Beancount and beangulp**: importer interface, dedupe, archive.
8. **Plaid and SimpleFIN**: the bank side, the cursor design, what a person can hold.
9. **Xero's own MCP server and JAX**: what the incumbent lets agents do.
10. **Merge**: `remote_data`, soft deletes, the agent handler.
