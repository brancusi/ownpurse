import { useEffect, useMemo, useState } from 'react';
import {
	AXES, FAMILIES, OWNPURSE, RELATIONS, SEGMENTS, familyOf, segmentLabel,
	type AxisKey, type Family, type Player,
} from './data';
import '../facts/facts.css';
import './landscape.css';

type View = 'map' | 'compare' | 'matrix' | 'segments';

const GONE = new Set(['shut', 'acquired', 'archived', 'pivoted']);

const PRESETS: { label: string; x: AxisKey; y: AxisKey }[] = [
	{ label: 'Who holds the record, and does it keep every version', x: 'custody', y: 'history' },
	{ label: 'Who holds the record, and how much it does alone', x: 'custody', y: 'autonomy' },
	{ label: 'Who holds the record, and can agents use it', x: 'custody', y: 'agent_readiness' },
	{ label: 'Who it serves, and how much it gathers', x: 'market', y: 'breadth' },
	{ label: 'Where it runs, and how easy it is to connect', x: 'locality', y: 'integration_ease' },
];

const axis = (k: AxisKey) => AXES.find((a) => a.key === k)!;
const colorOf = (p: Player) => (p.self ? 'var(--coin)' : FAMILIES[familyOf(p.segment)].color);

// The comparison opens on the products the synthesis found closest to ownpurse.
const CLOSEST = ['numeric', 'books-dispatchlabs', 'kick', 'xero'];

export default function Landscape({ players, initial }: { players: Player[]; initial?: { x: AxisKey; y: AxisKey } }) {
	const everyone = useMemo(() => [OWNPURSE, ...players], [players]);
	const bySlug = useMemo(() => new Map(everyone.map((p) => [p.slug, p])), [everyone]);

	const [view, setView] = useState<View>('map');
	const [x, setX] = useState<AxisKey>(initial?.x ?? 'custody');
	const [y, setY] = useState<AxisKey>(initial?.y ?? 'history');
	const [families, setFamilies] = useState<Set<Family>>(() => new Set(Object.keys(FAMILIES) as Family[]));
	const [showGone, setShowGone] = useState(true);
	const [q, setQ] = useState('');
	const [selected, setSelected] = useState<string>('ownpurse');
	const [hover, setHover] = useState<string | null>(null);
	const [compare, setCompare] = useState<string[]>(() => CLOSEST.filter((s) => players.some((p) => p.slug === s)));

	// A link to #player=slug opens that player.
	useEffect(() => {
		const read = () => {
			const m = location.hash.match(/player=([a-z0-9-]+)/);
			if (m && bySlug.has(m[1])) setSelected(m[1]);
		};
		read();
		addEventListener('hashchange', read);
		return () => removeEventListener('hashchange', read);
	}, [bySlug]);

	const shown = useMemo(() => {
		const needle = q.trim().toLowerCase();
		return everyone.filter((p) => p.self || (
			families.has(familyOf(p.segment)) &&
			(showGone || !GONE.has(p.status)) &&
			(!needle || `${p.name} ${p.tagline} ${segmentLabel(p.segment)}`.toLowerCase().includes(needle))
		));
	}, [everyone, families, showGone, q]);

	const pick = (slug: string) => {
		setSelected(slug);
		try { history.replaceState(null, '', `#player=${slug}`); } catch { /* previews may refuse */ }
	};
	const toggleCompare = (slug: string) =>
		setCompare((c) => (c.includes(slug) ? c.filter((s) => s !== slug) : [...c, slug].slice(-4)));

	const sel = bySlug.get(selected) ?? OWNPURSE;

	return (
		<div className="ls">
			<div className="ls-bar">
				<div className="ls-tabs" role="tablist" aria-label="View">
					{([['map', 'Map'], ['compare', 'Compare'], ['matrix', 'Matrix'], ['segments', 'Segments']] as [View, string][]).map(([v, l]) => (
						<button key={v} role="tab" aria-selected={view === v} className={view === v ? 'is-on' : ''} onClick={() => setView(v)}>{l}</button>
					))}
				</div>
				<input className="ls-search" name="ls-search" type="search" placeholder={`Search ${players.length} players`} value={q} onChange={(e) => setQ(e.target.value)} />
			</div>
			<div className="ls-filters">
				{(Object.keys(FAMILIES) as Family[]).map((f) => (
					<button key={f} className={`ls-chip ${families.has(f) ? 'is-on' : ''}`} aria-pressed={families.has(f)} title={FAMILIES[f].doc}
						onClick={() => setFamilies((s) => { const n = new Set(s); n.has(f) ? n.delete(f) : n.add(f); return n; })}>
						<i style={{ background: FAMILIES[f].color }} />{FAMILIES[f].label}
						<span className="ls-count">{everyone.filter((p) => !p.self && familyOf(p.segment) === f).length}</span>
					</button>
				))}
				<label className="ls-check"><input type="checkbox" name="ls-gone" checked={showGone} onChange={(e) => setShowGone(e.target.checked)} /> Include shut, sold and pivoted</label>
			</div>

			<div className={`ls-body ${view === 'matrix' || view === 'segments' ? 'is-full' : ''}`}>
				<div className="ls-main">
					{view === 'map' && (
						<MapView players={shown} x={x} y={y} setX={setX} setY={setY} selected={selected} hover={hover} setHover={setHover} pick={pick} />
					)}
					{view === 'compare' && (
						<CompareView everyone={everyone} compare={compare} toggle={toggleCompare} bySlug={bySlug} pick={pick} />
					)}
					{view === 'matrix' && (
						<MatrixView players={shown} selected={selected} pick={pick} compare={compare} toggle={toggleCompare} />
					)}
					{view === 'segments' && <SegmentsView players={shown} pick={pick} selected={selected} />}
				</div>
				<Details p={sel} inCompare={compare.includes(sel.slug)} toggle={toggleCompare} goCompare={() => setView('compare')} />
			</div>
		</div>
	);
}

