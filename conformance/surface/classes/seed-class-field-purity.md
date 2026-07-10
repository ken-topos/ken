# SURF-2 class-field purity conformance — seed cases

Format: `../../README.md`. This seed pins the SURF-2 mechanism that extends the
SURF-1 `const`/`fn`/`proc` checked-purity discipline from top-level definitions
to **class fields**. It is additive over `seed-classes.md` (class records,
instances, projection, coherence) and
`../declarations/seed-purity-keywords.md` (definition-level purity keywords).

The new surface is exactly an optional leading keyword on a class field:

```ken
class C A {
  fn op : A -> A
  plain  : A -> A
}
```

An unmarked field is status quo. A marked field is a checked surface signature:
the parser stores the marker, the class registry carries it, instance fields are
checked through the existing SURF-1 purity classifier, and `.field` projection
surfaces the field purity so a use site classifies the projection correctly.

**Trust posture.** The marker is elaborator metadata, erased before the kernel.
It must not alter the class record type, the right-nested Sigma of field types,
`sort_sigma`, or `trusted_base()`. The kernel remains the checker for the class
record and instance record value; the marker only changes front-end
classification and diagnostics. The non-interference cases below are therefore
mandatory: a marker that changes the Type/Omega class-kind discriminant is a
surface soundness bug even if the emitted core still checks.

**Build-forcing.** These cases are red on the pre-SURF-2 parser/elaborator: the
class parser accepts only `field : type`, `ClassInfo` carries no field purities,
instance checking does not enforce a class-field keyword, and projection does
not preserve field purity. A conforming build must drive the real producer path;
do not hand-feed a synthetic `ClassInfo` or inspect only a parser token stream.

**Citations.** `spec/30-surface/33-declarations.md §1` (definition purity
keywords) and `§5` (classes as records), `36-effects.md §1.6` (the checked
purity split), `39-elaboration.md §6` (class declaration, instance checking,
projection), and `50-stdlib/56-effectful-classes.md §5.1`/`§5.2`
(`proc traverse` is required and row-polymorphic).

---

## CFP1 — Marked class fields parse and elaborate

### surface/classes/class-field-proc-traverse-parses-elaborates
- spec: `33 §5` (class field grammar), `39 §6` (class declaration
  elaboration), `56 §5.1` (the `proc traverse` field)
- given: the Traversable signature shape from `56 §5.1`, including an optional
  leading purity marker on `traverse`:

  ```ken
  class Functor (f : Type -> Type) {
    map : (a b : Type) -> (a -> b) -> f a -> f b
  }

  class Foldable (f : Type -> Type) {
    foldr : (a b : Type) -> (a -> b -> b) -> b -> f a -> b
  }

  class Applicative (g : Type -> Type) {
    functor : Functor g
    pure    : (a : Type) -> a -> g a
    ap      : (a b : Type) -> g (a -> b) -> g a -> g b
  }

  class Traversable (f : Type -> Type) {
    functor  : Functor f
    foldable : Foldable f
    proc traverse :
      (g : Type -> Type) -> Applicative g -> (a b : Type) ->
      (a -> g b) -> f a -> g (f b)
  }
  ```

- expect: **accepts** through parse, resolution, class declaration elaboration,
  and kernel re-check of the class record type. The registered class metadata
  contains a `proc` purity marker for `traverse` and no marker for `functor` or
  `foldable`. The emitted class record type is still the Sigma chain over the
  three **field types** in declaration order.
- why: G1. This is the grammar and storage gate that makes the already-settled
  `56 §5.1` contract expressible. A build that treats `proc` as the field name,
  rejects it as an unexpected keyword, or drops the marker during class
  registration fails before CAT-2 D3 can resume.

### surface/classes/unmarked-class-fields-remain-status-quo
- spec: `33 §5`, `39 §6.1`, SURF-2 backward-compatibility guard
- given: an existing unmarked structure class and instance, e.g.

  ```ken
  class Endo A {
    apply : A -> A
  }

  fn id_int (x : Int) : Int = x
  instance Endo Int { apply = id_int }
  ```

- expect: **accepts** exactly as before SURF-2. The field has no stored purity
  marker, so the instance field is checked only against the substituted field
  type and projection has no extra purity classification. The existing Lc,
  CAT-1, and lawful-class suites remain green with no field annotations added.
- why: G4. The marker is optional and additive. A build that forces every class
  field to choose `const`/`fn`/`proc`, or changes unmarked projection behavior,
  breaks existing class programs rather than extending them.

---

## CFP2 — Projection retains class-field purity

### surface/classes/proc-field-projection-classifies-proc
- spec: `39 §6` (projection metadata), `36 §1.6` (proc classification),
  `56 §5.2` (traverse stays `proc`)
