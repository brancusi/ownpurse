# The model

How ownpurse keeps what its sources say, and how that becomes books. The why is in [PHILOSOPHY.md](PHILOSOPHY.md);
the working state is in [plans/NEXT.md](plans/NEXT.md).

## The words

| Word | Meaning |
|---|---|
| Entity | One business or household: a legal entity. One Xero organisation is one entity's books. |
| Book | The record of one entity, or of the group. Accounts, entries and postings belong to a book. |
| Group | The entities you own, taken together. The **group book** belongs to no single entity. |
| Source | Where data comes from: Xero, Mercury, QuickBooks, Quicken, a folder of receipts. |
| System of record | The one source a book's entries come from. Each book has exactly one. |
| Evidence | Every other source for a book: used to reconcile and to match, never posted. |
| Source account | Something a source holds that can belong to an entity: a Xero organisation, a Mercury account, a credit card. Every source lists its own in one shape. |
| Link | Says which entity a source account belongs to, and how: as the entity's system of record, or as evidence. |
| Group system of record | ownpurse itself, for what no single entity's system holds: intercompany, eliminations, group reports, matches across sources, and history. |
| Intercompany | A transaction between two of your entities. It appears in both books, on opposite sides (due to and due from), and is eliminated in the group book. |

Mercury is not a book. Its statements are evidence for a bank account in an entity's book (`1010 Operating
account`), or the system of record of a book no accounting system keeps, such as a household.

## Two systems of record

```
ownpurse
  group book ............. system of record: ownpurse
    intercompany matches, eliminations, group reports
       ▲              ▲              ▲
  entity book    entity book    entity book ... system of record: Xero (ownpurse holds a copy)
       ▲              ▲              ▲
  Xero org       Xero org       Xero org
```

An entity's books stay official in Xero: that is what the accountant and the tax filings use, and ownpurse's copy
must equal Xero's trial balance to the cent. ownpurse never edits that copy. When ownpurse finds something an
entity's books should change, such as a missing intercompany entry, the fix goes to Xero as a sealed package a
person approves, and comes back on the next fetch.

## Entities and their sources

ownpurse keeps the entities you own, and links each source account to the entity it belongs to. Three tables, none
of which knows any one source:

```
entities                 source_accounts                         entity_links
  _id   entity:<slug>      _id           the source row's id       _id           link:<source account id>
  name                     source        mercury | xero | ...      entity        entity:<slug>
  kind  business |         source_table  mercury.accounts          account       a source account
        household          org           the provider's org id     role          system-of-record | evidence
                           kind          org | bank | credit-card  evidence_for  an account in the entity's
                           name, code, currency, status                          system of record (optional)
