import { useCallback, useEffect, useMemo, useState } from 'react';
import {
	ReactFlow,
	ReactFlowProvider,
	Background,
	Controls,
	Handle,
	Position,
	useReactFlow,
	type Node,
	type Edge,
	type NodeProps,
	type NodeChange,
} from '@xyflow/react';
import '@xyflow/react/dist/base.css';
import type { Database, SqlValue } from 'sql.js';
import { openInConsole, rows } from './db';
import { Loading, Problem, useMonth, type Props } from './ui';

// How the log is stored: one table of datoms, [e a v tx added]. An entity is every datom with the same e; a reference
// is a datom whose v is another entity's e. This figure draws each entity as its own run of datoms, as the (e, a, tx)
// index keeps them, and wires each reference row to the entity it names. Entities are found by content, never by id.

interface Datom {
	id: number;
	e: number;
	a: number;
	attr: string;
	v: SqlValue;
	vt: number;
	tx: number;
	added: number;
}

interface Entity {
	e: number;
	kind: string;
	label: string;
	datoms: Datom[];
	incoming: { attr: string; n: number }[];
	note: string | null;
}

const KIND_BY: [string, string][] = [
	['db/ident', 'attribute'],
	['tx/instant', 'transaction'],
	['doc/id', 'document'],
	['obs/id', 'observation'],
	['reading/id', 'reading'],
	['region/id', 'region'],
	['link/id', 'link'],
	['entry/id', 'entry'],
	['posting/id', 'posting'],
	['account/id', 'account'],
	['party/id', 'party'],
	['commodity/code', 'commodity'],
	['source/id', 'source'],
	['book/id', 'book'],
	['file/sha256', 'file'],
];

const one = (db: Database, sql: string, params: SqlValue[] = []) => rows(db, sql, params)[0];
const num = (db: Database, sql: string, params: SqlValue[] = []) => Number(Object.values(one(db, sql, params) ?? { n: 0 })[0] ?? 0);

function load(db: Database, e: number): Entity {
	const datoms = rows(
		db,
		`SELECT d.rowid AS id, d.e, d.a, at.ident AS attr, d.v, d.vt, d.tx, d.added
     FROM datom d JOIN attr at ON at.id = d.a WHERE d.e = ? ORDER BY d.a, d.rowid`,
		[e],
	) as unknown as Datom[];
	const attrs = new Set(datoms.map((d) => d.attr));
	const kind = KIND_BY.find(([a]) => attrs.has(a))?.[1] ?? 'entity';
	const identity = KIND_BY.find(([a]) => attrs.has(a))?.[0];
	const label =
		kind === 'transaction'
			? `transaction ${e}`
			: String(one(db, 'SELECT label FROM label WHERE e = ?', [e])?.label ?? datoms.find((d) => d.attr === identity)?.v ?? `entity ${e}`);
	const incoming = rows(
		db,
		`SELECT at.ident AS attr, count(DISTINCT d.e) AS n FROM datom d JOIN attr at ON at.id = d.a
     WHERE d.vt = 1 AND d.v = ? AND d.added = 1 GROUP BY at.ident ORDER BY n DESC, at.ident`,
		[e],
	).map((r) => ({ attr: String(r.attr), n: Number(r.n) }));
	let note: string | null = null;
	if (kind === 'attribute') note = `a in ${num(db, 'SELECT count(*) FROM datom WHERE a = ?', [e]).toLocaleString('en-US')} datoms`;
	if (kind === 'transaction') note = `wrote ${num(db, 'SELECT count(*) FROM datom WHERE tx = ?', [e]).toLocaleString('en-US')} datoms`;
	return { e, kind, label, datoms, incoming, note };
}

interface Start {
	title: string;
	// entities to open, by content; the first is the root
	open: string[];
	// entities placed left of the root because they point at it
	left?: number;
}

