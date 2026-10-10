import { useEffect, useState, type ReactNode } from 'react';
import { LATEST, loadMonth, type Month, type Result } from './db';
import './facts.css';

/** Every figure takes `src`, the export to load; figures on one page share one copy. */
export interface Props {
	src?: string;
}

export function useMonth(src = LATEST): { month: Month | null; error: string | null } {
	const [month, setMonth] = useState<Month | null>(null);
	const [error, setError] = useState<string | null>(null);
	useEffect(() => {
		let live = true;
		loadMonth(src).then(
			(m) => live && setMonth(m),
			(e: unknown) => live && setError(e instanceof Error ? e.message : String(e)),
		);
		return () => {
			live = false;
		};
	}, [src]);
	return { month, error };
}

/** Loading is the coin blinking, never a spinner. */
export function Loading({ what = 'Building the log in your browser' }: { what?: string }) {
	return (
		<div className="fm-loading" role="status">
			<span className="fm-blink" aria-hidden="true" />
			<span>{what}…</span>
		</div>
	);
}

export function Problem({ children }: { children: ReactNode }) {
	return (
		<div className="fm-problem" role="alert">
			<span className="fm-k fm-k-coin">Problem</span>
			<span>{children}</span>
		</div>
	);
}

const NUM = /^-?\d+(\.\d+)?$/;

export function Cell({ v }: { v: unknown }) {
	if (v === null || v === undefined) return <td className="fm-null">null</td>;
	const s = String(v);
	const num = typeof v === 'number' || NUM.test(s);
	return (
		<td className={num ? 'fm-num' : undefined} title={s.length > 64 ? s : undefined}>
			{s.length > 96 ? `${s.slice(0, 95)}…` : s}
		</td>
	);
}

export function ResultTable({ result, max = 200 }: { result: Result; max?: number }) {
	const { columns, rows } = result;
	return (
		<div className="fm-result">
			<div className="fm-result-meta">
				<span>
					{rows.length} {rows.length === 1 ? 'row' : 'rows'}
				</span>
				<span>{result.ms < 1 ? '<1' : result.ms.toFixed(0)} ms</span>
				{rows.length > max && <span>showing the first {max}</span>}
			</div>
			{columns.length > 0 && (
				<div className="fm-result-scroll">
					<table>
						<thead>
							<tr>
								{columns.map((c, i) => (
									<th key={i}>{c}</th>
								))}
							</tr>
						</thead>
						<tbody>
							{rows.slice(0, max).map((row, i) => (
								<tr key={i}>
									{row.map((v, j) => (
										<Cell key={j} v={v} />
									))}
								</tr>
							))}
						</tbody>
					</table>
				</div>
			)}
		</div>
	);
}

/** A command as an agent would run it, in the terminal's colours. */
export function Command({ sql }: { sql: string }) {
	return (
		<pre className="fm-term">
			<span className="fm-term-prompt">$ </span>
			<span className="fm-term-cmd">ownpurse-facts sql </span>
			<span className="fm-term-str">"{sql}"</span>
		</pre>
	);
}
