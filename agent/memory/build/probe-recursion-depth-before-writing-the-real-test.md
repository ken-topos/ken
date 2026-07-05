---
scope: build
audience: (see scope README)
source: private memory `probe-recursion-depth-before-writing-the-real-test`
---

# Probe recursion depth before writing the real test

Ken `Nat` literals are unary (`Zero`/`Suc` chains, no numeral sugar). A
conformance seed can pin a discriminating value at a depth (e.g.
`slice 0 99 "abc"`) that nobody has evaluated before -- and `ken-interp`'s
current reduction strategy has a real, non-obvious cost cliff there.

**Why:** on `wp/L3-strings-surface` (2026-07-03), evaluating `take`/`drop`-
style structural recursion over a depth-99 `Suc`-chain (1) stack-overflows under
a default 8MB test-thread stack around depth ~85-90 (pure recursion depth, not
an algorithmic issue -- resolved by spawning the test on a thread with
`stack_size(256*1024*1024)`), and (2) separately costs ~3 CPU-minutes at depth
99 vs sub-millisecond at depth <=40 -- empirically ~O(n^3.5-4) in the recursion
depth, not exponential (confirmed via isolated timing at depths
10/20/30/40/50/70: each roughly (depth-ratio)^~4 slower). This is a genuine
`ken-interp` reduction-strategy characteristic (no prior test exercised Nat
depths anywhere near this range -- existing precedents topped out around
`fib ten`), NOT a bug in the derived defs being tested (the value produced was
correct), and NOT a soundness issue (tested-not-trusted ring: "a wrong value,
never a false proof" -- here the value was simply slow, not wrong).

**How to apply:** before writing the "real" acceptance test for any conformance
case that evaluates (not just elaborates) a Nat/List value at a depth that's new
territory (no landed precedent test goes that deep), spend a few disposable
scratch-test iterations (create/`rm -f`, never committed) bisecting the depth to
characterize (a) whether a bigger test-thread stack is needed, and (b) the
actual wall-clock cost at the target depth, BEFORE wiring it into the real test
suite. This turns a would-be mysterious CI hang or stack-overflow crash into an
understood, bounded cost. If the conformance seed's exact pinned value (not a
smaller substitute) is expensive but not pathological (bounded polynomial, not
exponential/non-terminating), ship the exact pinned value with the stack-size
fix and document the cost as a non-blocking forward-tracked finding (I filed it
in the package's `MANIFEST.md`) -- do not silently substitute a cheaper value
than what the already-gate-approved seed specifies just to dodge the cost.

Sibling of named floor must be grepped not assumed (verify empirically before
committing to an approach) and tested not trusted posture needs reachability
precondition (the tested-not-trusted ring's failure mode is a wrong/slow value,
never a false proof -- so a perf finding here is a forward-tracked item, not a
soundness escalation).
