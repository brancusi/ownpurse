import { useCallback, useEffect, useMemo, useState } from 'react';
import {
	ReactFlow,
	ReactFlowProvider,
	Background,
	Controls,
	Handle,
	Position,
	BaseEdge,
	EdgeLabelRenderer,
	getSmoothStepPath,
	useReactFlow,
	type Node,
	type Edge,
	type NodeProps,
	type EdgeProps,
} from '@xyflow/react';
import '@xyflow/react/dist/base.css';
import { KINDS, LAYERS, REFS, type Kind, type Layer, type Ref } from './model';
import { openInConsole, rows, run } from './db';
import { Loading, Problem, ResultTable, useMonth, type Props } from './ui';

const W = 220;
const height = (k: Kind) => 50 + 17 * k.attrs.length;
const width = (k: Kind) => (k.wide ? 4 * 300 + W : W);

// An example from the month for each kind, chosen along the dinner's path where there is one.
const EXAMPLES: Record<string, string> = {
	source: "SELECT e FROM facts WHERE attr = 'source/id' AND value = 'receipts:tallowmere'",
	file: "SELECT e FROM files WHERE names = 'copper-pot-2026-03-12.jpg'",
	document: "SELECT e FROM facts WHERE attr = 'doc/kind' AND value = 'doc.kind/receipt'",
	region: "SELECT e FROM facts WHERE attr = 'region/locator' AND value = 'page:1/box:566,1005,685,1036'",
	observation: "SELECT e FROM observations WHERE kind = 'obs.kind/receipt'",
	reading: "SELECT e FROM facts WHERE attr = 'reading/attr' AND value = 'extraction.receipt/tip'",
	link: "SELECT e FROM links WHERE kind = 'link.kind/evidences' AND from_id LIKE 'receipts:%' AND to_id LIKE 'xero:%'",
	commodity: "SELECT e FROM facts WHERE attr = 'commodity/code' AND value LIKE 'USDC%'",
	party: "SELECT e FROM facts WHERE attr = 'party/name' AND value = 'Fernhill Grocers'",
	book: "SELECT e FROM facts WHERE attr = 'book/id' AND value = 'xero:tallowmere'",
	account: "SELECT e FROM facts WHERE attr = 'account/id' AND value = 'ridgeway:checking:4821'",
	entry: "SELECT e FROM facts WHERE attr = 'entry/id' AND value LIKE '%c0ffee05-0000-4000-8000-000000000003#entry'",
	posting:
		"SELECT p.value FROM facts en JOIN facts p ON p.e = en.e AND p.attr = 'entry/posting' WHERE en.attr = 'entry/id' AND en.value LIKE '%c0ffee05-0000-4000-8000-000000000003#entry' ORDER BY p.value LIMIT 1",
	tx: "SELECT tx FROM facts WHERE attr = 'doc/kind' AND value = 'doc.kind/receipt'",
};

type KindData = { kind: Kind; count: number | null; state: 'idle' | 'on' | 'near' | 'dim' };

function KindNode({ data }: NodeProps<Node<KindData>>) {
	const { kind, count, state } = data;
	const refs = new Set(kind.refs);
	return (
		<div
			className={`fm-kn fm-kn-${state}${kind.empty ? ' fm-kn-empty' : ''}${kind.wide ? ' fm-kn-wide' : ''}`}
			style={{ width: width(kind) }}
		>
			{(['Top', 'Right', 'Bottom', 'Left'] as const).map((side) => (
				<span key={side}>
					<Handle type="source" id={`s${side}`} position={Position[side]} isConnectable={false} />
					<Handle type="target" id={`t${side}`} position={Position[side]} isConnectable={false} />
				</span>
			))}
			<div className="fm-kn-head">
				<span className="fm-kn-name">{kind.label}</span>
				<span className="fm-kn-count">{count === null ? '' : count.toLocaleString('en-US')}</span>
			</div>
			<ul className="fm-kn-attrs">
				{kind.attrs.map((a) => (
					<li key={a} className={refs.has(a) ? 'fm-ref' : undefined}>
						{a}
						{refs.has(a) ? ' →' : ''}
					</li>
				))}
			</ul>
		</div>
	);
}

/** A reference from a kind to itself: a loop out of the right edge and back in at the top. */
function LoopEdge({ id, sourceX, sourceY, targetX, targetY, label, style, markerEnd }: EdgeProps) {
	const path = `M ${sourceX} ${sourceY} C ${sourceX + 70} ${sourceY}, ${targetX + 10} ${targetY - 80}, ${targetX} ${targetY}`;
	return (
		<>
			<BaseEdge id={id} path={path} style={style} markerEnd={markerEnd} />
			<EdgeLabelRenderer>
				<div className="fm-edge-label nodrag nopan" style={{ transform: `translate(-50%, -50%) translate(${sourceX + 46}px, ${targetY - 44}px)` }}>
					{label}
				</div>
			</EdgeLabelRenderer>
		</>
	);
}