const OBS = "(SELECT e FROM observations WHERE kind = 'obs.kind/receipt')";
const TX = "(SELECT tx FROM facts WHERE attr = 'doc/kind' AND value = 'doc.kind/receipt')";
const STARTS: Start[] = [
	{
		title: 'The dinner receipt',
		open: [
			`SELECT ${OBS}`,
			"SELECT e FROM facts WHERE attr = 'reading/attr' AND value = 'extraction.receipt/tip'",
			`SELECT min(e) FROM facts WHERE attr = 'link/from' AND value = ${OBS}`,
			`SELECT value FROM facts WHERE attr = 'obs/doc' AND e = ${OBS}`,
			"SELECT r.value FROM facts t JOIN facts r ON r.e = t.e AND r.attr = 'reading/region' WHERE t.attr = 'reading/attr' AND t.value = 'extraction.receipt/tip'",
			`SELECT value FROM facts WHERE attr = 'obs/account' AND e = ${OBS}`,
			`SELECT value FROM facts WHERE attr = 'obs/commodity' AND e = ${OBS}`,
		],
		left: 2,
	},
	{
		title: 'The transaction that wrote it',
		open: [`SELECT ${TX}`, `SELECT value FROM facts WHERE attr = 'tx/source' AND e = ${TX}`, `SELECT min(value) FROM facts WHERE attr = 'tx/evidence' AND e = ${TX}`],
	},
	{ title: 'One of its attributes', open: ["SELECT id FROM attr WHERE ident = 'obs/amount'"] },
];

const W = 340;
const ROW = 22;
const nodeHeight = (en: Entity) => 34 + en.datoms.length * ROW + 30;

type Sel = { type: 'entity'; e: number } | { type: 'datom'; d: Datom } | null;

type EntityData = {
	en: Entity;
	ids: boolean;
	root: boolean;
	state: 'idle' | 'on' | 'dim';
	litRows: Set<number>;
	names: Map<number, string>;
	open: Set<number>;
	onRef: (from: number, to: number) => void;
	onIncoming: (e: number, attr: string) => void;
	onDatom: (d: Datom) => void;
	onClose: (e: number) => void;
	selRow: number | null;
};

function short(v: SqlValue, max = 34): string {
	const s = String(v);
	return s.length > max ? `${s.slice(0, max - 1)}…` : s;
}

function EntityNode({ data }: NodeProps<Node<EntityData>>) {
	const { en, ids, state, litRows, names, open, onRef, onIncoming, onDatom, onClose, root, selRow } = data;
	return (
		<div className={`fm-en fm-en-${state}`} style={{ width: W }}>
			<Handle type="target" id="in" position={Position.Left} isConnectable={false} style={{ top: 17 }} />
			<div className="fm-en-head">
				<span className="fm-en-kind">{en.kind}</span>
				<span className="fm-en-label" title={en.label}>
					{short(en.label, 40)}
				</span>
				{!root && (
					<button type="button" className="fm-en-x nodrag" aria-label={`Close ${en.label}`} onClick={(ev) => (ev.stopPropagation(), onClose(en.e))}>
						×
					</button>
				)}
			</div>
			<div className="fm-en-rows" role="table" aria-label={`The datoms of entity ${en.e}`}>
				{en.datoms.map((d) => {
					const ref = d.vt === 1;
					const target = ref ? Number(d.v) : 0;
					return (
						<div
							key={d.id}
							role="row"
							className={`fm-en-row nodrag${litRows.has(d.id) ? ' is-lit' : ''}${selRow === d.id ? ' is-sel' : ''}${d.added ? '' : ' is-retracted'}`}
							onClick={(ev) => (ev.stopPropagation(), onDatom(d))}
						>
							<span className="fm-en-e">{d.e}</span>
							<span className="fm-en-a" title={`${d.attr} (attribute ${d.a})`}>
								{ids ? d.a : d.attr}
							</span>
							{ref ? (
								<button
									type="button"
									className={`fm-en-v fm-en-ref nodrag${open.has(target) ? ' is-open' : ''}`}
									title={`${names.get(target) ?? ''} (entity ${target})`}
									onClick={(ev) => (ev.stopPropagation(), onRef(en.e, target))}
								>
									{ids ? target : `→ ${short(names.get(target) ?? target, 26)}`}
								</button>
							) : (
								<span className="fm-en-v" title={String(d.v)}>
									{short(d.v)}
								</span>
							)}
							<span className="fm-en-tx">{d.tx}</span>
							{ref && <Handle type="source" id={`r${d.id}`} position={Position.Right} isConnectable={false} />}
						</div>
					);
				})}
			</div>
			<div className="fm-en-foot">
				{en.note && <span>{en.note}</span>}
				{en.incoming.length > 0 && <span className="fm-muted">pointed at by</span>}
				{en.incoming.map((i) => (
					<button key={i.attr} type="button" className="fm-en-in nodrag" onClick={(ev) => (ev.stopPropagation(), onIncoming(en.e, i.attr))}>
						{i.n} {i.attr}
					</button>
				))}
				{!en.note && en.incoming.length === 0 && <span className="fm-muted">nothing points here</span>}
			</div>
		</div>
	);
}