```

- **Entities** are made through the API. Their names live only in the record, never in this repository.
- **Source accounts** are written by each source's component when it pulls, from its own rows: a Xero organisation
  and its bank accounts (`xero.organisations`, `xero.accounts` of type BANK), a Mercury organization, its deposit
  accounts and credit card accounts. A source account's `_id` is the id of the source row it comes from, and
  `source_table` says where that row is, so its every field is one join away. Adding a source means mapping its own
  accounts into this shape; nothing else changes.
- **Links** are made and ended through the API. A link's id comes from its account, so an account belongs to at
  most one entity at a time. Ending a link is a `DELETE` from the day it ends, which keeps its history: what was
  linked to what, and when, is a query (`FOR VALID_TIME AS OF`).

The rules, checked on every write:

1. A `system-of-record` link is an organisation (kind `org`), and an entity has at most one at a time.
2. An `evidence` link is an account (any kind but `org`). Its `evidence_for`, when given, is an account of the
   organisation the entity's system of record is: the Mercury account is evidence for Xero's `1010 Operating
   account`.
3. An account linked to one entity is not linked to another until its link ends.

| You say | Link |
|---|---|
| This Xero organisation is Tallowmere's books | Xero org, `system-of-record` |
| This Mercury account is Tallowmere's operating account | Mercury account, `evidence`, `evidence_for` Xero's 1010 |
| This Mercury card is Tallowmere's | Mercury credit card account, `evidence`, `evidence_for` Xero's card account |
| This Mercury account is the household's | Mercury account, `evidence` (no system of record yet) |

A book no accounting system keeps, such as a household, has no `system-of-record` link: ownpurse will be its system
of record, posting its accounts' lines with shared bank rules, when those rules exist. Until then its accounts are
linked as evidence.

## Bank lines and the compare

`bank_lines` is the first documents table: every line on a bank or credit card account, from every source, in one
shape (`account`, `date`, `amount`, `status`, `counterparty`, `description`, `reference`, and `source_row` back to
the source row). Money in is positive, from the account's side. Each source derives its own on every pull, from
every row it holds:

- **Xero:** each authorised bank transaction (a transfer's two legs among them) and each authorised payment, on the
  account it names. A bank transfer is no line of its own: its legs are bank transactions already, and counting
  both would count every transfer twice.
- **Mercury:** each pending or sent transaction, on its deposit account or its credit card account, dated by the
  day it posted (UTC), or the day it was made while pending. A failed, cancelled, reversed or blocked one moved no
  money.

A line that no longer holds (a payment deleted in Xero, a transaction cancelled at Mercury) ends by `DELETE`; its
history stays.

The compare (`GET /compare`) reads the links: for each evidence link with an `evidence_for`, the lines of the
account against the lines of the account it backs up. Two lines of the same amount at most a few days apart
(default 5) are the same money; each line matches at most one. It lists what is missing from the book, what is
missing from the evidence, and the evidence's pending lines apart, with each side's total. Lines just outside the
range are read too, so a line near its edge still finds its match.

## Three layers

Each layer is derived from the one below and never writes past it.

```
1 Sources                 2 Documents                     3 Ledger
  one schema per provider   one table per kind of thing     double entry
  ─────────────────────     ───────────────────────────     ─────────────
  xero.invoices     ─┐      parties                         books, sources
  xero.contacts      │      invoices    (bills: payable)    entries
  xero.payments      ├─map─►payments                  ─────►postings
  mercury.transactions      bank_lines                      the check: trial balance
  mercury.invoices  ─┘      accounts                          equals the source's
  qbo.invoices …            links (same-as, settles)
```

1. **Sources** keep each item exactly as the provider sent it, one row per item, in a schema named for the provider
   (`xero.invoices`). Only that provider's component reads them. A new version of an item is a new version of its
   row, valid from when the provider says it changed; nothing is deleted.
2. **Documents** hold the same kind of thing from every source in one table, with `book`, `source` and
   `source_row`. A client asks here for every open invoice, whoever issued it. A bill is a payable invoice; a credit
   note is an invoice of kind `credit_note`. The same invoice seen by two sources is two rows joined by a `same-as`
   link, never merged.
3. **Ledger** entries come only from a book's system of record. Where that is an accounting system, its own rules
   post from its source rows, so the trial balance can match its report. Where there is none, shared bank rules post
   from `bank_lines`. The group book's elimination entries are written by ownpurse.

## Example (synthetic)

Tallowmere invoices Fernhill Grocers 1,250.00 using Mercury's invoicing, records the same invoice in Xero as
INV-0418, and is paid by ACH into Mercury. Xero is the book's system of record; Mercury is evidence.

| Layer | Table | Row |
|---|---|---|
| source | `mercury.invoices` | Mercury's invoice, as sent: paid |
| source | `xero.invoices` | Xero's INV-0418, as sent: AUTHORISED |
| source | `mercury.transactions` | the ACH, 1,250.00 |
| document | `invoices` | source mercury: receivable, paid |
| document | `invoices` | source xero: receivable, INV-0418, open |
| document | `bank_lines` | source mercury: Operating account, 1,250.00 |
| link | `links` | same-as: the two invoices are one |
| link | `links` | settles: the bank line pays INV-0418 |
| ledger | `entries`, `postings` | from `xero.invoices` only: 1200 debit, 4000 credit, 1,250.00 |
| difference | a query | paid in Mercury, open in Xero: record the payment in Xero |

## Names in the store

- Source tables: `<provider>.<resource>` in kebab case as SQL folds it (`xero.bank_transactions`). Row ids:
  `<provider>:<org>:<kind>:<the provider's id>`.
- Documents and ledger tables: the `public` schema, plain names (`invoices`, `postings`), as are `entities`,
  `source_accounts` and `entity_links`.
- Columns: plain, snake case in SQL, no namespace (`invoice_number`, `org`).
