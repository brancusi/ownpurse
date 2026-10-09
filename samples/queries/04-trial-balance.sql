-- Each mirrored book's balances by account, summed exactly. The log holds more than one company's books (the
-- roastery's Xero, Quillfeather's QuickBooks), so a trial balance is always per book. dsum() is registered by
-- `ownpurse-facts sql`; SQLite's own sum() would turn the decimal text into floating point.
SELECT book, account, dsum(quantity) AS balance, count(*) AS postings
FROM postings
WHERE status = 'entry.status/posted'
GROUP BY book, account
ORDER BY book, account;
