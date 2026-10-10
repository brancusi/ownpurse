import { useMemo, useState, type ReactNode } from 'react';
import { openInConsole, rows, run, type Month } from './db';
import { selectKind } from './model';
import { Loading, Problem, ResultTable, useMonth, type Props } from './ui';

// The receipt is found by content, never by entity id, so a re-export of the month keeps this figure true.
const RECEIPT_TX = "(SELECT tx FROM facts WHERE attr = 'doc/kind' AND value = 'doc.kind/receipt')";
const RECEIPT_OBS = "(SELECT e FROM observations WHERE kind = 'obs.kind/receipt')";
const RECEIPT_ID = "(SELECT id FROM observations WHERE kind = 'obs.kind/receipt')";
const IMG = { src: '/data/copper-pot-2026-03-12.jpg', w: 942, h: 1710 };

interface Box {
	key: string;
	label: string;
	box: [number, number, number, number];
	confidence: number;
	handwritten: boolean;
}

function readings(m: Month): Box[] {
	return rows(
		m.db,
		`SELECT
      max(CASE WHEN f.attr = 'reading/attr' THEN f.value END) AS attr,
      max(CASE WHEN f.attr = 'reading/text' THEN f.value END) AS text,
      max(CASE WHEN f.attr = 'reading/confidence' THEN f.value END) AS confidence,
      max(CASE WHEN f.attr = 'reading/handwritten' THEN f.value END) AS handwritten,
      max(CASE WHEN f.attr = 'reading/region' THEN (SELECT value FROM facts WHERE e = f.value AND attr = 'region/locator') END) AS place
    FROM facts f
    WHERE f.e IN (SELECT e FROM facts WHERE attr = 'reading/of' AND value = ${RECEIPT_OBS})
    GROUP BY f.e`,
	)
		.map((r) => {
			const m2 = /box:(\d+),(\d+),(\d+),(\d+)/.exec(String(r.place ?? ''));
			if (!m2) return null;
			return {
				key: String(r.attr),
				label: `${String(r.attr).split('/').pop()}: ${r.text}`,
				box: [Number(m2[1]), Number(m2[2]), Number(m2[3]), Number(m2[4])] as [number, number, number, number],
				confidence: Number(r.confidence),
				handwritten: Number(r.handwritten) === 1,
			};
		})
		.filter((b): b is Box => b !== null && b.key !== 'obs/account');
}

interface Step {
	title: string;
	layer: string;
	kind: string;
	boxes: 'none' | 'all' | 'low';
	body: (m: Month, hover: (k: string | null) => void) => ReactNode;
	sql: (m: Month) => string;
}