- given: a marked operation field, a valid `proc` implementation registered in
  an instance, and two use sites with the same projected call:

  ```ken
  class Effectful A {
    proc step : A ->[FS] A
  }

  proc step_int (x : Int) : Int visits [FS] = x
  instance Effectful Int { step = step_int }

  proc ok_use (x : Int) : Int visits [FS] =
    (Effectful_instance_Int).step x

  fn bad_use (x : Int) : Int =
    (Effectful_instance_Int).step x
  ```

- expect: `ok_use` **accepts** and `bad_use` **rejects** with the SURF-1 purity
  error path, naming a projected `proc` field/effect escape rather than a bare
  type mismatch. Equivalently, the projection `(Effectful_instance_Int).step`
  is visible to the purity checker as a `proc` operation, not as an ordinary
  pure function.
- why: G2. This is the use-site half of the contract: storing the marker on
  `ClassInfo` is not enough unless `.field` projection carries it back into
  expression classification. The verdict flips only on the enclosing keyword
  (`proc ok_use` versus `fn bad_use`), with the projected body held fixed.

### surface/classes/where-dict-proc-field-projection-classifies-proc
- spec: `39 §6.2` (instance search supplies dictionary `d`), `39 §6` projection,
  `36 §1.6`
- given: the same `Effectful Int` instance, but the field is reached through an
  implicit dictionary supplied by a `where` constraint:

  ```ken
  fn bad_where_use (x : Int) : Int where Effectful Int =
    d.step x
  ```

- expect: **rejects** through the same purity/classification path as the named
  instance projection. The diagnostic identifies that the projected class field
  is `proc`-classified; it is not a `NoInstance`, `UnresolvedCon`, or generic
  projection error.
- why: G2, dictionary-form. CAT-2 uses dictionary fields in generic bodies; a
  build that only handles concrete `Class_instance_Head.field` and drops the
  marker for an opaque bound dictionary would still leave `d.traverse` wrongly
  classifiable as pure.

---

## CFP3 — Instance fields satisfy the declared class-field purity (covariant)

### surface/classes/instance-field-purity-covariant-subsumption
- spec: `33 §5.2`/`§5.3` (instance field checking), `36 §1.6.2` (DS-8b: the
  `∅ ⊆ proc` instance-field subsumption and its one-way covariance), `39 §6`
  (instance record construction)
- given: witnesses for marked fields exercising both the DS-8b widening and the
  still-live dangerous direction:

  ```ken
  class Effectful A {
    proc step : A ->[FS] A
  }

  proc step_int (x : Int) : Int visits [FS] = x
  instance Effectful Int  { step = step_int }    -- effectful witness, proc field

  fn step_bool (x : Bool) : Bool = x
  instance Effectful Bool { step = step_bool }   -- PURE witness, proc field

  class Pure A {
    fn compute : A -> A
  }

  proc compute_fs (x : Bool) : Bool visits [FS] = x
  instance Pure Bool { compute = compute_fs }     -- effectful witness, fn field
  ```

- expect:
  - `Effectful Int` **accepts** — an effectful witness for a `proc` field
    (unchanged).
  - `Effectful Bool` **accepts** — a **pure `∅`-row witness satisfies the
    `proc` field** by covariant subsumption `∅ ⊆ ρ_field` (`proc`'s contract is
    "may be effectful"; a pure witness is a more precise inhabitant). **This
    reverses the pre-DS-8b behavior**, which rejected "field requires `proc`";
    the reversal is the landed `check_instance_field_purity` change — the
    `Proc if !impure => Err` arm removed, so a pure witness for a `proc` field
    now falls through to `Ok` (`36 §1.6.2`, DS-8b).
  - `Pure Bool` **rejects** — the **dangerous direction is unchanged**: an
    effectful witness for a pure `fn` field is rejected on the specific variant
    `class field `Pure.compute` requires `Fn` but instance implementation is
    effectful: EffectEscapes … FS` (the retained `Const | Fn if impure => Err`
    arm). Assert this exact variant — not a bare `is_err()`, and not overlap,
    orphan, or a missing field.
- why: G3, re-cast for DS-8b's covariant subsumption. The covariance is
  **one-way**, and the **non-degenerate discriminating pair** is `Effectful
  Bool` (pure → `proc`, **accept**) against `Pure Bool` (effectful → `fn`,
  **reject**): a bug that made subsumption symmetric (accepting the
  effectful-into-`fn` direction), or a regression re-adding the pure-into-`proc`
  reject, flips exactly one of the pair — on the same
  `check_instance_field_purity` path. The structural discriminator is the stored
  field marker combined with the witness's SURF-1 effect-row classification
  (`∅` vs non-`∅`).