function RefEdge({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, label, style, markerEnd }: EdgeProps) {
	const [path, lx, ly] = getSmoothStepPath({ sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, borderRadius: 2, offset: 18 });
	return (
		<>
			<BaseEdge id={id} path={path} style={style} markerEnd={markerEnd} />
			<EdgeLabelRenderer>
				<div className="fm-edge-label nodrag nopan" style={{ transform: `translate(-50%, -50%) translate(${lx}px, ${ly}px)` }}>
					{label}
				</div>
			</EdgeLabelRenderer>
		</>
	);
}

const nodeTypes = { kind: KindNode };
const edgeTypes = { ref: RefEdge, loop: LoopEdge };

function sides(a: Kind, b: Kind): [string, string] {
	if (a.id === b.id) return ['sRight', 'tTop'];
	if (a.wide) return ['sTop', 'tBottom'];
	const ax = a.x + width(a) / 2;
	const ay = a.y + height(a) / 2;
	const bx = b.x + width(b) / 2;
	const by = b.y + height(b) / 2;
	if (Math.abs(ax - bx) > 40) return ax > bx ? ['sLeft', 'tRight'] : ['sRight', 'tLeft'];
	return ay > by ? ['sTop', 'tBottom'] : ['sBottom', 'tTop'];
}

type Selection = { type: 'kind'; id: string } | { type: 'ref'; id: string } | { type: 'layer'; id: Layer } | null;

function Canvas({ src }: Props) {
	const { month, error } = useMonth(src);
	const [sel, setSel] = useState<Selection>(null);
	const flow = useReactFlow();

	const counts = useMemo(() => {
		if (!month) return {} as Record<string, number>;
		return Object.fromEntries(KINDS.map((k) => [k.id, Number(run(month.db, k.count).rows[0]?.[0] ?? 0)]));
	}, [month]);

	const near = useMemo(() => {
		const s = new Set<string>();
		if (sel?.type === 'kind') {
			s.add(sel.id);
			for (const r of REFS) if (r.from === sel.id || r.to === sel.id) s.add(r.from).add(r.to);
		} else if (sel?.type === 'ref') {
			const r = REFS.find((x) => x.id === sel.id);
			if (r) s.add(r.from).add(r.to);
		} else if (sel?.type === 'layer') {
			for (const k of KINDS) if (k.layer === sel.id) s.add(k.id);
		}
		return s;
	}, [sel]);

	const nodes: Node<KindData>[] = useMemo(
		() =>
			KINDS.map((kind) => ({
				id: kind.id,
				type: 'kind',
				position: { x: kind.x, y: kind.y },
				data: {
					kind,
					count: month ? (counts[kind.id] ?? 0) : null,
					state: !sel ? 'idle' : sel.type === 'kind' && sel.id === kind.id ? 'on' : near.has(kind.id) ? 'near' : 'dim',
				},
				draggable: false,
				connectable: false,
			})),
		[month, counts, sel, near],
	);

	const edges: Edge[] = useMemo(() => {
		const byId = Object.fromEntries(KINDS.map((k) => [k.id, k]));
		return REFS.map((r) => {
			const [sourceHandle, targetHandle] = sides(byId[r.from], byId[r.to]);
			const lit = sel?.type === 'ref' ? sel.id === r.id : sel?.type === 'kind' ? r.from === sel.id || r.to === sel.id : false;
			const dim = sel !== null && !lit && sel.type !== 'layer';
			return {
				id: r.id,
				source: r.from,
				target: r.to,
				sourceHandle,
				targetHandle,
				type: r.from === r.to ? 'loop' : 'ref',
				label: r.attr,
				className: lit ? 'fm-edge-lit' : dim ? 'fm-edge-dim' : undefined,
				markerEnd: { type: 'arrowclosed' as const, width: 14, height: 14, color: lit ? 'var(--lapis)' : 'var(--ink-3)' },
				focusable: true,
			} as Edge;
		});
	}, [sel]);

	const pick = useCallback(
		(next: Selection) => {
			setSel(next);
			if (next?.type === 'kind') {
				const k = KINDS.find((x) => x.id === next.id);
				if (k) flow.setCenter(k.x + width(k) / 2, k.y + height(k) / 2, { zoom: Math.max(flow.getZoom(), 0.75), duration: 300 });
			}
		},
		[flow],
	);

	useEffect(() => {
		const on = (e: Event) => pick({ type: 'kind', id: (e as CustomEvent<{ id: string }>).detail.id });
		window.addEventListener('fm:select', on);
		return () => window.removeEventListener('fm:select', on);
	}, [pick]);

	return (
		<div className="fm-model">
			<div className="fm-layers" role="toolbar" aria-label="Layers">
				{LAYERS.map((l) => (
					<button
						key={l.id}
						type="button"
						className={sel?.type === 'layer' && sel.id === l.id ? 'is-on' : undefined}
						onClick={() => pick(sel?.type === 'layer' && sel.id === l.id ? null : { type: 'layer', id: l.id })}
					>
						<span className="fm-n">{l.n}</span> {l.label}
					</button>
				))}
			</div>
			<div className="fm-model-body">
				<div className="fm-canvas" aria-label="The fact model as a graph. Use the list below the canvas to choose a kind with the keyboard.">
					<ReactFlow
						nodes={nodes}
						edges={edges}
						nodeTypes={nodeTypes}
						edgeTypes={edgeTypes}
						fitView
						fitViewOptions={{ padding: 0.08 }}
						minZoom={0.2}
						maxZoom={1.6}
						nodesDraggable={false}
						nodesConnectable={false}
						elementsSelectable
						onNodeClick={(_, n) => pick(sel?.type === 'kind' && sel.id === n.id ? null : { type: 'kind', id: n.id })}
						onEdgeClick={(_, e) => pick({ type: 'ref', id: e.id })}
						onPaneClick={() => setSel(null)}
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
				<Details sel={sel} month={month} error={error} counts={counts} onPick={pick} />
			</div>
			<div className="fm-kindlist" role="list" aria-label="Kinds">
				{KINDS.map((k) => (
					<button
						key={k.id}
						role="listitem"
						type="button"
						className={sel?.type === 'kind' && sel.id === k.id ? 'is-on' : undefined}
						onClick={() => pick({ type: 'kind', id: k.id })}
					>
						{k.label}
					</button>
				))}
			</div>
		</div>
	);
}

