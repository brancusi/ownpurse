// The fact model as the canvas draws it: one node per kind of thing in the log, one edge per reference attribute.
// Attribute names are the log's own (`ownpurse-store` schema facts); counts and examples are queried live.

export type Layer = 'drawer' | 'documents' | 'observations' | 'links' | 'books' | 'log';

export const LAYERS: { id: Layer; n: string; label: string }[] = [
	{ id: 'drawer', n: '1', label: 'Drawer' },
	{ id: 'documents', n: '2', label: 'Documents' },
	{ id: 'observations', n: '3', label: 'Observations' },
	{ id: 'links', n: '4', label: 'Links' },
	{ id: 'books', n: '5', label: 'Books' },
	{ id: 'log', n: '0', label: 'The log' },
];

export interface Kind {
	id: string;
	label: string;
	layer: Layer;
	x: number;
	y: number;
	/** The attributes drawn on the node; refs are drawn in lapis. */
	attrs: string[];
	refs: string[];
	what: string;
	why: string;
	identity: string;
	/** How many of this kind the month holds. */
	count: string;
	/** What to open in the console. */
	query: string;
	queryTitle: string;
	empty?: boolean;
	wide?: boolean;
}

const C = 300; // column pitch

export const KINDS: Kind[] = [
	{
		id: 'source',
		label: 'Source',
		layer: 'drawer',
		x: 0,
		y: 0,
		attrs: ['source/id', 'source/kind', 'source/name'],
		refs: [],
		what: 'Where evidence comes from: a Xero org, a bank account, a folder of receipt photos, a supplier\'s QuickBooks, a wallet.',
		why: 'Every transaction names its source, so you can ask what one source has ever said, and retire or replace it without touching what others said.',
		identity: 'A stable id you choose when you connect it, such as bank:ridgeway-4821.',
		count: "SELECT count(*) FROM facts WHERE attr = 'source/id'",
		query: "SELECT s.value AS source,\n  (SELECT value FROM facts WHERE e = s.e AND attr = 'source/kind') AS kind,\n  (SELECT value FROM facts WHERE e = s.e AND attr = 'source/name') AS name,\n  (SELECT count(*) FROM facts t WHERE t.attr = 'tx/source' AND t.value = s.e) AS transactions\nFROM facts s WHERE s.attr = 'source/id'\nORDER BY transactions DESC;",
		queryTitle: 'Every source, and how many transactions each wrote',
	},
	{
		id: 'file',
		label: 'File',
		layer: 'drawer',
		x: 0,
		y: 230,
		attrs: ['file/sha256', 'file/name', 'file/media-type', 'file/size', 'file/derived-from'],
		refs: ['file/derived-from'],
		what: 'The bytes, exactly as they arrived: a CSV, a PDF, a photo, an API response body. The bytes live in the drawer; the log holds facts about them.',
		why: 'Nothing is read twice from a provider. If a projector had a bug, fix it and replay the drawer. An extraction is its own file, derived from the photo it read.',
		identity: 'The SHA-256 of the bytes. The same file captured twice is one file with two names.',
		count: "SELECT count(*) FROM files",
		query: "SELECT names, media_type, size, substr(sha256, 8, 12) AS sha256\nFROM files\nORDER BY media_type, names;",
		queryTitle: 'Every file in the drawer',
	},
	{
		id: 'document',
		label: 'Document',
		layer: 'documents',
		x: C,
		y: 0,
		attrs: ['doc/id', 'doc/kind', 'doc/source', 'doc/file', 'doc/external-id', 'doc/period-start'],
		refs: ['doc/source', 'doc/file'],
		what: 'What a file is: a statement, a receipt, a check, a Xero invoice. A document can have several files (a photo and its extraction; the same invoice in two pulls).',
		why: 'Files are bytes; documents are things people talk about. Separating them lets two pulls of one invoice be one document with two files.',
		identity: 'The source\'s own id when it has one (a Xero GUID, a check number, a statement period); otherwise the first file\'s hash.',
		count: "SELECT count(*) FROM facts WHERE attr = 'doc/id'",
		query: "SELECT k.value AS kind, count(*) AS documents\nFROM facts d JOIN facts k ON k.e = d.e AND k.attr = 'doc/kind'\nWHERE d.attr = 'doc/id'\nGROUP BY kind ORDER BY documents DESC;",
		queryTitle: 'Documents by kind',
	},
	{
		id: 'region',
		label: 'Region',
		layer: 'documents',
		x: C,
		y: 270,
		attrs: ['region/id', 'region/file', 'region/locator', 'region/page'],
		refs: ['region/file'],
		what: 'A place inside a file: a CSV row, a JSON pointer, an OFX element path, a page, or a page and a box.',
		why: 'Every figure can be traced to the exact place it was read, so a screen can open the statement at page 1 and draw a box around the line.',
		identity: 'The file and the locator: 4e9c…#page:1/box:566,1005,685,1036.',
		count: "SELECT count(*) FROM facts WHERE attr = 'region/id'",
		query: "SELECT CASE\n    WHEN value LIKE 'row:%' THEN 'CSV row'\n    WHEN value LIKE '/OFX%' THEN 'OFX path'\n    WHEN value LIKE '/%' THEN 'JSON pointer'\n    WHEN value LIKE 'page:%/box:%' THEN 'page and box'\n    WHEN value LIKE 'page:%' THEN 'page'\n    WHEN value LIKE 'cell:%' THEN 'cell'\n    ELSE 'other' END AS grain,\n  count(*) AS regions, min(value) AS example\nFROM facts WHERE attr = 'region/locator'\nGROUP BY grain ORDER BY regions DESC;",
		queryTitle: 'Regions by grain',
	},
	{
		id: 'observation',
		label: 'Observation',
		layer: 'observations',
		x: 2 * C,
		y: 0,
		attrs: ['obs/id', 'obs/kind', 'obs/doc', 'obs/region', 'obs/date', 'obs/amount', 'obs/commodity', 'obs/account', 'obs/counterparty', 'obs/parent', 'obs/fingerprint'],
		refs: ['obs/doc', 'obs/region', 'obs/commodity', 'obs/account', 'obs/parent'],
		what: 'One item a source states, at a place in a file, about an account, in a commodity, at a time. A bank line, a receipt, an invoice, a budget figure, a wallet transfer.',
		why: 'This is the molecule every source broke down into. Shared attributes (date, amount, account) let one rule read every source; typed extras (extraction.receipt/tip, eth.tx/hash) keep what is particular.',
		identity: 'The ladder: the source\'s own id; else a natural key; else a fingerprint and which occurrence it is; else a position. Different sources never share an identity: rules link them.',
		count: "SELECT count(*) FROM observations",
		query: "SELECT kind, count(*) AS observations,\n  sum(parent IS NOT NULL) AS legs\nFROM observations\nGROUP BY kind ORDER BY observations DESC;",
		queryTitle: 'Observations by kind, with how many are legs of another',
	},
	{
		id: 'reading',
		label: 'Reading',
		layer: 'observations',
		x: 2 * C,
		y: 380,
		attrs: ['reading/of', 'reading/attr', 'reading/text', 'reading/region', 'reading/confidence', 'reading/handwritten', 'reading/review'],
		refs: ['reading/of', 'reading/region'],
		what: 'One field as an extractor read it from a photo or PDF: the text it saw, where, and how sure it was.',
		why: 'For a CSV, the row is the provenance of every field in it. For a photo, each field has its own box and confidence. The value stays plain on the observation, so rules never have to read readings.',
		identity: 'The entity, the attribute and the extractor\'s field name.',
		count: "SELECT count(*) FROM facts WHERE attr = 'reading/id'",
		query: "SELECT\n  max(CASE WHEN attr = 'reading/attr' THEN value END) AS attr,\n  max(CASE WHEN attr = 'reading/text' THEN value END) AS seen,\n  max(CASE WHEN attr = 'reading/confidence' THEN value END) AS confidence,\n  max(CASE WHEN attr = 'reading/handwritten' THEN 'yes' END) AS handwritten\nFROM facts\nWHERE e IN (SELECT e FROM facts WHERE attr = 'reading/id')\nGROUP BY e ORDER BY confidence LIMIT 15;",
		queryTitle: 'The least certain readings in the month',
	},
	{
		id: 'link',
		label: 'Link',
		layer: 'links',
		x: 3 * C,
		y: 0,
		attrs: ['link/id', 'link/kind', 'link/from', 'link/to', 'link/confidence', 'link/rule', 'link/amount'],
		refs: ['link/from', 'link/to'],
		what: 'How two things relate: same-event (two witnesses of one movement), evidences (a receipt backs a line), settles (a payment pays an invoice), counterpart (one invoice in two companies\' books), mirrors (an entry mirrors a document).',
		why: 'Sources are never merged. A match is a fact with an author, a rule and a confidence, so a wrong match can be seen and retracted, and the witnesses stay as they were.',
		identity: 'Kind, from and to.',
		count: "SELECT count(*) FROM links",
		query: "SELECT kind, rule, count(*) AS links, min(confidence) AS lowest\nFROM links\nGROUP BY kind, rule ORDER BY links DESC;",
		queryTitle: 'Links by kind and rule',
	},
	{
		id: 'label',
		label: 'Label, note',
		layer: 'links',
		x: 3 * C,
		y: 330,
		attrs: ['label/target', 'label/key', 'label/value', 'note/target', 'note/text'],
		refs: [],
		empty: true,
		what: 'What a person or an agent adds about anything: a tag, a category, a sentence of explanation.',
		why: 'Your own knowledge sits beside the sources\' in the same log, with your name on its transaction. It is in the schema; the synthetic month has none yet.',
		identity: 'Its own id, minted when written.',
		count: "SELECT count(*) FROM facts WHERE attr IN ('label/target', 'note/target')",
		query: "SELECT ident FROM attr WHERE ident LIKE 'label/%' OR ident LIKE 'note/%';",
		queryTitle: 'The label and note attributes (no facts yet)',
	},
	{
		id: 'commodity',
		label: 'Commodity, price',
		layer: 'books',
		x: 3 * C,
		y: 560,
		attrs: ['commodity/code', 'commodity/kind', 'commodity/scale', 'price/commodity', 'price/quote', 'price/at', 'price/value'],
		refs: ['price/commodity', 'price/quote'],
		what: 'What an amount is counted in: USD, ETH, a token at a contract address. Prices are facts too: one commodity in another, at a time, from a source.',
		why: 'Amounts carry their commodity from day one, so a wallet payment in USDC and its gas fee in ETH sit in the same model as a bank line in dollars.',
		identity: 'A code: USD, or USDC@ethereum:0x… for a token.',
		count: "SELECT count(*) FROM facts WHERE attr = 'commodity/code'",
		query: "SELECT c.value AS commodity,\n  (SELECT count(*) FROM facts o WHERE o.attr = 'obs/commodity' AND o.value = c.e) AS observations,\n  (SELECT count(*) FROM facts p WHERE p.attr = 'price/commodity' AND p.value = c.e) AS prices\nFROM facts c WHERE c.attr = 'commodity/code';",
		queryTitle: 'Commodities, and what is counted in each',
	},
	{
		id: 'party',
		label: 'Party',
		layer: 'books',
		x: 2 * C,
		y: 640,
		attrs: ['party/id', 'party/name'],
		refs: [],
		what: 'A person or business the books deal with: a customer, a supplier, the bank.',
		why: 'Parties are in the log, but nothing points at them yet. Observations name their counterparty as text (obs/counterparty), so "Fernhill Grocers" on the wallet and on the Xero invoice match only because the strings are equal. A reference here would make that a join.',
		identity: 'The source\'s contact id, or a name.',
		count: "SELECT count(*) FROM facts WHERE attr = 'party/id'",
		query: "SELECT p.value AS party,\n  (SELECT value FROM facts WHERE e = p.e AND attr = 'party/name') AS name,\n  (SELECT count(*) FROM facts c WHERE c.attr = 'obs/counterparty'\n     AND c.value = (SELECT value FROM facts WHERE e = p.e AND attr = 'party/name')) AS named_by\nFROM facts p WHERE p.attr = 'party/id'\nORDER BY name;",
		queryTitle: 'Parties, and how many observations name each',
	},
	{
		id: 'book',
		label: 'Book',
		layer: 'books',
		x: 4 * C,
		y: 0,
		attrs: ['book/id', 'book/name', 'book/commodity', 'book/authority'],
		refs: ['book/commodity', 'book/authority'],
		what: 'One company\'s double-entry books. The log can hold several: the roastery\'s Xero and a supplier\'s QuickBooks sit side by side.',
		why: 'Every book names its authority. While Xero owns a book, ownpurse mirrors it; when you are ready, the authority becomes ownpurse, in one fact.',
		identity: 'The source it mirrors, or an id you choose.',
		count: "SELECT count(*) FROM facts WHERE attr = 'book/id'",
		query: "SELECT b.value AS book,\n  (SELECT value FROM facts WHERE e = b.e AND attr = 'book/name') AS name,\n  (SELECT count(*) FROM facts en WHERE en.attr = 'entry/book' AND en.value = b.e) AS entries\nFROM facts b WHERE b.attr = 'book/id';",
		queryTitle: 'Books, and how many entries each holds',
	},
	{
		id: 'account',
		label: 'Account',
		layer: 'books',
		x: 4 * C,
		y: 190,
		attrs: ['account/id', 'account/book', 'account/code', 'account/name', 'account/type', 'account/mask', 'account/commodity'],
		refs: ['account/book', 'account/commodity'],
		what: 'An account in a book (6100 Meals and entertainment), or an account at an institution (checking ending 4821, a wallet address).',
		why: 'A statement, a check and a receipt all say "ending 4821". The mask is the weak key every source shares; accounts across sources are linked, not merged.',
		identity: 'The book and its code, or the institution and the account number.',
		count: "SELECT count(*) FROM facts WHERE attr = 'account/id'",
		query: "SELECT a.value AS account,\n  (SELECT value FROM facts WHERE e = a.e AND attr = 'account/code') AS code,\n  (SELECT value FROM facts WHERE e = a.e AND attr = 'account/name') AS name,\n  (SELECT value FROM facts WHERE e = a.e AND attr = 'account/mask') AS mask\nFROM facts a WHERE a.attr = 'account/id'\nORDER BY account;",
		queryTitle: 'Every account',
	},
	{
		id: 'entry',
		label: 'Entry',
		layer: 'books',
		x: 4 * C,
		y: 420,
		attrs: ['entry/id', 'entry/book', 'entry/date', 'entry/narration', 'entry/status', 'entry/posting'],
		refs: ['entry/book', 'entry/posting'],
		what: 'A double-entry journal entry: a date, a narration, a status, and postings that sum to zero in each commodity.',
		why: 'Entries are held until posted. Posting is two facts in a transaction with your name on it. No source owns an entry; external ids are ordinary facts.',
		identity: 'Minted by ownpurse, or the document it mirrors (…#entry).',
		count: "SELECT count(*) FROM facts WHERE attr = 'entry/id'",
		query: "SELECT entry, date, status, account, quantity, commodity\nFROM postings\nWHERE book = 'xero:tallowmere'\nORDER BY date, entry LIMIT 40;",
		queryTitle: 'The roastery\'s entries and their postings',
	},
	{
		id: 'posting',
		label: 'Posting',
		layer: 'books',
		x: 4 * C,
		y: 650,
		attrs: ['posting/account', 'posting/quantity', 'posting/commodity', 'posting/memo'],
		refs: ['posting/account', 'posting/commodity'],
		what: 'One line of an entry: a signed quantity of a commodity in an account. Debits are positive.',
		why: 'A trial balance is a sum of postings per account, done exactly with dsum(). Any heavier engine later is a projection of these facts.',
		identity: 'The entry and its position.',
		count: "SELECT count(*) FROM facts WHERE attr = 'posting/id'",
		query: "SELECT book, account, dsum(quantity) AS balance, count(*) AS postings\nFROM postings\nWHERE status = 'entry.status/posted'\nGROUP BY book, account\nORDER BY book, account;",
		queryTitle: 'A trial balance per book',
	},
	{
		id: 'tx',
		label: 'Transaction',
		layer: 'log',
		x: 0,
		y: 880,
		wide: true,
		attrs: ['tx/actor', 'tx/projector', 'tx/source', 'tx/evidence', 'tx/observed-at', 'tx/instant', 'tx/reason', 'tx/pressure', 'prev · hash'],
		refs: ['tx/source', 'tx/evidence'],
		what: 'The unit of writing. Every datom [entity, attribute, value, transaction, added] names the transaction that wrote it, and the transaction is itself an entity with facts: who, when, why, from which files, with which projector.',
		why: 'Provenance is written once per batch and every fact inherits it. Each transaction carries the hash of the one before, so the whole log is a chain you can verify. Nothing is updated; a correction is a new transaction that retracts and asserts.',
		identity: 'Its position in the log.',
		count: "SELECT count(*) FROM tx",
		query: "SELECT tx, actor, projector, files, asserted, retracted\nFROM transactions\nORDER BY tx;",
		queryTitle: 'Every transaction in the log',
	},
];

