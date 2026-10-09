# Amount

Money to the cent, in DM Mono with tabular figures; negatives in parentheses and coin-text. Every amount in ownpurse goes through Amount, so columns line up and negatives read the same everywhere.

- Right-align amounts in columns. Never round: the record keeps Xero's own digits, and so does the screen.
- Negatives default to parentheses, the bookkeeper's convention; `sign="minus"` uses a true minus (U+2212) for running text.
- `total` adds the single rule above and the double rule below. Use it only on a real total.
- `currency` shows the ISO code before the figure in `ink-3`, when a view mixes currencies.
