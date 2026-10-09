-- Time travel in plain SQL: an invoice's facts as the log knew them right after transaction 400 (before the second
-- Xero pull). A fact is true at T if it was asserted at or before T and not retracted at or before T.
SELECT d.attr, d.value, d.tx
FROM history d
WHERE d.e = (SELECT e FROM facts WHERE attr = 'obs/reference' AND value = 'INV-0415')
  AND d.op = 'assert' AND d.tx <= 400
  AND NOT EXISTS (SELECT 1 FROM history r
                  WHERE r.e = d.e AND r.attr = d.attr AND r.value IS d.value
                    AND r.op = 'retract' AND r.tx > d.tx AND r.tx <= 400)
  AND d.attr IN ('obs/amount', 'obs/status', 'xero.Invoice/Total', 'xero.Invoice/UpdatedDateUTC')
ORDER BY d.attr;
