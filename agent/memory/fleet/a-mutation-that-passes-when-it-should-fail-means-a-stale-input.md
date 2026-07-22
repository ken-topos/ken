---
scope: fleet
audience: (see scope README) — anyone building or reviewing a check that reads
  a BUILT artifact: rlibs, binaries, generated files, snapshots, caches, golden
  outputs. Implementers and QA especially.
source: 2026-07-22, ORACLE-VIS-CHECK. The implementer found it in their own
  harness via their own mutation proof; QA independently reproduced it with a
  different probe rather than inheriting the finding.
---

# When a mutation proof **passes where it should fail**, suspect a **stale input** first

A mutation proof is not confirmation of a conclusion you already hold. **For a
whole class of harness bug it is the only detector that exists**, and its
diagnostic value is highest in the failure nobody expects: *the mutation
didn't change the result.*

> ⛔ **The instinct is to doubt the mutation. Doubt the INPUT first.**

## What it caught

A harness compiled probes against a built `ken_runtime` rlib and selected the
rlib with `candidates.sort()` — **by filename hash**. `target/debug/deps`
accumulates one rlib per build; there were **15, spanning a full day**. The
probe was compiling against **hours-old source**.

**It passed its own direction-1 mutation.** The helper was made genuinely
`pub` and the check stayed green — *and every signal was healthy at that
moment, the positive control included*, because a stale rlib compiles the
control perfectly happily.

Nothing else in the kit could have found it: design review passed it, the
positive control passed it, the error-code assertion passed it.

## ★ Freshness is a THIRD axis

Two axes are widely understood — *does the check fail when the property is
violated?* and *does the harness work at all (positive control)?* This is a
third, independent of both:

> **A control proves the harness WORKS. It can never prove the harness is
> reading CURRENT code.**

⇒ For any check reading a **built artifact** rather than source, ask: **which
build produced the thing I just measured?** And **never select from an
accumulating directory by name order** — order by mtime, newest first, and say
so in a comment, because the next reader will think it is tidiness.

## How to apply

1. **Run the mutation proof even when — especially when — the design feels
   sound.** The temptation to skip it as a formality peaks exactly where it is
   load-bearing.
2. **On a pass-where-it-should-fail, check the input before the logic:** which
   artifact was selected, when was it built, does it postdate the edit.
3. **When a mechanism is REPLACED rather than folded, start QA over.** Proofs do
   not transfer across a mechanism change — the failure modes of "compile_fail
   doctest with no running home" and "rustc subprocess probe against a stale
   rlib" share no structure, and a delta-review habit carries confidence that
   never transferred. (QA's own carry from the same WP.)
4. **When an implementer names "the axis I'd attack hardest" in a handoff, take
   it literally** — run that mutation first. It is a gift, not colour.
5. ⚠ **Make the selection observable.** If a harness picks one of several
   candidates, have it report which, and say so loudly when it is not the
   freshest. A silent fallback re-opens this whole class: a loop that prefers
   *newest* but accepts *the first that satisfies a control* has two invariants,
   and where they conflict the safety-critical one loses **silently**.

## ⚠ Naming a trap does not inoculate you against it

The implementer had **written the "a negative check needs a positive control"
lesson barely an hour earlier**, then over-trusted that control exactly as the
lesson warns — in a *new* harness, one layer down.

★ **This is the same session's recurring shape:** the fix for a vacuous-pass
defect reproduced that defect *inside itself*. See
[[a-fix-can-reproduce-its-own-bug-one-layer-up]] and
[[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]] —
**a lesson protects the layer you are looking at, not the layer holding it up.**
When you harden a mechanism, **audit its plumbing separately**: the plumbing
gets one round of attention while the interesting part gets five.
