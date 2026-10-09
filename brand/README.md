# ownpurse brand

The identity of ownpurse: the mark, the wordmark, colour, type, voice and the rules for using them. This folder is
the source of truth; the website (`site/`) and the TUI theme take their values from it.

| File | What |
|---|---|
| `logos/` | outlined SVGs: the mark (colour, mono, reversed, blinking, app icon) and the wordmark (colour, mono, reversed, blinking) |
| `tokens.json` | colour (Paper and Night themes, terminal colours), type, spacing, radius, rule and shadow tokens |
| `tokens.css` | the same tokens as CSS custom properties (generated from `tokens.json`) |
| `TERMINAL.md` | the terminal and TUI mapping: colours, 16-colour fallbacks, glyphs, the frame |
| `design-system/components/` | reference components (`window.Ownpurse.*`, React 18): Mark, Wordmark, Coin, Amount, Button, Tag, TAccount, LedgerTable, Note, Sheet, Terminal, each with a README and a preview |

## How the mark is built

- **A T-account read as a face.** On a 96-unit grid: the brow is a 72 × 10 bar at (12, 20); the nose a 10 × 54 stem
  at (43, 30); the filled eye a disc of radius 9 at (28, 50); the open eye a ring of outer radius 9 (radius 7,
  stroke 4) at (68, 50). The drawing is cropped to `12 20 72 64`.
- The brow is the account's top rule, the nose divides debit (left) from credit (right), and the two equal eyes are
  the two sides of an entry. The filled eye is the **coin**.
- **Balance is the rule: everything built from the mark is symmetric about its centre line.** Never offset parts,
  never extend the brow across a word.
- **The open eye has moods**: at rest (it blinks: closes for a moment every six seconds), thinking (a squint), minus
  (closed, a wink) and plus. Use them for state in living surfaces, always alongside words.

## How the wordmark is built

- `ownpurse` in Bricolage Grotesque 800 at optical size 96, letter-spacing −0.022em, always lowercase, one word.
- **The o is a solid coin** cut to the letter's own size: its diameter is the o's ink height (0.556 em) and it sits
  exactly where the o sits, on the baseline with the same overshoot. In live type: an inline block 0.556em square,
  `vertical-align: -0.014em`, `margin-left: 0.022em`. The outlined files were made by shaping the word with HarfBuzz from the Bricolage
  Grotesque variable font instanced at wght 800, opsz 96, and replacing the o with a circle measured from its glyph.
- Versions: colour (coin-o in `coin`), mono (one ink), reversed, and blinking (the coin closes to a minus).
- **The mark and the wordmark are never locked together.** Pick one per placement: the wordmark where the name must
  be read, the mark where it is known or space is tight.

## The brand book

ownpurse is a local accounting tool for people and their agents. It keeps an append-only, tamper-evident record of your books from any source (Xero first; spreadsheets, banks and household accounts next), holds every entry before it is posted anywhere, and lets you and your agents query, reconcile and push it wherever it needs to go. The identity takes its ideas from double-entry bookkeeping's oldest tools: the T-account, the coin, the drafting board and the balance.

## The idea in one picture

The mark is a T-account read as a face. The brow is the account's top rule, the nose is the line between debit and credit, the filled eye is the coin and the open eye is the other side of the entry. Two eyes of equal size on one axis means balance, and balance is the point: **every figure built from the mark is symmetric about its centre line.**

## Content fundamentals

