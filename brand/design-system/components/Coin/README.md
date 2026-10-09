# Coin

A status written as a coin glyph and a word: posted, balanced, open, held, short or over. The coin is the system's one status shape. Its glyphs come from the Mark's eyes: the filled coin, the open ring, the wink (minus) and the plus.

- Posted and Balanced: filled, `verdigris`. Open: ring, `ink-3`. Held (not yet posted): dashed ring, `gilt`, word in `gilt-text`. Short and Over: minus and plus, `coin-text`.
- The word always shows. `iconOnly` is for dense tables where the column header names the status; the word stays for screen readers.
- Short and Over carry the amount in the label when there is one: `label="Short 86.40"`.
- Don't invent new colours or shapes for states; add a word instead.
