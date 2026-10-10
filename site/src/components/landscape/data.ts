// The landscape: segments, the spectrum axes and ownpurse's own place on them. The dossiers themselves are the
// `landscape` content collection (src/content/landscape/*.md); this is what the figure needs to read them.

export type AxisKey =
	| 'custody' | 'locality' | 'history' | 'openness' | 'breadth'
	| 'autonomy' | 'agent_readiness' | 'integration_ease' | 'market';

export type Status = 'active' | 'acquired' | 'shut' | 'pivoted' | 'archived';

/** The compact form of a dossier the figure carries; the full one is on its own page. */
export type Player = {
	slug: string;
	name: string;
	url: string;
	segment: string;
	status: Status;
	tagline: string;
	domains: string[];
	customers: string[];
	scores: Record<AxisKey, number>;
	notes: Record<AxisKey, string>;
	mcp: string;
	approval: string;
	apiCost: string;
	export: string;
	pricing: string;
	relation: string;
	relationNote: string;
	href: string;
	self?: boolean;
	/** Where the work in progress puts it, for the axes that will move. Only ownpurse has one. */
	aim?: Partial<Record<AxisKey, number>>;
};

export const AXES: { key: AxisKey; label: string; short: string; lo: string; hi: string }[] = [
	{ key: 'custody', label: 'Custody', short: 'Custody', lo: 'Vendor holds your only copy', hi: 'You hold the record' },
	{ key: 'locality', label: 'Where it runs', short: 'Local', lo: 'Cloud only', hi: 'On your machine' },
	{ key: 'history', label: 'History', short: 'History', lo: 'Current state, overwritten', hi: 'Every version kept' },
	{ key: 'openness', label: 'Openness', short: 'Open', lo: 'Closed', hi: 'Open source' },
	{ key: 'breadth', label: 'Breadth', short: 'Breadth', lo: 'One source', hi: 'Books, banks, documents, payments' },
	{ key: 'autonomy', label: 'Autonomy', short: 'Autonomy', lo: 'People do the work', hi: 'Writes unattended' },
	{ key: 'agent_readiness', label: 'Agent-ready', short: 'Agents', lo: 'No API', hi: 'First-class agent interface' },
	{ key: 'integration_ease', label: 'Easy to connect', short: 'Connect', lo: 'Hard to connect to', hi: 'Self-serve, documented' },
	{ key: 'market', label: 'Market', short: 'Market', lo: 'People and freelancers', hi: 'Enterprise' },
];

export type Family = 'books' | 'work' | 'pipes' | 'agents' | 'primitives';

export const FAMILIES: Record<Family, { label: string; color: string; doc: string }> = {
	books: { label: 'Where the books live', color: 'var(--lapis)', doc: 'Ledgers, from the incumbents to plain text' },
	work: { label: 'Work on the books', color: 'var(--verdigris)', doc: 'Services, agents, close, capture and spend' },
	pipes: { label: 'Pipes', color: 'var(--gilt)', doc: 'Aggregators, unified APIs, integration and data platforms' },
	agents: { label: 'Agent-native', color: 'var(--ink-1)', doc: 'MCP servers and agent products, 2025 on' },
	primitives: { label: 'Primitives', color: 'var(--ink-3)', doc: 'Ledger databases and fact stores' },
};

export const SEGMENTS: Record<string, { label: string; family: Family }> = {
	'incumbent-ledger': { label: 'Incumbent ledgers', family: 'books' },
	'ai-ledger': { label: 'AI-native ledgers', family: 'books' },
	'plain-text-local': { label: 'Plain text and local-first', family: 'books' },
	'personal-finance': { label: 'Personal finance', family: 'books' },
	'bookkeeping-service': { label: 'Bookkeeping services', family: 'work' },
	'agent-beside-ledger': { label: 'Agents beside the ledger', family: 'work' },
	'close-analytics': { label: 'Close and analytics', family: 'work' },
	capture: { label: 'Capture', family: 'work' },
	'spend-management': { label: 'Spend management', family: 'work' },
	'crypto-accounting': { label: 'Crypto accounting', family: 'work' },
	'unified-api': { label: 'Unified accounting APIs', family: 'pipes' },
	'bank-aggregator': { label: 'Bank aggregators', family: 'pipes' },
	'ipaas-middleware': { label: 'Integration platforms', family: 'pipes' },
	'connector-framework': { label: 'Connector frameworks', family: 'pipes' },
	'data-platform': { label: 'Data platforms', family: 'pipes' },
	'agent-native': { label: 'Agent-native', family: 'agents' },
	'fact-ledger': { label: 'Ledger and fact databases', family: 'primitives' },
};

export const familyOf = (segment: string): Family => SEGMENTS[segment]?.family ?? 'pipes';
export const segmentLabel = (segment: string) => SEGMENTS[segment]?.label ?? segment;

export const RELATIONS: Record<string, string> = {
	competitor: 'Competitor',
	neighbour: 'Neighbour',
	source: 'Source',
	infrastructure: 'Infrastructure',
	kin: 'Kin',
	'cautionary-tale': 'Cautionary tale',
	'design-cousin': 'Design cousin',
	channel: 'Channel',
};

/** ownpurse, scored on the same axes by the same definitions, as it stands on the `facts` branch. */
export const OWNPURSE: Player = {
	slug: 'ownpurse',
	name: 'ownpurse',
	url: 'https://ownpurse.com',
	segment: 'plain-text-local',
	status: 'active',
	tagline: 'A local, append-only record of what every source reports, rebuilt into books you check and agents can read.',
	domains: ['ledger', 'bank', 'documents', 'crypto', 'integration'],
	customers: ['freelancer', 'smb', 'accountants', 'developers'],
	scores: {
		custody: 4, locality: 4, history: 4, openness: 4, breadth: 1,
		autonomy: 1, agent_readiness: 3, integration_ease: 1, market: 1,
	},
	notes: {
		custody: 'The record is a file on your machine; the source stays where it is.',
		locality: 'Runs on your machine; nothing passes through us.',
		history: 'Append-only and hash-chained; any past state can be queried.',
		openness: 'Open source.',
		breadth: 'One real connector, Xero. Bank files, receipts, wallets and two more ledgers are in the fact model on synthetic data, not yet in the binary.',
		autonomy: 'Agents read and propose; writes go out only as packages a person approves.',
		agent_readiness: 'A CLI with --json on every command; no MCP server yet.',
		integration_ease: 'You register your own Xero app and run a command line.',
		market: 'Freelancers and small businesses, and the people who keep their books.',
	},
	mcp: 'none',
	approval: 'Every write is a sealed package a person approves, recorded in an operations log.',
	apiCost: 'None: it uses your own Xero app on the free tier.',
	export: 'The whole record is a SQLite file you already hold.',
	pricing: 'Free and open source',
	relation: 'self',
	relationNote: 'This is us.',
	href: '/how-it-works/',
	self: true,
	aim: { breadth: 4 },
};
