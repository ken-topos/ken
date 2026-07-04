# CAT-2 effectful-constructor-class conformance — seed cases

Format: `../../README.md`. Second WP of the **catalog campaign**
(`docs/program/06-catalog-campaign.md`;
`docs/program/wp/CAT-2-applicative-monad-traversable.md`). CAT-2 lands the three
**effectful constructor classes** — `Applicative` / `Monad` / `Traversable` — in
the CAT-1 mould (a class is a record, each law an `Ω` value-equation **proved
not postulated** over an inductive carrier, zero `trusted_base()` delta,
kernel-untouched). This seed pins the **discriminating conformance** for D1–D3,
authored against the Architect's five fork rulings (`evt_p39qvcmh4gy2`) and
reconciled against spec-author's landed `55 §7`/D1/D2/D3 body at the merge gate.

The five rulings this seed encodes (Architect, `evt_p39qvcmh4gy2`):

- **Fork A — WIRE** (explicit superclass-dictionary field). `Applicative f`
  carries `functor : Functor f`; `Monad f` carries
  `applicative : Applicative f`; `Traversable f` carries `functor`+`foldable`.
  Applied **consistently up the whole chain** (AC7). Reverses CAT-1 `§2.2`'s
  value-class *restate* default; `55 §7` pt 5 pre-registered exactly this for
  the 3-deep chain. Rides existing record/projection machinery, **zero new
  capability** beyond CAT-1 `§6`. The win is **proof reuse**: a wired sub-dict
  supplies the already-built proofs; restate would duplicate them at every
  deeper instance.
- **Fork B — `bind`-primary.** The landed grafting `bind` (`declare_bind`,
  `effects/state.rs`, a single `Term::Elim` over `ITree e resp`):
  `bind (Ret a) f = f a` (**left-identity definitional**, ι on `Ret`),
  `bind (Vis e k) f = Vis e (λ r. bind (k r) f)`. `pure := Ret`; `join`/`map`
  derivable, **not** proliferated as primary fields. Field order
  `(m : f a) (k : a → f b)`.
- **Fork C — `Traversable.traverse`'s `Applicative g` = EXPLICIT (unbundled)
  dictionary parameter** (abstract `g` has no concrete head for implicit
  search). `traverse` classifies **`proc`** because its `f : a → g b` has an
  abstract codomain head `g`, which SURF-1's `classify_telescope` classifies
  `Unknown` → fail-closed → a fresh `RowVar` (SURF-1 `36 §1.5`). The RowVar
  **co-varies** with the dict: `g := Option`/`List` ⇒ RowVar → `∅`; `g := Eff e`
  ⇒ RowVar → `e` (`visits [e]`). **Same axis, two layers — not two mechanisms.**
- **Fork D — cartesian `Applicative List`.** Forced by Monad coherence: under
  the wired chain, `Monad List`'s `applicative` field must satisfy
  `ap = ap-from-bind` (`ap mf mx = bind mf (λ f. bind mx (λ x. pure (f x)))`);
  list `bind` is concatMap ⇒ the derived `ap` is cartesian. Ziplist `ap` is not
  `bind`-coherent and has no lawful `Monad`, so it cannot be the wired
  `Applicative List`. One canonical instance — ziplist **not** proliferated.
- **Fork E — ITree monad: ATTESTED CORRESPONDENCE** (not a surface `instance`).
  The carrier `ITree e resp` is a **parametric instance head** (free `e`,
  `resp`); a general `instance Monad (ITree e resp)` hits the still-open CAT-1
  `§6.1` parametric-instance-head gap and does **not** elaborate today. CAT-2
  reconciles by an **attested bridge**: `pure := Ret`, `bind :=` the landed
  `Term::Elim` bind; left-id definitional, right-id/assoc by ITree induction;
  **no second `bind` minted** (AC5).

## Grounding (content-verified against the landed targets)

