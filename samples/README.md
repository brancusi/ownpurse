# Samples: one month, every kind of source

Synthetic evidence for the fact model. Every name, number, address and id here is invented. One business, one
month (March 2026), seen through every kind of source ownpurse has to read, so the same events show up more than
once and the model has to say how the witnesses relate.

`ingest.json` lists every file in the order a person would bring them in. Load it all with:

```sh
cd rust
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite init
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite load ../samples/ingest.json
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite link
cargo run -p ownpurse-facts-cli -- --log /tmp/march.sqlite sql -f ../samples/queries/09-what-backs-the-dinner.sql
```

The files load in the order they were read (`observed_at`), across sources. Loading them again changes nothing.

`queries/` holds SQL you can run against the result; `docs/plans/FACTS-EDGES.md` is what loading all of this
taught us about the model, with one write-up per source family in `docs/plans/edges/`.

## The world

**Tallowmere Coffee Roasting Co.** keeps its books in **Xero** (base currency USD). It banks with **Ridgeway
Federal Bank**: operating checking ending **4821**, savings ending **9067**. Two of its suppliers keep their own
books in other systems and appear here too, because the same invoice exists on both sides.

| Source id | What | Files |
|---|---|---|
| `xero:tallowmere` | The roastery's Xero org | `xero/pull-1/*.json` (2026-03-20), `xero/pull-2/*.json` (2026-04-02) |
| `bank:ridgeway-4821` | Operating checking | `bank/activity-2026-03-20.csv` and `bank/activity-2026-04-02.csv` (overlapping downloads) + mapping, `bank/2026-03.ofx`, `bank/statement-2026-03.pdf` + extraction, `bank/check-1042.png` + extraction |
| `receipts:tallowmere` | Phone photos of receipts | `receipts/copper-pot-2026-03-12.jpg` + extraction |
| `qbo:quillfeather` | Quillfeather Design Consultancy LLP's QuickBooks Online | `qbo/*.json` |
| `freshbooks:saltmarsh` | Saltmarsh Cupping Lab LLC's FreshBooks | `freshbooks/*.json` |
| `sheet:tallowmere-budget-2026` | The 2026 budget, a Google Sheet | `sheets/budget-2026.values.json` + formulas + mapping; a second read after edits |
| `xlsx:ridgeway-loan-2207` | The lender's amortisation schedule for the roaster loan | `xlsx/loan-2207-schedule.xlsx` + mapping |
| `eth:0x51ab7c3e` | The roastery's Ethereum wallet `0x51ab9e04c2d7f3a1b6e85d20c4f9a7136b0e7c3e` | `eth/*.rpc.json` (JSON-RPC responses), `eth/token-list.json`, `eth/wallet.json`, `eth/prices-2026-03.csv` |

## The month

Operating account 4821. Opening balance on 2026-03-01: **24,310.55**. Closing on 2026-03-31: **13,413.22**.

| # | Bank posting date | Bank text | Amount | In Xero | Other witnesses |
|---|---|---|---|---|---|
| 1 | 03-02 | DEPOSIT OSTLEY JOINERY | +1,846.13 | Payment of INV-0412 (dated 02-16) | CSV, OFX, statement |
| 2 | 03-06 | HARBOR STREET COFFEE | −4.75 | Spend money, 6100 | CSV, OFX, statement (two identical lines the same day) |
| 3 | 03-06 | HARBOR STREET COFFEE | −4.75 | Spend money, 6100 | as above |
| 4 | 03-09 | CHECK 1042 | −3,215.47 | Payment of bill BO-7781 (Brindle & Oake Supply, 03-02) | CSV, OFX, statement, check image (written 03-03) |
| 5 | 03-13 | COPPER POT BISTRO | −186.40 | Spend money dated 03-12, 6100 | CSV, OFX, statement, receipt photo (subtotal 152.80, tax 13.60, tip 20.00) |
| 6 | 03-16 | TRANSFER TO SAV 9067 | −5,000.00 | Bank transfer 1010 → 1020 | CSV, OFX, statement |
| 7 | 03-18 | QUILLFEATHER DESIGN ACH | −2,400.00 | Payment of bill QF-1031 (03-04) | CSV, OFX, statement; QuickBooks invoice 1031 and its payment |
| 8 | 03-20 | LARKSPUR PROPERTY RENT | −2,750.00 | Spend money, 6200 | CSV, OFX, statement |
| 9 | 03-24 | SALTMARSH CUPPING LAB | −362.18 | Payment of bill SC-0057 (03-11), pull 2 only | CSV, OFX, statement; FreshBooks invoice 57 and its payment |
| 10 | 03-25 | STRIPE PAYOUT | +2,318.06 | Receive money, 4000 | CSV, OFX, statement |
| 11 | 03-28 | RIDGEWAY LOAN PMT 2207 | −1,126.84 | Spend money split: 2500 principal 1,004.12, 6500 interest 122.72 | CSV, OFX, statement; the loan schedule's March row |
| 12 | 03-31 | MONTHLY SERVICE FEE | −12.50 | Spend money, 6300 | CSV, OFX, statement |
| 13 | 03-31 | INTEREST PAID | +1.37 | Receive money, 4900 | CSV, OFX, statement |

Off the bank account:

- **INV-0415** to Ostley Joinery (03-10): 12 × 71.40 + 8.5 % tax = 929.63 in pull 1; edited to 14 units,
  1,084.57, by pull 2.
- **INV-0416** to Ostley Joinery (03-12): 412.08, authorised in pull 1, voided by pull 2.
- **INV-0418** to Fernhill Grocers (03-19): 1,250.00, no tax. Paid on 03-21 in **USDC** to the roastery's
  wallet. Xero never hears about it: the payment exists only on chain.
- **Depreciation** (manual journal, 03-31, pull 2): 6700 Dr 412.08 / 1510 Cr 412.08.
- The dinner's bank transaction gains a reference ("Client dinner, Ostley Joinery") between the pulls.
- **Wallet** `0x51ab…7c3e`: 0.05 ETH arrives from an exchange on 03-02; 1,250.00 USDC from Fernhill on 03-21;
  1,000.00 USDC goes back to the exchange on 03-26, paying gas in ETH.
- **Budget**: twelve months by category, with a total column computed by formula.
- **Loan 2207**: 2.40 % a year, monthly payment 1,126.84; 63,362.24 owed at the start of 2026, 61,360.00 before
  the March payment, which splits 1,004.12 principal and 122.72 interest.

## Chart (Xero)

1010 Operating account (bank, …4821) · 1020 Reserve savings (bank, …9067) · 1200 Accounts receivable · 1500
Roasting equipment · 1510 Accumulated depreciation · 2000 Accounts payable · 2200 Sales tax payable (system GST)
· 2500 Equipment loan · 4000 Wholesale coffee sales · 4900 Interest income · 5000 Green coffee · 6100 Meals and
entertainment · 6200 Rent · 6300 Bank fees · 6400 Design and branding · 6500 Interest expense · 6600 Cupping and
quality · 6700 Depreciation

## Ids

Xero ids are synthetic GUIDs of the form `c0ffee<kk>-0000-4000-8000-<n>`, listed in
`rust/scripts/uuid-allow.txt`. QuickBooks and FreshBooks ids are small integers, as those APIs use.