const nodeTypes = { entity: EntityNode };

/** Columns: every reference points right. Seeds keep their column; targets are pushed right of what names them. */
function layout(order: number[], ents: Map<number, Entity>, seed: Map<number, number>) {
	const col = new Map(seed);
	for (const e of order) if (!col.has(e)) col.set(e, 0);
	const open = new Set(order);
	for (let pass = 0; pass < order.length; pass++) {
		let moved = false;
		for (const e of order) {
			for (const d of ents.get(e)?.datoms ?? []) {
				const t = Number(d.v);
				if (d.vt !== 1 || !open.has(t) || t === e) continue;
				if ((col.get(t) ?? 0) < (col.get(e) ?? 0) + 1 && !seed.has(t)) {
					col.set(t, (col.get(e) ?? 0) + 1);
					moved = true;
				}
			}
		}
		if (!moved) break;
	}
	const y = new Map<number, number>();
	const pos = new Map<number, { x: number; y: number }>();
	for (const e of order) {
		const c = col.get(e) ?? 0;
		const top = y.get(c) ?? 0;
		pos.set(e, { x: c * (W + 90), y: top });
		y.set(c, top + nodeHeight(ents.get(e)!) + 36);
	}
	return pos;
}

function Graph({ src }: Props) {
	const { month, error } = useMonth(src);
	const flow = useReactFlow();
	const [tab, setTab] = useState<'stored' | 'view'>('stored');
	const [start, setStart] = useState(0);
	const [order, setOrder] = useState<number[]>([]);
	const [seed, setSeed] = useState<Map<number, number>>(new Map());
	const [ents, setEnts] = useState<Map<number, Entity>>(new Map());
	const [dragged, setDragged] = useState<Map<number, { x: number; y: number }>>(new Map());
	const [sel, setSel] = useState<Sel>(null);
	const [ids, setIds] = useState(false);

	const ensure = useCallback(
		(list: number[], into: Map<number, Entity>) => {
			if (!month) return into;
			const next = new Map(into);
			for (const e of list) if (!next.has(e)) next.set(e, load(month.db, e));
			return next;
		},
		[month],
	);

	const begin = useCallback(
		(i: number) => {
			if (!month) return;
			const s = STARTS[i];
			const list = s.open.map((q) => Number(Object.values(one(month.db, q) ?? {})[0])).filter((e) => Number.isFinite(e) && e > 0);
			const root = list[0];
			const sd = new Map<number, number>([[root, 0]]);
			list.slice(1, 1 + (s.left ?? 0)).forEach((e) => sd.set(e, -1));
			setEnts((m) => ensure(list, m));
			setOrder(list);
			setSeed(sd);
			setDragged(new Map());
			setSel({ type: 'entity', e: root });
			setStart(i);
		},
		[month, ensure],
	);

	useEffect(() => {
		if (month && order.length === 0) begin(0);
	}, [month, order.length, begin]);

	const names = useMemo(() => {
		const m = new Map<number, string>();
		if (!month) return m;
		const refs = new Set<number>();
		for (const e of order) for (const d of ents.get(e)?.datoms ?? []) if (d.vt === 1) refs.add(Number(d.v));
		if (refs.size === 0) return m;
		for (const r of rows(month.db, `SELECT e, label FROM label WHERE e IN (${[...refs].join(',')})`)) m.set(Number(r.e), String(r.label));
		for (const r of refs) if (!m.has(r) && ents.get(r)) m.set(r, ents.get(r)!.label);
		return m;
	}, [month, order, ents]);

	const openOne = useCallback(
		(e: number, col: number | null) => {
			setEnts((m) => ensure([e], m));
			setOrder((o) => (o.includes(e) ? o : [...o, e]));
			if (col !== null) setSeed((s) => (s.has(e) ? s : new Map(s).set(e, col)));
			setSel({ type: 'entity', e });
		},
		[ensure],
	);

	const onRef = useCallback((_from: number, to: number) => openOne(to, null), [openOne]);

	const onIncoming = useCallback(
		(e: number, attr: string) => {
			if (!month) return;
			const all = rows(
				month.db,
				`SELECT DISTINCT d.e FROM datom d JOIN attr at ON at.id = d.a WHERE d.vt = 1 AND d.v = ? AND at.ident = ? AND d.added = 1 ORDER BY d.e`,
				[e, attr],
			).map((r) => Number(r.e));
			const next = all.find((x) => !order.includes(x)) ?? all[0];
			if (next === undefined) return;
			const c = (seed.get(e) ?? 0) - 1;
			openOne(next, c);
		},
		[month, order, seed, openOne],
	);

	const onClose = useCallback((e: number) => {
		setOrder((o) => o.filter((x) => x !== e));
		setSel(null);
	}, []);

	const onDatom = useCallback((d: Datom) => setSel({ type: 'datom', d }), []);

	const pos = useMemo(() => layout(order, ents, seed), [order, ents, seed]);

	const focus = sel?.type === 'entity' ? sel.e : sel?.type === 'datom' ? sel.d.e : null;
	const target = sel?.type === 'datom' && sel.d.vt === 1 ? Number(sel.d.v) : null;

	const nodes: Node<EntityData>[] = useMemo(() => {
		const named = new Set<number>();
		if (focus !== null) for (const d of ents.get(focus)?.datoms ?? []) if (d.vt === 1) named.add(Number(d.v));
		return order
				.filter((e) => ents.has(e))
				.map((e) => {
					const en = ents.get(e)!;
					const litRows = new Set<number>();
					for (const d of en.datoms) if (d.vt === 1 && focus !== null && Number(d.v) === focus) litRows.add(d.id);
					const on = focus === e || target === e;
					const near = litRows.size > 0 || named.has(e);
					return {
						id: String(e),
						type: 'entity',
						position: dragged.get(e) ?? pos.get(e) ?? { x: 0, y: 0 },
						data: {
							en,
							ids,
							root: e === order[0],
							state: focus === null ? 'idle' : on ? 'on' : near ? 'idle' : 'dim',
							litRows,
							names,
							open: new Set(order),
							onRef,
							onIncoming,
							onDatom,
							onClose,
							selRow: sel?.type === 'datom' ? sel.d.id : null,
						},
						connectable: false,
					};
				});
	}, [order, ents, pos, dragged, focus, target, ids, names, onRef, onIncoming, onDatom, onClose, sel],
	);

	const edges: Edge[] = useMemo(() => {
		const open = new Set(order);
		const out: Edge[] = [];
		for (const e of order) {
			for (const d of ents.get(e)?.datoms ?? []) {
				const t = Number(d.v);
				if (d.vt !== 1 || !open.has(t) || !d.added) continue;
				const lit = (sel?.type === 'datom' && sel.d.id === d.id) || (sel?.type === 'entity' && (sel.e === t || sel.e === e));
				out.push({
					id: `d${d.id}`,
					source: String(e),
					target: String(t),
					sourceHandle: `r${d.id}`,
					targetHandle: 'in',
					className: lit ? 'fm-edge-lit' : focus !== null ? 'fm-edge-dim' : undefined,
					markerEnd: { type: 'arrowclosed' as const, width: 14, height: 14, color: lit ? 'var(--lapis)' : 'var(--ink-3)' },
				});
			}
		}
		return out;
	}, [order, ents, sel, focus]);

	useEffect(() => {
		if (order.length === 0) return;
		const id = requestAnimationFrame(() => flow.fitView({ padding: 0.08, duration: 250, maxZoom: 1 }));
		return () => cancelAnimationFrame(id);
	}, [order, flow]);

	const onNodesChange = useCallback((changes: NodeChange[]) => {
		const moves = changes.filter((c) => c.type === 'position' && c.position);
		if (moves.length === 0) return;
		setDragged((m) => {
			const next = new Map(m);
			for (const c of moves) if (c.type === 'position' && c.position) next.set(Number(c.id), c.position);
			return next;
		});
	}, []);

	if (error) return <Problem>{error}</Problem>;

	const total = order.reduce((n, e) => n + (ents.get(e)?.datoms.length ?? 0), 0);

	return (
		<div className="fm-store">
			<div className="fm-layers" role="tablist" aria-label="Two ways to read the same facts">
				<button type="button" role="tab" aria-selected={tab === 'stored'} className={tab === 'stored' ? 'is-on' : undefined} onClick={() => setTab('stored')}>
					<span className="fm-n">1</span> As stored
				</button>
				<button type="button" role="tab" aria-selected={tab === 'view'} className={tab === 'view' ? 'is-on' : undefined} onClick={() => setTab('view')}>
					<span className="fm-n">2</span> As a view
				</button>
			</div>
			{!month ? (
				<div className="fm-store-pad">
					<Loading />
				</div>
			) : tab === 'stored' ? (
				<>
					<div className="fm-store-bar">
						<div className="fm-pills" role="group" aria-label="Start from">
							{STARTS.map((s, i) => (
								<button key={s.title} type="button" className={start === i ? 'is-on' : undefined} onClick={() => begin(i)}>
									{s.title}
								</button>
							))}
						</div>
						<label className="fm-store-ids">
							<input type="checkbox" name="fm-store-ids" checked={ids} onChange={(e) => setIds(e.target.checked)} /> Show a and refs as the integers stored
						</label>
					</div>
					<div className="fm-store-canvas" aria-label="Entities as runs of datoms. Click a reference to open the entity it names.">
						<ReactFlow
							nodes={nodes}
							edges={edges}
							nodeTypes={nodeTypes}
							onNodesChange={onNodesChange}
							onNodeClick={(_, n) => setSel({ type: 'entity', e: Number(n.id) })}
							onPaneClick={() => setSel(null)}
							fitView
							fitViewOptions={{ padding: 0.08, maxZoom: 1 }}
							minZoom={0.2}
							maxZoom={1.6}
							nodesConnectable={false}
							proOptions={{ hideAttribution: true }}
							preventScrolling={false}
							zoomOnScroll={false}
							zoomOnPinch
							panOnDrag
						>
							<Background gap={24} size={1} color="var(--grid-minor)" />
							<Controls showInteractive={false} position="bottom-right" />
						</ReactFlow>
					</div>
					<StoredDetails month={month.db} sel={sel} ents={ents} total={total} count={order.length} onOpen={(e) => openOne(e, null)} />
				</>
			) : (
				<ViewRow db={month.db} />
			)}
		</div>
	);
}