function Details({
	sel,
	month,
	error,
	counts,
	onPick,
}: {
	sel: Selection;
	month: ReturnType<typeof useMonth>['month'];
	error: string | null;
	counts: Record<string, number>;
	onPick: (s: Selection) => void;
}) {
	if (error) return <aside className="fm-details"><Problem>{error}</Problem></aside>;
	if (!sel)
		return (
			<aside className="fm-details">
				<span className="fm-k">How to read it</span>
				<p>
					Each box is a kind of thing in the log, with the attributes it is described by. Arrows are references, drawn in lapis on the box:
					they point from the thing that refers to the thing it refers to. The figure on each box is how many the synthetic month holds.
				</p>
				<p>Click a box or an arrow for what it is, why it exists and an example from the month. Drag to pan; pinch or use the buttons to zoom.</p>
				{month ? (
					<p className="fm-fine-inline">
						{month.data.datoms.length.toLocaleString('en-US')} datoms in {month.data.tx.length} transactions, rebuilt in your browser in{' '}
						{month.ms.toFixed(0)} ms, from commit <code>{month.data.commit.slice(0, 7)}</code>.
					</p>
				) : (
					<Loading />
				)}
			</aside>
		);
	if (sel.type === 'layer') {
		const layer = LAYERS.find((l) => l.id === sel.id)!;
		return (
			<aside className="fm-details">
				<span className="fm-k">Layer {layer.n}</span>
				<h3>{layer.label}</h3>
				<p>{LAYER_TEXT[sel.id]}</p>
				<ul className="fm-pills">
					{KINDS.filter((k) => k.layer === sel.id).map((k) => (
						<li key={k.id}>
							<button type="button" onClick={() => onPick({ type: 'kind', id: k.id })}>
								{k.label}
							</button>
						</li>
					))}
				</ul>
			</aside>
		);
	}
	if (sel.type === 'ref') return <RefDetails r={REFS.find((r) => r.id === sel.id)!} month={month} onPick={onPick} />;
	return <KindDetails kind={KINDS.find((k) => k.id === sel.id)!} month={month} count={counts[sel.id]} />;
}

const LAYER_TEXT: Record<Layer, string> = {
	drawer: 'The bytes as they arrived and where they came from. Every connector does the same thing: fetch bytes, put them in the drawer, and let a versioned projector turn them into facts.',
	documents: 'What a file is, and places inside it. A region is as exact as the format allows for free: a row, a pointer, a page, a box.',
	observations: 'What each document says, one item at a time, pinned to its region. Readings keep each extracted field\'s own place and confidence.',
	links: 'How things relate, with an author and a confidence. Matching is a fact, so it can be reviewed and undone. Your labels and notes live here too.',
	books: 'Double-entry books: accounts, entries held until posted, postings in any commodity, and prices between commodities.',
	log: 'Underneath everything: the datom [entity, attribute, value, transaction, added] and the transaction that stamps it. The schema itself is facts in the same log.',
};

