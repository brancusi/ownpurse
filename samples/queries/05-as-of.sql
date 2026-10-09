-- Time travel in plain SQL: INV-0415's facts as the log knew them right after the last transaction read before the
-- second Xero pull (observed 2026-04-02). A fact is true at T if it was asserted at or before T and not retracted
-- at or before T. `--as-of T` does the same for every command.
WITH t(at) AS (SELECT max(e) FROM facts WHERE attr = 'tx/observed-at' AND value < '2026-04-02')
SELECT d.attr, d.value, d.tx
FROM history d, t
WHERE d.e = (SELECT e FROM facts WHERE attr = 'obs/reference' AND value = 'INV-0415')
  AND d.op = 'assert' AND d.tx <= t.at
  AND NOT EXISTS (SELECT 1 FROM history r
                  WHERE r.e = d.e AND r.attr = d.attr AND r.value IS d.value
                    AND r.op = 'retract' AND r.tx > d.tx AND r.tx <= t.at)
  AND d.attr IN ('obs/amount', 'obs/status', 'xero.Invoice/Total', 'xero.Invoice/UpdatedDateUTC')
ORDER BY d.attr;