/* ---------- the map ---------- */

const W = 1000, H = 760, PAD = { l: 64, r: 20, t: 20, b: 56 };

function MapView({ players, x, y, setX, setY, selected, hover, setHover, pick }: {
	players: Player[]; x: AxisKey; y: AxisKey; setX: (k: AxisKey) => void; setY: (k: AxisKey) => void;
	selected: string; hover: string | null; setHover: (s: string | null) => void; pick: (s: string) => void;
}) {
	const cw = (W - PAD.l - PAD.r) / 5, ch = (H - PAD.t - PAD.b) / 5;
	// Scores are whole numbers, so players share cells: lay each cell's players out in a small grid.
	const placed = useMemo(() => {
		const cells = new Map<string, Player[]>();
		for (const p of players) {
			const k = `${p.scores[x]},${p.scores[y]}`;
			(cells.get(k) ?? cells.set(k, []).get(k)!).push(p);
		}
		const out: { p: Player; cx: number; cy: number }[] = [];
		for (const [k, ps] of cells) {
			const [sx, sy] = k.split(',').map(Number);
			ps.sort((a, b) => (a.self ? -1 : b.self ? 1 : a.name.localeCompare(b.name)));
			const cols = Math.min(ps.length, Math.max(3, Math.ceil(Math.sqrt((ps.length * cw) / ch))));
			const rows = Math.ceil(ps.length / cols);
			const gx = Math.min(30, (cw - 16) / cols), gy = Math.min(26, (ch - 16) / rows);
			ps.forEach((p, i) => {
				const c = i % cols, r = Math.floor(i / cols);
				out.push({
					p,
					cx: PAD.l + sx * cw + cw / 2 + (c - (cols - 1) / 2) * gx,
					cy: PAD.t + (4 - sy) * ch + ch / 2 + (r - (rows - 1) / 2) * gy,
				});
			});
		}
		return out;
	}, [players, x, y, cw, ch]);

	// Size is breadth: how much of a business's money one product gathers.
	const radius = (p: Player, b = p.scores.breadth) => (p.self ? 6 : 3) + b * 1.3;
	const self = placed.find((d) => d.p.self);
	const aim = self && OWNPURSE.aim ? {
		sx: OWNPURSE.aim[x] ?? OWNPURSE.scores[x], sy: OWNPURSE.aim[y] ?? OWNPURSE.scores[y],
		r: radius(OWNPURSE, OWNPURSE.aim.breadth ?? OWNPURSE.scores.breadth) + 3,
	} : null;
	const aimAt = aim && self ? (
		aim.sx === OWNPURSE.scores[x] && aim.sy === OWNPURSE.scores[y]
			? { cx: self.cx, cy: self.cy }
			: { cx: PAD.l + aim.sx * cw + cw / 2, cy: PAD.t + (4 - aim.sy) * ch + ch / 2 }
	) : null;

	const focus = hover ?? selected;
	const label = placed.find((d) => d.p.slug === focus);
	const preset = PRESETS.find((p) => p.x === x && p.y === y);

	return (
		<div className="ls-map">
			<div className="ls-axes">
				<label>Across <select name="ls-x" value={x} onChange={(e) => setX(e.target.value as AxisKey)}>{AXES.map((a) => <option key={a.key} value={a.key}>{a.label}</option>)}</select></label>
				<label>Up <select name="ls-y" value={y} onChange={(e) => setY(e.target.value as AxisKey)}>{AXES.map((a) => <option key={a.key} value={a.key}>{a.label}</option>)}</select></label>
				<div className="ls-presets">
					{PRESETS.map((p) => (
						<button key={p.label} className={preset === p ? 'is-on' : ''} onClick={() => { setX(p.x); setY(p.y); }}>{p.label}</button>
					))}
				</div>
			</div>
			<svg viewBox={`0 0 ${W} ${H}`} className="ls-svg" role="img" aria-label={`${players.length} players by ${axis(x).label} and ${axis(y).label}`}>
				{[0, 1, 2, 3, 4].map((i) => (
					<g key={i}>
						<line x1={PAD.l + i * cw} x2={PAD.l + i * cw} y1={PAD.t} y2={H - PAD.b} className="ls-grid" />
						<line x1={PAD.l} x2={W - PAD.r} y1={PAD.t + i * ch} y2={PAD.t + i * ch} className="ls-grid" />
						<text x={PAD.l + i * cw + cw / 2} y={H - PAD.b + 18} className="ls-tick" textAnchor="middle">{i}</text>
						<text x={PAD.l - 12} y={PAD.t + (4 - i) * ch + ch / 2 + 4} className="ls-tick" textAnchor="end">{i}</text>
					</g>
				))}
				<rect x={PAD.l} y={PAD.t} width={W - PAD.l - PAD.r} height={H - PAD.t - PAD.b} className="ls-frame" />
				<text x={PAD.l} y={H - 12} className="ls-axis-lo">{axis(x).lo}</text>
				<text x={W - PAD.r} y={H - 12} className="ls-axis-hi" textAnchor="end">{axis(x).hi} →</text>
				<text transform={`translate(18 ${H - PAD.b}) rotate(-90)`} className="ls-axis-lo">{axis(y).lo}</text>
				<text transform={`translate(18 ${PAD.t}) rotate(-90)`} className="ls-axis-hi" textAnchor="end">{axis(y).hi} →</text>
				{placed.map(({ p, cx, cy }) => {
					const gone = GONE.has(p.status);
					const on = p.slug === selected;
					return (
						<g key={p.slug} className={`ls-dot ${on ? 'is-on' : ''}`} transform={`translate(${cx} ${cy})`}
							onMouseEnter={() => setHover(p.slug)} onMouseLeave={() => setHover(null)} onClick={() => pick(p.slug)}
							tabIndex={0} role="button" aria-label={`${p.name}, ${segmentLabel(p.segment)}`}
							onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); pick(p.slug); } }}>
							<circle r={14} className="ls-hit" />
							{on && <circle r={radius(p) + 4} className="ls-ring" />}
							<circle r={radius(p)} style={gone ? { stroke: colorOf(p) } : { fill: colorOf(p) }} className={gone ? 'ls-gone' : ''} />
						</g>
					);
				})}
				{aim && aimAt && self && (
					<g className="ls-aim" pointerEvents="none">
						{(aimAt.cx !== self.cx || aimAt.cy !== self.cy) && <line x1={self.cx} y1={self.cy} x2={aimAt.cx} y2={aimAt.cy} />}
						<circle cx={aimAt.cx} cy={aimAt.cy} r={aim.r} />
					</g>
				)}
				{label && (
					<g className="ls-label" transform={`translate(${label.cx} ${label.cy - 15})`} pointerEvents="none">
						<text textAnchor={label.cx > W - 160 ? 'end' : label.cx < 160 ? 'start' : 'middle'}>{label.p.name}</text>
					</g>
				)}
			</svg>
			<p className="ls-legend">
				{(Object.keys(FAMILIES) as Family[]).map((f) => <span key={f}><i style={{ background: FAMILIES[f].color }} />{FAMILIES[f].label}</span>)}
				<span><i style={{ background: 'var(--coin)' }} />ownpurse</span>
				<span><i className="ls-legend-aim" />ownpurse with the connectors in progress</span>
				<span>Larger means broader</span>
				<span><i className="ls-legend-gone" />Shut, sold or pivoted</span>
			</p>
		</div>
	);
}

