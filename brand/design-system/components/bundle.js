/* @ds-bundle: {"format":4,"namespace":"Ownpurse","components":[{"name":"Mark"},{"name":"Wordmark"},{"name":"Coin"},{"name":"Amount"},{"name":"Button"},{"name":"Tag"},{"name":"TAccount"},{"name":"LedgerTable"},{"name":"Note"},{"name":"Sheet"},{"name":"Terminal"}]} */
(function () {
  var React = window.React, h = React.createElement;
  function cx() { return Array.prototype.filter.call(arguments, Boolean).join(' '); }

  // The mark on its 96 grid, cropped to the drawing (12 20 72 64).
  function Mark(p) {
    var size = p.size || 48, tone = p.tone || 'color', state = p.state || 'rest';
    var right;
    if (state === 'minus') right = h('rect', { className: 'fg', x: 59, y: 47.75, width: 18, height: 4.5 });
    else if (state === 'plus') right = h('g', null, h('rect', { className: 'fg', x: 59, y: 47.75, width: 18, height: 4.5 }), h('rect', { className: 'fg', x: 65.75, y: 41, width: 4.5, height: 18 }));
    else if (state === 'thinking') right = h('ellipse', { className: 'fgs', cx: 68, cy: 50, rx: 7, ry: 2.6, fill: 'none', strokeWidth: 4 });
    else right = h('circle', { className: cx('fgs', p.blink && 'op-blink'), cx: 68, cy: 50, r: 7, fill: 'none', strokeWidth: 4 });
    return h('svg', { className: cx('op-mark', tone !== 'color' && tone, p.className), viewBox: '12 20 72 64', width: size * 72 / 64, height: size,
      role: 'img', 'aria-label': p.title || 'ownpurse' },
      h('rect', { className: 'fg', x: 12, y: 20, width: 72, height: 10 }),
      h('rect', { className: 'fg', x: 43, y: 30, width: 10, height: 54 }),
      h('circle', { className: 'eye', cx: 28, cy: 50, r: 9 }),
      right);
  }

  // The wordmark as live type: the o is a coin cut to the letter's own size and baseline.
  function Wordmark(p) {
    var tone = p.tone || 'color';
    return h('span', { className: cx('op-wordmark', tone !== 'color' && tone, p.className), style: { fontSize: (p.size || 32) + 'px' }, role: 'img', 'aria-label': 'ownpurse' },
      h('span', { className: cx('o', p.blink && 'op-blink'), 'aria-hidden': 'true' }),
      h('span', { 'aria-hidden': 'true' }, 'wnpurse'));
  }

  // A status written as a coin glyph plus its word. The word always shows unless iconOnly.
  var COIN_WORD = { posted: 'Posted', balanced: 'Balanced', open: 'Open', held: 'Held', short: 'Short', over: 'Over' };
  function coinGlyph(kind, s) {
    var c = s / 2, r = s / 2 - 0.5;
    if (kind === 'posted' || kind === 'balanced') return h('svg', { width: s, height: s, viewBox: '0 0 ' + s + ' ' + s, 'aria-hidden': 'true' }, h('circle', { cx: c, cy: c, r: r, fill: 'currentColor' }));
    if (kind === 'open') return h('svg', { width: s, height: s, viewBox: '0 0 ' + s + ' ' + s, 'aria-hidden': 'true' }, h('circle', { cx: c, cy: c, r: r - s * 0.12, fill: 'none', stroke: 'currentColor', strokeWidth: s * 0.22 }));
    if (kind === 'held') return h('svg', { className: 'g-held', width: s, height: s, viewBox: '0 0 ' + s + ' ' + s, 'aria-hidden': 'true' }, h('circle', { cx: c, cy: c, r: r - s * 0.12, fill: 'none', stroke: 'currentColor', strokeWidth: s * 0.22, strokeDasharray: (s * 0.42) + ' ' + (s * 0.2) }));
    var bar = h('rect', { x: s * 0.18, y: c - s * 0.12, width: s * 0.64, height: s * 0.24, fill: 'currentColor' });
    if (kind === 'short') return h('svg', { width: s, height: s, viewBox: '0 0 ' + s + ' ' + s, 'aria-hidden': 'true' }, bar);
    return h('svg', { width: s, height: s, viewBox: '0 0 ' + s + ' ' + s, 'aria-hidden': 'true' }, bar, h('rect', { x: c - s * 0.12, y: s * 0.18, width: s * 0.24, height: s * 0.64, fill: 'currentColor' }));
  }
  function Coin(p) {
    var kind = p.kind || 'posted', word = p.label || COIN_WORD[kind];
    return h('span', { className: cx('op-coin', kind, p.className), title: p.iconOnly ? word : undefined },
      coinGlyph(kind, p.size || 12),
      p.iconOnly ? h('span', { style: { position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0 0 0 0)' } }, word) : h('span', null, word));
  }

  // Money to the cent. Negatives in parentheses and coin-text, as a bookkeeper writes them.
  function fmt(v) {
    var n = Math.abs(Number(v) || 0);
    return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
  function Amount(p) {
    var v = Number(p.value) || 0, neg = v < 0;
    var text = neg ? (p.sign === 'minus' ? '−' + fmt(v) : '(' + fmt(v) + ')') : fmt(v);
    return h('span', { className: cx('op-amount', neg && 'neg', p.total && 'total', p.className) },
      p.currency ? h('span', { className: 'cur' }, p.currency) : null, text);
  }

  function Button(p) {
    var rest = Object.assign({}, p); ['variant', 'size', 'className', 'children'].forEach(function (k) { delete rest[k]; });
    return h('button', Object.assign({ type: 'button' }, rest, { className: cx('op-btn', p.variant || 'quiet', p.size === 'sm' && 'sm', p.className) }), p.children);
  }

  function Tag(p) {
    return h('span', { className: cx('op-tag', p.tone && p.tone !== 'neutral' && p.tone, p.className) },
      p.dot ? h('span', { className: 'dot', 'aria-hidden': 'true' }) : null, p.children);
  }

  function sum(xs) { return Math.round((xs || []).reduce(function (a, x) { return a + (Number(x.amount) || 0) * 100; }, 0)) / 100; }
  function TAccount(p) {
    var dr = sum(p.debits), cr = sum(p.credits), diff = Math.round((dr - cr) * 100) / 100;
    function side(label, rows, total) {
      return h('div', { className: 'side' },
        h('div', { className: 'side-l' }, label),
        (rows || []).map(function (r, i) { return h('div', { className: 'ln', key: i }, h('span', null, r.label), h(Amount, { value: r.amount })); }),
        h('div', { className: 'tot' }, h(Amount, { value: total, total: true })));
    }
    return h('div', { className: cx('op-taccount', p.className) },
      h('div', { className: 'head' }, h('span', { className: 'name' }, p.title), p.code ? h('span', { className: 'code' }, p.code) : null),
      h('div', { className: 'body' }, side('DR', p.debits, dr), side('CR', p.credits, cr)),
      h('div', { className: 'foot' }, diff === 0 ? h(Coin, { kind: 'balanced', label: 'Nil balance' }) :
        h('span', { className: 'bal' }, (diff < 0 ? 'Credit balance ' : 'Debit balance '), h(Amount, { value: Math.abs(diff) }))));
  }

  function LedgerTable(p) {
    var rows = p.rows || [];
    var dr = Math.round(rows.reduce(function (a, r) { return a + (Number(r.debit) || 0) * 100; }, 0)) / 100;
    var cr = Math.round(rows.reduce(function (a, r) { return a + (Number(r.credit) || 0) * 100; }, 0)) / 100;
    var diff = Math.round((dr - cr) * 100) / 100;
    return h('div', { className: 'op-ledger-wrap' }, h('table', { className: cx('op-ledger', p.className) },
      h('thead', null, h('tr', null, ['Date', 'Source', 'Book', 'Account', 'Debit', 'Credit'].map(function (c, i) { return h('th', { key: c, className: i > 3 ? 'num' : undefined }, c); }))),
      h('tbody', null,
        rows.map(function (r, i) {
          return h('tr', { key: i, className: cx(r.selected && 'sel', r.difference && 'diff') },
            h('td', { className: 'mono' }, r.date), h('td', null, h(Tag, null, r.source)), h('td', null, r.book),
            h('td', null, r.account),
            h('td', { className: 'num' }, r.debit != null ? h(Amount, { value: r.debit }) : null),
            h('td', { className: 'num' }, r.credit != null ? h(Amount, { value: r.credit }) : null));
        }),
        h('tr', { className: 'totals' },
          h('td', { colSpan: 4 }, diff === 0 ? h(Coin, { kind: 'balanced' }) : h(Coin, { kind: diff < 0 ? 'short' : 'over', label: (diff < 0 ? 'Short ' : 'Over ') + fmt(diff) })),
          h('td', { className: 'num' }, h(Amount, { value: dr, total: true })),
          h('td', { className: 'num' }, h(Amount, { value: cr, total: true }))))));
  }

  function Note(p) {
    return h('div', { className: cx('op-note', p.className) }, h('span', { className: 'k' }, p.label || 'Note'), h('div', null, p.children));
  }

  function Sheet(p) {
    var tb = p.titleBlock;
    return h('div', { className: cx('op-sheet', p.className) }, p.children,
      tb ? h('div', { className: 'tb', 'aria-label': 'Title block' },
        Object.keys(tb).map(function (k) { return h('div', { key: k }, h('span', null, k), tb[k]); })) : null);
  }

  // Lines: strings, or arrays of [text, tone] where tone is coin | ok | gilt | note | dim.
  function Terminal(p) {
    var lines = p.lines || [];
    return h('div', { className: cx('op-term', p.className) },
      h('div', { className: 'bar' }, h(Mark, { size: 13, blink: p.blink }), h('span', null, p.title || 'ownpurse'), p.meta ? h('span', { className: 'meta' }, p.meta) : null),
      h('pre', null, lines.map(function (ln, i) {
        var parts = typeof ln === 'string' ? [[ln]] : ln;
        return h(React.Fragment, { key: i }, parts.map(function (pt, j) { return h('span', { key: j, className: pt[1] ? 't-' + pt[1] : undefined }, pt[0]); }), i < lines.length - 1 ? '\n' : null);
      })));
  }

  window.Ownpurse = { Mark: Mark, Wordmark: Wordmark, Coin: Coin, Amount: Amount, Button: Button, Tag: Tag, TAccount: TAccount,
    LedgerTable: LedgerTable, Note: Note, Sheet: Sheet, Terminal: Terminal };
})();
