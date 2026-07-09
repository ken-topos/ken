# Effectful constructor classes — `Applicative`, `Monad`, `Traversable`

> Status: **DRAFT v0 (CAT-2).** The fast-follow after CAT-1
> (`55-lawful-functors.md`) and the monadic/traversal layer the whole catalog
> (parsers, effectful folds, validation, stateful computation) leans on. It
> **inherits CAT-1's template unchanged** — a class is a record
> (`../30-surface/33 §5.2`), a law is an `Ω` value-equation **proved not
> postulated** over an inductive carrier, stated **pointwise** (funext is
> definitional, `55 §5.2`), **zero `trusted_base()` delta, kernel-untouched**.
> It makes **one** template change CAT-1 pre-registered: the deep constructor
> chain **wires** superclass fields rather than restating them (`55 §7` pt 5,
> resolved here — `§2`). **Traversable is gated on SURF-1's row-variable
> surface** (`../30-surface/36 §1.5`, landed `main@ef791a3`) and **SURF-2's
> class-field purity marker** (`../30-surface/33 §5.2`, `../30-surface/39
> §6.0`); Applicative and Monad are not.

## 1. What CAT-2 inherits from CAT-1 (the template, unchanged)

Every structural rule is `55 §7`'s reusable constructor-class template, applied
verbatim:

1. **A class is a record** (`33 §5.2`); **a law is `Ω`** (`16 §1`), a direct
   `Equal (f _) u v` value-equation — no `‖·‖` truncation (`55 §4`; these are
   not proof-relevant `∨`/`∃`).
2. **Higher-kinded classes** (`f : Type → Type`) use CAT-1's bounded
   `ken-elaborator` param-kind binder (`55 §6`, the four-piece outer-ring
   extension); instances resolve to the **bare indformer** head. CAT-2 adds
   **no new elaborator capability** beyond it (`§6`); a genuinely required one
   re-forks to Steward.
3. **Laws are stated pointwise** — funext-definitional (`55 §5.2`), so the
   function-level law whnf-reduces to the pointwise form and each instance
   discharges by **direct induction on the carrier**. **One canonical field per
   law**, no point-free duplicate.
4. **Instances land over inductive carriers** (`List`/`Option`) so every law is
   **proved** by induction + `cong` (`55 §3.1`) with the `tt`-vs-`Refl`
   discrimination (`55 §3.2`) — **zero `Axiom`, zero delta**. A postulate on an
   inductive carrier is a **defect** (`51 §5`), never an honest delta.

The **one** change: `55 §7` pt 5 left open whether the stronger class *restates*
the weaker's operation+law (the value-class default, `55 §2.2`) or **wires** a
superclass field when the chain is deep. CAT-2 is where the chain
(`Functor → Applicative → Monad`) first gets deep enough; `§2` resolves it.

## 2. The wired superclass chain (`55 §7` pt 5 — resolved: **WIRE**)

**Ruling (Architect, CAT-2 core).** The constructor-class chain **wires** an
explicit superclass-dictionary field, applied **consistently up the whole
chain** (AC7):

- **`Applicative f`** carries a field `functor : Functor f`.
- **`Monad f`** carries a field `applicative : Applicative f`.
- **`Traversable f`** carries `functor : Functor f` + `foldable : Foldable f`.

