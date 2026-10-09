-- The schema is facts too: attributes by namespace, and the transaction that installed each group.
SELECT substr(a.ident, 1, instr(a.ident, '/') - 1) AS namespace, count(*) AS attributes, min(d.tx) AS first_tx
FROM attr a
JOIN datom d ON d.e = a.id AND d.a = 1
GROUP BY namespace
ORDER BY first_tx, namespace;
