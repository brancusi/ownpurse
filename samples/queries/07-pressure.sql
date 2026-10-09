-- Where the sources did not fit the model, as the projectors reported it (also: `ownpurse-facts pressure`).
SELECT substr(value, 1, instr(value, ' | ') - 1) AS kind, count(*) AS times, min(value) AS example
FROM facts
WHERE attr = 'tx/pressure'
GROUP BY kind
ORDER BY times DESC;
