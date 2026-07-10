# DS-8b · Pure-witness ⊆ `proc`-field widening (instance-field purity completeness)

**Owned by the Steward** (frame); **home: Ergo team** (surface-purity
discipline — ergo-implementer → ergo-leader → ergo-qa → **Architect gate**).
The small elaborator completeness fix that unblocks DS-8's `instance
Traversable List/Option` assembly. Kicked in the operator's autonomous window
(2026-07-10) under the run's boundary rules (elaborator fixes may LAND this run
if they are the right fix, through the ring + gate). **Logged for the operator's
review as `K2`** (second elaborator-behavior land of the run — but **lighter
than DS-5b**: pre-kernel-erased, zero TCB/kernel/sort delta).

## The bug (Architect-grounded on `origin/main`, `evt_6vbgk65sj4jva`)

`class Traversable`'s `proc traverse` field is *inherently* row-polymorphic
(its action has an abstract-`g`-headed codomain → fail-closed → `proc`;
declaring it `fn` is correctly rejected at the class-decl level). But every
lawful witness (`list_traverse`/`option_traverse`) is **genuinely pure** — it
only calls DS-7's `apg.ap`/`apg.pure`, which landed (correctly) as plain pure
`fn`s, so its inferred effect row is `∅`. The effects of a traversal live in
the `g b` **values**, not in `traverse`'s execution.

`check_instance_field_purity` (`crates/ken-elaborator/src/elab.rs:3182`) is an
**exact-match** gate:

```
DefKeyword::Proc if !impure => Err("class field requires `proc`
                                    but instance implementation is pure")
```

So the pure (`∅`-row) witness is **over-rejected** for the `proc` field, and
there is no honest escape: the `visits [e]` surface binder (SURF-1 D1) is
unwired (`surface_declared_row_type` hardcodes `row_var_map(&[])`,
elab.rs:2130/:2165) — **and using it would be dishonest**, forcing a false
effect annotation on a function that visits nothing. Result: `class Traversable`
has **no possible inhabitants** today. That is a **completeness bug** (a valid
program rejected, fail-closed, safe), not a soundness hole.

## The fix (the RIGHT one — not SURF-1 D1)

Relax the `Proc if !impure` arm (elab.rs:3182) to **accept a pure/`∅`
witness for a `proc` field** — covariant subsumption `∅ ⊆ open row`: a pure
implementation is a valid, *more precise* inhabitant of a "may-be-effectful"
contract. This is exactly SURF-1 §1.6's **do-not-optimize** semantics (a `proc`
may instantiate to `∅`; a pure witness **is** that instantiation).

**Why this and not the SURF-1 D1 `visits [e]` binder:** the D1 route would
force a *false* `visits [e]` annotation on a genuinely pure `list_traverse`
(dishonest, and it's a whole surface-syntax feature that was deliberately
deferred at OQ-8). The widening accepts `list_traverse`'s **true** pure
classification — honest, minimal (~1 arm vs a surface feature), and it subsumes.
SURF-1 D1 stays deferred, correctly (not needed here).

## Hard bars (safe-direction only — why it's a completeness fix, not a relaxation)

- **The dangerous direction STAYS CLOSED.** Only `∅ ⊆ proc` opens. The
  `Const | Fn if impure` arm (elab.rs:3189, `check_decl_poly` vs the empty
  row) is **untouched** — an *effectful* witness for a `fn`/`const`/pure field
  still **rejects**. Touching that arm is the STOP signal.
- **AC6 preserved — the field stays `proc`.** The class-record contract is
  unchanged; `d.traverse` still classifies **`proc`** at projection/use sites
  (conservative). The widening is only about which witnesses *inhabit* a `proc`
  field, never about weakening the field's own classification.
- **Zero TCB / kernel / sort delta.** Purity is erased before the kernel and
  does **not** touch the class-record Type/Ω sort discriminant (computed from
  field types alone). Assert it: **no `ken-kernel` diff, no `prelude.rs`
  diff**, and an executable check that the `Traversable` class-record sort is
  identical before/after. This is a pre-kernel surface-purity-discipline fix,
  nothing more.
- **AC8 discriminator (the gate net the Architect named).** A test must show
  the dangerous direction still rejects: an **actually-effectful** witness
  assigned to a `fn`/`const` (pure) field still fails, asserted as the
  **specific** error variant (not bare `is_err()`), at that field. Plus a
  positive: a pure witness on a `proc` field now **succeeds** (the
  `Traversable`-shaped case), and `d.field` still classifies `proc`.
- **Zero regression — full pre-existing suite green.** Every existing
  instance-field purity check (the FR-series / class-instance tests) must
  behave identically; the change is additive on exactly the
  `Proc`-field-with-pure-witness case. Run the whole suite, not a targeted
  subset.

## Spec fidelity

If SURF-1's write-up (`spec/30-surface/36-effects.md` /
`33-declarations.md`) does not already state that a pure witness satisfies a
`proc` class field (the `∅ ⊆ proc` do-not-optimize subsumption for instance
fields), add the one-line rule so spec and elaborator agree. CV grounds any
conformance-fixture implication. **CI-gated** if it touches `crates/`/
`conformance/` (it does — an elaborator change), not doc-only.

## Reversibility / boundary

- **Reversibility: moderate-class** (a landed elaborator completeness fix,
  pre-kernel-erased, zero TCB delta) — lighter than DS-5b's hard-class;
  revert-clean; flagged for operator review as `K2`.
- **Outer-ring purity discipline only** — no kernel, no `data.rs`, no new
  surface syntax (explicitly NOT the SURF-1 D1 binder). If the fix appears to
  need any of those, STOP and hand back (the diagnosis would be wrong — it's a
  1-arm change).

## Gate

Ergo ring: ergo-implementer build → ergo-leader re-derivation → ergo-qa
independent re-derivation (run the AC8 dangerous-direction discriminator + the
positive `proc`-field-pure-witness case yourself) → **Architect gate**
(safe-direction-only, AC6-preserved, zero-TCB/sort-delta) → `git_request` to
Steward. CI-gated. Own retro. Resource discipline (`CARGO_BUILD_JOBS=2`,
scoped `-p` tests). This unblocks DS-8's `instance Traversable List/Option`
assembly — flag the completion path in the handback.
