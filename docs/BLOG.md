# Writing for the dev blog

The blog at ownpurse.com/blog is where working sessions become something others can read, run and argue with. A
post captures the essence of a design discussion: what we asked, what we tried, what held, what broke and what we
decided. The docs hold what stays true; the blog holds how we got there, pinned to the version it describes.

## What a post is

- **A position, distilled from the sessions (since 2026-10-29).** A post is not a transcript of a design session.
  It argues for the direction we are taking and why it is insightful: the problem as a reader feels it, the idea
  that answers it, the evidence (numbers, what broke), and where it goes next. Excite the reader by explaining, not
  by selling: intelligent, calm, never aggressive, never "obviously". Quote Aram sparingly, only where his words
  carry the idea better than a summary; never invent a quote. Every post opens with the "From our working sessions"
  note (`.fm-convo-note`) naming the sessions it was distilled from and linking the post before it.
  Byline: `author: Aram Zadikian (@brancusi) and Tally`.
- **Something to run.** A post about a change opens with "Run this version": one command that fetches ownpurse at
  a pinned commit and plays the demo (`samples/demo.sh`). The commit is also tagged `demo/<date>-<slug>`, so the
  posts form a time machine: run them in order and you watch the model grow.
- **Something to touch, when it helps.** Interactive figures (the fact-model canvas, the receipt flow, the SQL
  console, the agent traces) query the real synthetic log in the reader's browser. Never a mock-up: if a figure
  shows a number, a query returned it.
- **What we learned.** End with what the session sharpened: the gaps found, the decisions made, what goes on the
  next phase's list. Findings that change plans also go into `docs/plans/`.

## What goes where

| It is… | It goes in |
|---|---|
| How the system works now, a command, a flag, a schema | The docs (`site/src/content/docs/docs/`), in the same change |
| A plan, a finding, an open decision | `docs/plans/` |
| The story of a session: the question, the attempt, the result | The blog (`site/src/content/blog/`) |

When a post explains something a user or contributor will need later, put the lasting version in the docs and link
it from the post. Docs follow the code as it changes; a post stays at its commit.

## Steps

1. **Capture as you go.** During the session, keep Aram's own words for the questions and decisions worth quoting.
   Note the numbers the work produced, with the query or command that produced them.
2. **Build the thing to run.** Make the demo or figure work against the synthetic samples first. Every name,
   amount and id must be invented (`samples/README.md`).
3. **Pin it.** Commit the code, then tag that commit: `git tag demo/YYYY-MM-DD-<slug> <sha>` and push the tag. If the
   post has interactive figures over the fact log, export the log from a clean checkout:
   `bash site/scripts/month-log.sh`. It writes `site/public/data/month-<commit>.json` (pinned, for the post) and
   `month.json` (latest, for the docs). Keep old pinned exports; deleting one breaks its post.
4. **Write the post** as `site/src/content/blog/<slug>.md`, or `.mdx` when it embeds figures. Front matter:

   ```yaml
   ---
   title: The fact model, up close        # sentence case, no colon-subtitles
   description: One or two plain sentences of what the reader gets.
   date: 2026-10-10T12:00:00Z
   author: Aram Zadikian (@brancusi) and Tally
   ---
   ```

   Link GitHub files at the pinned commit (or the branch they live on), never at a branch the file is not on.
5. **Embed figures** from `site/src/components/facts/` with `client:only="react"`, a `src` pointing at the pinned
   export, and a `slot="fallback"` paragraph for readers without JavaScript. Wrap wide figures in
   `<div class="fm-wide" id="…">`. New figures use the brand tokens only, so they work in the docs' Night theme;
   follow `brand/README.md` (square corners, rules not shadows, the coin spent once, gilt for held, lapis for
   selection and a query's path, the blink as the only motion).
6. **Check the copy** against `brand/README.md` and AGENTS.md: ownpurse lowercase, second person, sentence case,
   numbers to the cent, no emoji, no hype. Keep the "not affiliated" fine print for any provider named.
7. **Check the privacy rules.** Read the diff for credentials, real records, real ids, personal names, emails, local
   paths and real amounts (`SECURITY.md`). The exported log is synthetic by construction; check it anyway with
   `grep -iE 'users/|home/|@' site/public/data/month.json` (only `.example` addresses should appear).
8. **Build and look.** `cd site && npm run build && npx astro preview`. Open the post at desktop width and at 390px
   (no horizontal scroll), click through every figure, and open the docs pages it touches in both themes.
9. **Publish.** Commit the post (a second commit, after the pinned one), push the branch and the tag, then
   `cd site && npm run deploy`. Open the live page and run the post's command from an empty folder.

## Corrections

A post stays at its commit, but it should not mislead. When a reader's question shows a post gives the wrong
picture, add the answer to the post as a section in the same conversation voice, say so in the conversation note
("Updated after publishing with …"), and put the lasting version in the docs. Figures may be improved in place; the
data they read stays pinned.

## Checklist

- [ ] Opens with the conversation note and, for a change, the pinned command and tag
- [ ] It argues a position, not a session's play-by-play; any quote of Aram's is real
- [ ] Every number came from a query or command; figures read the pinned export
- [ ] Lasting material is in the docs too, linked from the post
- [ ] Copy follows the brand; fine print names any provider mentioned
- [ ] Diff read for private data
- [ ] Built, checked at desktop and phone width, docs checked in both themes
- [ ] Pinned commit tagged and pushed; post committed, pushed and deployed; live page checked
