import { useCallback, useEffect, useMemo, useState } from 'react';
import { run, type Result } from './db';
import { Loading, Problem, ResultTable, useMonth, type Props } from './ui';

interface Preset {
	title: string;
	sql: string;
	from?: string;
}

// A few to start with, then every query in samples/queries/ as it was at the exported commit.
const STARTERS: Preset[] = [
	{ title: 'The datom, raw', sql: 'SELECT e, a, v, vt, tx, added FROM datom\nORDER BY rowid DESC\nLIMIT 20;' },
	{ title: 'Every fact about the dinner, by name', sql: "SELECT attr, value, tx FROM facts_shown\nWHERE entity = 'xero:tallowmere/BankTransaction/c0ffee05-0000-4000-8000-000000000003'\nORDER BY attr;" },
	{ title: 'The views you can query', sql: "SELECT name FROM sqlite_master\nWHERE type = 'view'\nORDER BY name;" },
	{ title: 'Try to change history', sql: "UPDATE datom SET v = '-1.00' WHERE vt = 4;" },
];

function title(file: string, sql: string): string {
	const first = sql.split('\n').find((l) => l.startsWith('--'));
	const text = first ? first.replace(/^--\s?/, '') : file;
	const cut = text.search(/[:.](\s|$)/);
	return cut > 0 ? text.slice(0, cut) : text;
}

export default function QueryConsole({ src }: Props) {
	const { month, error } = useMonth(src);
	const presets = useMemo<Preset[]>(
		() => [...STARTERS, ...(month?.data.queries ?? []).map((q) => ({ title: title(q.file, q.sql), sql: q.sql, from: q.file }))],
		[month],
	);
	const [sql, setSql] = useState(STARTERS[1].sql);
	const [label, setLabel] = useState<string | null>(STARTERS[1].title);
	const [result, setResult] = useState<Result | null>(null);
	const [problem, setProblem] = useState<string | null>(null);

	const go = useCallback(
		(text: string) => {
			if (!month) return;
			try {
				setResult(run(month.db, text));
				setProblem(null);
			} catch (e) {
				setResult(null);
				setProblem(e instanceof Error ? e.message : String(e));
			}
		},
		[month],
	);

	useEffect(() => {
		if (month) go(sql);
		// Run the opening query once the log is built.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [month]);

	useEffect(() => {
		const on = (e: Event) => {
			const d = (e as CustomEvent<{ sql: string; title?: string }>).detail;
			setSql(d.sql);
			setLabel(d.title ?? null);
			go(d.sql);
		};
		window.addEventListener('fm:query', on);
		return () => window.removeEventListener('fm:query', on);
	}, [go]);

	if (error) return <Problem>{error}</Problem>;

	return (
		<div className="fm-console">
			<div className="fm-console-bar">
				<label>
					<span className="fm-k">Questions</span>
					<select
						value=""
						onChange={(e) => {
							const p = presets[Number(e.target.value)];
							if (!p) return;
							setSql(p.sql);
							setLabel(p.title);
							go(p.sql);
						}}
					>
						<option value="">Pick a question…</option>
						<optgroup label="To start">
							{presets.slice(0, STARTERS.length).map((p, i) => (
								<option key={p.title} value={i}>
									{p.title}
								</option>
							))}
						</optgroup>
						<optgroup label="samples/queries">
							{presets.slice(STARTERS.length).map((p, i) => (
								<option key={p.from} value={i + STARTERS.length}>
									{p.from?.slice(0, 2)} · {p.title}
								</option>
							))}
						</optgroup>
					</select>
				</label>
				{month ? (
					<span className="fm-console-meta">
						SQLite in your browser · {month.data.datoms.length.toLocaleString('en-US')} datoms · commit <code>{month.data.commit.slice(0, 7)}</code>
					</span>
				) : (
					<Loading />
				)}
			</div>
			{label && <div className="fm-console-title">{label}</div>}
			<textarea
				className="fm-sql"
				spellCheck={false}
				value={sql}
				aria-label="SQL to run against the month's log"
				rows={Math.min(16, Math.max(4, sql.split('\n').length + 1))}
				onChange={(e) => {
					setSql(e.target.value);
					setLabel(null);
				}}
				onKeyDown={(e) => {
					if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
						e.preventDefault();
						go(sql);
					}
				}}
			/>
			<div className="fm-console-actions">
				<button type="button" className="fm-btn" disabled={!month} onClick={() => go(sql)}>
					Run
				</button>
				<span className="fm-muted">Cmd or Ctrl + Enter. Views: facts, history, transactions, files, observations, links, postings. Sum money with dsum().</span>
			</div>
			{problem && (
				<Problem>
					{problem}
					{/append-only/.test(problem) ? ' The triggers that guard the real log came along with its schema; a correction is a new transaction.' : ''}
				</Problem>
			)}
			{result && <ResultTable result={result} />}
		</div>
	);
}