/* ---------- compare ---------- */

function Radar({ ps }: { ps: Player[] }) {
	const R = 150, C = 200, n = AXES.length;
	const pt = (i: number, v: number) => {
		const a = (Math.PI * 2 * i) / n - Math.PI / 2;
		return [C + Math.cos(a) * (R * v) / 4, C + Math.sin(a) * (R * v) / 4];
	};
	const strokes = ['var(--coin)', 'var(--lapis)', 'var(--verdigris)', 'var(--gilt)', 'var(--ink-2)'];
	return (
		<svg viewBox="-50 -10 500 420" className="ls-radar" role="img" aria-label={`Radar of ${ps.map((p) => p.name).join(', ')}`}>
			{[1, 2, 3, 4].map((v) => (
				<polygon key={v} className="ls-grid" points={AXES.map((_, i) => pt(i, v).join(',')).join(' ')} fill="none" />
			))}
			{AXES.map((a, i) => {
				const [lx, ly] = pt(i, 4.9);
				const [ex, ey] = pt(i, 4);
				return (
					<g key={a.key}>
						<line x1={C} y1={C} x2={ex} y2={ey} className="ls-grid" />
						<text x={lx} y={ly + 4} textAnchor={lx < C - 10 ? 'end' : lx > C + 10 ? 'start' : 'middle'} className="ls-tick">{a.short}</text>
					</g>
				);
			})}
			{ps.map((p, j) => (
				<polygon key={p.slug} points={AXES.map((a, i) => pt(i, p.scores[a.key]).join(',')).join(' ')}
					style={{ stroke: strokes[j % strokes.length], fill: strokes[j % strokes.length] }} className="ls-shape" />
			))}
		</svg>
	);
}

