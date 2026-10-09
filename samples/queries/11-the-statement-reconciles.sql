-- A check, not a fact: the statement states an opening and a closing balance; every stream of bank lines for the
-- same account (the CSV downloads, OFX, the statement's own lines) must carry the opening to the closing. A pending
-- line that lapsed is not money that moved. `difference` is exact (dsum) and should be 0.
WITH bal AS (
  SELECT f.value AS account, o.date, o.amount
  FROM observations o JOIN facts f ON f.e = o.e AND f.attr = 'obs/account'
  WHERE o.kind = 'obs.kind/balance'),
period AS (
  SELECT account, min(date) AS start, max(date) AS finish,
    (SELECT amount FROM bal b WHERE b.account = p.account ORDER BY date LIMIT 1) AS opening,
    (SELECT amount FROM bal b WHERE b.account = p.account ORDER BY date DESC LIMIT 1) AS closing
  FROM bal p GROUP BY account),
moves AS (
  SELECT f.value AS account, o.date, o.amount, t.projector
  FROM observations o
  JOIN facts f ON f.e = o.e AND f.attr = 'obs/account'
  JOIN history h ON h.e = o.e AND h.attr = 'obs/kind' AND h.op = 'assert'
  JOIN transactions t ON t.tx = h.tx
  WHERE o.kind = 'obs.kind/money-movement'
    AND NOT EXISTS (SELECT 1 FROM facts s WHERE s.e = o.e AND s.attr = 'obs/status'
                    AND s.value = 'obs.status/lapsed'))
SELECT m.projector, p.opening, dsum(m.amount) AS moved, p.closing, count(*) AS lines,
  (SELECT dsum(x) FROM (SELECT p.opening AS x UNION ALL SELECT amount FROM moves m2
     WHERE m2.account = p.account AND m2.projector = m.projector AND m2.date BETWEEN p.start AND p.finish
     UNION ALL SELECT CASE WHEN p.closing LIKE '-%' THEN substr(p.closing, 2)
                                   ELSE '-' || p.closing END)) AS difference
FROM period p JOIN moves m ON m.account = p.account AND m.date BETWEEN p.start AND p.finish
GROUP BY m.projector
ORDER BY m.projector;
