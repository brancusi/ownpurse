-- Every top-level observation, in date order: what each source said happened.
SELECT kind, date, amount, commodity, account, counterparty, doc
FROM observations
WHERE parent IS NULL
ORDER BY date, kind;