function CompareView({ everyone, compare, toggle, bySlug, pick }: {
	everyone: Player[]; compare: string[]; toggle: (s: string) => void; bySlug: Map<string, Player>; pick: (s: string) => void;
}) {
	const ps = [OWNPURSE, ...compare.map((s) => bySlug.get(s)).filter((p): p is Player => !!p && !p.self)];
	const strokes = ['var(--coin)', 'var(--lapis)', 'var(--verdigris)', 'var(--gilt)', 'var(--ink-2)'];
	const rest = everyone.filter((p) => !p.self && !compare.includes(p.slug)).sort((a, b) => a.name.localeCompare(b.name));
	return (
		<div className="ls-compare">
			<div className="ls-compare-head">
				<p>ownpurse against up to four others. Add one here, or with “Compare” on any player.</p>
				<select name="ls-add" value="" onChange={(e) => e.target.value && toggle(e.target.value)}>
					<option value="">Add a player…</option>
					{rest.map((p) => <option key={p.slug} value={p.slug}>{p.name}</option>)}
				</select>
			</div>
			<div className="ls-compare-grid">
				<Radar ps={ps} />
				<div className="ls-scroll-x">
				<table className="ls-table ls-table-compare">
					<thead>
						<tr><th />{ps.map((p, j) => (
							<th key={p.slug}>
								<span className="ls-swatch" style={{ background: strokes[j] }} />
								<button className="ls-name" onClick={() => pick(p.slug)}>{p.name}</button>
								{!p.self && <button className="ls-x" aria-label={`Remove ${p.name}`} onClick={() => toggle(p.slug)}>×</button>}
							</th>
						))}</tr>
					</thead>
					<tbody>
						{AXES.map((a) => (
							<tr key={a.key}>
								<th scope="row" title={`0 ${a.lo}; 4 ${a.hi}`}>{a.label}</th>
								{ps.map((p) => <td key={p.slug} className="ls-heat" data-v={p.scores[a.key]} title={p.notes[a.key]}>{p.scores[a.key]}</td>)}
							</tr>
						))}
						<tr><th scope="row">MCP</th>{ps.map((p) => <td key={p.slug}>{p.mcp}</td>)}</tr>
						<tr><th scope="row">Entry price</th>{ps.map((p) => <td key={p.slug} className="ls-wrap"><span className="ls-clamp" title={p.pricing}>{p.pricing}</span></td>)}</tr>
					</tbody>
				</table>
				</div>
			</div>
		</div>
	);
}

