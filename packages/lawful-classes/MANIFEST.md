# `lawful-classes` — `Eq`, `DecEq`, `Ord`

**Spec catalog entry:** `spec/50-stdlib/51-lawful-classes.md`. The first
`packages/` catalog tranche and the pattern-setter for every later ES4
package.

## Public API

- `IsTrue (b : Bool) : Prop` — bridging notation, `IsTrue b := Equal Bool b
  True` (`51 §2`).
- `class Eq a { eq; refl; sym; trans }` — decidable Boolean equality, an
  equivalence (`51 §2.1`).
- `class DecEq a { eq; sound; complete }` — decides the kernel's
  propositional `Equal` (`51 §2.2`).
- `class Ord a { leq; refl; antisym; trans; total }` — total order; `leq`
  is the comparator `sort`/`isSorted` thread explicitly (`51 §2.3`/`§4`,
  ES2-remainder `2358b4d`); `where Ord a` supplies it via the resolved
  dictionary's `d.leq` projection (`33 §5.4` implicit-instance desugaring).
- `instance Eq Int`, `instance DecEq Int`, `instance Ord Int` — canonical
  instances over the `Int` primitive.

## Derivation path + `trusted_base()` delta

- **The three classes** are `class` declarations = record types (`33 §5.2`,
  right-nested Σ over `13 §3`), built from `Bool` (prelude, `30 §4`) + the
  kernel's `Eq`/logic vocabulary (`15`/`16`) + the Σ/record machinery. **No
  new kernel former, zero delta.**
- **The `Int` instances are AUDITED-DELTA, not zero-delta** (`51 §6`): `Int`
  is a K1 primitive — opaque to δ (a primitive's registered reduction only
  fires on literals; the kernel's `whnf` only unfolds `Decl::Transparent`,
  never `Decl::Primitive`, `ken-kernel/src/conv.rs`) and has no induction
  principle, so its ∀-quantified laws (`refl`/`sym`/`trans`/`sound`/
  `complete`/`antisym`/`total`) are **not kernel-provable** — proving any of
  them would require a `declare_postulate` regardless of how the law is
  phrased (confirmed empirically: `Term::Elim`'s motive-codomain check,
  `check.rs::infer_motive_level`, requires `Type(l)`, never `Omega(l)`, so
  even an inductive carrier's laws can't yet be proved by case-split — a
  kernel-capability gap, tracked as a forward WP, not specific to `Int`).
  The **operation** fields (`leq`/`eq`) wrap the audited `leq_int`/`eq_int`
  primitives (`numbers.rs`) — existing `trusted_base()` entries, adding
  nothing new. The **law** fields are honest, visible postulates (`Axiom` —
  a real `Decl::Opaque`, grep-able, never hidden) — each contributes one
  entry to `trusted_base_delta`. `Int` is **illustrative-only**: it is
  **not** claimed as a zero-delta/AC3-lawful instance.
- **The zero-delta exemplar** (a law-carrying instance over an *inductive*
  carrier, e.g. a future `Ord Bool`/`Ord <user data>`, with every law field a
  real kernel proof) is a **named forward WP**, gated on the kernel gaining
  Ω-motive `Elim` support (Architect ruling, `evt_68ppz77ysh5ne`) — not
  delivered by this package yet.

## `where Ord a` wiring (AC2)

`where Ord a` desugars to an implicit `{d : Ord a}` (`33 §5.4`); the
resolved dictionary is bound under the surface name `d` for the duration of
that one declaration's elaboration (never leaks to sibling decls), so the
body/refinement can project its fields (`d.leq`) exactly as the spec's own
illustration shows — ordinary implicit-dictionary insertion, the *same*
`sort`/`isSorted` view as the explicit-comparator form, no second mechanism.
