import { useMemo, useState } from 'react';
import { openInConsole, run, type Result } from './db';
import { TRACES } from './traces';
import { Command, Loading, Problem, ResultTable, useMonth, type Props } from './ui';

export default function AgentTrace({ src }: Props) {
	const { month, error } = useMonth(src);
	const [id, setId] = useState(TRACES[0].id);
	const [shown, setShown] = useState(1);
	const trace = TRACES.find((t) => t.id === id)!;
	const done = shown > trace.steps.length;

	const results = useMemo<(Result | string)[]>(() => {
		if (!month) return [];
		return trace.steps.map((s) => {
			try {
				return run(month.db, s.sql);
			} catch (e) {
				return e instanceof Error ? e.message : String(e);
			}
		});
	}, [month, trace]);

	if (error) return <Problem>{error}</Problem>;

	return (
		<div className="fm-agent">
			<div className="fm-agent-qs" role="tablist" aria-label="Questions">
				{TRACES.map((t) => (
					<button
						key={t.id}
						role="tab"
						type="button"
						aria-selected={t.id === id}
						className={t.id === id ? 'is-on' : undefined}
						onClick={() => {
							setId(t.id);
							setShown(1);
						}}
					>
						{t.question}
					</button>
				))}
			</div>
			<div className="fm-agent-body" role="tabpanel">
				<div className="fm-agent-ask">
					<span className="fm-k">You ask</span>
					<p>{trace.question}</p>
				</div>
				{!month && <Loading />}
				{month &&
					trace.steps.slice(0, Math.min(shown, trace.steps.length)).map((s, i) => {
						const r = results[i];
						return (
							<div className="fm-agent-step" key={i}>
								<div className="fm-agent-who">
									<span className="fm-avatar" aria-hidden="true" />
									<span className="fm-k">Tally · step {i + 1}</span>
								</div>
								<p className="fm-agent-think">{s.think}</p>
								<Command sql={s.sql} />
								{typeof r === 'string' ? <Problem>{r}</Problem> : r && <ResultTable result={r} max={12} />}
								<p className="fm-agent-read">{s.read}</p>
								<button type="button" className="fm-linkbtn" onClick={() => openInConsole(s.sql, `${trace.question} Step ${i + 1}`)}>
									Open this step in the console
								</button>
							</div>
						);
					})}
				{month && !done && (
					<div className="fm-agent-nav">
						<button type="button" className="fm-btn" onClick={() => setShown(shown + 1)}>
							{shown < trace.steps.length ? `Next step (${shown + 1} of ${trace.steps.length})` : 'Show the answer'}
						</button>
						<button type="button" className="fm-btn fm-btn-quiet" onClick={() => setShown(trace.steps.length + 1)}>
							Show all
						</button>
					</div>
				)}
				{month && done && (
					<div className="fm-agent-answer">
						<span className="fm-k">Answer</span>
						<p>{trace.answer}</p>
						<span className="fm-k">From</span>
						<ul className="fm-cites">
							{trace.cites.map((c) => (
								<li key={c}>{c}</li>
							))}
						</ul>
						{trace.proposal && (
							<div className="fm-proposal">
								<div className="fm-proposal-head">
									<span className="fm-status-held">
										<i />
										Held
									</span>
									<span>Proposed transaction: {trace.proposal.why}</span>
								</div>
								<table>
									<tbody>
										{trace.proposal.facts.map((f, i) => (
											<tr key={i}>
												<td className="fm-op">{f[0]}</td>
												<td>{f[1]}</td>
												<td className="fm-a">{f[2]}</td>
												<td>{f[3]}</td>
											</tr>
										))}
									</tbody>
								</table>
								<p className="fm-muted">A proposal is a speculative transaction: the log can apply it to a copy of itself (<code>with</code>) to show you the result without writing anything. Approving it would append it with your name on it. The approval step is designed, not built yet.</p>
							</div>
						)}
					</div>
				)}
			</div>
		</div>
	);
}
