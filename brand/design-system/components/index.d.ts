// ownpurse components. window.Ownpurse.<Name>. React 18.

export interface MarkProps {
  /** Height in px. The drawing is 72:64, so width = size × 1.125. Default 48. */
  size?: number;
  /** color: ink with the coin eye. mono: one ink. reversed: ink-inverse on a dark ground. */
  tone?: 'color' | 'mono' | 'reversed';
  /** The open eye's mood. rest (default), thinking (squint), minus (wink), plus. */
  state?: 'rest' | 'thinking' | 'minus' | 'plus';
  /** The open eye blinks every 6s. Only with state rest. Off under reduced motion. */
  blink?: boolean;
  /** Accessible name. Default "ownpurse". */
  title?: string;
  className?: string;
}

export interface WordmarkProps {
  /** Font size in px. Default 32. Smallest 16. */
  size?: number;
  /** color: coin o. mono: the o in the text colour. reversed: ink-inverse letters. */
  tone?: 'color' | 'mono' | 'reversed';
  /** The coin-o blinks into a minus every 6s. Headers and splash screens only. */
  blink?: boolean;
  className?: string;
}

export interface CoinProps {
  /** posted and balanced: filled verdigris. open: ink-3 ring. held: gilt dashed ring. short: minus. over: plus. */
  kind?: 'posted' | 'balanced' | 'open' | 'held' | 'short' | 'over';
  /** The word. Defaults to the kind's own word (Posted, Balanced, Open, Held, Short, Over). */
  label?: string;
  /** Glyph size in px. Default 12. */
  size?: number;
  /** Hide the word visually (kept for screen readers and as a tooltip). Dense tables only. */
  iconOnly?: boolean;
  className?: string;
}

export interface AmountProps {
  /** The amount. Negative renders in parentheses and coin-text. */
  value: number;
  /** ISO code shown before the figure in ink-3, e.g. "NZD". */
  currency?: string;
  /** A total: single rule above, double rule below, medium weight. */
  total?: boolean;
  /** parens (default) or minus for negatives. */
  sign?: 'parens' | 'minus';
  className?: string;
}

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** primary: ink. coin: the one action a view exists for (at most one). quiet: outlined (default). ghost: lapis text. */
  variant?: 'primary' | 'coin' | 'quiet' | 'ghost';
  size?: 'md' | 'sm';
  children?: React.ReactNode;
}

export interface TagProps {
  /** neutral (default) for sources and books; lapis, gilt, verdigris, coin echo statuses. */
  tone?: 'neutral' | 'lapis' | 'gilt' | 'verdigris' | 'coin';
  /** A small coin before the text. */
  dot?: boolean;
  children?: React.ReactNode;
  className?: string;
}

export interface TAccountLine { label: string; amount: number }
export interface TAccountProps {
  /** Account name, above the brow rule. */
  title: string;
  /** Account code, e.g. "2410". */
  code?: string;
  debits?: TAccountLine[];
  credits?: TAccountLine[];
  className?: string;
}

export interface LedgerRow {
  date: string; source: string; book: string; account: string;
  debit?: number; credit?: number;
  /** Lapis marker: the row a person or agent is looking at. */
  selected?: boolean;
  /** coin-tint: this row is the cause of a difference. */
  difference?: boolean;
}
export interface LedgerTableProps { rows: LedgerRow[]; className?: string }

export interface NoteProps {
  /** The annotation's key, set in capitals. Default "Note". */
  label?: string;
  children?: React.ReactNode;
  className?: string;
}

export interface SheetProps {
  /** Title block fields, bottom right, e.g. { sheet: "A-002", scale: "1:3" }. */
  titleBlock?: Record<string, string>;
  children?: React.ReactNode;
  className?: string;
}

export interface TerminalProps {
  /** Each line a string, or [text, tone][] where tone is coin | ok | gilt | note | dim. */
  lines: (string | [string, ('coin' | 'ok' | 'gilt' | 'note' | 'dim')?][])[];
  /** Frame title. Default "ownpurse". */
  title?: string;
  /** Muted text after the title, e.g. "0.17 · 4 books". */
  meta?: string;
  /** The mark in the frame blinks. */
  blink?: boolean;
  className?: string;
}
