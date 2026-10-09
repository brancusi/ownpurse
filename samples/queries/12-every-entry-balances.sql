-- A check: every posted entry in every mirrored book sums to zero, per commodity. Any row here is an entry that
-- does not balance; an empty result is a pass.
SELECT book, entry, commodity, dsum(quantity) AS off_by, count(*) AS postings
FROM postings
WHERE status = 'entry.status/posted'
GROUP BY book, entry, commodity
HAVING dsum(quantity) + 0 <> 0
ORDER BY book, entry;