This **reverses** the value-class *restate* default (`55 §2.2`, `Monoid`
restating `Semigroup`'s `op`/`assoc`) — and that reversal is exactly what
`55 §7` pt 5 pre-registered: *restate … unless the chain is deep enough
(Functor→Applicative→Monad) that wiring wins*. A three-deep chain is that
condition.

**Why wire (mechanism-cost, grounded on the landed elaborator):**

- **Zero new capability beyond `55 §6`.** `elab_class_decl` elaborates each
  field type in the class-parameter context, so a field typed `Functor f` (an
  `App` of the `Functor` class-`Const` to the parameter) elaborates like any
  other field. Nested projection `d.applicative.functor.map` composes:
  `infer_proj` resolves each `.field` by matching the base type's head `Const`
  against the class's `type_id`, re-entering cleanly on the inner dict — and it
  already supports an **opaque bound-variable base** (needed for the generic
  case).
- **Proof reuse is the win, and it is real.** An instance's field values are
  checked against their substituted expected types
  (`compute_ordered_field_values`), so `instance Monad List` supplies the
  already-built `Applicative List` dict as its `applicative` field: the six
  `Functor` + `Applicative` law proofs are **not re-proved** — only `bind` + the
  three monad laws are new. **Restatement would duplicate those six proofs at
  every deeper instance** — a divergence surface and a
  subsume-don't-proliferate violation.
- **Same trust level, no TCB regression.** Both the class record and the nested
  sub-dict are kernel-re-checked terms (`declare_def`); wiring moves nothing out
  of kernel protection (the `coexist-when-trust-differs` guard does **not** bite
  — unlike folding a kernel-checked op into a trusted one).
- **Closed-carrier instances** (`List`/`Option`) wire cleanly — a closed head,
  no parametric-head gap.

**Honest boundary (bounded, non-blocking).** Explicit wiring makes *use sites*
verbose: `d.functor.map`, not a bare `map`. The ergonomic fix — **implicit
superclass coercion** (auto-`map` in an `Applicative` context, the Haskell
`Functor f ⇒ Applicative f` reading) — **would** need a new elaborator
capability (instance resolution walking the superclass edge), i.e. a `55 §6`
guardrail re-fork to Steward. **CAT-2 does not take it:** it ships **explicit**
wiring (field + projection, all existing machinery); implicit-coercion sugar is
deferred as an `OQ-syntax`/elaborator follow-on (`90 §OQ-syntax`), re-forked to
Steward if and when wanted. Nothing in this chapter depends on it.

## 3. D1 — `Applicative f` (wires `Functor f`)

### 3.1 Signature

```
class Applicative (f : Type → Type) {
  functor : Functor f
  pure    : (a : Type) → a → f a
  ap      : (a b : Type) → f (a → b) → f a → f b
  -- laws: §3.2
}
```

`ap` is the ergonomic `<*>` (a plain identifier field; infix sugar deferred
`OQ-syntax`, `55 §2.1`). `pure` lifts a value into the effect-free position.
`functor` is the wired superclass (`§2`); its `map` agrees with the applicative
structure by the coherence law (`§3.2`, `map_coh`).

### 3.2 The four laws, stated pointwise (AC2/AC3)

Each is an `Ω` value-equation, one canonical field, the exact `pure`/`ap`
phrasing character-for-character (as `55 §5.2` did for `Functor`). `idf`/`comp`
are ordinary Ken views (`55 §5.2`); `applyTo a b y := λ(g : a → b). g y` and
`compose a b c := λ(g : b → c)(h : a → b)(x : a). g (h x)`.

```
-- identity:  pure id <*> v = v
ap_id  : (a : Type) → (v : f a) →
           Equal (f a) (ap a a (pure (a → a) (idf a)) v) v

-- homomorphism:  pure g <*> pure x = pure (g x)
ap_hom : (a b : Type) → (g : a → b) → (x : a) →
           Equal (f b) (ap a b (pure (a → b) g) (pure a x)) (pure b (g x))

-- interchange:  u <*> pure y = pure ($ y) <*> u
ap_ich : (a b : Type) → (u : f (a → b)) → (y : a) →
           Equal (f b) (ap a b u (pure a y))
                       (ap (a → b) b (pure ((a → b) → b) (applyTo a b y)) u)

-- composition:  pure (∘) <*> u <*> v <*> w = u <*> (v <*> w)
ap_cmp : (a b c : Type) → (u : f (b → c)) → (v : f (a → b)) → (w : f a) →
           Equal (f c)
             (ap a c
                (ap (a → b) (a → c)
                   (ap (b → c) ((a → b) → (a → c))
                      (pure ((b → c) → (a → b) → (a → c)) (compose a b c)) u)
                   v)
                w)
             (ap b c u (ap a b v w))
```

The wired `functor` field is pinned to the applicative structure by one
coherence equation (so `map` is not a second, divergent operation):

```
-- map coherence:  map g x = pure g <*> x
map_coh : (a b : Type) → (g : a → b) → (x : f a) →
            Equal (f b) (functor.map a b g x) (ap a b (pure (a → b) g) x)
```

All five are `Ω`-clean value equations (`§1` pt 1) — the truncation catch does
not fire.

### 3.3 Instances — proved, zero-delta

- **`List` applicative — cartesian** (`§4.4` / Fork D): `pure a x = Cons a x
  (Nil a)`; `ap` is the cartesian product-of-effects (`concatMap`-shaped),
  forced by coherence with `Monad List` (`§4`). Every law by induction + `cong`
  (`55 §3.1`), `tt`-vs-`Refl` per endpoint (`55 §3.2`).
- **`Option` applicative:** `pure = Some`; `ap (Some g) (Some x) = Some (g x)`,
  else `None`. Laws by finite case-split.

The wired `functor` field is the already-built `Functor List`/`Functor Option`
dict (`55`), supplied whole — its laws are **not re-proved** (`§2`).

**Ziplist is not proliferated.** A second lawful `Applicative List` (`ap =
zipWith`) exists but is **not** `Monad List`-coherent (`§4.4`); if wanted it
rides a `newtype` wrapper — deferred, not CAT-2 (subsume-don't-proliferate).

## 4. D2 — `Monad f` (wires `Applicative f`)

### 4.1 Signature — `bind`-primary (Fork B)

```
class Monad (f : Type → Type) {
  applicative : Applicative f
  bind        : (a b : Type) → f a → (a → f b) → f b
  -- pure is inherited: applicative.pure
  -- laws: §4.2
}
```

**`bind`-primary** (Architect, Fork B), grounded on the landed interaction-tree
`bind` (`declare_bind`, `ken-elaborator/src/effects/state.rs`): a single
`Term::Elim` over `ITree e resp` whose `Ret` method is `λx. k x` — so
`bind (Ret a) k = k a`, **left-identity is definitional** (`ι` on `Ret`). Making
`bind` primary makes the ITree bridge (`§4.3`) a **direct correspondence**,
not a re-derivation. `pure := applicative.pure`; **`join`/`map` are derivable
convenience, not primary — not proliferated as separate primary fields**. Field
order `(m : f a) (k : a → f b)` matches the landed bind exactly.

### 4.2 The three laws, stated pointwise (AC2/AC3)

```
-- left identity:  bind (pure a) k = k a
bind_lid : (a b : Type) → (x : a) → (k : a → f b) →
             Equal (f b) (bind a b (applicative.pure a x) k) (k x)

-- right identity:  bind m pure = m
bind_rid : (a : Type) → (m : f a) →
             Equal (f a) (bind a a m (applicative.pure a)) m

-- associativity:  bind (bind m k) h = bind m (λx. bind (k x) h)
bind_asc : (a b c : Type) → (m : f a) → (k : a → f b) → (h : b → f c) →
             Equal (f c) (bind b c (bind a b m k) h)
                         (bind a c m (λ(x : a). bind b c (k x) h))
```

`Ω`-clean value equations. The wired `applicative` field carries `pure`/`ap` +
their four laws already-proved (`§2`); `Monad` adds only `bind` + these three.

### 4.3 The ITree bridge — **attested correspondence** (Fork E, AC5)

`Monad`'s `bind`/`pure` fields and its three laws are **satisfied by the landed
interaction-tree `bind`** (effect-composition `ed34129d`): `pure := Ret`,
`bind :=` the landed `Term::Elim` bind, `bind_lid` **definitional** (`ι` on
`Ret`), `bind_rid`/`bind_asc` by induction on the `ITree`. **No second `bind` is
minted** — the effect system's denotation is a lawful monad *by construction*,
one denotation, not two (AC5).

The reconciliation is an **attested correspondence, not a surface `instance
Monad (ITree e resp)`**. The carrier `ITree e resp` is a **parametric instance
head** (free `e`, `resp`); `elab_instance_decl` elaborates the head in an
*empty* context, so free head vars raise `UnresolvedCon` — the **CAT-1 `55 §6.1`
parametric-instance-head gap**, still open with Steward. A *general* surface
`instance Monad (ITree e resp)` therefore does **not** elaborate today. So:

- **CAT-2 deliverable:** the attested correspondence above — the fields + laws
  are satisfied-by the landed bind, documented as the bridge, discriminated in
  the seed (`§7`, AC5). Zero new code, no minted `bind`.
- **Generality upgrade (not CAT-2):** the general `instance Monad (ITree e
  resp)` is gated on `55 §6.1`'s parametric-head path — the already-open Steward
  fork, **not** reopened here. (A closed-effect `instance Monad (ITree E₀)`
  *would* elaborate, but it is not the general bridge, so it is not the CAT-2
  deliverable.)

### 4.4 Instances — proved, zero-delta

- **`List` monad:** `bind m k = concatMap k m` (flatten-map); `pure = §3.3`.
  Coherence forces the **cartesian** `Applicative List` (`§3.3` / Fork D):
  `ap mf mx = bind mf (λg. bind mx (λx. pure (g x)))` reduces to the cartesian
  product; ziplist `ap` is **not** `bind`-coherent and ziplist has no lawful
  `Monad`, so it cannot be the wired `applicative` field. Laws by induction —
  `bind_lid` closes via the inductive `list_right_unit` lemma at neutral `k a`
  (**not** definitional here, unlike `ITree`/`Option`), `bind_rid`/`bind_asc`
  bases at `Nil` are constructor-headed → `tt` (`55 §3.2`).
- **`Option` monad:** `bind (Some x) k = k x`, `bind None k = None`;
  `bind_lid` neutral `k x` → `Refl`; `bind_rid` both branches constructor-headed
  → `tt`; `bind_asc` `None` → `tt`, `Some x` neutral → `Refl`.

## 5. D3 — `Traversable f` (wires `Functor f` + `Foldable f`; **SURF-1/SURF-2-gated**)

### 5.1 Signature — `traverse` a `proc`; explicit `Applicative g` dict (Fork C)

```
class Traversable (f : Type → Type) {
  functor  : Functor f
  foldable : Foldable f
  proc traverse :
    (g : Type → Type) → Applicative g → (a b : Type) →
    (a → g b) → f a → g (f b)
  -- sequence a = traverse g ap_g (g a) a (idf (g a)) — the id-specialization
  -- laws: §5.3
}
```

`proc traverse` is a class-field purity marker, made grammatical by SURF-2
(`33 §5.2`, `39 §6.0`). The marker is the checked SURF-1 static-purity signal
for the field and its projections; it is erased before the kernel and does not
affect the class record's AC4 Type/Ω sort discriminant, which remains computed
from the field types alone. This note reconciles the surface spelling here; it
does not weaken the Traversable contract or move `proc` to instance bodies.

**`Applicative g` is an EXPLICIT (unbundled) dictionary parameter** (Architect,
Fork C), **not** an implicit `where` constraint. Forced by the landed
elaborator: an abstract `g` has **no concrete head** for implicit instance
search, whereas `infer_proj` projects `ap_g.ap`/`ap_g.pure` off an opaque
bound-variable dict fine — so the explicit form is buildable *today* and the
implicit form is not. `sequence` is `traverse` specialized at the identity
action.

### 5.2 Row-polymorphism — the same axis as the `Applicative g` abstraction

`traverse` is the **first library definition polymorphic over an arbitrary
applicative `g`**, and it classifies **`proc`** under SURF-1's bidirectional
check — **via SURF-1's row-variable mechanism verbatim, no new machinery**
(AC6). The abstraction and the effect row are **one axis at two layers**:

- `traverse`'s action `f : a → g b` has an **abstract codomain head `g`**.
  SURF-1's `classify_telescope` (`ken-elaborator/src/effects/extract.rs`)
  classifies that abstract head as `Unknown` → **fail-closed** → a fresh row
  variable (`RowVar`, `extract.rs`). So `traverse` is row-polymorphic
  **because** `g` is abstract, and is therefore `proc` (`36 §1.5`).
- The row variable **co-varies with the dict** under instantiation:
  - `g := Option` / `g := List` (a **pure** applicative) ⇒ `RowVar → ∅`, and
    `traverse` **stays `proc`** — the `36 §1.6` *do-not-optimize* guard
    (a `proc` that may instantiate to `∅` is still `proc`).
  - `g := Eff e` (an **effect** applicative) ⇒ `RowVar → e`, surfacing as
    `visits [e]` — precisely SURF-1's `36 §1.5.1` exemplar
    (`proc traverse (f : a →[e] b) … visits [e]`), the effect-specialized slice
    of this general form.
- **No double-count.** The surface row `[e]` is the conservative,
  signature-level face; the `ITree` denotation reifies **the same** effects as
  `g`-data. They agree at `g := Eff e` and both collapse at `g := Option` — one
  set of effects, two presentations.

The **surface spelling** of the row variable (`[e]` / `[E | e]` / `→[e]`) stays
`OQ-syntax` (`36 §1.5.1`); the **construct** — an explicit `Applicative g` dict
plus a row variable on the abstract-`g` action, co-varying under instantiation —
is the normative pin. Zero new mechanism (explicit dict + SURF-1's `RowVar`,
both landed).

> **Gate (frame D3, guardrail).** SURF-1's row-variable surface is on
> `main@ef791a3`, and SURF-2 supplies the class-field `proc` marker (§5.1). D3
> is clear only with both pins: had SURF-1 not landed, D1+D2 land first and D3
> holds; had SURF-2 not landed, keep the `proc traverse` contract and hold D3
> rather than moving `proc` to instance bodies or hand-rolling a monomorphic
> `traverse` to dodge the gate.

### 5.3 The traversal coherence laws (pointwise)

Stated pointwise over the carrier, `Ω`-clean value-equations. The instruments
are the standard derived applicatives — the **identity** applicative
(`g := λa. a`, `pure = id`, `ap = app`) and the **composition** applicative
`Compose g h` (both ordinary CAT-2/CAT-3 derived instances):

- **identity** — traversal at the identity action is the identity:
  `traverse Identity pure ≡ pure` (equivalently, at the identity applicative
  `traverse` reduces to `map`; the load-bearing base coherence).
- **naturality** — `traverse` commutes with any applicative morphism
  `η : g ⇒ h`: `η (traverse t x) ≡ traverse (η ∘ t) x` (a consequence of
  parametricity over `g`; stated, discharged structurally).
- **composition** — traversals compose through `Compose`:
  `traverse (Compose ∘ map t2 ∘ t1) ≡ Compose ∘ map (traverse t2) ∘ traverse
  t1`, i.e. two nested traversals fuse into one over the composite applicative.

The **composition** law is the load-bearing one (it pins `traverse`'s
interaction with the applicative structure); **identity** anchors the base;
**naturality** is the parametricity face. Their exact per-instance discharge is
by induction on the carrier (`55 §3.1`).

### 5.4 Instances — proved, zero-delta

- **`List` traverse** (canonical): `traverse g ap_g t (Nil a) = ap_g.pure (Nil
  b)`; `traverse g ap_g t (Cons a h u) = ap_g.ap … (ap_g … (t h)) (traverse g
  ap_g t u)` — the effect-sequencing fold, effect-polymorphic in `g`. Laws by
  induction + the applicative laws of `g` (`§3.2`).
- **`Option` traverse:** `traverse g ap_g t (None a) = ap_g.pure (None b)`;
  `traverse g ap_g t (Some a x) = ap_g.ap … (ap_g.pure … (Some b)) (t x)`.

## 6. Derivation paths and `trusted_base()` delta (AC1/AC4)

- **The classes** are `class` declarations = record types (`33 §5.2`,
  right-nested Σ), built from the kernel's `Equal`/`Ω` vocabulary + the wired
  superclass fields (themselves `55` records). **No new kernel former, zero
  delta.** The wiring + explicit dicts + attested bridge all ride **CAT-1's
  `55 §6` extension + existing record/projection machinery** (`elab_class_decl`,
  `infer_proj`, `compute_ordered_field_values`) — **zero new elaborator
  capability** (`§2`; AC1). If the build finds one genuinely required, it
  re-forks to Steward, not smuggled.
- **The instances are ZERO-DELTA — the inductive-carrier exemplar** (`55 §8`).
  `List`/`Option` are real inductives with eliminators, so every ∀-law is a real
  kernel proof; **no `Axiom`, nothing enters `trusted_base()`**. The
  ITree-bridge (`§4.3`) mints no new definition — it attests the landed `bind`.
- **Reused, never re-defined** (subsume-don't-proliferate): the wired `functor`/
  `applicative`/`foldable` fields are the already-built `55` dicts;
  `cong`/`sym`/`trans` (`catalog/packages/transport`); `list_append`/`concatMap`
  (`catalog/packages/collections`); the landed `bind` (`ken-elaborator/effects/`).

> **Build note (perishable).** The `.ken` source
> (`catalog/packages/lawful-functors/` additions) is a Language-build deliverable, held
> for the GPT window. CV's grounding confirms `map`/`bind`/`foldr`/`traverse`
> for `List`/`Option` are **not yet landed** — the instance-law conformance
> cases are **red-until-built** (the CAT-1 `Functor`-case posture), reconciled
> against the built package at the CAT-2 build gate. Architect re-certs
> AC1/AC3/AC5 on the built diff.

## 7. Acceptance

- **AC1 (kernel-untouched, extension-reused).** No `ken-kernel` diff; **no new
  elaborator capability beyond CAT-1's `55 §6` extension** (wiring + explicit
  dicts + attested bridge all ride existing machinery, `§6`); zero
  `trusted_base()` delta.
- **AC2 (laws `Ω`, no truncation).** Every law field is `Equal (f _) u v : Ω`
  (`§3.2`/`§4.2`/`§5.3`); no `‖·‖`.
- **AC3 (pointwise, one field per law).** Every law pointwise, one canonical
  field, no point-free duplicate (`55 §5.2`).
- **AC4 (proved, zero Axiom, zero-delta).** `List`/`Option` instances: every law
  a real kernel proof, **zero `Axiom`/postulate/opaque**, zero delta; instances
  bundle at inductive carriers.
- **AC5 (Monad ⇔ ITree reconciliation).** `Monad`'s fields/laws are satisfied by
  the landed interaction-tree `bind` (`ed34129d`) — attested correspondence
  (`§4.3`), no second divergent `bind`; discriminated in the seed.
- **AC6 (Traversable ⇔ SURF-1/SURF-2).** `traverse`'s explicit
  `Applicative g` dictionary uses SURF-1's row-variable mechanism verbatim
  (`§5.2`), and its class field uses SURF-2's checked `proc` marker (`§5.1`);
  `d.traverse` classifies `proc` at projection/use sites.
- **AC7 (superclass template pinned).** Fork A resolved once — **WIRE** — and
  applied consistently across `Functor → Applicative → Monad → Traversable`
  (`§2`); `55 §7` pt 5 + `§2.2` updated to record the resolution.
- **AC8 (discriminators genuinely flip).** Every conformance soundness case
  flips accept→reject on the wrong witness, at the named law field, asserted as
  the specific variant (D4 seed): a wired `applicative` field that is a
  non-cartesian / law-breaking `Applicative`; a masked `Axiom` inhabiting
  `Bottom`; the ITree-bridge discriminator (no second divergent `bind`).
