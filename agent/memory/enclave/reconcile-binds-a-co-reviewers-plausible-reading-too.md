---
scope: enclave
audience: (see scope README)
source: private memory `reconcile-binds-a-co-reviewers-plausible-reading-too`
---

# The reconcile-re-derive duty binds a co-reviewer's plausible reading too

Un-oracling a deferred `(oracle)` case at the merge gate: the trigger is often a
co-reviewer (the leader assembling the Decision) saying "the landed §X appears
to resolve your open item as Y — un-oracle if it holds." **Re-derive Y against
the landed §-body yourself; do not un-oracle to their framing.** A plausible
reading from a trusted teammate can be the **inverse** of what the body actually
says.

**Why:** SURF-1 `seed-purity-keywords.md` PK2b. I flagged an over-declared
`proc` (declares `visits [Console]` headroom, body infers `∅`) as `(oracle)` —
legit `proc` or should-be-`fn`? spec-leader's assembly-thread reading:
"`36 §1.6.3(a)`'s hard-error mismatch covers it, same as the clean case."
Plausible — but re-deriving against the landed **`§1.6.2`** gave the
**opposite**: classification keys on the **declared** row `ρ_decl` (not the
inferred `ρ_inf`), and `§1.6.2` carves it out verbatim — a `proc` with a
non-empty declared row and a now-pure body is **honest** (the `§1.4`
over-declaration/headroom rule), NOT a mismatch. Only the *empty*-declared case
is the hard error. Had I un-oracled to the hypothesized "mismatch," the seed
would have shipped a conformance case **contradicting its own spec** — a build
following it would reject a valid over-declared `proc`. Caught pre-merge because
the un-oracle was gated on my own content-reconcile, not the co-reviewer's word.

**How to apply:** (1) A leader/co-reviewer's "§X resolves your oracle as Y" is a
**pointer to re-derive**, never a verdict to transcribe — read the §-body and
work the classification yourself (differential verify which mechanism is the net
sibling: don't over-credit a plausible attribution). (2) The failure mode is
sharp for **classification/axis** ambiguities (declared-vs-inferred,
source-vs-target, direction, level): a plausible reading can pick the wrong axis
and read as "settled" while inverting the verdict. Name the axis and check which
one the body keys on (here: `ρ_decl`, so over-declaration is an escape valve
that keeps the keyword honest). (3) This is the co-reviewer analog of disclaimed
framing still binds your own companion artifact and the CAT-1 tt-vs-Refl catch:
the reconcile-re-derive duty binds **any** framing you'd un-oracle to — your own
draft OR a teammate's hypothesis. Report the corrected reading explicitly so the
record carries the body's answer, not the hypothesis.