function DatomLine({ d }: { d: Datom }) {
	return (
		<code className="fm-datom">
			[{d.e} {d.a} {d.vt === 1 ? d.v : JSON.stringify(d.v)} {d.tx} {d.added ? 'true' : 'false'}]
		</code>
	);
}

function MiniDatoms({ en, limit = 12 }: { en: Entity; limit?: number }) {
	return (
		<table className="fm-mini">
			<colgroup>
				<col style={{ width: '42%' }} />
				<col />
				<col style={{ width: 44 }} />
			</colgroup>
			<tbody>
				{en.datoms.slice(0, limit).map((d) => (
					<tr key={d.id}>
						<td>{d.attr}</td>
						<td title={String(d.v)}>{d.vt === 1 ? `→ ${d.v}` : short(d.v, 60)}</td>
						<td className="fm-muted fm-num">{d.tx}</td>
					</tr>
				))}
				{en.datoms.length > limit && (
					<tr>
						<td colSpan={3} className="fm-muted">
							and {en.datoms.length - limit} more
						</td>
					</tr>
				)}
			</tbody>
		</table>
	);
}

function StoredDetails({
	month: db,
	sel,
	ents,
	total,
	count,
	onOpen,
}: {
	month: Database;
	sel: Sel;
	ents: Map<number, Entity>;
	total: number;
	count: number;
	onOpen: (e: number) => void;
}) {
	if (!sel)
		return (
			<div className="fm-store-details">
				<p>
					{count} entities on the canvas, {total} datoms between them. Each box is not a table: it's every row of the one <code>datom</code> table whose{' '}
					<code>e</code> is that entity, in the order the <code>(e, a, tx)</code> index keeps them. Click a reference to open the entity it names, a row to
					see the datom whole, or a "pointed at by" button to walk a reference backwards.
				</p>
			</div>
		);
	if (sel.type === 'entity') {
		const en = ents.get(sel.e) ?? load(db, sel.e);
		const txs = [...new Set(en.datoms.map((d) => d.tx))];
		const attrs = new Set(en.datoms.map((d) => d.attr)).size;
		return (
			<div className="fm-store-details">
				<p>
					<span className="fm-k">Entity {en.e}</span> {en.kind === 'entity' ? '' : `is ${/^[aeiou]/.test(en.kind) ? 'an' : 'a'} ${en.kind}: `}
					{en.datoms.length} datoms with <code>e = {en.e}</code>, using {attrs} attributes, written by {txs.length === 1 ? 'transaction' : `${txs.length} transactions:`}{' '}
					{txs.join(', ')}. Nothing else is stored for it: no row with a column per attribute, so an attribute it lacks takes no space.
				</p>
				<div className="fm-console-actions">
					<button type="button" className="fm-linkbtn" onClick={() => openInConsole(`SELECT e, a, v, vt, tx, added FROM datom\nWHERE e = ${en.e}\nORDER BY a, rowid;`, `Entity ${en.e}, as stored`)}>
						Run it in the console
					</button>
				</div>
			</div>
		);
	}
	const d = sel.d;
	const attr = load(db, d.a);
	const tx = load(db, d.tx);
	const uses = num(db, 'SELECT count(*) FROM datom WHERE a = ?', [d.a]);
	return (
		<div className="fm-store-details fm-store-details-grid">
			<div>
				<span className="fm-k">This datom</span>
				<p>
					<DatomLine d={d} />
				</p>
				<p className="fm-muted">
					Entity {d.e}, attribute {d.a} ({d.attr}), value {d.vt === 1 ? `entity ${d.v}, a reference` : short(d.v, 60)}, written by transaction {d.tx},{' '}
					{d.added ? 'asserted' : 'retracted'}.
				</p>
				{d.vt === 1 && (
					<button type="button" className="fm-linkbtn" onClick={() => onOpen(Number(d.v))}>
						Open entity {String(d.v)}
					</button>
				)}
			</div>
			<div>
				<span className="fm-k">Attribute {d.a} is an entity</span>
				<MiniDatoms en={attr} />
				<p className="fm-muted">A new attribute is a few datoms like these. The table never gains a column. This one is the <code>a</code> of {uses.toLocaleString('en-US')} datoms.</p>
			</div>
			<div>
				<span className="fm-k">Transaction {d.tx} is an entity</span>
				<MiniDatoms en={tx} limit={8} />
				<p className="fm-muted">
					The <code>tx</code> table holds only the hash chain (<code>t, instant, prev, hash</code>). Who, why and from what are datoms like these. It{' '}
					{tx.note}.
				</p>
				<button type="button" className="fm-linkbtn" onClick={() => onOpen(d.tx)}>
					Open transaction {d.tx}
				</button>
			</div>
		</div>
	);
}

