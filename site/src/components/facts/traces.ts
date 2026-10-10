// How an agent answers a question from the log. Each trace was worked by Tally against this month's log: the
// reasoning is Tally's, and every step's SQL runs live in the reader's browser when the trace plays. The answer
// cites the files and places the rows point to, never a number the queries did not return.

export interface Step {
	think: string;
	sql: string;
	/** What the agent takes from the rows. */
	read: string;
}

export interface Trace {
	id: string;
	question: string;
	steps: Step[];
	answer: string;
	cites: string[];
	/** A transaction the agent would propose rather than write. */
	proposal?: { why: string; facts: [string, string, string, string][] };
}

export const TRACES: Trace[] = [
	{
		id: 'tip',
		question: 'Was the tip on the 12 March client dinner really 20.00?',
		steps: [
			{
				think: 'Start from what the log offers. The views are the readable way in.',
				sql: "SELECT name FROM sqlite_master WHERE type = 'view' ORDER BY name;",
				read: 'observations, links, facts and history will do. Readings hold what an extractor saw, field by field.',
			},
			{
				think: 'Find the receipt for the dinner, and what each of its fields was read as, least certain first.',
				sql: "SELECT\n  max(CASE WHEN f.attr = 'reading/attr' THEN f.value END) AS field,\n  max(CASE WHEN f.attr = 'reading/text' THEN f.value END) AS seen,\n  max(CASE WHEN f.attr = 'reading/confidence' THEN f.value END) AS confidence,\n  max(CASE WHEN f.attr = 'reading/handwritten' THEN 'by hand' END) AS written\nFROM facts f\nWHERE f.e IN (SELECT r.e FROM facts r\n  JOIN observations o ON o.e = r.value AND o.kind = 'obs.kind/receipt'\n  WHERE r.attr = 'reading/of' AND o.date = '2026-03-12')\nGROUP BY f.e ORDER BY confidence;",
				read: 'The tip, 20.00, was written by hand and read at 0.62. The paid total, 186.40, is handwritten too (0.81). The printed total is 166.40, and subtotal 152.80 plus tax 13.60 makes exactly that.',
			},
			{
				think: 'A handwritten figure at 0.62 is not enough. If the paid total is right, the tip is the paid total less the printed one. What else saw this payment?',
				sql: "SELECT l.kind, l.confidence, o.kind AS witness, o.date, o.amount, o.doc\nFROM links l\nJOIN observations o ON o.id = l.to_id\nWHERE l.from_id = (SELECT id FROM observations\n                   WHERE kind = 'obs.kind/receipt' AND date = '2026-03-12');",
				read: 'Four witnesses the receipt evidences: the bank CSV line, the OFX line, the statement line and Xero\'s bank transaction. Every one says 186.40.',
			},
			{
				think: 'Check the arithmetic exactly, not in floating point.',
				sql: "SELECT dsum(x) AS tip FROM (SELECT '186.40' AS x UNION ALL SELECT '-166.40');",
				read: '186.40 − 166.40 = 20.00.',
			},
		],
		answer:
			'Yes. The tip was handwritten and read with low confidence (0.62), but four independent sources (the bank\'s CSV download, its OFX file, the March statement and Xero) all show 186.40 leaving checking ending 4821, and the printed total was 166.40. The difference is 20.00 exactly. You could confirm the reading, which records your verdict as a new fact.',
		cites: ['copper-pot-2026-03-12.jpg · page 1, box 566,1005,685,1036 (tip)', 'statement-2026-03.pdf · page 1', '2026-03.ofx · STMTTRN[5]', 'activity-2026-03-20.csv · row 9'],
	},
	{
		id: 'fernhill',
		question: 'Has Fernhill Grocers paid INV-0418?',
		steps: [
			{
				think: 'Find the invoice by its reference, and its status as Xero last reported it.',
				sql: "SELECT o.id, o.date, o.amount, o.counterparty,\n  (SELECT value FROM facts WHERE e = o.e AND attr = 'obs/status') AS status\nFROM observations o\nJOIN facts r ON r.e = o.e AND r.attr = 'obs/reference' AND r.value = 'INV-0418';",
				read: 'Issued 2026-03-19 for 1,250.00. Xero still calls it authorised: issued, not paid.',
			},
			{
				think: 'Does anything settle it? A payment would be linked to it.',
				sql: "SELECT kind, from_id, amount, rule FROM links\nWHERE to_id = (SELECT o.id FROM observations o\n  JOIN facts r ON r.e = o.e AND r.attr = 'obs/reference' AND r.value = 'INV-0418');",
				read: 'No rows. Nothing in the log settles INV-0418.',
			},
			{
				think: 'Xero only knows what it was told. Look for money from Fernhill in any source, in any commodity.',
				sql: "SELECT kind, date, amount, commodity, account, doc\nFROM observations\nWHERE counterparty LIKE '%Fernhill%' AND kind <> 'obs.kind/invoice';",
				read: 'One transfer on chain: 1,250.000000 USDC into the roastery wallet on 2026-03-21.',
			},
			{
				think: 'Make sure it happened: the transaction around that transfer, and when exactly.',
				sql: "SELECT h.attr, h.shown AS value\nFROM history h\nWHERE h.e = (SELECT e FROM observations\n             WHERE kind = 'obs.kind/chain-transaction' AND date = '2026-03-21')\n  AND h.attr IN ('obs/status', 'obs/date', 'obs/instant', 'eth.tx/block', 'obs/account');",
				read: 'It succeeded, in block 24866402 at 03:41 UTC on 22 March, which is still 21 March in the books\' time zone. Two times for one event: the instant and the business date.',
			},
		],
		answer:
			'In substance, yes; in Xero, no. Fernhill paid 1,250.000000 USDC to the roastery wallet on 21 March (block 24866402), but Xero never heard about it, and no rule in the log yet links a payment in one commodity to an invoice in another. The invoice will look unpaid until someone links them. Here is the link to propose: it waits for your approval and changes nothing until you give it.',
		cites: ['Invoices.json (pull 2) · INV-0418', '2026-03-21-usdc-in.rpc.json · the transfer log', 'wallet.json · the counterparty label'],
		proposal: {
			why: 'Fernhill paid INV-0418 in USDC on chain',
			facts: [
				['+', 'link:new', 'link/kind', 'link.kind/settles'],
				['+', 'link:new', 'link/from', 'ethereum:0x2a9c…#transfer (1,250.000000 USDC)'],
				['+', 'link:new', 'link/to', 'xero:tallowmere/Invoice/…0004 (INV-0418)'],
				['+', 'link:new', 'link/amount', '1250.00'],
				['+', 'tx', 'tx/actor', 'agent:tally'],
				['+', 'tx', 'tx/reason', 'Fernhill paid INV-0418 in USDC on chain'],
			],
		},
	},
	{
		id: 'changes',
		question: 'What changed in Xero between the two pulls?',
		steps: [
			{
				think: 'Each Xero read is a transaction. Find them, and which ones retracted anything.',
				sql: "SELECT tx, (SELECT value FROM facts WHERE e = t.tx AND attr = 'tx/observed-at') AS read_at,\n  (SELECT group_concat(DISTINCT fl.names) FROM facts ev JOIN files fl ON fl.e = ev.value\n    WHERE ev.e = t.tx AND ev.attr = 'tx/evidence') AS file,\n  asserted, retracted\nFROM transactions t\nWHERE projector = 'xero.json@1' AND retracted > 0;",
				read: 'Two reads of 2 April retracted something: Invoices.json (19 facts) and BankTransactions.json (4). Everything else only added.',
			},
			{
				think: 'Pair each retraction with what replaced it in the same transaction. Skip Xero\'s raw mirror fields and read the model\'s own.',
				sql: "SELECT coalesce((SELECT value FROM facts WHERE e = r.e AND attr = 'obs/reference'),\n                (SELECT label FROM label WHERE e = r.e)) AS what,\n  r.attr, r.shown AS was, a.shown AS now\nFROM history r\nJOIN transactions t ON t.tx = r.tx AND t.projector = 'xero.json@1'\nLEFT JOIN history a ON a.e = r.e AND a.attr = r.attr AND a.tx = r.tx AND a.op = 'assert'\nWHERE r.op = 'retract' AND r.attr NOT LIKE 'xero.%'\nORDER BY what, r.attr;",
				read: 'INV-0415 went from 929.63 to 1,084.57 (its line from 856.80 to 999.60). INV-0416 was voided. Saltmarsh\'s bill SC-0057 was paid. The dinner\'s description became "Dinner with Ostley Joinery buyers".',
			},
			{
				think: 'And what appeared for the first time in the second pull?',
				sql: "SELECT k.value AS kind, d.value AS document\nFROM facts d\nJOIN facts k ON k.e = d.e AND k.attr = 'doc/kind'\nJOIN transactions t ON t.tx = d.tx AND t.projector = 'xero.json@1'\nWHERE d.attr = 'doc/id'\n  AND (SELECT value FROM facts WHERE e = d.tx AND attr = 'tx/observed-at') >= '2026-04-01'\nORDER BY kind;",
				read: 'Seven new documents: an account, four bank transactions from late March, the payment of SC-0057 and the depreciation journal.',
			},
		],
		answer:
			'Between the 20 March and 2 April pulls: INV-0415 was edited from 929.63 to 1,084.57; INV-0416 (412.08) was voided; the Saltmarsh bill SC-0057 was paid; the dinner\'s description gained "with Ostley Joinery buyers". Seven documents are new, among them four late-March bank transactions and the 31 March depreciation journal. The earlier values are still in the log; ask for any of them as of the first pull.',
		cites: ['pull-1/Invoices.json and pull-2/Invoices.json', 'pull-2/BankTransactions.json', 'pull-2/ManualJournals.json'],
	},
	{
		id: 'check',
		question: 'Who signed check 1042, and when did it clear?',
		steps: [
			{
				think: 'Find the check and how sure the extractor was of each field.',
				sql: "SELECT\n  max(CASE WHEN f.attr = 'reading/attr' THEN f.value END) AS field,\n  max(CASE WHEN f.attr = 'reading/text' THEN f.value END) AS seen,\n  max(CASE WHEN f.attr = 'reading/confidence' THEN f.value END) AS confidence\nFROM facts f\nWHERE f.e IN (SELECT r.e FROM facts r\n  JOIN observations o ON o.e = r.value AND o.kind = 'obs.kind/check'\n  WHERE r.attr = 'reading/of')\nGROUP BY f.e ORDER BY confidence;",
				read: 'The signature reads "Odile Fenwick", but only at 0.38. The amount and payee are read well.',
			},
			{
				think: 'The check is dated when it was written. The bank says when it cleared. Which bank lines does it evidence?',
				sql: "SELECT o.date, o.amount, o.description, o.doc, l.confidence\nFROM links l\nJOIN observations o ON o.id = l.to_id\nWHERE l.kind = 'link.kind/evidences'\n  AND l.from_id = (SELECT id FROM observations WHERE kind = 'obs.kind/check');",
				read: 'Written on 03-03; every bank witness posts it on 03-09 for −3,215.47.',
			},
			{
				think: 'What did it pay? Follow Xero\'s payment to the bill it settles.',
				sql: "SELECT p.date, l.amount, b.counterparty,\n  (SELECT value FROM facts WHERE e = b.e AND attr = 'obs/reference') AS bill\nFROM links l\nJOIN observations p ON p.id = l.from_id\nJOIN observations b ON b.id = l.to_id\nWHERE l.kind = 'link.kind/settles' AND l.amount = '3215.47';",
				read: 'It paid bill BO-7781 to Brindle & Oake Supply.',
			},
		],
		answer:
			'The signature reads as Odile Fenwick, but with confidence 0.38: look at the image before relying on it. The check was written on 3 March and cleared on 9 March for 3,215.47, paying Brindle & Oake Supply\'s bill BO-7781.',
		cites: ['check-1042.png · page 1, the signature box', 'statement-2026-03.pdf · page 1', 'pull-1/Payments.json'],
	},
];
