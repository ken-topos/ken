---
scope: fleet
audience: all agents
source: merges former private memories `markdown-80col-reflow-gotchas` and
  `safe-reflow-space-substitution-only` (overlapping write-ups of the same
  recurring reflow gotchas from different sessions)
related: correcting-scope-must-sweep-whole-doc
---

# 80-column markdown reflow: the recurring gotchas

CLAUDE.md mandates wrapping markdown at 80 columns (Mermaid/code fences exempt).
Every spec-authoring or doc-editing WP eventually needs to reflow a file to fit,
and a naive script or by-hand pass keeps re-hitting the same traps. A working
reflow script lives at `scratchpad/reflow_md.py` in-session; these are the
durable, portable lessons.

**The only safe algorithm is space-substitution.** Join the block, then convert
*existing* inter-word spaces to newlines at wrap points — **never re-tokenize
and rejoin**. Two corruption modes both pass a naive eyeball (hit twice: F1 +
Decimal/Char DEMOTE):

1. **Tokenize-and-rejoin injects spaces around every backtick span.**
   Splitting on backticks then rejoining with single spaces turns a code
   span immediately followed by punctuation (e.g. `` `README.md`. ``) into
   `` `README.md` . `` — silent mid-token corruption.
2. **A fence-unaware reflow mangles triple-backtick-fenced code blocks.**
   Tracking only "line starts with the fence marker" (not fence *state*)
   treats lines *inside* the fence as reflowable prose and destroys layout
   diagrams / code. This passes a whitespace-collapse invariant check yet
   ruins the meaningful line structure — **skip files with fences** (reflow
   by hand or make the reflow fence-state-aware).

**Measure DISPLAY width, not bytes.** `awk 'length>80'` counts *bytes*, so
em-dashes (—, 3 bytes) and math symbols (⊑ ℒ ⊥ ⊤ Σ ≤ ⇒) over-report — a line
flagged "83" is often 80 on screen. Use `wc -L` (or Python codepoint count via
`east_asian_width`; these symbols are width-1) for the real column count. Filter
`grep -nP '^.{81,}$'` then re-measure each with `wc -L`.

**Backtick-parity-safe break points.** Break only at spaces where
backtick-parity is even — never inside a code span (its internal spaces
aren't valid break points, and breaking there leaks a continuation indent
into rendered code). Correct tokenizer: split on whitespace, then merge
tokens while backtick-parity is odd, so a multi-word code span stays one
token. Any code span too wide to fit an indented line must be shortened by
hand first (or moved to a fenced block) — the reflow cannot break it.

**Handle lists nested in blockquotes (`> - item`).** A contract blockquote with
bullets is a standard shape in spec chapters. A reflow that classifies `> - ...`
as plain blockquote flattens the bullets into one paragraph (the `-` becomes an
inline dash). Detect `^((?:> )+)\s*[-*]\s+` as its own kind; each `> - ` bullet
is a new paragraph, its `>   text` continuations attach to it.

**NEVER overwrite in place with `open(f,'w')`.**
`open(f,'w').write(open(f) .read())` **truncates `f` to empty before the inner
read runs** — this destroyed a finished chapter mid-turn. Reflow to a
**scratch** file, verify, then `cp` over the target. Recovery if it happens
anyway: the destroyed file's content is usually still in the turn's earlier
`Read` output — re-`Write` it (Read the destroyed version first to clear the
modified-since-read guard), then re-run the fixed reflow.

**Content-integrity gate is mandatory, and it has a blind spot.**
`re.sub(r'\s+',' ', file).strip()` must be byte-identical before and after —
proves only whitespace moved. It does **not** catch fenced-block mangling (skip
those files) and it can **falsely flag "differs"** on a reflowed blockquote,
because re-wrapping changes the *count* of `> ` prefixes, which a naive
whitespace-collapse doesn't strip — that's expected, not corruption. The real
gate: strip line-prefixes (`(?:> )+`, list markers, indent) from both files
first, then assert the token streams are identical.

**Verify header↔coverage-map id sync after any rename.** A grep artifact: a
lowercase-only regex silently truncates a capitalized id like `...-false-True`
(corpus convention is all-lowercase kebab ids) — after reflowing or renaming,
diff case-header ids against coverage-map ids (`grep case-headers` vs
`grep coverage-map-ids | sort | diff`) the same way you'd sweep a corrected
claim through a whole document.
