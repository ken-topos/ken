---
scope: build/qa
audience: (see scope README)
source: private memory `discriminating-flip-must-be-checked-per-test`
---

# An empirical discriminating flip must be checked test-by-test

**Finding (sct-reconstruction-descent (b) QA gate, 2026-07-03).** Ran the
standard discriminating flip (revert only `sct.rs` to pre-fix, keep the new
`sct_reconstruction_descent.rs` tests, rerun). Two of the four accept-side tests
(`shape_b_ackermann_accepts*`) correctly FAILED pre-fix and PASSED post-fix —
genuine, load-bearing. The other two
(`shape_b_second_lexicographic_shape_accepts*`, the `walkDown` shape) PASSED on
*both* sides of the flip — they don't discriminate anything, despite the
implementer's doc comment claiming this second shape exercises "the general
reconstruction-`DownEq` mechanism, not an Ackermann-specific special case."

**Why this almost slipped through.** It's tempting to run the flip once, see "2
tests newly fail, good, the fix is load-bearing," and stop — the aggregate
signal (some tests flip) is exactly what a discriminating-flip check is looking
for. But a test suite commonly bundles a genuine discriminator alongside a
"second example for generality," and nothing forces the second example to
actually need the mechanism. Here: `walkDown` has exactly ONE recursive call
site, and its untouched parameter (`n`) already gets a genuine field-`Down` via
the pre-existing (a)-only mechanism — `has_strict_diagonal` only needs ONE
diagonal position `Down` in an idempotent single-edge matrix, so the OTHER
parameter's reconstruction status (`DownEq` vs. pre-fix `Unknown`) never affects
the outcome at all. Ackermann needs the fix specifically because it has THREE
call edges whose pairwise composition can zero out a diagonal a single edge
can't. This is a real structural fact about the SCT criterion (single-edge
self-loops are far weaker tests than multi-edge ones), not a test-writing
mistake exactly — but the doc-comment's generality claim was still empirically
false, and only the per-test flip check caught it.

**Rule:** after a discriminating flip, check EACH accept-side test's individual
pass/fail delta, not just "did the suite report new failures." A suite can be
net-discriminating (some tests flip) while individual tests within it are not
(they pass on both sides) — the aggregate signal hides exactly the distinction
the flip exists to establish.

**Corollary for future SCT-mechanism test design:** a self-recursive definition
with exactly one call site can satisfy `has_strict_diagonal` off a single
genuinely-decreasing parameter regardless of what any other parameter does. "Add
a second parameter preserved via the new mechanism" is NOT automatically a
harder/more-general test than the single-parameter case — only a shape with
multiple call-edges (so composition can zero out an otherwise-fine diagonal, as
in Ackermann's inner+outer+base-case calls) actually stresses the mechanism.
Check a proposed "second example shape" against the UNFIXED code first, before
trusting that it broadens coverage.

**Complements grep the producer not the cited proxy** (there: a "mirrors Y"
claim was checked against the wrong ground truth; here: a "exercises the general
mechanism" claim wasn't checked against ground truth — unpatched code — at all,
until the per-test flip was actually run).