- `55-lawful-functors.md` — the class-as-record / law-as-`Ω`-prop pattern, the
  **two-line proof grammar** (`§3.1`, induction + `cong`), the **`tt`-vs-`Refl`
  discrimination** (`§3.2`, the load-bearing K7 subtlety), the **pointwise law
  form** (`§5.2`, which states verbatim *"this is the form CAT-2's Monad laws
  inherit"*), the **higher-kinded elaborator extension** (`§6`, 4 pieces,
  kernel-untouched), the **reusable constructor-class template** (`§7`; **pt 5**
  pre-registers Fork A: restate-vs-wire deferred to CAT-2), and the
  **parametric-instance-head gap** (`§6.1`, still open — Fork E's upgrade path).
  Re-read HEAD↔`origin/main@ef791a3`: no drift (spec-author confirms
  byte-identical). *Note the citation fix:* `§6` is the **4-piece** extension,
  `§7` is the **5-piece template**; Fork A is `§7` pt 5, not `§6`.
- `36-effects.md §2.2` — the interaction-tree `bind` (grafting, via `elim_ITree`
  / a single `Term::Elim`): `bind (Ret a) k = k a`,
  `bind (Vis e f) k = Vis e (λ r. bind (f r) k)`; `§2.4` the denotation `⟦·⟧`
  sequences by `bind`. This is the landed effect denotation `Monad` reconciles
  with (AC5 / Fork B / Fork E), merged effect-composition `ed34129d`. Re-derived
  from the real code (`effects/state.rs` `declare_bind`), not memory.
- `36 §1.5` (SURF-1, landed `ef791a3`) — the row-variable surface (`[e]` /
  `[E | e]`) + the bidirectional `const`/`fn`/`proc` classification (`§1.6`):
  effect-polymorphic ⇒ `proc`; classification keys on the **declared** row
  `ρ_decl`. `traverse` rides this verbatim (Fork C / AC6). `classify_telescope`
  emits a fresh `RowVar` for an abstract codomain head (SURF-1 build seam).
- Carriers: `data List a = Nil | Cons a (List a)` /
  `data Option a = None | Some a` (`prelude.rs:189/191`) — real `Type0 → Type0`
  indformers. `list_append` landed (`packages/collections/collections.ken:52`,
  recurses on its **first** arg → left-unit definitional, right-unit/assoc
  inductive). `Semigroup`/`Monoid` + `List Nat`/`Bool` instances landed
  (`packages/lawful-functors/`). **NOT yet landed:**
  `map`/`bind`/`foldr`/`traverse` for `List`/`Option`, and the three CAT-2
  classes — so the instance-law cases are **red-until-built** (§below).
- `14 §3` — K4 `Ω`-motive elimination (the capability the inductive law proofs
  consume); `16 §1`/`§6` — `Ω` / truncation; `25 §3` — `trusted_base_delta`;
  `obs.rs:90` — funext-definitional (`Eq (Π..) f g ⇝` pointwise, `16 §2.2`).

## Scope — what this seed pins, and the static-vs-runtime split

Like the CAT-1 seed, the soundness content here is **not** a new mechanism: it
is the CAT-1 discipline (lawful ≡ zero-delta on an inductive carrier; a false
witness fails the law re-check at the named field; laws are `Ω`-clean pointwise
value equations) lifted to the effectful classes, **plus** the three
CAT-2-specific reconciliations the fork rulings pin: **superclass wiring +
proof-reuse** (Fork A / AC7), the **Monad ⇔ ITree attested bridge** (Fork B+E /
AC5), and the **effect-row-polymorphic `traverse`** (Fork C / AC6).

**Static vs runtime face.** These pin the **static face** — the verdict shapes,
the `tt`-vs-`Refl` endpoint map, the `proc` classification, the attested bridge,
the wired-sub-dict reuse. The **runtime/build face** — the actual
`packages/lawful-functors/` additions carrying real `elim_List`/`elim_Option`
proof terms and the wired sub-dicts, producer-grepped for
`declare_postulate`/holes on the law fields — is the **CAT-2 Language build**
(held for the GPT window), reconciled against the **landed** package there, not
asserted from this pre-package draft ([[soundness-ac-static-vs-runtime-face]],
[[lock-structural-output-against-landed-body]]).

**Red-until-built.** The three classes and the `List`/`Option`
`map`/`bind`/`traverse` ops are not on `main`; every instance-law case is
**red-until-built** against the CAT-2 build + SURF-1's `proc` checker (which
lands with the SURF-1 Language build) — the exact CAT-1-Functor posture. The
cases are build-forcing (not hand-fed): a green result requires the real
elaborator + instances, not a stubbed verdict.

**Reconcile-at-the-landed-body.** This seed is authored **in parallel with**
spec-author's `55 §7`/D1/D2/D3 transcription of the rulings. The exact field
spellings, the wired-field names, and the law statements are **reconciled
against the landed `55` body** at the merge gate — a ruling-thread reading is
not ground until the section lands
([[reconcile-binds-a-co-reviewers-plausible-reading-too]]). The
**proof-structure claims** (left-id definitional on `Ret`/`Some`, the per-law
`tt`-vs-`Refl` map, cartesian-forced-by-coherence, `proc`-via-RowVar) are
re-derived from the landed `bind`/`list_append`/`classify_telescope` and hold
**regardless** of package naming.

---

## AC7 — Fork A: the superclass chain WIRES, and the wired sub-dict is reused

`55 §7` pt 5's deferred template hole is resolved to **WIRE**: each class
carries its superclass as an explicit dictionary **field**, not a restatement of
the superclass ops+laws. The load-bearing consequence — and the discriminator —
is **proof reuse**: a wired `applicative` field supplies the already-built
`Applicative` proofs, so a deeper instance re-proves **only** its own new laws.

### stdlib/classes/monad-wires-applicative-subdict-reused (soundness)
- spec: Fork A ruling (`evt_p39qvcmh4gy2`), `55 §7` pt 5 (WIRE), `55 §5.1`/`§6`
  (record fields elaborate in the param context; `infer_proj` nested
  projection), `33 §5.3` (instance = record value), `13 §2` (Σ-Intro re-check)
- given: two `Monad List`-shaped instances, **identical** in `bind` and the
  three monad-law proofs, differing **only** in the wired `applicative` field:
  (a) the **already-built cartesian `Applicative List`** dict (Fork D), whose
  four applicative laws are real proofs; (b) a **law-breaking**
  `Applicative List` wired as the field — same field *shape* `Applicative List`,
  but its `ap` is non-cartesian (e.g. ziplist / a dropping `ap`), so it either
  fails its own applicative law or violates the Monad coherence
  `ap = ap-from-bind`
- expect: **the verdict flips on the wired sub-dict, with `bind` + the monad-law
  proofs held fixed.** (a) **accepts** — the wired `applicative` field
  type-checks as `Applicative List`, its four law proofs re-check, and the six
  Functor+Applicative proofs are **reused from the sub-dict, not re-proved** in
  `Monad List`; (b) **rejected — at the `applicative` field / the ap-coherence
  obligation**: the law-breaking sub-dict's own false applicative law fails
  conversion at that named law field (or, if it type-checks in isolation, the
  `ap = ap-from-bind` coherence obligation is a false `Equal` at a concrete
  `mf`/`mx`). Assert the **specific observable**: (a) elaborates + the
  `applicative` field's proofs re-check + `Monad List` re-proves only `bind`+3
  laws (grep: no duplicated Functor/Applicative proof terms in `Monad List`);
  (b) elaboration fails **at the wired `applicative` field or the coherence
  obligation** with a conversion/type-mismatch — **not** `is_err()`, not a
  missing-field/kind error ([[assert-specific-error-variant-not-is-err]])
