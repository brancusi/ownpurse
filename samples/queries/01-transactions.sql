-- Every transaction in the log: who wrote it, with which projector, from how many files, and what it changed.
SELECT tx, instant, actor, projector, files, asserted, retracted
FROM transactions
ORDER BY tx;