// The `observations` view's columns, each with the attribute it reads and, for a reference, the attribute it reads on
// the entity named. Kept beside schema.sql's view by hand; the figure shows the view's own row next to it.
const VIEW_COLS: { col: string; attr: string; via?: string }[] = [
	{ col: 'id', attr: 'obs/id' },
	{ col: 'kind', attr: 'obs/kind' },
	{ col: 'date', attr: 'obs/date' },
	{ col: 'amount', attr: 'obs/amount' },
	{ col: 'commodity', attr: 'obs/commodity', via: 'commodity/code' },
	{ col: 'account', attr: 'obs/account', via: 'account/name' },
	{ col: 'counterparty', attr: 'obs/counterparty' },
	{ col: 'description', attr: 'obs/description' },
	{ col: 'confidence', attr: 'obs/confidence' },
	{ col: 'parent', attr: 'obs/parent', via: 'obs/id' },
	{ col: 'doc', attr: 'obs/doc', via: 'doc/id' },
];

function ViewRow({ db }: { db: Database }) {
	const [hover, setHover] = useState<string | null>(null);
	const data = useMemo(() => {
		const e = Number(Object.values(one(db, `SELECT ${OBS}`) ?? {})[0]);
		const row = one(db, 'SELECT * FROM observations WHERE e = ?', [e]) ?? {};
		const en = load(db, e);
		const vias = new Map<string, Datom | undefined>();
		for (const c of VIEW_COLS) {
			if (!c.via) continue;
			const ref = en.datoms.find((d) => d.attr === c.attr);
			if (ref) vias.set(c.col, load(db, Number(ref.v)).datoms.find((d) => d.attr === c.via));
		}
		const used = new Set(VIEW_COLS.map((c) => c.attr));
		const left = new Map<string, number>();
		for (const d of en.datoms) if (!used.has(d.attr)) left.set(d.attr, (left.get(d.attr) ?? 0) + 1);
		const nulls = VIEW_COLS.filter((c) => row[c.col] === null || row[c.col] === undefined).length;
		const hidden = en.datoms.filter((d) => !used.has(d.attr)).length;
		return { e, row, en, vias, left, nulls, hidden };
	}, [db]);
	const { e, row, en, vias, left, nulls, hidden } = data;
	const lit = VIEW_COLS.find((c) => c.col === hover);
	return (
		<div className="fm-store-pad fm-view">
			<p className="fm-view-lede">
				The same receipt as one row of the <code>observations</code> view. A view is a query: it reads the datoms on the right and lays them out as columns.
				Its {nulls} nulls are attributes this entity has no datom for. They take no space, because nothing is there. And {hidden}{' '}
				of its {en.datoms.length} datoms aren't in the view at all; <code>facts</code> still has every one.
			</p>
			<div className="fm-view-grid">
				<div>
					<span className="fm-k">SELECT * FROM observations WHERE e = {e}</span>
					<table className="fm-view-row">
						<tbody>
							{VIEW_COLS.map((c) => {
								const v = row[c.col];
								const missing = v === null || v === undefined;
								return (
									<tr
										key={c.col}
										tabIndex={0}
										className={hover === c.col ? 'is-lit' : undefined}
										onMouseEnter={() => setHover(c.col)}
										onMouseLeave={() => setHover(null)}
										onFocus={() => setHover(c.col)}
										onBlur={() => setHover(null)}
									>
										<th scope="row">{c.col}</th>
										{missing ? (
											<td className="fm-null">
												null <span className="fm-view-why">no {c.attr} datom</span>
											</td>
										) : (
											<td title={String(v)}>{short(v as SqlValue, 44)}</td>
										)}
										<td className="fm-muted">{c.via ? `${c.attr} → ${c.via}` : c.attr}</td>
									</tr>
								);
							})}
						</tbody>
					</table>
				</div>
				<div>
					<span className="fm-k">SELECT * FROM datom WHERE e = {e}</span>
					<table className="fm-view-datoms">
						<tbody>
							{en.datoms.map((d) => {
								const isLit = lit !== undefined && d.attr === lit.attr;
								const unused = !VIEW_COLS.some((c) => c.attr === d.attr);
								return (
									<tr key={d.id} className={`${isLit ? 'is-lit' : ''}${unused ? ' is-unused' : ''}`}>
										<td className="fm-muted">{d.e}</td>
										<td>{d.attr}</td>
										<td title={String(d.v)}>{d.vt === 1 ? `→ ${d.v}` : short(d.v, 30)}</td>
										<td className="fm-muted fm-num">{d.tx}</td>
									</tr>
								);
							})}
							{lit?.via && vias.get(lit.col) && (
								<tr className="is-lit fm-view-via">
									<td className="fm-muted">{vias.get(lit.col)!.e}</td>
									<td>{vias.get(lit.col)!.attr}</td>
									<td>{short(vias.get(lit.col)!.v, 30)}</td>
									<td className="fm-muted fm-num">{vias.get(lit.col)!.tx}</td>
								</tr>
							)}
						</tbody>
					</table>
					<p className="fm-muted fm-view-left">
						Not in the view:{' '}
						{[...left.entries()].map(([a, n], i) => (
							<span key={a}>
								{i > 0 ? ', ' : ''}
								<code>{a}</code>
								{n > 1 ? ` ×${n}` : ''}
							</span>
						))}
						.
					</p>
				</div>
			</div>
			<div className="fm-console-actions">
				<button
					type="button"
					className="fm-linkbtn"
					onClick={() => openInConsole(`SELECT attr, value, tx FROM facts_shown\nWHERE e = ${OBS}\nORDER BY attr;`, 'Every fact about the receipt, none left out')}
				>
					Every fact about it, in the console
				</button>
			</div>
		</div>
	);
}

export default function StoredGraph({ src }: Props) {
	return (
		<ReactFlowProvider>
			<Graph src={src} />
		</ReactFlowProvider>
	);
}