function KindDetails({ kind, month, count }: { kind: Kind; month: ReturnType<typeof useMonth>['month']; count?: number }) {
	const layer = LAYERS.find((l) => l.id === kind.layer)!;
	const attrs = useMemo(() => {
		if (!month) return [];
		return kind.attrs
			.filter((a) => a.includes('/'))
			.map((a) => {
				const r = rows(
					month.db,
					"SELECT (SELECT value FROM facts WHERE e = a.id AND attr = 'db/doc') AS doc, (SELECT count(*) FROM datom d WHERE d.a = a.id AND d.added = 1) AS n FROM attr a WHERE a.ident = ?",
					[a],
				)[0];
				return { a, doc: r?.doc as string | null, n: Number(r?.n ?? 0) };
			});
	}, [month, kind]);
	const example = useMemo(() => {
		if (!month || !EXAMPLES[kind.id]) return null;
		const e = run(month.db, EXAMPLES[kind.id]).rows[0]?.[0];
		if (e === undefined || e === null) return null;
		const sql =
			kind.id === 'tx'
				? 'SELECT attr, value FROM facts_shown WHERE e = ? ORDER BY attr'
				: "SELECT attr, value FROM facts_shown WHERE e = ? AND attr NOT LIKE 'xero.%' ORDER BY attr";
		return { e, result: run(month.db, sql, [e]) };
	}, [month, kind]);

	return (
		<aside className="fm-details">
			<span className="fm-k">
				Layer {layer.n} · {layer.label}
			</span>
			<h3>{kind.label}</h3>
			<p>{kind.what}</p>
			<dl>
				<dt>Why</dt>
				<dd>{kind.why}</dd>
				<dt>What makes it the same thing</dt>
				<dd>{kind.identity}</dd>
				<dt>In this month</dt>
				<dd>{count === undefined ? '…' : kind.empty && count === 0 ? 'None yet' : count.toLocaleString('en-US')}</dd>
			</dl>
			{attrs.length > 0 && (
				<>
					<span className="fm-k">Attributes</span>
					<table className="fm-attrs">
						<tbody>
							{attrs.map(({ a, doc, n }) => (
								<tr key={a}>
									<td className={kind.refs.includes(a) ? 'fm-ref' : undefined}>{a}</td>
									<td>{doc ?? <span className="fm-muted">no doc yet</span>}</td>
									<td className="fm-num">{n.toLocaleString('en-US')}</td>
								</tr>
							))}
						</tbody>
					</table>
				</>
			)}
			{example && (
				<>
					<span className="fm-k">Example: entity {String(example.e)}</span>
					<ResultTable result={example.result} max={14} />
				</>
			)}
			<button type="button" className="fm-btn" onClick={() => openInConsole(kind.query, kind.queryTitle)}>
				Query it: {kind.queryTitle.toLowerCase()}
			</button>
		</aside>
	);
}

function RefDetails({ r, month, onPick }: { r: Ref; month: ReturnType<typeof useMonth>['month']; onPick: (s: Selection) => void }) {
	const from = KINDS.find((k) => k.id === r.from)!;
	const to = KINDS.find((k) => k.id === r.to)!;
	const attrs = r.attr.split(' · ');
	const n = month
		? attrs.reduce((s, a) => s + Number(run(month.db, 'SELECT count(*) FROM facts WHERE attr = ?', [a]).rows[0]?.[0] ?? 0), 0)
		: null;
	const sql = `SELECT entity, attr, value FROM facts_shown\nWHERE attr IN (${attrs.map((a) => `'${a}'`).join(', ')})\nLIMIT 50;`;
	return (
		<aside className="fm-details">
			<span className="fm-k">Reference</span>
			<h3 className="fm-mono-h">{r.attr}</h3>
			<p>{r.note}</p>
			<dl>
				<dt>From</dt>
				<dd>
					<button type="button" className="fm-linkbtn" onClick={() => onPick({ type: 'kind', id: from.id })}>
						{from.label}
					</button>
				</dd>
				<dt>To</dt>
				<dd>
					<button type="button" className="fm-linkbtn" onClick={() => onPick({ type: 'kind', id: to.id })}>
						{to.label}
					</button>
				</dd>
				<dt>In this month</dt>
				<dd>{n === null ? '…' : `${n.toLocaleString('en-US')} facts`}</dd>
			</dl>
			<p className="fm-muted">
				A reference is an ordinary fact whose value is another entity's id. The query follows it by joining on that id.
			</p>
			<button type="button" className="fm-btn" onClick={() => openInConsole(sql, `Facts that hold ${r.attr}`)}>
				Query it: every {r.attr} fact
			</button>
		</aside>
	);
}

export default function ModelCanvas({ src }: Props) {
	return (
		<ReactFlowProvider>
			<Canvas src={src} />
		</ReactFlowProvider>
	);
}
