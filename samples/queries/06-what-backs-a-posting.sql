-- From a posting in the books back to the file it came from: posting → entry → the document it mirrors →
-- every file that carried a version of that document.
SELECT p.entry, p.account, p.quantity, l.kind AS link, f.names AS file, f.sha256
FROM postings p
JOIN facts en ON en.attr = 'entry/id' AND en.value = p.entry
JOIN links l ON l.from_id = p.entry
JOIN facts dd ON dd.attr = 'doc/id' AND dd.value = l.to_id
JOIN facts df ON df.e = dd.e AND df.attr = 'doc/file'
JOIN files f ON f.e = df.value
WHERE p.account = 'Meals and entertainment'
ORDER BY p.entry, f.names;