- why: (soundness) AC7 — wiring is only sound if the wired field is
  **kernel-re-checked at the same trust level** (Architect: both the class
  record and the nested sub-dict are `declare_def` terms; wiring moves nothing
  out of kernel protection, so `coexist-when-trust-differs` does **not** bite).
  The discriminator proves the wire is **load-bearing**: a broken sub-dict
  cannot ride through as the superclass field. **Proof reuse is the DRY win**
  (subsume-don't-proliferate): restatement would duplicate the six
  Functor+Applicative proofs at `Monad List`, a divergence/soundness surface;
  wiring re-checks them **once** in the sub-dict and reuses. **Verdict-flip**,
  keyed solely on the wired sub-dict.

### stdlib/classes/superclass-chain-wired-consistently (property)
- spec: Fork A ruling (`evt_p39qvcmh4gy2`), `55 §7` pt 5 (WIRE, applied up the
  whole chain — AC7), `33 §5.4` (superclass field vs `where`-desugaring)
- given: the three class records as authored (field types elided; each
  superclass field is typed by its class):

  ```
  Applicative f { functor    : Functor f              ; pure ; ap ; <4 laws> }
  Monad f       { applicative : Applicative f          ; bind      ; <3 laws> }
  Traversable f { functor : Functor f ; foldable : Foldable f ; traverse ; <laws> }
  ```
- expect: each class carries its superclass(es) as an **explicit dict field**,
  and the chain is wired **consistently** (Applicative→Functor,
  Monad→Applicative, Traversable→Functor+Foldable) — **not** a per-class mix of
  wire-here/restate-there. Assert the **structural** observable: each superclass
  appears as a single dict field of the superclass type (not duplicated
  ops+laws), and the *same* choice (WIRE) is made at every link. **Not** a
  verdict-flip: this is a `(property)` case asserting the layering shape the
  ruling pins, the AC7 analog of CAT-1's `semigroup-assoc-shared-with-monoid`
- why: (property) AC7 — the template must be resolved **once** and applied
  consistently ([[correcting-scope-must-sweep-whole-doc]] dual: pin the choice
  at every link, not just where first forced). Reconciled against the landed
  `55 §7` pt 5 at the merge gate — the field **spellings**
  (`functor`/`applicative`/ `foldable`) are spec-author's to finalize; the
  **wire-consistently** invariant is normative. **Ergonomics flag (bounded,
  non-blocking):** explicit wiring makes use sites verbose (`d.functor.map`, not
  `map`); the implicit-coercion sugar Architect deferred (auto-`map` in an
  `Applicative` context) **would** need a new elaborator capability (resolution
  walking the superclass edge) = an `OQ-syntax`/`§6`-guardrail re-fork, **not**
  in CAT-2. A case asserting `map` resolves implicitly in an `Applicative` body
  is **red** (correctly — that sugar is not shipped).

---

## AC5 — Fork B + Fork E: Monad ⇔ ITree (bind-primary, attested bridge)

`bind`-primary (Fork B) fixes the reduction shapes; the ITree bridge (Fork E) is
an **attested correspondence**, not a surface instance. The load-bearing pin:
**one denotation, no second `bind`**.

### stdlib/classes/monad-left-id-definitional-bind-primary (soundness)
- spec: Fork B ruling (`evt_p39qvcmh4gy2`, `bind`-primary), `36 §2.2` (landed
  grafting `bind`, `bind (Ret a) f = f a`), `55 §5.2` (pointwise laws),
  `55 §3.2` (`tt`-vs-`Refl`), `51 §5` (laws PROVED)
- given: the `Monad` left-identity law `bind (pure a) k = k a` on two carriers
  with `bind`-primary + `pure := Ret`/`Some`, and a **bogus** `bind` that does
  not graft onto the leaf (e.g. `bad_bind m k = m` — well-typed, drops `k`): (a)
  `Option` — `bind (Some a) k` ι-reduces to `k a`; (b) `ITree` —
  `bind (Ret a) k` ι-reduces to `k a`
- expect: **left-id closes by the definitional ι with the honest `bind`, and
  flips under the bogus one.** honest arm: goal
  `Equal (f b) (bind (pure a) k) (k a)` reduces to `Equal (f b) (k a) (k a)` —
  `k a` is a **neutral** (`k` free) → an `Eq`-shaped goal closed by **`Refl`**
  (`55 §3.2`: neutral endpoint → `Refl`, **not** `tt`); bogus arm:
  `bad_bind (Some a) k = Some a`, goal `Equal (Option b) (Some a) (k a)` — a
  false `Equal` (`Some`-headed vs the neutral `k a`) → **conversion failure** at
  the left-id field. Assert the **observable**: honest `bind` closes left-id by
  `Refl` at both carriers; bogus `bind` rejects at the left-id field with a
  conversion/type-mismatch on `Some a`/`Ret a` vs `k a` — not `is_err()`
- why: (soundness) AC5 — `bind`-primary makes left-identity a **definitional**
  reduction (ι on the leaf), which is *why* the ITree bridge is a direct
  correspondence not a re-derivation. The endpoint is **`Refl` (neutral `k a`),
  not `tt`** — the CAT-1 trap
  [[tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases]] lifted: left-id's
  endpoints are a neutral application, so `Refl`; contrast the right-id/assoc
  constructor bases below (`tt`). **Verdict-flip**, keyed solely on `bind`.
  Two-arm net: a **masked** `left_unit = Axiom` on the bogus `bind` fails the
  delta gate and inhabits `Bottom` (`Axiom : Equal (Option b) (Some a) (k a)` is
  a false postulate, [[deceq-on-noncanonical-carrier-inhabits-bottom]]).

### stdlib/classes/monad-law-endpoints-tt-vs-refl (soundness)
- spec: `55 §3.2` (the `tt`-vs-`Refl` discrimination), Fork B ruling
  (`bind`-primary fixes the reductions), `14 §3` (`elim_List`/`elim_Option`),
  `51 §5`
- given: the three monad laws on `List`/`Option`, each closed at its base/branch
  by the **endpoint-determined** terminal (`tt` for constructor-headed, `Refl`
  for neutral), and a build that **swaps** the terminal (`Refl` where `tt` is
  required, or vice versa)
- expect: **the terminal is fixed per-law-per-branch and a swap fails.**
  - **Option** (`bind None k = None`, `bind (Some a) k = k a`): left-id →
    neutral `k a` → **`Refl`**; right-id `bind m pure = m` → case-split,
    `None`→`None` and `Some a`→`Some a`, both **constructor-headed → `Top` →
    `tt`**; assoc → `None` branch **`tt`**, `Some a` branch both sides neutral
    `bind (k a) h` → **`Refl`**.
  - **List** (`bind` = concatMap): left-id
    `bind (Cons a Nil) k = append (k a) Nil` — **NOT definitional**, closes via
    the inductive `right_unit` lemma at neutral `k a`; right-id/assoc bases are
    `Nil`-constructor → **`tt`**, steps lift the IH with `cong`.
  A swap (e.g. closing Option right-id by `Refl`, or Monad left-id by `tt`) is
  **rejected**: `Refl` on a `Top`-collapsed goal fails (goal not `Eq`-shaped),
  `tt` on a neutral `Eq` goal fails (goal not `Top`). Assert the **observable**:
  each terminal matches its endpoint reduction; a swapped terminal fails to
  check
- why: (soundness) AC5/AC3 — the multi-dimensional-guard rule
  ([[tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases]]): the terminal is
  **not uniform**, it is a two-way net keyed on what the endpoints reduce to
  (constructor → `Top` → `tt`; neutral → `Eq` → `Refl`). A build that hard-codes
  "base → `Refl`" is wrong for the constructor-headed bases (right-id/assoc) and
  a build that hard-codes "base → `tt`" is wrong for the neutral bases (left-id,
  Option-assoc `Some`). This pins the endpoint map Fork B's `bind`-primary
  reductions realize. **The List left-id subtlety** (not definitional — needs
  `right_unit`) is the sharpest sub-case: a build assuming it closes by `Refl`
  fails, because `append (k a) Nil` is stuck on the neutral `k a`.

### stdlib/classes/itree-bridge-attested-no-second-bind (soundness)
- spec: Fork E ruling (`evt_p39qvcmh4gy2`, attested correspondence), `36 §2.2`
  (landed grafting `bind`), `55 §7` (template), AC5 (`§2` pin 6 — one
  denotation), effect-composition `ed34129d`
- given: the Monad⇔ITree reconciliation as authored — `pure := Ret`, `bind :=`
  the landed `Term::Elim` grafting bind (`effects/state.rs`), the three laws
  attested (left-id definitional, right-id/assoc by ITree induction) —
  contrasted with a build that **mints a second `bind'`** for the `Monad ITree`
  bridge that **disagrees** with the landed `bind` (e.g. a `bind'` that does not
  graft onto `Vis` continuations, or wraps rather than substitutes at `Ret`)
- expect: **the attested bridge uses the one landed `bind`; a divergent second
  `bind'` is caught.** (a) the bridge's `bind` field **is** the landed
  `declare_bind` term (grep: same `GlobalId` / the same `Term::Elim`, not a
  fresh definition) — one denotation; (b) a second `bind'` disagreeing with the
  landed `bind` produces an **observably different tree** at a concrete input —
  e.g. at `bind (Vis e k) f`, the landed `bind` yields
  `Vis e (λ r. bind (k r) f)` while a non-grafting `bind'` yields a different
  `ITree` — so the "one denotation" attestation is **false** and flagged (the
  two `bind`s are not convertible on a concrete `Vis` tree). Assert the
  **structural observable**: the bridge's `bind` field resolves to the
  **landed** `bind` (same term/`GlobalId`), and a minted divergent `bind'` fails
  the concrete-tree agreement — **not** a message string
- why: (soundness) AC5, the **load-bearing reconciliation** — the effect
  system's denotation **is** a monad, so `Monad` must not mint a second,
  divergent `bind` (frame `§2` pin 6, `§6` guardrail). The net is
  **structural**: the bridge reuses the landed `bind` (one `GlobalId`), and any
  second `bind'` is discriminated by **tree disagreement at a concrete
  `Vis`/`Ret`** ([[differential-verify-which-mechanism-is-the-net]]: the
  discriminator is concrete-tree agreement, not "both type-check as `bind`").
  This is the effect-composition verdict-flip discipline (AC8 models on it): a
  divergent denotation is a wrong *value* (a different tree), caught at a
  concrete input.

### stdlib/classes/itree-monad-general-instance-gated (property)
- spec: Fork E ruling (`evt_p39qvcmh4gy2`), `55 §6.1` (the parametric-instance-
  head gap, **still open** with Steward), `33 §5.3` (instance head elaboration)
- given: two forms of the ITree monad reconciliation — (a) the **attested
  correspondence** CAT-2 ships (the bridge, no surface `instance`); (b) a
  general surface `instance Monad (ITree e resp)` with a **parametric head**
  (free `e`, `resp`)
- expect: (a) the attested bridge is the CAT-2 deliverable (documented, uses the
  landed `bind`); (b) the general surface `instance Monad (ITree e resp)` **does
  NOT elaborate today** — `elab_instance_decl` elaborates the head via
  `elab_type` in an empty context, so the free head vars `e`/`resp` resolve to
  `UnresolvedCon` (the `55 §6.1` gap). Assert the **observable**: the general
  parametric-head instance fails with the **`§6.1` parametric-head resolution
  error** (`UnresolvedCon` on the free head vars), **not** a law-check failure —
  it is a **generality-upgrade gated on the open `§6.1` fork**, not a CAT-2 bug.
  (A *closed*-effect `instance Monad (ITree E0)` **would** elaborate — closed
  head — but it is not the general bridge, so not the deliverable.)
- why: (property) AC5 — pins **why** the bridge is *attested* not *instanced*:
  the general instance is blocked by an **already-open** Steward fork (`§6.1`),
  not a new re-fork ([[named-floor-must-be-grepped-not-assumed]]: the blocker is
  the real landed `elab_instance_decl` empty-context head elaboration,
  grep-verified, not assumed). Prevents a build from either (i) claiming the
  general instance elaborates today (it does not) or (ii) hand-rolling a
  divergent closed-form to dodge the gate. Red-until the `§6.1` path lands; the
  **attested bridge** is green-until-built on the landed `bind`.

---

## AC4 — Applicative laws + Fork D: cartesian `List`, false-witness rejected

### stdlib/classes/applicative-list-cartesian-forced (soundness)
- spec: Fork D ruling (`evt_p39qvcmh4gy2`, cartesian), Fork A (wired chain →
  coherence obligation), `55 §5.2` (pointwise laws), `51 §5` (laws PROVED)
- given: two candidate `Applicative List` instances offered as the `applicative`
  field wired into `Monad List`: (a) **cartesian** `ap`
  (`ap fs xs = concatMap (λ f. map f xs) fs`); (b) **ziplist** `ap` (`zipWith`
  application) — both are lawful `Applicative`s **in isolation**, but only one
  is `bind`-coherent
- expect: **the verdict flips on Monad coherence, not on the applicative laws
  alone.** (a) cartesian **accepts** — it satisfies `ap = ap-from-bind`
  (`ap mf mx = bind mf (λ f. bind mx (λ x. pure (f x)))`) because list `bind` is
  concatMap; (b) ziplist **rejected as the wired field** — the coherence
  `Equal (List c) (ap_zip mf mx) (ap-from-bind mf mx)` is a **false** equation
  at a concrete `mf`/`mx` (e.g. `mf = [f,g]`, `mx = [x,y]`: cartesian yields
  `[f x, f y, g x, g y]`, ziplist yields `[f x, g y]`) → **conversion failure**
  at the coherence obligation. Assert the **observable**: cartesian satisfies
  the coherence equation at the concrete witness; ziplist fails it — a
  constructor-level list mismatch, **not** a missing-field error. **One
  canonical instance:** ziplist is **not** proliferated as a second
  `Applicative List`
- why: (soundness) AC4/Fork D — Monad coherence *forces* cartesian: the wired
  `applicative` field must be the one `ap` the `Monad`'s `bind` induces, else
  the `ap = ap-from-bind` obligation is false. Ziplist is a lawful `Applicative`
  but **not** a lawful `Monad`'s applicative, so it cannot be wired here
  (subsume-don't-proliferate: if ever wanted, ziplist rides a `newtype`,
  deferred, not CAT-2). The discriminator is the **concrete-list flip** —
  cartesian vs ziplist give **different values** at `[f,g] <*> [x,y]`, the
  effect-composition verdict-flip discipline made concrete.

### stdlib/classes/applicative-law-false-witness-rejected (soundness)
- spec: `55 §5.2` (pointwise applicative laws), `51 §5` (laws PROVED), `13 §2`
  (Σ-Intro re-check); the four laws
  identity/homomorphism/interchange/composition
- given: two `Applicative Option`-shaped instances identical except in
  `pure`/`ap` with the **same** proof term attempted for a named applicative
  law: (a) the canonical `pure a = Some a` / `ap`; (b) a **bogus**
  `pure' a = None` (well-typed at `pure : a → f a`), whose **homomorphism** law
  `ap (pure f) (pure x) = pure (f x)` is offered at a concrete `f`/`x`
- expect: **the verdict flips on the witness at the named law field.** (a)
  **accepts** — the homomorphism goal
  `Equal (Option b) (ap (Some f) (Some x)) (Some (f x))` reduces to
  `Equal (Option b) (Some (f x)) (Some (f x))`, constructor-headed both sides →
  closed (`tt`/`Refl` per the endpoint); (b) **rejected — conversion failure at
  the homomorphism field**: `ap (pure' f) (pure' x) = ap None None = None`, so
  the goal is `Equal (Option b) None (Some (f x))`, a **false** equation (`None`
  vs `Some`-headed). Assert the **specific observable**: (a) law re-checks; (b)
  elaboration fails **at the homomorphism law field** with a `None`-vs-`Some`
  constructor clash — not `is_err()`, not a missing-field/kind error
- why: (soundness) AC4 for `Applicative` — the false-witness flip lifted from
  `Monoid`/`Functor` to the applicative laws (the exact
  `monoid-unit-law-false-witness-rejected` discipline). **Two-arm net**: honest
  proof fails **conversion** (this case); a masked `homomorphism = Axiom` fails
  the **delta gate** and inhabits `Bottom`
  (`Axiom : Equal (Option b) None (Some (f x))` is a false postulate).
  **Verdict-flip**, keyed solely on `pure`/`ap`.

---

## AC6 — Fork C: `traverse` is effect-row-polymorphic (SURF-1 verbatim)

The signature

```
traverse : (g : Type→Type) → Applicative g → (a b : Type)
         → (a → g b) → f a → g (f b)
```

classifies **`proc`** because its `f : a → g b` has an **abstract codomain head
`g`**; SURF-1's `classify_telescope` classifies that `Unknown` → fail-closed → a
fresh `RowVar`. The RowVar **co-varies** with the dict. The `Applicative g`
constraint is an **explicit (unbundled) dict parameter**.

### stdlib/classes/traverse-classifies-proc-via-rowvar (soundness)
- spec: Fork C ruling (`evt_p39qvcmh4gy2`), SURF-1 `36 §1.5` (row variable) +
  `§1.6.1` (effect-polymorphic ⇒ `proc`, classify on `ρ_decl`), the landed
  `classify_telescope` (`extract.rs`); ties to SURF-1 PK8
  `poly-def-is-proc-not-fn`
- given: the `traverse` definition with its abstract codomain head `g`, offered
  under two keywords with the **identical** signature: (a) `proc traverse …`;
  (b) `fn traverse …`
- expect: **the verdict flips on the keyword** (the SURF-1 PK8 shape lifted to
  `traverse`). (a) `proc` **accepts** — the abstract `g` makes
  `classify_telescope` emit a fresh `RowVar`; the declared row contains a row
  variable, so `proc` is honest (`§1.6.1`); (b) `fn` **rejects** — `fn` claims
  the empty closed row `∅`, but the abstract-`g` row variable is **not** `∅`, so
  the bidirectional check fails (a row variable is not `∅`). Assert the
  **observable**: `proc traverse` classifies + elaborates; `fn traverse` is
  rejected by the SURF-1 purity check (the declared `∅` contradicts the inferred
  row variable), the **specific keyword-mismatch** — not `is_err()`
- why: (soundness) AC6 — `traverse` is **the first library definition
  polymorphic over an arbitrary applicative `g`** (frame `§1`), and it rides
  SURF-1's row variable **verbatim**: it is `proc` **because** `g` is abstract,
  the same axis at the type-classification layer that SURF-1's `[e]` is at the
  effect layer (Architect: *"same axis, two layers — not two mechanisms"*). This
  is SURF-1's PK8 `poly-def-is-proc-not-fn` realized by `traverse` — the
  effect-polymorphic case SURF-1 sequenced first *for* this. **Verdict-flip**,
  keyed on the keyword. Red-until the SURF-1 `proc` checker + the CAT-2
  `traverse` land.

### stdlib/classes/traverse-rowvar-covaries-with-applicative-g (soundness)
- spec: Fork C ruling (`evt_p39qvcmh4gy2`, RowVar co-varies), SURF-1 `36 §1.5.1`
  (the `g := Eff e` exemplar) + PK8 `proc-stays-proc-at-pure-instantiation`,
  `36 §2.2` (the ITree denotation reifies the same effects as `g`-data)
- given: the **one** `traverse` definition instantiated at two applicatives: (a)
  `g := Option` (a pure applicative) ⇒ the RowVar substitutes to `∅`; (b)
  `g := Eff e` (an effect monad) ⇒ the RowVar substitutes to `e`
- expect: **the same definition's row variable tracks `g` under instantiation.**
  (a) at `g := Option`, `apply_subst` sends the RowVar → `∅`, so the *call* runs
  pure (`traverse … : Option (List b)`, no `visits`), yet the *definition* stays
  `proc` (SURF-1 PK8 `proc-stays-proc-at-pure-instantiation` — classify on
  `ρ_decl`, not the instantiated `ρ_inf`); (b) at `g := Eff e`, the RowVar →
  `e`, surfacing as `visits [e]` (SURF-1 `§1.5.1`'s exemplar). Assert the
  **observable**: the pure-`g` instantiation type-checks as an effect-free
  result while the definition's keyword stays `proc`; the `Eff e` instantiation
  carries `[e]`. **No double-count**: the surface row is the conservative
  signature-level face; the ITree denotation reifies the same effects as
  `g`-data; they **agree** at `g := Eff e` and **both collapse** at
  `g := Option`
- why: (soundness/oracle) AC6 — pins the **co-variation** that makes `traverse`
  one polymorphic definition rather than two (a pure `traverse` + an effectful
  `traverse`). It is the round-trip crux SURF-1's PK8 pins, realized by the
  first real consumer: instantiate the polymorphic `proc` at a pure applicative
  and it runs pure without changing its classification. The `[e]`/`[E | e]`
  **surface glyph** stays `(oracle)` (SURF-1 `§1.5.1`, still `OQ-syntax`); the
  **co-variation construct** (RowVar tracks `g`, collapses at pure `g`) is
  normative. Red-until the SURF-1 fixpoint-lift seam + CAT-2 `traverse` land.

### stdlib/classes/traverse-applicative-g-explicit-dict-not-implicit (property)
- spec: Fork C ruling (`evt_p39qvcmh4gy2`, explicit unbundled dict), `55 §5.1`
  (`infer_proj` projects off an opaque bound-var dict), `33 §5.4` (implicit
  `where` needs a concrete head for instance search)
- given: two forms of the `Applicative g` constraint on `traverse`: (a)
  **explicit** unbundled dict parameter `(ap_g : Applicative g)`, projected as
  `ap_g.ap`/`ap_g.pure`; (b) **implicit** `where Applicative g`
- expect: (a) the **explicit** form **elaborates today** — `g` is an abstract
  bound var, and `infer_proj` projects `ap_g.ap`/`ap_g.pure` off the opaque
  bound-var dict fine (its own comment states it supports an opaque bound-var
  base); (b) the **implicit** `where Applicative g` form does **NOT** elaborate
  — an abstract `g` has **no concrete head** for implicit instance search, so
  resolution has nothing to match. Assert the **observable**: explicit-dict
  `traverse` elaborates; implicit-`where` `traverse` fails at instance
  resolution (no concrete head for `g`) — a **class-resolution** failure, not a
  law failure
- why: (property) AC6 — the explicit-vs-implicit dict fact
  ([[class-dict-explicit-vs-implicit-abstract-tyvar]]): a class constraint over
  an **abstract** type variable must be an **explicit** (unbundled) dict because
  implicit search needs a concrete head; the `.field` projection is
  head-agnostic (a Σ-record projected by `type_id`), so the explicit form is
  **buildable today** with zero new mechanism. This is why Fork C rides existing
  machinery (explicit dict + SURF-1 RowVar, both landed) — **no re-fork**.
  Reconciled against the landed `55` D3 body; the mechanism home is
  `../surface/classes/` (referenced, not re-pinned).

### stdlib/classes/traverse-coherence-false-witness-rejected (soundness)
- spec: Fork C ruling, `55 §5.2` (pointwise), `51 §5` (laws PROVED), `14 §3`
  (`elim_List`); the traversal coherence laws (naturality, identity,
  composition)
- given: two `Traversable List`-shaped instances differing only in `traverse`,
  with the **identity** coherence law (`traverse (Identity) pure ≡ pure` under
  the identity applicative) offered at a concrete carrier: (a) the canonical
  `traverse`; (b) a **bogus** `traverse` that **drops** an element (skips the
  head), so the identity law is false at a concrete `xs`
- expect: **verdict flips at a concrete carrier** — (a) the identity law closes
  by `elim_List` induction (base `Nil`→`Nil` constructor → `tt`; step lifts the
  IH under `Cons` with `cong` through the applicative); (b) the element-dropping
  `traverse`'s identity law is a **false** `Equal` at (e.g.)
  `xs = Cons True (Cons False Nil)` (LHS drops `True`, RHS preserves) →
  **conversion failure**, right-reason (not a missing-field/kind error). Assert
  the concrete-carrier conversion-failure observable at the **named coherence
  field**
- why: (soundness) AC6 — the AC4 false-witness flip for `Traversable`: a
  `traverse` that drops/reorders elements is the traversal family's "wrong
  witness," caught by the identity coherence law at a concrete carrier, the same
  discipline as the Functor id-law and Monad left-id flips
  ([[two-arm-producer-needs-a-case-per-arm]]: masked-postulate arm → delta gate
  + `Bottom`). The **exact set** of stated-vs-derivable coherence laws
  (naturality, identity, composition) is spec-author's `55` D3 call; the
  **element-preservation discriminator** is normative regardless.

---

## AC2 / AC3 — laws are `Ω`-clean, pointwise, one canonical field

### stdlib/classes/cat2-laws-omega-clean-pointwise-one-field (soundness)
- spec: `51 §3` (law-field sorts — every law `Ω`), `55 §5.2` (pointwise,
  funext-definitional, one field), `obs.rs:90` (`Eq (Π..) f g ⇝` pointwise),
  `16 §1`/`§6` (`Ω`/truncation), `13 §4` (`sort_sigma`)
- given: the Applicative/Monad/Traversable law fields as authored (each a `Π`
  into `Equal (f _) u v`), contrasted with (i) a **proof-relevant**
  `Type`-valued "law" and (ii) a **second, point-free** restatement of a law
  carried as a separate field
- expect: (a) every law field is `Equal (f _) u v : Ω` — proof-irrelevant, **no
  truncation** (direct value equations, the `51 §3` truncation catch does
  **not** fire); the record is `Type`-sorted because of its **op** fields
  (`pure`/`ap`/`bind`/`traverse`), never because a law leaked to `Type`; (b) a
  `Type`-valued "law" is **not** an admissible law field; (c) a point-free
  restatement (`Equal (f a → f b) …`) is **definitionally equal** to the
  pointwise field (funext, `obs.rs:90`), so a **second** field carrying it is
  **redundant proliferation** — one canonical pointwise field per law. Assert:
  each law is a single pointwise `Ω`-field; a duplicated point-free field is
  flagged as reflect-don't-extend proliferation, not a distinct obligation
- why: (soundness) AC2+AC3 — the `Ω`-cleanliness + no-proliferation pins lifted
  to the effectful classes, the `cat2` analog of CAT-1's
  `monoid-laws-omega-clean-carrier-equality` +
  `functor-one-canonical-pointwise-field`. `55 §5.2` states verbatim the Monad
  laws **inherit** the pointwise form, so each discharges by **direct
  induction** on the carrier (no funext strip) and a point-free second field is
  content-free duplication. Guards a build that adds the categorical/point-free
  law "too." The Applicative/Monad/Traversable laws are **value equations**
  (`Equal (f _) u v`), so — unlike a disjunction needing the `Bool`-equation
  trick — they are **already** proof-irrelevant, no truncation
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]]).

