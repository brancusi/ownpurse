-- One invoice, two books: the supplier's invoice (QuickBooks, FreshBooks) and the roastery's bill (Xero),
-- joined by the counterpart rule. Opposite amounts, the same digits in the reference.
SELECT a.id AS issued, a.amount, a.date, b.id AS received, b.amount, b.date, l.confidence
FROM links l
JOIN observations a ON a.id = l.from_id
JOIN observations b ON b.id = l.to_id
WHERE l.kind = 'link.kind/counterpart';