/* ---------- matrix ---------- */

type SortKey = 'name' | 'segment' | AxisKey;

function MatrixView({ players, selected, pick, compare, toggle }: {
	players: Player[]; selected: string; pick: (s: string) => void; compare: string[]; toggle: (s: string) => void;
}) {
	const [sort, setSort] = useState<{ k: SortKey; desc: boolean }>({ k: 'custody', desc: true });
	const rows = useMemo(() => {
		const r = [...players];
		r.sort((a, b) => {
			if (a.self || b.self) return a.self ? -1 : 1;
			const va = sort.k === 'name' ? a.name : sort.k === 'segment' ? segmentLabel(a.segment) : a.scores[sort.k];
			const vb = sort.k === 'name' ? b.name : sort.k === 'segment' ? segmentLabel(b.segment) : b.scores[sort.k];
			const c = typeof va === 'number' ? va - (vb as number) : String(va).localeCompare(String(vb));
			return (sort.desc ? -c : c) || a.name.localeCompare(b.name);
		});
		return r;
	}, [players, sort]);
	const head = (k: SortKey, label: string, title?: string) => (
		<th title={title} aria-sort={sort.k === k ? (sort.desc ? 'descending' : 'ascending') : 'none'}>
			<button onClick={() => setSort((s) => ({ k, desc: s.k === k ? !s.desc : k !== 'name' && k !== 'segment' }))}>
				{label}{sort.k === k ? (sort.desc ? ' ↓' : ' ↑') : ''}
			</button>
		</th>
	);
	return (
		<div className="ls-matrix">
			<p className="ls-hint">Every axis is scored 0 to 4. Hover a score for the reason, click a header to sort, click a row for the summary.</p>
			<div className="ls-scroll">
				<table className="ls-table ls-table-matrix">
					<thead>
						<tr>
							{head('name', 'Player')}
							{head('segment', 'Segment')}
							{AXES.map((a) => head(a.key, a.short, `${a.label}: 0 ${a.lo}; 4 ${a.hi}`))}
							<th>MCP</th>
							<th>Compare</th>
						</tr>
					</thead>
					<tbody>
						{rows.map((p) => (
							<tr key={p.slug} className={`${p.slug === selected ? 'is-on' : ''} ${p.self ? 'is-self' : ''}`} onClick={() => pick(p.slug)}>
								<th scope="row">
									<i className="ls-swatch" style={GONE.has(p.status) ? { border: `2px solid ${colorOf(p)}` } : { background: colorOf(p) }} />
									<button className="ls-name" onClick={(e) => { e.stopPropagation(); pick(p.slug); }}>{p.name}</button>
								</th>
								<td className="ls-muted ls-seg-cell">{p.self ? 'This project' : segmentLabel(p.segment)}</td>
								{AXES.map((a) => <td key={a.key} className="ls-heat" data-v={p.scores[a.key]} title={p.notes[a.key]}>{p.scores[a.key]}</td>)}
								<td className="ls-muted">{p.mcp}</td>
								<td>{!p.self && (
									<input type="checkbox" name={`ls-cmp-${p.slug}`} aria-label={`Compare ${p.name}`} checked={compare.includes(p.slug)}
										onClick={(e) => e.stopPropagation()} onChange={() => toggle(p.slug)} />
								)}</td>
							</tr>
						))}
					</tbody>
				</table>
			</div>
		</div>
	);
}