const STEPS: Step[] = [
	{
		title: 'The photo lands in the drawer',
		layer: '1 Drawer',
		kind: 'file',
		boxes: 'none',
		body: (m) => {
			const f = rows(m.db, "SELECT names, media_type, size, sha256 FROM files WHERE names = 'copper-pot-2026-03-12.jpg'")[0];
			return (
				<>
					<p>
						A phone photo of the receipt from a client dinner. ownpurse keeps the bytes exactly as they arrived, named by their hash. The same photo
						uploaded twice is one file.
					</p>
					<dl className="fm-kv">
						<dt>file/name</dt>
						<dd>{String(f?.names)}</dd>
						<dt>file/media-type</dt>
						<dd>{String(f?.media_type)}</dd>
						<dt>file/size</dt>
						<dd>{Number(f?.size).toLocaleString('en-US')} bytes</dd>
						<dt>file/sha256</dt>
						<dd className="fm-hash">{String(f?.sha256)}</dd>
					</dl>
				</>
			);
		},
		sql: () => "SELECT * FROM files WHERE names LIKE 'copper-pot%';",
	},
	{
		title: 'An extractor reads it',
		layer: '1 Drawer',
		kind: 'file',
		boxes: 'all',
		body: (m) => {
			const x = rows(
				m.db,
				`SELECT
          (SELECT value FROM facts WHERE e = d.e AND attr = 'extraction/extractor') AS who,
          (SELECT value FROM facts WHERE e = d.e AND attr = 'extraction/extractor-version') AS version,
          (SELECT value FROM facts WHERE e = d.e AND attr = 'extraction/read-at') AS read_at
        FROM facts d WHERE d.attr = 'extraction/extractor' AND d.tx = ${RECEIPT_TX}`,
			)[0];
			return (
				<>
					<p>
						An extractor (here {String(x?.who)}, {String(x?.version)}) reads the photo and writes what it saw as a second file: each field's text,
						the box it sits in, and a confidence. That file is evidence too. It goes in the drawer beside the photo, marked as derived from it.
					</p>
					<p>
						The boxes on the photo are the extractor's. Two of them were written by hand: the tip and the total. Those are the ones it was least
						sure of.
					</p>
				</>
			);
		},
		sql: () => "SELECT names, media_type, size FROM files WHERE names LIKE 'copper-pot%';",
	},
	{
		title: 'A projector writes one transaction',
		layer: '0 The log',
		kind: 'tx',
		boxes: 'none',
		body: (m) => {
			const t = rows(m.db, `SELECT * FROM transactions WHERE tx = ${RECEIPT_TX}`)[0];
			const obs = run(
				m.db,
				`SELECT kind, date, amount, counterparty, description FROM observations
         WHERE e = ${RECEIPT_OBS} OR parent = ${RECEIPT_ID}
         ORDER BY parent IS NOT NULL, e`,
			);
			return (
				<>
					<p>
						The projector <code>{String(t?.projector)}</code> turns the two files into one transaction of {Number(t?.asserted).toLocaleString('en-US')}{' '}
						facts, all stamped with transaction {String(t?.tx)}: who wrote it, from which files, with which projector. It writes a document, one
						observation for the receipt, a leg for each line, and the regions and readings behind them.
					</p>
					<ResultTable result={obs} />
				</>
			);
		},
		sql: () => `SELECT attr, value FROM facts_shown\nWHERE e = ${RECEIPT_OBS}\nORDER BY attr;`,
	},
	{
		title: 'Every field keeps its place',
		layer: '3 Observations',
		kind: 'reading',
		boxes: 'all',
		body: (m, hover) => {
			const r = rows(
				m.db,
				`SELECT
          max(CASE WHEN f.attr = 'reading/attr' THEN f.value END) AS attr,
          max(CASE WHEN f.attr = 'reading/text' THEN f.value END) AS seen,
          max(CASE WHEN f.attr = 'reading/confidence' THEN f.value END) AS confidence,
          max(CASE WHEN f.attr = 'reading/handwritten' THEN 'by hand' END) AS written
        FROM facts f
        WHERE f.e IN (SELECT e FROM facts WHERE attr = 'reading/of' AND value = ${RECEIPT_OBS})
        GROUP BY f.e ORDER BY confidence`,
			);
			return (
				<>
					<p>
						The observation holds plain values (amount −186.40, date 2026-03-12), so rules never have to think about pixels. Beside each value is a
						reading: the text as seen, where, and how sure. Point at a row to see its box.
					</p>
					<div className="fm-result-scroll">
						<table className="fm-readings">
							<thead>
								<tr>
									<th>Attribute</th>
									<th>Seen</th>
									<th>Confidence</th>
								</tr>
							</thead>
							<tbody>
								{r.map((x) => (
									<tr
										key={String(x.attr)}
										onMouseEnter={() => hover(String(x.attr))}
										onMouseLeave={() => hover(null)}
										onFocus={() => hover(String(x.attr))}
										onBlur={() => hover(null)}
										tabIndex={0}
										className={Number(x.confidence) < 0.85 ? 'fm-low' : undefined}
									>
										<td>{String(x.attr)}</td>
										<td>
											{String(x.seen)}
											{x.written ? <span className="fm-tag-held">by hand</span> : null}
										</td>
										<td className="fm-num">{String(x.confidence)}</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				</>
			);
		},
		sql: () =>
			`SELECT\n  max(CASE WHEN f.attr = 'reading/attr' THEN f.value END) AS attr,\n  max(CASE WHEN f.attr = 'reading/text' THEN f.value END) AS seen,\n  max(CASE WHEN f.attr = 'reading/confidence' THEN f.value END) AS confidence\nFROM facts f\nWHERE f.e IN (SELECT e FROM facts WHERE attr = 'reading/of'\n  AND value = ${RECEIPT_OBS})\nGROUP BY f.e ORDER BY confidence;`,
	},
	{
		title: 'What did not fit is reported',
		layer: '0 The log',
		kind: 'tx',
		boxes: 'low',
		body: (m) => {
			const raw = run(m.db, `SELECT value FROM facts WHERE attr = 'tx/pressure' AND e = ${RECEIPT_TX} ORDER BY value`);
			const p = {
				columns: ['kind', 'where', 'detail'],
				rows: raw.rows.map(([v]) => {
					const [kind, where, ...rest] = String(v).split(' | ');
					return [kind, where, rest.join(' | ')];
				}),
				ms: raw.ms,
			};
			return (
				<>
					<p>
						A projector never silently drops what doesn't fit. The receipt prints a total of 166.40, and someone wrote 186.40 under it after adding
						the tip. Both are kept. Which one is "the amount" is the projector's policy, and the policy is reported as pressure on the transaction.
					</p>
					<ResultTable result={p} />
				</>
			);
		},
		sql: () => `SELECT value FROM facts\nWHERE attr = 'tx/pressure' AND e = ${RECEIPT_TX};`,
	},
	{
		title: 'Rules link it to the other witnesses',
		layer: '4 Links',
		kind: 'link',
		boxes: 'none',
		body: (m) => {
			const l = run(
				m.db,
				`SELECT l.kind, l.confidence, o.date, o.amount, coalesce(o.doc, o.id) AS witness
         FROM links l JOIN observations o ON o.id = l.to_id
         WHERE l.from_id = ${RECEIPT_ID}`,
			);
			const t = rows(m.db, `SELECT actor FROM transactions WHERE tx = (SELECT max(tx) FROM facts WHERE attr = 'link/id')`)[0];
			return (
				<>
					<p>
						The bank saw the same dinner three times (a CSV download, an OFX file, the statement) and Xero once. None of them is merged into another.
						The rules ({String(t?.actor).replace('rules:', '')}) write links instead, each with a confidence: the receipt <em>evidences</em> each
						witness. The receipt is dated the 12th; the bank posted it on the 13th. Two witnesses, two dates, one event.
					</p>
					<ResultTable result={l} />
				</>
			);
		},
		sql: () => `SELECT * FROM links\nWHERE from_id = ${RECEIPT_ID};`,
	},
	{
		title: 'In the books',
		layer: '5 Books',
		kind: 'entry',
		boxes: 'none',
		body: (m) => {
			const p = run(
				m.db,
				`SELECT p.book, p.date, p.status, p.account, p.quantity, p.memo
         FROM postings p
         WHERE p.entry IN (SELECT l.from_id FROM links l
           WHERE l.kind = 'link.kind/mirrors'
             AND l.to_id IN (SELECT to_id FROM links WHERE from_id = ${RECEIPT_ID}))`,
			);
			return (
				<>
					<p>
						The entry in the books mirrors Xero's bank transaction: 186.40 to Meals and entertainment, out of the operating account. The receipt is
						not the entry. It is evidence for it, and from the posting you can walk back through the links to the box on the photo where the tip
						was written.
					</p>
					<ResultTable result={p} />
				</>
			);
		},
		sql: () => "SELECT * FROM postings\nWHERE account = 'Meals and entertainment';",
	},
];

export default function ReceiptFlow({ src }: Props) {
	const { month, error } = useMonth(src);
	const [i, setI] = useState(0);
	const [hover, setHover] = useState<string | null>(null);
	const boxes = useMemo(() => (month ? readings(month) : []), [month]);

	if (error) return <Problem>{error}</Problem>;
	const step = STEPS[i];
	const shown = step.boxes === 'all' ? boxes : step.boxes === 'low' ? boxes.filter((b) => b.confidence < 0.85) : hover ? boxes : [];

	return (
		<div className="fm-flow">
			<ol className="fm-flow-steps">
				{STEPS.map((s, j) => (
					<li key={s.title}>
						<button type="button" className={j === i ? 'is-on' : j < i ? 'is-done' : undefined} onClick={() => setI(j)} aria-current={j === i ? 'step' : undefined}>
							<span className="fm-n">{j + 1}</span>
							<span>{s.title}</span>
						</button>
					</li>
				))}
			</ol>
			<div className="fm-flow-body">
				<figure className="fm-photo">
					<div className="fm-photo-frame">
						<img src={IMG.src} width={IMG.w} height={IMG.h} alt="A phone photo of a restaurant receipt from Copper Pot Bistro, with a handwritten tip and total. Invented for the samples." />
						<svg viewBox={`0 0 ${IMG.w} ${IMG.h}`} aria-hidden="true">
							{shown
								.filter((b) => step.boxes !== 'none' || b.key === hover)
								.map((b) => (
									<g key={b.key} className={`${b.confidence < 0.85 ? 'fm-box-low' : 'fm-box'}${b.key === hover ? ' fm-box-on' : ''}`}>
										<rect x={b.box[0]} y={b.box[1]} width={b.box[2] - b.box[0]} height={b.box[3] - b.box[1]} />
										<text x={b.box[2] + 8} y={b.box[3] - 6}>
											{b.confidence.toFixed(2)}
										</text>
									</g>
								))}
						</svg>
					</div>
					<figcaption>copper-pot-2026-03-12.jpg · invented for the samples</figcaption>
				</figure>
				<div className="fm-flow-text">
					<span className="fm-k">
						Step {i + 1} of {STEPS.length} · layer {step.layer}
					</span>
					<h3>{step.title}</h3>
					{month ? step.body(month, setHover) : <Loading />}
					<div className="fm-flow-nav">
						<button type="button" className="fm-btn fm-btn-quiet" disabled={i === 0} onClick={() => setI(i - 1)}>
							Back
						</button>
						<button type="button" className="fm-btn" disabled={i === STEPS.length - 1} onClick={() => setI(i + 1)}>
							Next: {STEPS[Math.min(i + 1, STEPS.length - 1)].title.toLowerCase()}
						</button>
					</div>
					<div className="fm-flow-more">
						<button type="button" className="fm-linkbtn" onClick={() => selectKind(step.kind)}>
							See it on the model
						</button>
						{month && (
							<button type="button" className="fm-linkbtn" onClick={() => openInConsole(step.sql(month), step.title)}>
								Open the query
							</button>
						)}
					</div>
				</div>
			</div>
		</div>
	);
}