---

## AC1 — kernel-untouched, extension-reused (build-verify, home in surface)

Fork A wiring + Fork C explicit dicts + Fork E's attested bridge all ride
**CAT-1 `§6`'s 4-piece higher-kinded extension + existing record/projection
machinery** — **zero new elaborator capability** (Architect,
`evt_p39qvcmh4gy2`). No new kernel `Term`/`Decl`; no `trusted_base()` delta. The
**capability-boundary** property (no fifth piece; nested projection
`d.applicative.functor.map` rides `infer_proj`; the wired field elaborates like
any record field) is a **class-mechanism** property whose home is
**`../surface/classes/seed-classes.md`** (`33 §5`), not this lawful-content seed
(one home per property). Noted here as the **build-verify dependency** the
instance-law cases sit on: they are **red-until-built** against the CAT-2 build,
and the CV producer-grep at the build gate confirms AC1
(`git diff origin/main -- crates/ken-kernel/` empty, no new `Term`/`Decl`, zero
`trusted_base()` delta) on the built diff. Two pointers to **existing** open
items — `55 §6.1` parametric-head (Fork E's upgrade) and an `OQ-syntax`
implicit-superclass-coercion follow-on (Fork A's ergonomics) — are **not** new
re-forks.

---

## Coverage map

- **AC7** (Fork A WIRE): `monad-wires-applicative-subdict-reused` (wired
  sub-dict is load-bearing + proofs reused, verdict-flip),
  `superclass-chain-wired-consistently` (property: wire consistently up the
  chain; implicit-coercion sugar deferred).
- **AC5** (Fork B + Fork E, Monad ⇔ ITree):
  `monad-left-id-definitional-bind-primary` (left-id ι on `Ret`/`Some`, `Refl`
  at neutral), `monad-law-endpoints-tt-vs-refl` (the per-law-per-carrier
  endpoint map; List left-id NOT definitional),
  `itree-bridge-attested-no-second-bind` (one denotation, structural reuse of
  the landed `bind`, divergent `bind'` caught at a concrete tree),
  `itree-monad-general-instance-gated` (property: general surface instance
  blocked by the open `§6.1` gap).
- **AC4** (Applicative + Fork D): `applicative-list-cartesian-forced` (cartesian
  forced by `ap = ap-from-bind`; ziplist fails coherence at a concrete list, not
  proliferated), `applicative-law-false-witness-rejected` (bogus `pure`/`ap` →
  false homomorphism law, verdict-flip).
- **AC6** (Fork C, Traversable ⇔ SURF-1): `traverse-classifies-proc-via-rowvar`
  (`proc` accepts / `fn` rejects — PK8 lifted),
  `traverse-rowvar-covaries-with-applicative-g` (RowVar tracks `g`; collapses at
  pure `g`, `visits [e]` at `Eff e`),
  `traverse-applicative-g-explicit-dict-not-implicit` (property: explicit dict
  buildable, implicit not), `traverse-coherence-false-witness-rejected`
  (element-dropping `traverse` → false identity law at a concrete carrier).
- **AC2/AC3** (Ω-clean, pointwise): `cat2-laws-omega-clean-pointwise-one-field`
  (laws are `Equal (f _) u v : Ω`, proof-irrelevant, no truncation; one
  canonical pointwise field per law, funext-definitional ⇒ no second point-free
  field).
- **AC1** (kernel-untouched): build-verify dependency; mechanism home in
  `../surface/classes/`; instance-law cases red-until-built; producer-grep at
  the build gate (`ken-kernel` diff empty, zero `trusted_base()` delta).
- **AC8** (discriminators flip): satisfied by construction — every soundness
  case is a witness-flip accept↔reject at the **named** law/coherence field,
  asserted as the specific error variant (not `is_err()`), two-arm net (honest →
  conversion / masked `Axiom` → delta gate + `Bottom`).

## Cross-case consistency sweep

- **`bind`-primary fixes both the reductions and the endpoint map.**
  `monad-left-id-definitional-bind-primary` (left-id ι on the leaf, `Refl` at
  neutral `k a`) and `monad-law-endpoints-tt-vs-refl` (right-id/assoc bases are
  constructor-headed → `tt`) agree and compose: Fork B's `bind (Ret a) f = f a`
  makes left-id definitional and the constructor bases `tt`, the neutral bases
  `Refl`. A case closing Monad left-id by `tt`, or Option right-id by `Refl`,
  contradicts this pair (the endpoint discrimination, `55 §3.2`).
- **One denotation across the ITree cases.**
  `itree-bridge-attested-no-second-bind` (the bridge reuses the landed `bind`,
  one `GlobalId`) and `itree-monad-general-instance-gated` (the general instance
  is gated, not minted) agree: CAT-2 ships **no** second `bind` and **no**
  divergent ITree monad — the bridge is attested against the landed grafting
  `bind`, the general instance waits on `§6.1`. A case minting a fresh `bind`
  for the bridge, or claiming the general parametric-head instance elaborates
  today, contradicts this pair (frame `§2` pin 6 / AC5).
- **Cartesian is the wired `Applicative List`; ziplist is not proliferated.**
  `applicative-list-cartesian-forced` (cartesian satisfies `ap = ap-from-bind`,
  ziplist fails it) and `monad-wires-applicative-subdict-reused` (a law-breaking
  wired `applicative` rejects) agree: the `Monad List`'s wired `applicative`
  field **must** be cartesian, so a ziplist `Applicative List` cannot be the
  wired field. A case wiring ziplist into `Monad List`, or proliferating ziplist
  as a second canonical `Applicative List`, contradicts Fork D + Fork A.
- **`traverse` is `proc` because `g` is abstract, and the RowVar tracks `g`.**
  `traverse-classifies-proc-via-rowvar` (`proc` accepts / `fn` rejects) and
  `traverse-rowvar-covaries-with-applicative-g` (RowVar → `∅` at pure `g`, → `e`
  at `Eff e`) agree with SURF-1's PK8 (`proc-stays-proc-at-pure-instantiation`):
  the definition is `proc` on `ρ_decl` regardless of the instantiation, and the
  instantiated row tracks `g`. A case classifying `traverse` as `fn`, or
  expecting the pure-`g` instantiation to change the definition's keyword,
  contradicts this pair + SURF-1 `§1.6.1`.
- **The false-witness discipline is one rule across all classes.**
  `monad-left-id-definitional-bind-primary`,
  `applicative-law-false-witness-rejected`, and
  `traverse-coherence-false-witness-rejected` agree with CAT-1's
  `monoid-unit-law-false-witness-rejected` /
  `functor-id-law-false-map-rejected`: a wrong operation witness (a non-grafting
  `bind`, a bogus `pure`, an element-dropping `traverse`) makes a law a **false
  `Equal`**, caught at a **concrete carrier** by a conversion failure on the
  honest proof and by the delta gate (+ `Bottom`) on a masked postulate. A case
  rejecting a false-witness instance for an *unrelated* reason (missing field /
  kind error) contradicts the right-reason discipline.
- **Every law field is `Ω`; one canonical pointwise field.**
  `cat2-laws-omega-clean-pointwise-one-field` agrees with the `51 §3`
  law-field-sort pin and `55 §5.2`: the record is `Type`-sorted because of its
  op fields, never because a law leaked to `Type`; the pointwise field **is**
  the prover's goal (no funext strip); a second point-free field is
  proliferation. A case with a `Type`-valued "law" or dual point-free/pointwise
  fields contradicts this pin.

## Subsumed / not-duplicated (one home per property)

- **The `trusted_base()` / zero-delta mechanism** is **ES1/Sec4's**
  (`../surface/taxonomy/minimality.md`, `../../security/trust-model/`), its
  law-side reading pinned once in **`seed-lawful-classes.md`**. This seed
  **references** the provenance flip (a postulated law → non-empty delta on an
  inductive carrier), it does not re-pin the delta computation.
- **The K4 `Ω`-motive elimination** capability (`14 §3`) is
  **`../../kernel/inductive/seed-k4-omega-motive-elim.md`'s**. The
  Applicative/Monad/Traversable inductions consume it (`elim_List`/`elim_Option`
  into an `Equal`-motive); this seed does not re-pin the elimination rule.
- **The funext-definitional reduction** (`obs.rs:90`, `16 §2.2`) is the
  **observational kernel's** (`../../kernel/observational/`). The pointwise
  cases **consume** it as a ground fact; they do not re-pin it.
- **The class mechanism** — record elaboration, `infer_proj` nested projection,
  `instance_search`, the higher-kinded param binder, **and the AC1 capability
  boundary** (no fifth piece; the wired superclass field rides `infer_proj`) —
  is **`33 §5`'s / `../surface/classes/`'s**. This seed pins the **lawful
  content** (the law proofs are real and, when the witness is wrong, fail for
  the right reason; the wired sub-dict is load-bearing), not the resolution
  machinery.
- **SURF-1's row-variable surface + the `const`/`fn`/`proc` classifier**
  (`36 §1.5`/`§1.6`) is **`../surface/declarations/seed-purity-keywords.md`'s**.
  This seed's `traverse` cases **consume** the `proc` classification + RowVar
  co-variation (PK8) as landed facts; the `[e]`/`[E | e]` **surface glyph**
  stays `(oracle)` (SURF-1 `§1.5.1`, still `OQ-syntax`), and the classifier
  mechanism is not re-pinned here.
- **The landed grafting `bind`** (`36 §2.2`, `effects/state.rs` `declare_bind`)
  is the **effect system's** (`../runtime/effects/`, `../surface/effects/`). The
  ITree-bridge cases **attest** the `Monad` fields against it; they do not
  re-pin the `bind` reduction.
