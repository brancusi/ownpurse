-- The mirror book's balances by account, summed exactly. dsum() is registered by `ownpurse-facts sql`;
-- SQLite's own sum() would turn the decimal text into floating point.
SELECT account, dsum(quantity) AS balance, count(*) AS postings
FROM postings
WHERE status = 'entry.status/posted'
GROUP BY account
ORDER BY account;
