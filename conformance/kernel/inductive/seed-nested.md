# Nested strictly-positive inductive conformance

Format: `../../README.md`. These are the conformance obligations for the
nested-positive extension of `spec/10-kernel/14-inductive.md`: structural
admission through checked strictly-positive parameter positions, a generated
lifted induction hypothesis whose leaves cover every contained recursive
occurrence, nested ι, and the fail-closed boundary.

Ground truth: `14 §3.2` (lifted IH), `§7.8` (nested ι), `§8.5` (structural
parameter-polarity rule), and `§9.5` (subject reduction, termination, and
required population). The stable declaration entry is `18 §4.3`. Frame:
`docs/program/issues/SPEC-NESTED-IND.md`; build node:
`docs/program/issues/KERNEL-NESTED-IND.md`. Clean-room: derived from those
Ken-owned objects and first principles; no reference implementation or
`local/refs/` source was consulted.

Cases marked `[KERNEL-NESTED-IND]` are design-locked but implementation-gated.
Before that node lands, the kernel's conservative blanket rejection remains the
live safe boundary. The existing direct and W-style controls at the end remain
live throughout, so staging the new completeness class does not suspend the
positivity posture.

The custom positive carrier used below is deliberately not a standard-library
name:

```ken
data Bag (A : Type 0) : Type 0 where
  empty : Bag A
  one   : A -> Bag A
  join  : Bag A -> Bag A -> Bag A

data Rose : Type 0 where
  leaf : Rose
  node : Bag Rose -> Rose
```

`Bag`'s parameter is checked strictly positive from its own constructor
telescopes. No later rule may recognize `Bag` by name.

---

## AC1 — structural admission, never a constructor-name allow-list

Spec: `14 §8.5` clauses 1–3.

### kernel/inductive/nested-ds9-shapes-admitted [KERNEL-NESTED-IND]

- spec: `14 §8.5`; `18 §4.3`
- given: previously admitted positive `List` and `Pair`, followed by an ordinary
  `Json` declaration containing both `JsonArray : List Json -> Json` and
  `JsonObject : List (Pair String Json) -> Json`
- expect: **accepted** by `declare_inductive`; `Json`, its constructors, and
  `elim_Json` are admitted
- why: both paths to `Json` contain only checked strictly-positive parameter
  positions: `List`'s sole parameter, and that parameter followed by `Pair`'s
  second parameter. This is the concrete DS-9 consumer and proves that finite
  positive paths compose. It is necessary but not sufficient: the
  custom-carrier case below prevents a `List`/`Pair` allow-list from satisfying
  the corpus.

### kernel/inductive/nested-fresh-carrier-admitted [KERNEL-NESTED-IND]

- spec: `14 §8.5` clauses 1–3; `18 §4.3`
- given: admit the `Bag A` declaration above, then submit the `Rose` declaration
  through the same `declare_inductive` API without changing the kernel
- expect: **accepted**; `Rose`, `leaf`, `node`, and `elim_Rose` are generated
- why: the producer must derive `Bag`'s sole parameter as strictly positive when
  `Bag` is admitted, and the later `Rose` check must consume that recorded fact.
  **Verdict flip:** a name allow-list containing only pre-existing formers
  rejects this fresh `Bag Rose`; the structural rule accepts it. Conversely,
  accepting every application argument would fail AC4's controlled negative
  cases.

### kernel/inductive/nested-positive-chain-composes [KERNEL-NESTED-IND]

- spec: `14 §8.5` clause 3
- given: after admitting `Bag`, admit a second fresh positive carrier
  `Wrap A` with `wrap : A -> Wrap A`, then declare
  `Deep` with `deep : Bag (Wrap Deep) -> Deep`
- expect: **accepted** and `elim_Deep` generated
- why: the path through `Bag`'s and then `Wrap`'s sole parameters is a finite
  composition of two independently checked positive positions. A checker that
  inspects only the immediate
  application or caps traversal at one former rejects this case, while the
  normative compositional rule accepts it.

---

## AC2 — the lifted IH is generated, reachable, and load-bearing

Spec: `14 §3.2`, `§7.8`, `§9.5`; `34 §3.1`; `39 §2.2`; `43 §1`.

### kernel/inductive/nested-method-structured-lift [KERNEL-NESTED-IND]

- spec: `14 §3.2`
- given: the generated `elim_Rose` for the `Bag`/`Rose` declarations above,
  under motive `M : Rose -> Type 0`
