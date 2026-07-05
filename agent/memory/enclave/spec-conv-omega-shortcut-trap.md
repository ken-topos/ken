---
scope: enclave
audience: (see scope README)
source: private memory `spec-conv-omega-shortcut-trap`
---

# A proof-irrelevance shortcut over the universe itself, not just its
elements, is unsound

When authoring a **type-directed conversion** algorithm with an Ω (SProp)
proof-irrelevance shortcut, the guard must distinguish **`A : Ω_l`** (A is a
*proposition* — its proofs are definitionally equal, shortcut → `true`) from
**`A = Ω_l`** (A *is the universe* — `a, b` are propositions compared as
*elements*, which must fall through to structural comparison). In K2c
`17-conversion.md §3.3` I wrote `if A is Ω_l OR typeOf(A) is Ω_l: return true`;
the `A is Ω_l` disjunct made `conv(Ω_l, Top, Bottom) → true`, so `Top ≡ Bottom`
and a closed inhabitant of `Empty` follows. Architect caught it as the merge
blocker (dec_4727ygqgpm1y5). Fix: fire **only** on `typeOf(A) is Ω_l`.

A sibling trap in the same review: an SCT `↓=` ("not larger") class that admits
**constructor-wrapping** (`x ↦ c x`, which *grows*) is unsound, because
`compose(↓=, ↓) = ↓` turns one mis-record into a spurious decreasing thread and
admits a non-terminating definition. `↓=` must be identity or a non-growing
projection/permutation only; everything else (any ctor application, app, prim,
cast) is `?`.

A **third** over-recording instance, same chapter (`17 §4.3`, kernel-impl/QA
flagged): I collapsed the SCT compose table to `compose(↓, e) = ↓` ("a strict
decrease dominates"). Wrong for `e = ?`. It conflated **two operations with
opposite special elements**: `compose` *chains* a relation **along one thread**
`i→j→k` (relation composition over the subterm order, `↓`=`>`,`↓=`=`≥`,`?`=no
relation), so `?` is **absorbing** — `compose(↓, ?) = ?` (the thread breaks);
`max` picks the best thread **across** intermediates, and *there* `↓`
**dominates**. Collapsing them recorded a decrease through an unknown step →
flipped `sct-reject-ctor-wrap-compose` to accept. Lesson: in any path/relation
algebra, *compose-along* and *max-across* are different; an unknown/absorbing
element breaks a chain but is dominated in a max — never write one rule for
both.

A **fourth** instance, K2c series-1 *implementation* (Architect-caught at merge
review, fixed `9e36918`): the SCT closure aggregated self-loops by **`union`**
(element-wise max per `(caller, callee)` pair) instead of enumerating the
closure **set**. `↓ ∪ ↓= = ↓` **merged two distinct idempotent self-loops into
one**, hiding the non-decreasing `↓=` loop and admitting a non-terminating
definition (`f x = elim x base (λn ih. g (f n) (f x))`). Fix:
`composition_closure_self_loops` keeps every `(caller, callee, matrix)` triple
**distinct**, closes under `∘` to a fixpoint, and checks each idempotent loop
**separately** — no union before the test (the correct Ben-Amram/Lee–Jones
algorithm). Same class: **merging distinct entities before testing a property
the merge can mask.** The test corpus missed it because it had mutual `p→q→p`
but **no two distinct edges on the same pair** (two self-loops) — the coverage
gap to pin for any merge/closure algorithm.

**Why:** at the trust root, the unsound direction is *over*-equating /
*over*-recording-decrease / *over*-merging. Both traps are "the shortcut is
sound for the case I was picturing (proofs of one prop / a shrinking arg) but
the guard also fires on the adjacent case (props as universe elements / a
growing arg)."

**How to apply:** for every conversion/termination shortcut, write the *exact*
predicate, then ask "what else satisfies this guard?" and check the adjacent
case can't reach `true`/`↓`/`↓=`. Add a conformance case for the adjacent case
proving it is **not** equated/accepted (`Top ≢ Bottom` at `Ω_l`; a ctor-wrap
recursion **rejected**) — the property test, not the obvious case. Extends trust
root test coverage discipline.
