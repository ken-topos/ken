---
scope: build
audience: (see scope README)
source: private memory `bundled-changes-need-per-mechanism-isolation-flip`
---

# Bundled changes need a per-mechanism isolation-flip

**Finding (`obs-eq-termination` QA gate, 2026-07-03).** The candidate bundled
two changes in one `conv.rs` diff: a congruence-first/lazy-δ fast path (the
actual termination fix) and a re-landed `(Term::Eq, Term::Eq)` congruence arm
(previously reverted for exposing the divergence). Both new test suites
(`conv_eq_congruence.rs`, `obs_eq_termination_congruence.rs`) passed cleanly
together post-fix. A standard bundle-level isolation-flip (revert `conv.rs` to
pre-fix, rerun everything) would have shown "5 tests newly pass" — true, but
silent on which change caused which pass.

**Why this matters beyond thoroughness.** The report's own causal claim was
specific: "the fast path stops `allKeys P1 l ≟ allKeys P2 l` from δ-unfolding
into the regenerating Elim" — i.e., the fast path fixes `ordBelowL`, and the
`Eq`-arm re-land is *safe only because* the fast path is present. That's a
falsifiable claim about which mechanism does what, not just "the bundle works."
A bundle-level flip can't test it: if the causal attribution were backwards (say
the `Eq` arm alone happened to fix it, and the fast path were decorative or
wrong-but-harmless), the bundle-level test would look identical — 5/5 pass
post-fix, 3/5 fail pre-fix (or however the un-isolated split falls out). Only
splitting the bundle and flipping each half independently distinguishes "both
load-bearing for their own tests, no interaction" from "the causal story is
wrong."

**What I did:** built two synthetic variants of `conv.rs` from the same post-fix
source — (a) `Eq`-arm present, fast path removed; (b) fast path present, `Eq`
arm removed — and reran both new test files against each. Result: (a) `allKeys`
test still stack-overflows (fast path is necessary, not just correlated with the
fix); (b) `allKeys` test passes clean, `Eq` tests correctly fail (no arm
present). Clean 2×2, no confounding — this time the causal claim held. It might
not have.

**Rule:** when a candidate's diff contains more than one independently-
motivated change (a perf/termination fix + a re-landed feature, two refactors
riding together, a fix + its own regression guard), don't stop at a whole-bundle
isolation-flip. Synthesize per-mechanism variants (delete just one change from
the post-fix source, keep the rest) and flip each separately against the tests
that are supposed to depend on it. This is the bundled-change generalization of
discriminating flip must be checked per test — that lesson says check
per-*test*, not aggregate; this one says when the fix itself isn't a single
mechanism, also check per-*mechanism*, not per-bundle.