- AC6 (field classification unchanged): accepting a pure witness does **not**
  weaken the field's own purity — `d.step` still projects as `proc` at a use
  site (a pure caller still cannot call it through the dictionary), pinned by
  `surface/classes/proc-field-projection-classifies-proc` (CFP2). Only the set
  of witnesses that may **inhabit** the field widens.

### surface/classes/fn-and-const-field-signatures-follow-declared-type
- spec: `33 §5.2` (class-field marker reads the field type/telescope),
  `36 §1.6.3` (arity and purity mismatch errors), `39 §6`
- given: a class with marked pure fields whose declared field types determine
  the `const`/`fn` split, plus the two malformed marker/type flips:

  ```ken
  class PureFields {
    const seed : Int
    fn unary_fn : Int -> Int
  }

  const seed_ok : Int = 0
  fn unary_fn_ok (x : Int) : Int = x
  instance PureFields { seed = seed_ok ; unary_fn = unary_fn_ok }

  class BadConstField {
    const not_const : Int -> Int
  }

  class BadFnField {
    fn not_fn : Int
  }
  ```

- expect: `PureFields` and its instance **accept**: `seed : Int` is
  zero-explicit-value-argument and therefore `const`-shaped, while
  `unary_fn : Int -> Int` is one-explicit-value-argument and therefore
  `fn`-shaped. `BadConstField` **rejects** as should-be-`fn` because
  `Int -> Int` is not a `const` field signature; `BadFnField` **rejects** as
  should-be-`const` because `Int` has zero explicit value arguments. The
  diagnostic is the same SURF-1 arity/purity family, but the arity is read from
  the declared class-field type/telescope, not from a separate field binder list
  and not from instance-body convention.
- why: G3 for the whole marker set, not just `proc`. SURF-2 reuses the
  existing purity path; it must not special-case only the CAT-2 `proc traverse`
  spelling.

---

## CFP4 — Purity markers do not affect the class sort discriminant

### surface/classes/class-field-purity-marker-does-not-enter-ac4-sort
- spec: `33 §5.1` (property vs structure class), `13 §4` (`sort_sigma`),
  SURF-2 AC4 non-interference
- given: pairs of classes whose **field types** are identical and whose only
  difference is the optional marker:

  ```ken
  class PlainStruct A {
    op : A -> A
  }

  class MarkedStruct A {
    fn op : A -> A
  }

  class PlainProp {
    witness : Equal Bool True True
  }

  class MarkedProp {
    const witness : Equal Bool True True
  }
  ```

- expect: `PlainStruct` and `MarkedStruct` have the same structure-vs-property
  classification (`Structure`), because `A -> A` is a relevant Type-valued
  field. `PlainProp` and `MarkedProp` have the same classification (`Property`),
  because the field type is Omega-valued. The emitted Sigma field types are
  byte-for-byte the same within each pair; only the surface metadata differs.
- why: G5 and the AC4 guard. The marker must never be a hidden third input to
  the Type/Omega discriminant. The marked examples deliberately use
  arity-compatible pure markers so this case isolates sort non-interference; a
  build that lets the marker influence class-kind selection would change
  coherence behavior even though the field types are unchanged.

### surface/classes/class-field-purity-zero-kernel-delta
- spec: `33 §5.2` (classes as Sigma records), `39 §1` (elaborator untrusted),
  SURF-2 zero-kernel-delta guard
- given: the SURF-2 implementation diff and an elaborated program containing
  marked and unmarked class fields
- expect: `crates/ken-kernel/**` and `Cargo.lock` have no diff for SURF-2;
  `trusted_base()` for the marked-field program differs from the unmarked
  counterpart only by the ordinary user declarations that already existed, not
  by any new primitive/postulate for purity. In particular, the class record
  type and instance record value are still kernel-checked Sigma/Pair terms.
- why: G5. Class-field purity is a surface classification layer. Any kernel
  constructor, trusted-base entry, or sort rule added for it is outside the WP
  frame and would invalidate the "erased before kernel" guarantee.

---

## CFP5 — CAT-2 unblock hook

### surface/classes/traversable-real-proc-field-unblocks-cat2-d3
- spec: `56 §5.1`/`§5.2` (Traversable), `33 §5`, `36 §1.5`/`§1.6`
- given: the real CAT-2 D3 `Traversable` class with `proc traverse`, plus the
  `List` and `Option` instances resumed on top of SURF-2
- expect: `proc traverse` is grammatical, instance fields are accepted only when
  their implementations are `proc`-classifiable over the applicative argument,
  and use sites see `d.traverse` as `proc`. The identity, naturality, and
  composition laws remain CAT-2's own law-proof obligations and are not
  discharged by SURF-2.
- why: G7. SURF-2 is the surface prerequisite, not a replacement for CAT-2 D3.
  This hook prevents a workaround that drops the `proc` marker or hand-rolls a
  monomorphic `traverse` merely to make CAT-2 compile.
