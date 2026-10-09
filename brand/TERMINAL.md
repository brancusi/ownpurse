# Terminal and TUI

The terminal is ownpurse's home. These rules map the system onto a character grid.

## Colour

The TUI theme is always dark and uses the `term-*` tokens. Truecolor terminals get the exact values; 16-colour terminals fall back as below.

| Meaning | Token | Hex | 16-colour fallback |
|---|---|---|---|
| Ground | `term-ground` | #1B1B19 | default background |
| Text | `term-ink` | #E9E6DD | default foreground |
| Secondary, hints | `term-dim` | #9A978D | bright black |
| Negative, difference, the coin | `term-coin` | #EE5A3F | red |
| Agreed, balanced | `term-ok` | #6FC0AA | cyan (not green, so it stays apart from red for red-green colour blindness) |
| Held, warnings | `term-gilt` | #E0B04F | yellow |
| Selection, links, notes | `term-note` | #8DB6EC | blue |
| Frame lines | `term-line` | #3A3934 | bright black |

With `--theme no-color`, meaning is carried by the glyphs and words alone, which is why every status has both.

## Glyphs

| Meaning | Glyph | ASCII (`--glyphs ascii`) |
|---|---|---|
| Posted, balanced | ● | `*` |
| Open | ○ | `o` |
| Held | ◌ | `~` |
| Short | − | `-` |
| Over | + | `+` |
| The Mark in a frame header | ┳ followed by ● ○ | `T *o` |

## Figures

- Amounts right-aligned, two decimals, thousands separators, negatives in parentheses in `term-coin`.
- A total sits under a line of `─`, with `═` beneath it (the double rule).
- Account codes lead their names: `6420 Repairs`.

## The frame

- The header reads `┳ ●○ ownpurse 0.17 · roast, hold · 2026-09-30`: mark, name, version, scope, as-of date.
- The open eye in the header blinks (○ to − and back, every six seconds) while a sync or an agent is working; it is still otherwise.
- The footer lists the keys for the current screen in `term-dim`, the next useful one in `term-note`.

## Voice in the terminal

- Every list states its count first: `142 accounts`.
- End output with the next step as a hint: `hint: ownpurse post --held to review them`.
- Errors: what failed, why, and the command that fixes it. Structured output (`--json`) carries the same words.
