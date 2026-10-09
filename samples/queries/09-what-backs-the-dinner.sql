-- Everything that backs the 186.40 dinner in the books, from every source: the posting's entry mirrors Xero's
-- bank transaction; the link rules joined it to the bank's CSV line, OFX line and statement line (same-event)
-- and to the receipt photo (evidences). Each witness is shown with the file it came from and where in it.
WITH RECURSIVE
  start(e) AS (SELECT e FROM facts WHERE attr = 'doc/id'
               AND value = 'xero:tallowmere/BankTransaction/c0ffee05-0000-4000-8000-000000000003'),
  ends(a, b) AS (
    SELECT f.value, t.value FROM facts l
      JOIN facts f ON f.e = l.e AND f.attr = 'link/from'
      JOIN facts t ON t.e = l.e AND t.attr = 'link/to'
     WHERE l.attr = 'link/kind' AND l.value IN ('link.kind/same-event', 'link.kind/evidences')),
  witness(e) AS (
    SELECT e FROM start
    UNION SELECT b FROM ends JOIN witness ON a = witness.e
    UNION SELECT a FROM ends JOIN witness ON b = witness.e)
SELECT o.kind, o.date, o.amount, src.value AS source,
  (SELECT group_concat(DISTINCT fl.names) FROM facts r JOIN facts rf ON rf.e = r.value AND rf.attr = 'region/file'
     JOIN files fl ON fl.e = rf.value WHERE r.e = w.e AND r.attr = 'obs/region') AS file,
  (SELECT group_concat(DISTINCT lc.value) FROM facts r JOIN facts lc ON lc.e = r.value AND lc.attr = 'region/locator'
     WHERE r.e = w.e AND r.attr = 'obs/region') AS place
FROM witness w
JOIN observations o ON o.e = w.e
LEFT JOIN facts ds ON ds.e = coalesce((SELECT value FROM facts WHERE e = w.e AND attr = 'obs/doc'), w.e)
  AND ds.attr = 'doc/source'
LEFT JOIN facts src ON src.e = ds.value AND src.attr = 'source/id'
ORDER BY o.kind, source;