/* ---------- segments ---------- */

function SegmentsView({ players, pick, selected }: { players: Player[]; pick: (s: string) => void; selected: string }) {
	const segs = Object.entries(SEGMENTS).map(([k, s]) => ({ k, ...s, ps: players.filter((p) => !p.self && p.segment === k) })).filter((s) => s.ps.length);
	return (
		<div className="ls-segs">
			{(Object.keys(FAMILIES) as Family[]).map((f) => {
				const mine = segs.filter((s) => s.family === f);
				if (!mine.length) return null;
				return (
					<section key={f} className="ls-fam">
						<h3><i style={{ background: FAMILIES[f].color }} />{FAMILIES[f].label}</h3>
						<p className="ls-muted">{FAMILIES[f].doc}</p>
						<div className="ls-seg-grid">
							{mine.map((s) => (
								<div key={s.k} className="ls-seg">
									<p className="fm-k">{s.label} · {s.ps.length}</p>
									<ul>
										{s.ps.sort((a, b) => a.name.localeCompare(b.name)).map((p) => (
											<li key={p.slug}>
												<button className={`ls-card ${p.slug === selected ? 'is-on' : ''}`} onClick={() => pick(p.slug)}>
													<b>{p.name}{GONE.has(p.status) && <span className="ls-status"> {p.status}</span>}</b>
													<span>{p.tagline}</span>
												</button>
											</li>
										))}
									</ul>
								</div>
							))}
						</div>
					</section>
				);
			})}
		</div>
	);
}

/* ---------- the selected player ---------- */

function Details({ p, inCompare, toggle, goCompare }: { p: Player; inCompare: boolean; toggle: (s: string) => void; goCompare: () => void }) {
	return (
		<aside className="ls-detail" aria-live="polite">
			<p className="fm-k" style={{ color: colorOf(p) === 'var(--ink-3)' ? undefined : colorOf(p) }}>
				{p.self ? 'This project' : segmentLabel(p.segment)}{GONE.has(p.status) ? ` · ${p.status}` : ''}
			</p>
			<h3>{p.name}</h3>
			<p>{p.tagline}</p>
			{!p.self && <p className="ls-rel"><b>{RELATIONS[p.relation] ?? p.relation}.</b> {p.relationNote}</p>}
			<dl className="ls-bars">
				{AXES.map((a) => (
					<div key={a.key} title={p.notes[a.key]}>
						<dt>{a.label}</dt>
						<dd><span className="ls-bar-track"><span style={{ width: `${p.scores[a.key] * 25}%`, background: colorOf(p) }} /></span><span className="ls-bar-n">{p.scores[a.key]}</span></dd>
					</div>
				))}
			</dl>
			<dl className="ls-facts">
				<div><dt>Agents</dt><dd>MCP: {p.mcp}. {p.approval}</dd></div>
				<div><dt>Data access</dt><dd>{p.apiCost}</dd></div>
				<div><dt>Export</dt><dd>{p.export}</dd></div>
				<div><dt>Price</dt><dd>{p.pricing}</dd></div>
			</dl>
			<div className="ls-actions">
				<a className="fm-btn" href={p.href}>{p.self ? 'How ownpurse works' : 'Read the dossier'}</a>
				{!p.self && (
					<button className="fm-btn fm-btn-quiet" onClick={() => { if (!inCompare) toggle(p.slug); goCompare(); }}>
						{inCompare ? 'In the comparison' : 'Compare with ownpurse'}
					</button>
				)}
			</div>
		</aside>
	);
}
