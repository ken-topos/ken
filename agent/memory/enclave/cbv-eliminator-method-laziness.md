---
scope: enclave
audience: (see scope README)
source: private memory `cbv-eliminator-method-laziness`
---

# CBV eliminator methods must be held unevaluated

When elaborating an **operational semantics over a strict (CBV) core**, do not
let "call-by-value" imply *strict everywhere* — **name the non-strict positions
explicitly** and derive the observable laziness from them. The load-bearing case
(X1, `42-evaluation.md`): an eliminator `elim_D M m̄ ī s` must force the
**scrutinee** `s` (strict) but hold its **methods `m̄` unevaluated**, evaluating
**only** the method `mₖ` selected by `s`'s head constructor (ι fires one method,
`14 §3`). The natural-but-wrong default — evaluate `elim_D`'s method arguments
eagerly like any application — **silently breaks** three things at once:
**branch laziness** (`if`/`match` evaluate only the taken arm),
**short-circuit** (`&&`/`||`, which desugar to `if`), and **`unknown`
absorption** (`unknown ∧ false = false`, `∨ true = true` — the absorbing
connective is the untaken-eliminator-arm rule in disguise). A strict-`elim`
implementation passes happy-path tests yet forces an untaken arm's
`unknown`/divergence into a result selected away from it.

**Why:** in a strict core, *which positions are non-strict* is exactly the part
a build team (or a weaker model) gets wrong by omission — "CBV = strict" is the
seductive default, and the one carve-out (eliminator methods) is where all the
user-visible laziness lives. Stating "CBV with sharing" without pinning the
carve-out leaves the single subtle seam unspecified. Sibling of playbooks state
mechanism not intent (write the exact mechanism weak agents reconstruct wrong)
applied to evaluation semantics.

**How to apply:** in any evaluator/operational-semantics spec, (1) enumerate the
**non-strict positions** (eliminator methods; `Lazy`/thunk forcing; a
`perform`/effect node if effects are in scope) and make every other position
strict-by-default; (2) **derive** the observable laziness properties (branch
selection, short-circuit, `unknown`/⊥ absorption) from the one carve-out rather
than restating them ad hoc; (3) require a **structural** conformance assertion
that an untaken arm is **never forced** (e.g. an untaken branch that would yield
`unknown` or diverge does not contaminate the result) — a value-only check
passes vacuously when the implementation is wrongly strict. Extends trust root
test coverage discipline (assert the property, not the representative case) into
runtime semantics.
