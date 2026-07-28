---
name: line-closure-check-must-compare-whole-files-not-diff-hunks
description: A removed-line closure check run over git diff hunks is meaningless for a large move — git re-pairs retained code around big deletions. Compare whole-file content multisets instead.
metadata:
  type: feedback
---

RT-SPLIT slice 4 moved ~7,400 lines out of a 21k-line file. Running the AC-3b
**removed-line closure** ("every line removed from the parent is present in a
new module or listed in the ledger") over `git diff` hunks reported **1,339
unaccounted lines** and 1,308 *insertions* into a file that only had code
deleted from it.

Both numbers were artifacts. When a large interleaved block is deleted, git's
diff algorithm **re-pairs surrounding retained code**, emitting it as
`-`/`+` pairs. The hunks describe a minimal edit script, **not** the set of
lines that actually left the file.

**The correct oracle compares whole-file content:**

```
before := content of the source file at the merge base
after  := content of the source file now  +  every new module it was split into
lost   := multiset(before) - multiset(after)
gained := multiset(after)  - multiset(before)
```

On the same commit this gave **40 lost / 142 gained**, every one attributable
to a visibility prefix, a module header, the `impl` wrapper, or rustfmt reflow
— a residue small enough to read line by line, which is the entire point of a
closure check.

**Why:** the check exists to be *readable*. A hunk-derived residue of 1,339
lines cannot be audited, so it silently converts a gate into a formality — and
the number is wrong in both directions, so it can hide a real loss as easily as
invent a fake one.

**How to apply:** for any move/split verification, derive residue from
**whole-file content on both sides**, never from `git diff` output. If a
closure residue comes out implausibly large, suspect the oracle before the
change. Note also that line-level comparison goes red on lawful `rustfmt`
reflow — pair it with a **token-level** ordered check, which reformatting
cannot move.

Related: [[differential-oracle-is-blind-to-a-shared-premise]]. ⭐ Two
neighbouring shapes: a completeness gate has to be **bidirectional** — exact
set closure in both directions, not just "everything claimed is present" —
and a rustdoc symbol dump **cannot see trait impls**, so it under-reports the
surface it looks authoritative about.