- expect: the `node` method receives the original `b : Bag Rose` and exactly
  one structured hypothesis `ih : Lift_Rose(M, Bag Rose, b)`. The lift preserves
  `b`'s `empty`/`one`/`join` topology and has exactly one leaf of type `M r` for
  every contained `r : Rose`; it has no leaf where `b` contains no `Rose`
- why: this asserts the spec's locked granularity: one structured lifted
  hypothesis per nested field, with leaves in bijection with dynamic recursive
  occurrences. It does not pin a new surface type former or a private kernel
  representation for `Lift`. **Structural discriminator:** omitting the lift,
  emitting a single unstructured summary, or emitting one leaf per constructor
  field rather than per contained `Rose` produces a different generated method
  type.

### kernel/inductive/nested-iota-preserves-topology [KERNEL-NESTED-IND]

- spec: `14 §7.8`; `42 §3.3`
- given: `elim_Rose M ml mn (node b)` where
  `b = join (one r1) (join empty (one r2))`
- expect: the outer ι-step selects only `mn` and supplies
  `lift-elim_Rose(M, ml, mn, Bag Rose, b)`. Reducing that lift preserves the
  `join (one _) (join empty (one _))` topology and places
  `elim_Rose M ml mn r1` and `elim_Rose M ml mn r2` at its two recursive leaves
- why: nested ι is the admitted host former's constructor/eliminator traversal
  followed by `Rose` elimination at each child. **Structural discriminator:**
  dropping either leaf, changing the enclosing topology, evaluating an untaken
  `Rose` method, or passing `b` unchanged yields a different reduct. This pins
  reduction without over-specifying an internal runtime representation.

### kernel/inductive/nested-size-uses-lift [KERNEL-NESTED-IND]

- spec: `14 §3.2`, `§7.8`, `§9.5`; `39 §2.2`; `43 §1`
- given: define `size : Rose -> Nat` by the generated eliminator. The `leaf`
  method returns `1`; the `node` method folds the supplied `Bag`-shaped lifted
  `Nat` values and adds `1`. Evaluate
  `size (node (join (one leaf) (one (node empty))))`
- expect: **reduces-to `3`**. The two contained children contribute `1` each,
  and the outer node contributes `1`
- why: this is the load-bearing value flip. With the correct lifted IH the
  result is `3`; a guard-deletion-only implementation that admits `Rose` but
  supplies no lift cannot type-check the definition, and an implementation that
  supplies a lift but drops/ignores its leaves computes `1`. Surface
  elaboration and termination must preserve the generated hypotheses; they may
  not reconstruct unrestricted self-calls.

### kernel/inductive/nested-dependent-motive-uses-lift [KERNEL-NESTED-IND]

- spec: `14 §3.2`, `§9.5`; `34 §3.1`; `39 §2.2`
- given: a dependent motive `AllGood : Rose -> Omega_0` whose `node` proof
  matches the `Bag Rose` field and consumes the motive instance attached to each
  exposed child by the lifted IH
- expect: **accepts**. Each exposed `Rose` child is accompanied by its exact
  `AllGood child` proof; each residual `Bag Rose` value is accompanied by the
  correspondingly residual lift
- why: a non-dependent recursor or an elaborator that binds the field but loses
  the correlated lift cannot construct the branch proof. This pins that the
  feature is induction, not merely a constant-result fold.

---

## AC3 — unknown and non-positive positions fail closed separately

Spec: `14 §8.5` clauses 1, 2, 4, and 6.

### kernel/inductive/nested-unknown-head-rejected [KERNEL-NESTED-IND] (soundness)

- spec: `14 §8.5` clauses 1 and 6; `18 §4.3`
- given: declare
  `data UnknownNest (F : Type 0 -> Type 0) : Type 0 where
  mk : F (UnknownNest F) -> UnknownNest F`. The application head `F` is a bound
  variable, not a previously admitted former with checked parameter polarity
- expect: **`Err(PositivityViolation)`**; the declaration and eliminator are not
  admitted
- why: `F` cannot be resolved to checked polarity metadata, so its argument
  position is **unknown** and the retained `occurs` guard rejects it.
  **Disconfirming check:** a buggy fallback `unknown => positive` admits this
  exact declaration, while the correct rule rejects. The nearby fresh-`Bag`
  accept proves the corpus is not merely demanding blanket rejection of every
  nested application.

### kernel/inductive/nested-nonpositive-rejected [KERNEL-NESTED-IND] (soundness)

