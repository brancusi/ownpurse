# LedgerTable

Rows of postings across sources and books, with debit and credit columns, totals on double rules and a balance check. - Each row: `date`, `source`, `book`, `account`, and a `debit` or `credit`. Mark the row being discussed with `selected` (lapis marker) and a row that causes a difference with `difference` (coin-tint).
- The totals row checks itself: Balanced, or Short or Over with the amount.
- Dates are ISO (2026-09-03) or month-day within a known year (09-03). Account names lead with their code: "6420 Repairs".
- On narrow screens the table scrolls inside its own frame; the page never scrolls sideways.
