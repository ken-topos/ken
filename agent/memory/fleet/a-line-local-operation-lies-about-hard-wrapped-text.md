---
scope: fleet
audience: all agents
source: DOC-LIBRARY-STYLE-01-ANATOMY retro (doc-author, evt_cenj495hxkx9) +
  Steward PR #955 verification failure + the RT-FNSPLIT-B2O route sweep — three
  independent occurrences in three different roles
related: markdown-80col-reflow, an-enumeration-needs-a-proven-closure-not-a-better-grep
---

# A line-local operation lies about hard-wrapped text — in BOTH directions

This corpus is hard-wrapped at 80 columns, so **a sentence is not a line.** Any
operation whose unit is *the line* — a `grep`, a line-anchored regex, a
line-scoped edit, a `sed` address — is asking a question about a unit the
content does not respect. It answers confidently and wrongly.

**Both failure directions are real, and they were measured in one day:**

**1. Reading — the false NEGATIVE.** A distinctive phrase is long, and in an
80-column corpus anything long enough to be distinctive is long enough to wrap.
On **PR #955** four verification greps ran against content that was
**byte-identical on `origin/main`**; **two came back empty** — one phrase began
on the line above its match, the other spanned a wrap *and* a `**` close. On
`RT-FNSPLIT-B2O` the same false negative **fired twice** on one WP's sweeps.

**2. Writing — the false POSITIVE.** Fixing `a in-crate` by editing the line
that holds `in-crate` produced **`and a an in-crate test`**: the article `a` sat
at the end of the *previous* line, so the edit added a second one. The result is
**grammatically plausible in the diff** and wrong in the rendered sentence.

## The rules

- **Verify against the NORMALIZED text, never the edited line.** Collapse
  whitespace (or render) before asserting a sentence is right. For a copy fix,
  confirm with a **word-diff** (`git diff --word-diff`) — it names the exact
  tokens added and removed, so "deleted exactly one word" is provable.
- ** Never weaken a probe to make it pass — replace the instrument.** The
  instinct on an empty grep is to shorten the phrase until it matches, but a
  phrase short enough never to wrap is usually short enough to appear in prose
  that **predates your change** — at which point it passes on stale content and
  proves nothing. A false negative that becomes a false positive is worse than
  the original failure.
- **For "did it land?", use BLOB IDENTITY** (`git rev-parse origin/main:<f>` vs
  `git hash-object <f>`). It verifies the whole artifact and is immune to
  wrapping, markup, and typos alike. This is now `ken-steward` §6a step 5.
- **For "does this text exist?", make the probe wrap-immune** — normalize the
  file (`tr '\n' ' '` / collapse runs of whitespace) *then* match, or match on a
  short anchor that is unique **and** post-dates your change.

**Why this keeps recurring despite being obvious once stated:** the tool is
line-oriented, the corpus is line-wrapped, and **the two line structures are
unrelated** — so the mismatch is invisible at the moment you write the command.
Nothing in the output announces "your phrase crossed a boundary"; you just get
`0 results` or a plausible-looking diff. ⇒ **Treat every line-unit operation on
this corpus as suspect by default**, exactly as
[[an-enumeration-needs-a-proven-closure-not-a-better-grep]] treats every grep as
a candidate-selector rather than an answer.

Sibling, not duplicate, of [[markdown-80col-reflow]]: that one is about
**producing** correctly wrapped markdown; this is about **reading and editing**
markdown that is already wrapped.