export interface Ref {
	id: string;
	from: string;
	to: string;
	attr: string;
	note: string;
}

export const REFS: Ref[] = [
	{ id: 'doc-file', from: 'document', to: 'file', attr: 'doc/file', note: 'The files that carry this document. A Xero invoice read in two pulls has two.' },
	{ id: 'doc-source', from: 'document', to: 'source', attr: 'doc/source', note: 'Which source issued the document.' },
	{ id: 'file-derived', from: 'file', to: 'file', attr: 'file/derived-from', note: 'An extraction is a file of its own, derived from the photo or PDF it read.' },
	{ id: 'region-file', from: 'region', to: 'file', attr: 'region/file', note: 'The file the place is in.' },
	{ id: 'obs-doc', from: 'observation', to: 'document', attr: 'obs/doc', note: 'The document that reports the item.' },
	{ id: 'obs-region', from: 'observation', to: 'region', attr: 'obs/region', note: 'Where the item is. Several when its fields sit in different places, as on a receipt.' },
	{ id: 'obs-parent', from: 'observation', to: 'observation', attr: 'obs/parent', note: 'An event has legs: a receipt\'s lines, a transfer\'s two sides, a loan payment\'s principal and interest.' },
	{ id: 'obs-account', from: 'observation', to: 'account', attr: 'obs/account', note: 'The account the item is on: checking ending 4821, a wallet.' },
	{ id: 'obs-commodity', from: 'observation', to: 'commodity', attr: 'obs/commodity', note: 'What the amount is counted in.' },
	{ id: 'reading-of', from: 'reading', to: 'observation', attr: 'reading/of', note: 'The observation (or document, or account) the field supports.' },
	{ id: 'reading-region', from: 'reading', to: 'region', attr: 'reading/region', note: 'The page and box where the text was read.' },
	{ id: 'link-from', from: 'link', to: 'entry', attr: 'link/from', note: 'One end of the link: usually an observation; for a mirrors link, the entry that mirrors a document.' },
	{ id: 'link-to', from: 'link', to: 'observation', attr: 'link/to', note: 'The other end: an observation, or the document an entry mirrors.' },
	{ id: 'price-commodity', from: 'commodity', to: 'commodity', attr: 'price/commodity · price/quote', note: 'A price is one commodity in another.' },
	{ id: 'account-book', from: 'account', to: 'book', attr: 'account/book', note: 'The book the account belongs to.' },
	{ id: 'entry-book', from: 'entry', to: 'book', attr: 'entry/book', note: 'The book the entry is in.' },
	{ id: 'entry-posting', from: 'entry', to: 'posting', attr: 'entry/posting', note: 'The entry\'s lines.' },
	{ id: 'posting-account', from: 'posting', to: 'account', attr: 'posting/account', note: 'The account the line posts to.' },
	{ id: 'posting-commodity', from: 'posting', to: 'commodity', attr: 'posting/commodity', note: 'What the quantity is counted in.' },
	{ id: 'tx-evidence', from: 'tx', to: 'file', attr: 'tx/evidence', note: 'The drawer files the transaction was read from.' },
	{ id: 'tx-source', from: 'tx', to: 'source', attr: 'tx/source', note: 'The source the evidence came from.' },
];

/** Ask the canvas on this page to select a kind. */
export function selectKind(id: string) {
	window.dispatchEvent(new CustomEvent('fm:select', { detail: { id } }));
	document.getElementById('fm-model')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
}