- **Write like a careful bookkeeper talking to the owner.** Plain, exact, unhurried. Second person: "your books", "you and your agents". The product never says "we".
- **The name is always lowercase, one word: ownpurse.** It stays lowercase at the start of a sentence too ("ownpurse keeps every version Xero reported").
- **Numbers to the cent, always.** Never round in copy ("12,480.06", not "about 12.5k"). Negatives in parentheses in tables, a true minus in running text.
- **Say what happened and what to do next.** "Sync appended 14 versions. 3 entries are held; run `ownpurse post --held` to review them." Errors name the cause and the fix, without apology.
- **Name things as people know them**: books, accounts, entries, sources. Not "tenants", "entities" or "records" in the interface (the docs may explain the terms).
- **No hype.** No "magic", "AI-powered", "seamless" or "effortless". The quantum idea (entries held before they collapse into one system's view) is for explaining the data model to builders, never a slogan.
- **Sentence case** for headings, buttons and labels. Capitals only for eyebrow labels and title-block annotations, which are set in `eyebrow`.
- **No emoji** anywhere: product, docs or social. The coin glyphs and the Mark's states do that job.

Real copy, for reference:

> Reads cost no API calls. A sync appends; it never overwrites.
> Short 86.40: card 4417 paid one invoice from two books, and the household half isn't posted yet.
> Held, not posted. Observe to collapse.

## Colour

One brand colour, the **coin**, on warm paper and ink, with three supporting pigments taken from the Renaissance workshop: **gilt**, **verdigris** and **lapis**.

- Ground: `surface-0` (paper) for pages, `surface-1` (vellum) for raised panels and table headers, `surface-2` for pressed and selected fills, `surface-inverse` for at most one dark band per page.
- Text: `ink-1` primary, `ink-2` secondary, `ink-3` muted (only on `surface-0` and `surface-1`). Text on `surface-inverse` is `ink-inverse`.
- **The coin (`coin`) is spent sparingly**: the filled eye, the coin-o, one coin button per view, the one figure on a page that matters most. Never as a background wash or for decoration. For text, use `coin-text`, which is a shade darker so small text passes 4.5:1.
- **Meaning colours always come with a word or a glyph**: `verdigris` for agreed and balanced, `gilt` and `gilt-text` for held (not yet posted), `coin-text` for negatives, short and over, `lapis` for notes, links, selection and a query's path. Their `-tint` tokens are the matching row backgrounds.
- **Gilt is the coin as money.** Use it where a figure is about value itself: illustrations of coins, a purse's contents, a held amount waiting to be posted.
- Borders: `line-1` for hairlines between rows, `line-2` for control edges (3:1 on paper).
- The terminal is always dark, in both themes: `term-ground` with `term-ink`, and the `term-*` meaning colours. See the Terminal and TUI section.
- Every text token's usage note names the grounds it passes 4.5:1 on, in both themes.

## Type

- **Bricolage Grotesque** for everything with words, from the 72px hero to 13px labels, with optical size following the type size (96 for display, 12 to 14 for text). The wordmark is Bricolage 800.
- **DM Mono** for every figure, code, account code, date, command and eyebrow label. Amounts always use tabular figures, right-aligned in columns.
- Styles: `display-xl` (once per page, website only), `display`, `title`, `heading`, `body-lg`, `body`, `body-sm`, `label`; figures in `figure` and `figure-total`; commands in `code`; capitalised annotations in `eyebrow`.
- Keep running text near 65 characters. Headings balance their lines.

## Shape, space and rules

- **Square by default** (`radius-0`), like a drawing board. Buttons, inputs and chips take `radius-1`. **The only round thing is the coin** (`radius-coin`): status glyphs, the coin-o, avatars.
- Spacing on a 4px base, `space-1` to `space-24`. Panels pad `space-6`; sections sit `space-12` apart.
- **Rules, not shadows.** A section opens with a `rule-strong` (2px `ink-1`) line; rows divide with `rule-hair` in `line-1`; a total sits on a `rule-double`. The single shadow, `shadow-float`, is for things that float (menus, the command palette).

## The coin as a motif

The coin is money, the eye, the purse's contents and the Renaissance in one shape. Use it as a recurring prop:

- as **the status glyph** (Coin component);
- as **bullets** in marketing lists (a 0.4em `coin` disc in place of a dot);
- as **the avatar** of the agent (the Mark in a coin-sized frame, blinking while idle);
- as **a loading state**: the coin-o or the open eye blinking, never a spinner;
- in illustration, stacks and single coins in `gilt` with the brand's one `coin`.

Don't put words inside coins, use more than one `coin`-coloured coin per view, or make coins 3D.

## Graph paper is an element

The drafting board (`grid-major`, `grid-minor`, the Sheet component with its title block) is a prop for diagrams, worked examples and the occasional hero. **At most one Sheet per screen**, never as a page background, never behind running text. Its annotations use `lapis` and `eyebrow`.

## Motion

One motion: **the blink**. The open eye (or the coin-o) closes for a moment every six seconds. It means "alive and watching": the TUI frame, the agent's avatar while it works, splash screens and the website header. Nothing else animates beyond 150ms colour and opacity transitions. Under `prefers-reduced-motion` the blink stops.

## Focus and states

- Focus: a 2px solid `focus` (lapis) ring with a 2px offset, at least 3:1 on every surface.
- Selected row: `lapis-tint` background with a 3px `lapis` marker on the left edge of the first cell.
- Difference: `coin-tint` row with the amount in `coin-text`, and a Coin saying Short or Over.
- Disabled: 45% opacity, with the reason in nearby text.

## Logos

The Logos group holds outlined SVGs: the Mark (colour, mono, reversed, blinking, app icon) and the Wordmark (colour, mono, reversed, blinking).

- **The Mark and the Wordmark are never locked together.** Choose one per placement: the Wordmark where the name must be read (website header, docs, social cards), the Mark where it is known or space is tight (app icon, favicon, TUI frame, avatar).
- Clear space: one eye's diameter around the Mark; one coin's diameter around the Wordmark.
- Minimum sizes: Mark 16px tall, Wordmark 16px type size (about 80px wide).
- Colour versions on `surface-0` and `surface-1`; reversed versions on `surface-inverse` and `term-ground`; mono where one ink is all there is, or where another `coin` is already in view.
- Don't recolour parts, stretch, rotate, outline, add effects, or set the name in another face.

## Iconography

ownpurse has no general icon set yet. The Mark and its eye states (rest, thinking, minus, plus) and the Coin glyphs (filled, ring, dashed ring, minus, plus) are the whole vocabulary. In the TUI, use box-drawing characters and ● ○ − + for the same meanings. When an interface needs more icons, draw them on the Mark's 96 grid with 10-unit strokes and square ends, in `ink-1`; never emoji.

## Components

Mark, Wordmark, Coin, Amount, Button, Tag, TAccount, LedgerTable, Note, Sheet and Terminal, in `window.Ownpurse`. Each has a README with when to use it and what to provide.
