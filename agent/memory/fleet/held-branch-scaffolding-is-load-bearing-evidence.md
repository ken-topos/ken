---
scope: fleet
audience: (see scope README)
source: private memory `held-branch-scaffolding-is-load-bearing-evidence`
---

# A held branch's scaffolding is load-bearing evidence

On `wp/elab-match-expected-whnf` (2026-07-03, Map capstone "Wall 2"),
Foundation's bug report described a "2-line self-recursive `Prop` predicate over
`List Bool`" repro, with the actual scratch-probe files living only on the held,
un-checked-out `wp/map-verified-laws @ 4d4aaad` branch (which also carries
`andIntro`/`andFst`/`andSnd`/`Or`/`Bottom` prelude helpers not present on clean
`origin/main@f11c61d`, the WP's own base). I hand- reconstructed the repro from
the commit-message prose alone, on clean `f11c61d`, necessarily using different
(simpler) primitives -- and every variant I built elaborated the same with or
without the candidate fix (green-vs-green, see green vs green does not confirm a
fix).

foundation-leader's own grounding (grepping `prelude.rs` on `f11c61d`, finding
the cited helpers absent) surfaced this as the likely reason for my
non-reproduction. Architect and language-qa then independently re-ran
Foundation's *actual, unmodified* saved probe files against the real held-
branch base and got a clean isolation result (the probe fails identically
pre/post-fix, unrelated to my hand-built variants).

**How to apply:** when a WP's grounding cites a repro living on a DIFFERENT
branch than the one I'm building on -- especially a held/parked branch carrying
additive helpers or packages -- do not reconstruct it from prose. Either (a) ask
the reporting team for the exact file, or (b) check out that branch directly
(read-only, for grounding) to run the real repro before concluding anything
about whether a fix helps or doesn't. A hand reconstruction that omits
load-bearing scaffolding can produce a false green-vs-green in EITHER direction
-- it is not a safe substitute for the real input, even when it's built
faithfully to the report's stated shape. Sibling of green vs green does not
confirm a fix and named floor must be grepped not assumed (same family: don't
trust a prose description of code/scaffolding you haven't actually looked at).