- spec: `14 §8.5` clauses 2 and 4; `18 §4.3`
- given: first admit
  `data Contra (A : Type 0) : Type 0 where
  contra : (A -> Bool) -> Contra A`, whose parameter `A` is checked
  non-positive; then declare `data Bad : Type 0 where mk : Contra Bad -> Bad`
- expect: **`Err(PositivityViolation)`** at the `Contra` parameter boundary;
  `Bad` and `elim_Bad` are not admitted
- why: this position is known and classified **non-positive**, distinct from the
  unknown-head case. **Controlled verdict flip:** replace only `Contra` by the
  identically kinded positive `Bag` and the declaration admits; a checker that
  records every parameter as positive accepts `Contra Bad` incorrectly.

---

## AC4 — negative polarity remains negative under a positive carrier

Spec: `14 §8.3`, `§8.5` clause 5.

### kernel/inductive/nested-negative-under-positive [KERNEL-NESTED-IND] (soundness)

- spec: `14 §8.3`; `§8.5` clauses 3 and 5
- given: with positive `Bag` already admitted, declare
  `data Bad : Type 0 where mk : Bag (Bad -> Empty) -> Bad`
- expect: **`Err(PositivityViolation)`**; no `Bad` declarations are committed
- why: the checker may traverse `Bag`'s sole parameter, but then reaches `Bad`
  in the domain of `Bad -> Empty`, at negative polarisation. **Disconfirming
  check:** a bug
  that treats a positive outer parameter as making its entire argument positive
  admits this term. The positive control changes only the payload to `Bag Bad`
  and is accepted, so the rejection is specifically the inner arrow polarity,
  not the outer application.

### kernel/inductive/nested-negative-existing-pair-control

- spec: `14 §8.3`; `§8.5` clause 5
- given: the existing `kernel/seed-k1.md` case
  `nested-negative-in-application-rejected`,
  `Pair (Bad3 -> Empty) Unit`
- expect: **unchanged rejection**
- why: after nested-positive admission lands, the old case is no longer
  explained by a blanket ban on application arguments. The checker traverses
  `Pair`'s checked positive parameter and rejects when it reaches `Bad3` at
  negative polarity. This retains the original soundness verdict while
  reconciling its mechanism with `§8.5`.

---

## AC5 — mutual remains separate; direct and W-style stay live

Spec: `14 §8.6`; `§3`, `§3.1`, `§7.3`, `§7.7`, `§8.4`.

### kernel/inductive/mutual-family-block-still-rejected

- spec: `14 §8.6`; `18 §4.1`, `§4.3`
- given: an attempted simultaneous inductive block for two families that refer
  to one another, such as `Even` and `Odd`
- expect: **rejected before either family, constructor set, or eliminator is
  committed**
- why: nested-only does not add a simultaneous declaration API, joint polarity
  judgment, jointly generated eliminators, or joint termination argument. A
  frontend syntax for such a block, if introduced later, cannot elaborate it
  into two independently admitted declarations that temporarily trust the
  other. This asserts the semantic boundary without pinning a future surface
  spelling.

### kernel/inductive/nested-direct-and-wstyle-controls-unchanged

- spec: `14 §3`, `§3.1`, `§7.3`, `§7.7`, `§8.4`
- given: rerun `kernel/seed-k1.md`'s direct `Nat`/`Vec` admission and ι cases,
  plus this directory's `seed-wstyle.md` admission, Π-abstracted-IH, ι, and
  termination cases with their executable fixtures and expected behavior
  unchanged
- expect: **all unchanged-green**
- why: nested lifting extends only constructor arguments reached through
  checked positive parameter paths. Direct recursive arguments retain one
  direct motive instance; Π-bound arguments retain their function-shaped IH.
  The existing live suites are the adjacent enforcement while the new cases are
  implementation-gated.

---

## Internal consistency and stage audit

- Correct custom-positive and DS-9 paths accept; unknown, known non-positive,
  nested-negative, and mutual paths reject for distinct named guards.
- The real `size` computation and the structural nested-ι case both require the
  same `Bag`-shaped lift. Neither can pass by deleting the old `occurs` guard.
- The negative cases are not coincidental blanket rejects: after
  `KERNEL-NESTED-IND`, their paired positive controls admit while the exact
  negative/unknown mutation flips the verdict.
- Until that node lands, the positive/lift cases and the reason-specific
  negative cases remain gated together. The direct/W-style corpus stays live,
  and the current blanket nested rejection remains a safe implementation
  boundary.
