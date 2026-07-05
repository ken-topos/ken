---
scope: fleet
audience: (see scope README)
source: private memory `grep-the-producer-not-the-cited-proxy`
---

# Verify against the real producer, not a cited proxy

**Finding (sct-completeness (a) QA gate, 2026-07-03).** A kickoff/report framed
a new helper (`sct.rs::is_recursive_field`) as "mirrors
`ken-interp::eval::is_recursive_arg`, which independently re-derives the same
info for the identical reason." True in spirit (same re-derivation *technique*,
working around the same unpopulated `ConstructorDecl.recursive_positions` field)
but not an exact mirror: the new helper has an extra `Term::Pi(_, cod) => ...`
arm (Π-domain peeling, for W-style recursive fields) that the cited comparandum
lacks.

**Why this almost slipped through.** The cited helper (`eval::is_recursive_arg`)
is real, precedented, and directionally correct — reading it and confirming
"yes, same idea" is a natural stopping point, especially when the confinement
check (does the new code touch only its own file?) already passes. The trap: the
correctness property that actually matters isn't "does this match the cited
proxy" — it's "does this match what the actual downstream consumer needs," which
requires tracing to the REAL producer. Here: the match-compiler (`elab.rs:1436`)
builds its IH-count (`p_ihs`) by calling `ken_kernel::inductive::recursive_args`
(`inductive.rs:174`), NOT `eval::is_recursive_arg` — and `recursive_args` DOES
peel Π domains. So the new helper's extra arm wasn't superfluous generality, it
was NECESSARY to match the true producer; the cited "mirror" was merely a
same-shaped, coincidentally-often- agreeing but not-identical proxy. Missing
this would have meant (a) approving a doc-comment overclaim as fact, and (b)
missing the real finding: `eval.rs`'s own simpler version under-counts IHs for
W-style constructors — a genuine pre-existing, latent, untrusted-layer
(ken-interp, tested-not-trusted, not a soundness hole) bug that only became
visible by tracing to the true producer.

**Rule:** when a report says new code X "mirrors precedented helper Y," don't
stop at re-reading Y and nodding. Trace to whoever ACTUALLY CONSUMES the value X
computes (the real call site downstream), find what THAT call site's
ground-truth computation is, and diff X against that — not against Y. Y may be a
fair precedent for the general technique while still being the wrong equivalence
class for the specific property being checked.

**Complements named floor must be grepped not assumed** (there: a named floor
didn't exist at all; here: the named comparandum existed and was even
directionally right, just not the operative one — a subtler miss).

**Paired discipline that also paid off here:** don't trust "verified against the
actual dumped compiled term" in a report — reproduce the dump yourself in a
scratch worktree, elaborate real surface syntax, and hand-check the de Bruijn
indices. That's what surfaced enough confidence to also validate the ≥2-level
hand-built kernel-level tests as a faithful extrapolation (not a strawman)
rather than just asserting it.
