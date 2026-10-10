// The synthetic month's fact log, rebuilt in the browser. `scripts/month-log.sh` exports the log's own schema.sql,
// every transaction and datom, and the sample queries; this module replays them into SQLite (sql.js) once per page
// and registers `dsum()`, the exact decimal sum `ownpurse-facts sql` registers in Rust.
import initSqlJs, { type Database, type SqlValue } from 'sql.js';
import wasmUrl from 'sql.js/dist/sql-wasm.wasm?url';

export interface MonthExport {
	commit: string;
	format: string;
	schema: string;
	tx: [number, string, string, string][];
	datoms: [number, number, SqlValue, number, number, number][];
	queries: { file: string; sql: string }[];
}

export interface Month {
	db: Database;
	data: MonthExport;
	ms: number;
}

export interface Result {
	columns: string[];
	rows: SqlValue[][];
	ms: number;
}

// Decimal text as a scaled BigInt, so sums are exact to the last digit, as rust_decimal's are.
function parseDec(t: string): { n: bigint; s: number } {
	const m = /^([+-]?)(\d*)(?:\.(\d*))?$/.exec(t.trim());
	if (!m || (m[2] === '' && !m[3])) throw new Error(`dsum takes decimal text; got "${t}"`);
	const frac = m[3] ?? '';
	const n = BigInt((m[2] || '0') + frac);
	return { n: m[1] === '-' ? -n : n, s: frac.length };
}

function showDec(n: bigint, s: number): string {
	const neg = n < 0n;
	let digits = (neg ? -n : n).toString().padStart(s + 1, '0');
	if (s > 0) digits = `${digits.slice(0, -s)}.${digits.slice(-s)}`;
	return neg ? `-${digits}` : digits;
}

function registerDsum(db: Database) {
	db.create_aggregate('dsum', {
		init: () => null as { n: bigint; s: number } | null,
		step: (acc: { n: bigint; s: number } | null, v: SqlValue) => {
			if (v === null || v === undefined) return acc;
			if (typeof v === 'number' && !Number.isInteger(v)) throw new Error('dsum takes decimal text; a REAL would already have lost cents');
			const d = parseDec(String(v));
			if (!acc) return d;
			const s = Math.max(acc.s, d.s);
			return { n: acc.n * 10n ** BigInt(s - acc.s) + d.n * 10n ** BigInt(s - d.s), s };
		},
		finalize: (acc: { n: bigint; s: number } | null) => (acc ? showDec(acc.n, acc.s) : null),
	});
}

export async function buildMonth(data: MonthExport, locateFile?: (f: string) => string): Promise<Month> {
	const t0 = performance.now();
	const SQL = await initSqlJs({ locateFile: locateFile ?? (() => wasmUrl) });
	const db = new SQL.Database();
	db.exec(data.schema);
	db.exec('BEGIN');
	db.run('INSERT INTO meta(key, value) VALUES (?, ?)', ['format', data.format]);
	const tx = db.prepare('INSERT INTO tx(t, instant, prev, hash) VALUES (?, ?, ?, ?)');
	for (const row of data.tx) tx.run(row);
	tx.free();
	const datom = db.prepare('INSERT INTO datom(e, a, v, vt, tx, added) VALUES (?, ?, ?, ?, ?, ?)');
	for (const row of data.datoms) datom.run(row);
	datom.free();
	db.exec('COMMIT');
	registerDsum(db);
	return { db, data, ms: performance.now() - t0 };
}

/** The latest export. A post pins its own (`/data/month-<commit>.json`) so it keeps showing the version it was written at. */
export const LATEST = '/data/month.json';
const months = new Map<string, Promise<Month>>();

/** The month, loaded once per page however many figures ask for it. */
export function loadMonth(src = LATEST): Promise<Month> {
	let m = months.get(src);
	if (!m) {
		m = fetch(src)
			.then((r) => {
				if (!r.ok) throw new Error(`could not fetch ${src} (${r.status}); run site/scripts/month-log.sh`);
				return r.json() as Promise<MonthExport>;
			})
			.then((data) => buildMonth(data));
		months.set(src, m);
	}
	return m;
}

export function run(db: Database, sql: string, params: SqlValue[] = []): Result {
	const t0 = performance.now();
	const out = db.exec(sql, params);
	const last = out[out.length - 1];
	return { columns: last?.columns ?? [], rows: last?.values ?? [], ms: performance.now() - t0 };
}

/** One row as an object, for the figures' own lookups. */
export function rows(db: Database, sql: string, params: SqlValue[] = []): Record<string, SqlValue>[] {
	const r = run(db, sql, params);
	return r.rows.map((row) => Object.fromEntries(r.columns.map((c, i) => [c, row[i]])));
}

/** Ask other figures on the page to open this SQL in the console. */
export function openInConsole(sql: string, title?: string) {
	window.dispatchEvent(new CustomEvent('fm:query', { detail: { sql, title } }));
	document.getElementById('fm-console')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
}
