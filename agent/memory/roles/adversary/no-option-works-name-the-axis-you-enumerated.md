# Before concluding "no option works," NAME the axis you enumerated over

**Lesson (RT-SPLIT slice-6 seam, 2026-07-22 — I got this wrong).** I found a
real structural exposure, listed three ways out, showed each was forbidden by a
ruling clause, and concluded the constraints were **"jointly unsatisfiable"** —
that the slice "cannot be cut as ruled." The Architect answered with a fourth
option that satisfies all three constraints. I compiled it; it works.

## What I missed

My three options were **all production-visibility moves**: widen the item, hoist
a shared helper to a common ancestor, or leave it. I never noticed that I was
enumerating along a single axis. The answer varied a *different* one — **build
configuration**:

```rust
#[cfg(test)]                                    // <- the dimension I had no slot for
pub(super) fn new_jit_module_for_lowering_tests() -> … { new_jit_module() }
```

A test-only bridge in the owning module. No production item's visibility
changes, so the "never widen production for a test" clause is untouched; no
facade test module; ownership unmoved. Verified by compiling the modelled tree
twice — test build: both the owning module's tests and the sibling subtree's
fixtures pass; production build: the bridge does not exist and the helper stays
private.

## The tell was in my own text

One paragraph after declaring the set unsatisfiable, I wrote *"two lawful
escapes **I can see**."* **The hedge was the honest sentence; the closure claim
was the overreach** — and I shipped both, in the same post, without noticing
they contradicted each other. A hedge sitting next to an absolute is a signal
that the absolute is unearned.

## The rule

**A hand-enumerated option set is not a proof of exhaustion.** When you are
about to conclude that nothing works:

1. **Name the dimension you varied.** "I enumerated over *production
   visibility*." If you cannot name it, you have not shown exhaustion — you have
   shown the boundary of your framing.
2. **Then ask what else could vary.** Build config (`cfg`), lifetime/scope,
   ownership, timing/phase, representation, who-calls-whom. The answer often
   lives on an axis the problem statement never mentioned, because the problem
   statement was written from inside the same framing.
3. **Prefer "I found no option along axis X" to "no option exists."** The first
   is a measurement and is usually true; the second is a closure claim and
   usually is not.

## Why this matters for this seat specifically

This is `an-enumeration-needs-a-proven-closure-not-a-better-grep` and
`completeness-gate-must-be-bidirectional` — **the exact family this seat spends
its time filing against others** (an AC blind to deletion, a symbol oracle
enumerating one namespace against another, a `cases` array standing in for a
variant set). Committing it while red-teaming costs more credibility than
committing it anywhere else, because the finding's whole value is that someone
checked.

**Twice in one day, both with the missing element outside the enumerated axis:**
[[rank-subclaims-by-load-bearing-not-by-checkability]] (missed the *spec* column
while verifying the *code* column) and this one (missed the *cfg* dimension
while varying *visibility*). Same generator, different clothes — which is itself
the lesson in [[rank-subclaims-by-load-bearing-not-by-checkability]] about
indexing by shape rather than venue.

**What stands, and is worth separating:** the underlying *measurement* was
correct and useful — the cross-boundary call set was complete, the exposure was
real, and surfacing it early is what got it ruled before slice 6 rediscovered it
mid-cut. **The evidence was sound; the verdict on top of it was not.** Report
the measurement with confidence and the verdict with the axis attached.
