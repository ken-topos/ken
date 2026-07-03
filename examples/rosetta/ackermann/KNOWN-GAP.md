# KNOWN-GAP: GAP-ackermann-sct — SCT cannot infer lexicographic termination

## What's missing

Ken's structural termination checker (SCT) requires a **single**
structurally-decreasing parameter per recursive call. The Ackermann
function's standard definition:

```
A(0, n) = n + 1
A(m, 0) = A(m-1, 1)
A(m, n) = A(m-1, A(m, n-1))
```

terminates under the **lexicographic** order on `(m, n)` (the outer
recursive call decreases `m`; the inner call, `A(m, n-1)`, decreases `n`
while holding `m` fixed) — a well-known, standard termination argument, but
one the current SCT does not infer. Elaborating the natural encoding fails
with `SCT: idempotent self-loop has no strictly-decreasing parameter`.

## Impact

The function is total and well-founded (it's the textbook example of
terminating-but-not-primitive-recursive) — this is purely a limitation of
the SCT's termination-detection power, not a soundness gap or a missing
library feature. It blocks any recursive definition whose termination
argument needs lexicographic (or otherwise multi-parameter) descent instead
of a single always-decreasing argument.

## Fix needed (capability, not a Language-lane workaround)

Extend the SCT to accept lexicographic (or more generally, well-founded
multi-measure) descent — recognize that even though no single argument
strictly decreases on every call, a lexicographically-ordered tuple of
arguments does. This is a termination-checker capability, not something
expressible by restructuring the Ken source; there is no workaround that
doesn't either (a) fabricate an artificial fuel parameter (changes the
program's meaning/complexity — Ackermann's whole interest is that it's NOT
expressible with a simple decreasing fuel bound at reasonable size) or (b)
give up on real Ackermann semantics. Routed to language-leader / Steward as
its own properly-gated capability WP (VAL2 frame's "gap whose fix needs a
new capability" boundary).

## Intended program (once resolved)

See the commented-out definition in `ackermann.ken` — `A(3, 2) = 29`
(`A(3, 4) = 125` is the more common oracle but needs far more recursive
calls at runtime; `A(3, 2)` is the feasible pinned value once the SCT
gains lexicographic support).
