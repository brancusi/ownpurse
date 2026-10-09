# TAccount

A T-account: the account on the brow rule, debits left of the stem, credits right, totals on double rules, and the balance below. The T-account is the Mark's own shape at work, and the clearest way to show one account's movements.

- Provide `title`, optional `code`, and `debits` and `credits` as `{label, amount}` lists. The component totals both sides and states the balance beneath: Debit balance or Credit balance with the amount, or Nil balance with a balanced coin. Short and Over are for reconciliations, not for an account's normal balance.
- Keep labels short (a date and a counterparty). For more than eight lines a side, use LedgerTable.
- It sits on `surface-0` or `surface-1`; never inside a Sheet's title block.
