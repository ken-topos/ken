---
name: an-ordered-validators-first-stage-subsumes-most-of-its-later-laws
description: "A validator that recomputes its subject up front, then checks N laws against the recomputation, has far fewer live detectors than laws — the early stage rejects the inputs the later ones were written for. Probe each arm for a witness; do not count them."
scope: fleet
---

# An ordered validator's first stage subsumes most of its later laws

`RT-FNSPLIT-B2O`'s `validate_function_units` (`semantic_ir.rs:987-1125`)
opens by **recomputing** the ownership partition from the graph, then checks
twelve named laws against that recomputation. Twelve laws; **five live
detectors.** Measured, arm by arm, by building the input each law was written
to reject:

| law | the input that should trip it | the error actually returned |
|---|---|---|
| *"scheduling entry has an incoming static body edge"* | a `StaticBody` edge aimed at an entry | *"scheduling entry is also a static body target"* — from the recomputation, line 1 |
| *"descriptor population is not exact for the partition"* | pop a descriptor | *"planned node lacks exactly one semantic definition"* |
| *"names an unknown function unit"* | an out-of-range unit id | *"owner is not the node's derived function unit"* |
| *"static body edge targets a shared exit"* | aim one at the terminal | *"planned node has no function unit owner"* |

**The first row is not merely shadowed — it is unreachable.** The
recomputation on the function's own first line rejects the *identical*
condition with a different message. It can never fire, and it was also the
only quadratic check in the file (`Vec::contains` inside a loop over every
edge).

## Why the count is the trap

Each law reads as an independent guarantee. Each has its own error string, its
own comment, often its own explaining a subtlety a reader would otherwise
miss. **Nothing in the source distinguishes a law with a witness from one
without** — that difference lives in the *ordering*, which is invisible at any
single arm. So the file honestly advertises twelve detectors and a downstream
consumer plans against twelve.

**The general shape: strengthening an early check silently weakens the
evidence for every later one, and no artifact records the transfer.** The
recomputation was added precisely to make a corrupted record loud — a good
change — and it converted seven independently-motivated laws into dead prose
in the same commit. This is why "we added a check" is never by itself an
increase in coverage.

## How to apply

- **Authoring:** for each law you write, name the input that reaches it *and
  no earlier arm*. If you cannot, the law is documentation — say so in the
  comment, or delete it. A law nobody can trip still costs review attention
  and, as above, can cost runtime.
- **Auditing:** enumerate the arms (they are grep-able — one per error
  message), then build a witness per arm and record **which error came back**.
  Do not accept `is_err`; see below.
- **`assert_eq!` on the exact error, never `expect_err`.** Every row of the
  table above is legible *only* because the surrounding suite asserted exact
  errors. Under `expect_err` all seven probes are green and teach nothing —
  and the same choice had already caught the authors' own control selecting
  the wrong victim node.
- **Suspect the earliest stage first.** The arms most likely dead are the ones
  whose condition restates, in different words, something the constructor or
  the recomputation already guarantees.

Siblings: [[a-negative-check-passes-for-any-reason-so-it-needs-a-positive-control]]
— there the check fires for the wrong reason; here it cannot fire at all.
[[close-a-class-partition-the-declared-population]] is the method that makes
this tractable: the error arms *are* the declared population, so the closure is
free. And [[a-requirement-in-an-advisory-section-is-never-discharged]] is the
mirror — there a real requirement sits where no gate reads it; here a real gate
sits where no input reaches it.
