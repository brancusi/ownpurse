-- The dinner's bank transaction in Xero, every fact ever asserted or retracted about it, with when and by whom.
-- The reference arrives in the second pull; nothing is overwritten.
SELECT h.tx, t.instant, h.op, h.attr, h.shown AS value, t.actor
FROM history h
JOIN transactions t ON t.tx = h.tx
WHERE h.e = (SELECT e FROM facts WHERE attr = 'doc/id'
             AND value = 'xero:tallowmere/BankTransaction/c0ffee05-0000-4000-8000-000000000003')
  AND h.attr NOT LIKE 'xero.%'
ORDER BY h.tx, h.op DESC, h.attr;
