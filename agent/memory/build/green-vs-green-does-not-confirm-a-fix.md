---
scope: build
audience: (see scope README)
source: private memory `green-vs-green-does-not-confirm-a-fix`
---

# Green-vs-green does not confirm a fix

On `wp/elab-match-expected-whnf` (2026-07-03, Map capstone "Wall 2", DROPPED not
merged), I implemented an Architect-approved, code-site-grounded fix (whnf the
per-branch `expected_here` before `check` in `check_match_dependent`) exactly as
specified. Every hand-built repro I could construct (plain self-recursive
`List Bool` predicate, an `And`/`andIntro` conjunction, a nested match, a
two-recursive-field `Tree`, a `Type`-valued family, a universe-generic version)
elaborated identically with the fix applied AND with it reverted --
green-vs-green.

**I refused to declare AC3 (the isolation-flip requirement) closed on that
basis** and flagged it transparently instead of either rubber-stamping the green
suite or silently expanding scope to chase a discriminating case. This turned
out to be exactly right: Architect (running Foundation's *actual* saved
scratch-probe file, not a reconstruction) and language-qa (independently, via a
different diagnostic) both confirmed the fix is **provably inert** -- the real
gap was a proof-*structuring* issue (a scrutinee-dependent hypothesis never
threaded into the match's motive -- the "convoy" idiom fixes it with zero
elaborator change), not a reduction gap in `check_match_dependent` at all. The
bug *report*'s code-site read was accurate; its causal *mechanism* was wrong.
The WP was dropped, not merged -- correctly, since landing an inert fix would
have falsely claimed a capability (misdiagnosing "Wall 2 closed" when it was
never the real blocker).

**How to apply:** when a fix's own acceptance test can't be made to fail without
the fix, do not read that as "the fix must be subtly working anyway" or "my
repro must just be too weak, ship it, note a caveat" -- read it as "this fix has
not been shown to do anything," full stop, and escalate before merging. A clean,
zero-delta, vector-approved diff can still be a no-op; only a genuine
red-before/green-after flip on the SAME input establishes that a completeness
fix fixes anything. This is the literal purpose of an isolation-flip AC, and
honoring it caught a wrong causal diagnosis that a plausible-sounding code-site
read and an Architect sign-off had both missed. Sibling of held branch
scaffolding is load bearing evidence.
