# surface-transport — hypothesis-driven rewriting reachable from `.ken` (transport / `J` / `subst` at the surface)

**Steward frame → Team Language (build), with a spec-enclave elaboration step
first. LANE: surface term-former (equality/transport machinery).** Architect's
approach-review — the central soundness gate — is **DONE, APPROVE**
(`evt_1g5bx52mdv5g6`, grounded against landed `term.rs`/`obs.rs`/`check.rs`/
`conv.rs`/`elab.rs`/`prelude.rs`): the ruling is **elaborator-only, single Language
lane, ZERO kernel change, ZERO `trusted_base` delta** — see the ratified
decomposition below. Owner: **Language**. Gate: Architect approach-review (DONE) →
**spec-leader elaborates ONE `/spec/30-surface` typing rule** → merge to `main`
(Integrator) → Language builds → **Architect soundness** on the candidate +
**Language QA** + **CI** + **CV reviews `/spec`**. Findings → **Steward**.

Base: `origin/main`. Branch (pre-staged by Steward): **`wp/surface-transport`**.

## ★ RATIFIED DECOMPOSITION (Architect approach-review `evt_1g5bx52mdv5g6`) — build to THIS

**VERDICT: sound to build, ELABORATOR-ONLY (single Language lane), ZERO kernel
change, ZERO `trusted_base` delta. ONE surface former: `J`. `cong`/`subst`/
`transport`/`cast`/`sym`/`trans` are DERIVED `.ken` library, not formers. The
positional-`K6` `conv_struct` arm is NOT needed — ruled out of scope (not unsound,
simply not invoked). Cross-wise `K6` stays a hard NO.** The four approach-review
questions below (in "Objective") are now ANSWERED — do not relitigate them:

**(1) Former + kernel target + typing rule.** ONE former **`J`** (the identity
eliminator), **infer-mode**, mirroring the existing checked-sugar idiom
(`absurd`/`Refl`/`tt` are `RApp`/`RCon` special forms in `elab.rs::check` that
construct a kernel term directly). Surface `J <motive> <base> <eq>` elaborates to
**`Term::J(motive, base, eq)`** (which the kernel itself derives to `Term::Cast`
via `obs.rs::j_nonrefl`); both already exist and are already in `trusted_base()`.
**Typing rule (verbatim from `check.rs::infer_j` — this is the spec pin):**
`eq : Eq A a b`; `motive : (b':A) → Eq A a b' → sort` (first Π-domain must convert
to `A`; the **codomain sort is UNCONSTRAINED** — accepts `Type ℓ` AND `Ω`, which is
what lets `cong` target `Ω`); `base : motive a (refl a)`; result `motive b eq`.
`Cast` rule: `e : Eq Type A B`, `t : A` ⊢ `cast A B e t : B` — it does **not**
require `A ≡ B` (that is the whole point of transport).

**(2) Elaborator-only — the soundness pivot, grounded.** Explicit proof-carrying
transport **type-checks with zero conv change**. For the real Map shape (prove
`G'[leq k k']` under `q : Equal Bool (leq k k') True`):
`proof = J (λ x _. G'[x]) (pf_true : G'[True]) (sym q)`. `infer_j` returns
`motive b eq = (λ x _. G'[x]) (leq k k') (sym q)`; checked against goal
`G'[leq k k']`, conv β-reduces the redex and compares **by identity**. The only
conv work is β-reduction + checking `pf_true : G'[True]` (where `if True …` fires
by ordinary `Bool` ι). **No `Eq`/`Eq` cross-congruence, no positional-`K6`, is ever
invoked** — transport discharges the goal by the `J`/`Cast` *typing rule* (the
equation obligation lands on the user's `q`), never by asking conv to identify the
differing endpoints. **Single lane (Language). Positional-`K6` OUT of scope (not
needed; bundling it = scope creep into an orthogonal kernel-completeness question).
Cross-wise `K6` stays the hard NO.**

**(3) `cong` is DERIVED, not a former.** `J` alone is the complete primitive;
ship the derived combinators as a **non-recursive `.ken` prelude module**
(SCT-trivial; needs only `J` + the already-surface `Equal`/`Refl`/λ). Architect
verified each type-checks by `infer_j`:
- `subst P p pa := J (λ b' _. P b') pa p`
- `cong P p := J (λ b' _. Equal Type (P a) (P b')) (Refl (P a)) p` (motive → `Ω`, legal per the unconstrained codomain sort)
- `cast/coe A B e t := J (λ X _. X) t e` (raw type-transport derives too — no separate former)
- `sym`, `trans` likewise.

**(4) Spec footprint: ONE `/spec/30-surface` addition; NO `/spec/1x-kernel`
note.** Add the surface `J`-former syntax + typing rule (pointing at the
already-specified kernel rules in `15-identity`/`16-observational`). Because there
is **no conv change**, there is **no conv-completeness note**. The derived
combinators are library (a thin `50-stdlib` prelude listing). **@spec-leader: the
elaboration is this single surface-former typing rule + the prelude listing —
small.**

**NAMED BOUNDARY (honesty — the scope line):** the motive `λx. G'[x]` is
**user-written** (the user abstracts the scrutinee occurrence explicitly — the
Agda-`subst` posture). Motive-inference / with-abstraction / a `rewrite` auto-motive
spelling is a **SEPARATE, non-soundness ergonomic sugar, explicitly OUT of this
WP** (its own later Language surface-syntax WP). AC3's repro uses a **hand-written
motive**. (Sibling note from the Gap B ruling: the `J` motive should be elaborated
**bidirectionally from `eq`'s type**, not naive `infer` of an unannotated lambda —
so Gap A needs no `\(h:Ty).` syntax either.)

## The capability gap (grounded — verify against landed code, do not trust this line)

A `.ken` proof today **cannot rewrite a goal using a propositional equality
hypothesis.** Given `p : Equal A a b` (e.g. `IsTrue (leq k k') = Equal Bool (leq
k k') True`), there is **no surface term-former** that transports along `p` to
turn a goal mentioning `a` into one mentioning `b` — so a **stuck** `if leq k k'
then … else …` inside `insert`/`lookup` over an **abstract** key `k` can never be
made to fire from its order hypothesis. Independently grounded twice
(foundation-implementer + foundation-leader) and **Architect-confirmed**
(`evt_556pgcpjqbf0n`, `evt_7wdxesg7d7jam`) against source:

- **`elab.rs` never constructs `Term::J` / `Term::Cast`** (zero hits). The only
  surface equality builder is `Term::Refl`, which **checks** convertibility and
  **fails** when the two sides are not already definitionally convertible — a
  neutral `leq k k'` is never convertible to `True`, so `Refl` cannot bridge a
  *propositional* order fact into a *definitional* reduction.
- **`check_match_dependent` hard-requires a `Term::Var` scrutinee** — a computed
  `leq k k'` cannot be dependently matched (no in-elaborator generalization /
  with-abstraction).
- Ordinary modus-ponens (**combining** `IsTrue`/`Equal` hypotheses by
  application) can only *build* new `IsTrue`/`Equal` values — it can **never
  collapse a stuck scrutinee**. So the "reduce the operation redex so the goal
  whnf-collapses" idiom that closes concrete-carrier laws does not apply when the
  redex is stuck on an abstract argument.

**The kernel already has the sound machinery.** `Term::J` and `Term::Cast` exist
kernel-side (`ken-kernel/src/term.rs` ~313-316; used in `obs.rs`) and are checked
by the kernel's existing rules — they are **already in `trusted_base()`**. The
gap is purely that **surface syntax cannot reach them.** This WP surfaces an
**already-trusted eliminator**; it does **not** invent trust.

## Why this is sound to attempt — and the two anti-patterns it must NOT become

This is exactly the "user writes an explicit transport, the kernel verifies it"
capability every dependent type theory provides (Agda `subst`, Lean `▸`/`Eq.mpr`,
Coq `eq_rect`). It is sound **because the kernel type-checks the emitted `J`/`cast`
term** — the motive, the equality proof, and the result type are all kernel
obligations. **The elaborator asserts nothing the kernel cannot see.**

`packages/lawful-classes/lawful_classes.ken`'s own commentary records **two
rejected approaches this WP must be distinguished from** — cite them as
guardrails, they are the failure modes:

1. **The rejected elaborator-side workaround (K7 arc).** When `eq_at_inductive`
   failed to whnf its operands, an *"elaborator-side transport/`cast` workaround"*
   was **rejected** — because it would *"grow the TCB to route around a
   kernel-completeness gap that belongs in the kernel."* K7 was fixed **in the
   kernel** (`obs.rs`, `4ae2baf`). **Lesson for this WP:** if realistic transport
   goals fail to *compute* because of a **kernel** conversion gap, the fix is a
   **sound kernel completeness fix**, never an elaborator that asserts the result.
2. **The unsound cross-wise `K6` congruence arm.** `conv.rs::conv_struct` lacks a
   congruence case comparing two `Term::Eq(…)` nodes ("K6", Architect-ruled). A
   **sound *positional*** arm (compare `Eq A a b` vs `Eq A a' b'` argument-by-
   argument) is a legitimate kernel completeness fix. A **cross-wise** arm
   (identifying `bool_eq x y` with `bool_eq y x`) is **UNSOUND** — it *"smuggles
   propositional symmetry into definitional equality, collapses directed `Eq`,
   enables unproven-symmetry transport via `cast`"* — and stays a **hard NO.**
   **Lesson:** this WP delivers *explicit, proof-carrying* transport (the user
   supplies `p`), **never** an implicit congruence that manufactures an equality
   the user did not prove.

**The discriminator between the sound target and both anti-patterns:** the
emitted term is a **real `Term::J`/`Term::Cast` the kernel checks**, and every
transport is **witnessed by a user-supplied equality proof**. Nothing is asserted;
nothing is identified implicitly.

## The leverage — why this is fleet-wide, stated honestly

This unblocks **verified *generic* code** — proofs about operations that branch on
an **abstract/opaque** comparison, discharged via the comparison's *propositional*
law hypotheses. It does **not** magically close the concrete-carrier
`lawful-classes` stubs (`Ord Bool`/`DecEq Bool` are already proved by finite
case-split + K7; `Ord Int`'s laws are `Axiom` by **primitive-opacity** — `leq_int`
never reduces on variables regardless of transport). The honest leverage:

- **Flagship consumer: `map-verified-laws`** — the 4 Branch-B laws (preservation,
  found-after-insert, locality, agreement) deferred from `Map-build`. **This WP is
  their hard gate** (two-level: surface-transport → `map-verified-laws`).
- **Future consumers:** any verified generic container/algorithm parametric over
  `Ord`/`Eq` (generic verified search, generic ordered structures) — the same wall
  the Map laws hit. This is the *verified-generic-code* capability, not a
  Map-local patch.

## Objective + the four approach-review questions — ★ NOW ANSWERED (see ratified decomposition above)

> **⚠ These four questions are RESOLVED by the ratified decomposition at the top
> (`evt_1g5bx52mdv5g6`). Do NOT reopen them.** Kept below verbatim for the record of
> what was decided and why. The answers: (1) former = `J`, target = `Term::J`→
> `Cast`, typing rule = `infer_j`; (2) elaborator-only, positional-`K6` OUT; (3)
> `cong` DERIVED; (4) ONE `/spec/30-surface` rule, no kernel note.

The approach-review chose the mechanism; it pinned as fixed inputs for the build:

1. **The surface former + its kernel target.** Which surface spelling
   (`subst` / `transport` / `rewrite` / a `cast`-exposing form) and which kernel
   primitive it elaborates to (**OTT `Term::Cast` vs `Term::J`** — Ken is
   observational; transport along a `Bool`-equation into a motive may route
   through `cast` on a `cong`-derived type equality rather than intensional `J`).
   Pin the **typing rule** (motive, equation, result).
2. **The elaborator-vs-kernel split — the load-bearing design question.** Does the
   emitted `cast`/`J` **compute** on realistic Map goals with only an elaborator
   change, or does making it fire **also require the sound *positional* `K6`
   `conv_struct` congruence arm** (a kernel completeness fix)? If the latter, this
   WP has **two lanes** (elaborator surface former + kernel positional-congruence
   fix); if the former, it is elaborator-only. **Explicitly rule the unsound
   cross-wise arm out of scope.**
3. **Whether a `cong` combinator is also needed** (to lift `Equal A a b` to
   `Equal (P a) (P b)` for the motive) and, if so, whether it is derivable in
   `.ken` from the former in (1) or needs its own surface support.
4. **Spec footprint.** A new surface term-former is a `/spec/30-surface` addition
   (syntax + typing rule); a positional-`K6` conv arm is a `/spec/1x-kernel`
   completeness note. The approach-review states **which spec sections change**, so
   the elaborated brief (via spec-leader, if a `/spec` note is needed) and the
   build are scoped correctly.

The approach-review output **is** the shovel-ready decomposition; the build team
executes it mechanically.

## Acceptance criteria

- **AC1 — SOUNDNESS: kernel-checked emission, nothing asserted (load-bearing).**
  The surface former elaborates to a **real `Term::J`/`Term::Cast`** that the
  **kernel type-checks** — grep the elaboration emits the kernel former (not an
  elaborator-side equality assertion); and a **discriminating negative**: an
  **ill-typed** transport (motive/equation/result mismatched, or an equality proof
  of the *wrong* equation) is **REJECTED by the kernel**, not silently accepted
  ([[kernel-backed-claim-grep-the-emission-not-the-name]]). The mechanism is
  **explicit and proof-carrying** — no implicit congruence manufactures an unproven
  equality (the cross-wise-`K6` anti-pattern stays rejected).
- **AC2 — TRUST SURFACE: ZERO delta (ruled elaborator-only).** `trusted_base()`
  gains **no** `declare_primitive`/`declare_postulate` and no new `Decl` variant —
  the transport routes through the **existing** `Term::J`/`Term::Cast`, already in
  `trusted_base()`. **Grep the diff: NO `crates/ken-kernel/` file touched, NO
  `conv.rs` change** (the ratified decomposition proves no conv change is needed —
  §(2)). This is a pure `ken-elaborator` surface-former addition + a `.ken` prelude
  module. (The positional-`K6` conv arm is OUT of scope; if the build discovers a
  realistic goal that fails to *compute* without a conv change, that is a **finding
  → Steward / Architect**, not an in-scope patch — it would reopen the
  elaborator-vs-kernel split the approach-review closed.)
- **AC3 — CAPABILITY: a real transport-blocked proof now discharges.** A genuine
  Branch-B obligation elaborates and is **kernel-accepted** — minimally, a
  standalone repro (transport to fire a stuck `if leq k k` over an **abstract** `k`,
  with a **hand-written motive**); stretch = one full `map-verified-laws` proof
  (e.g. found-after-insert) built on top. **Note (Architect):** the Branch-B
  obligations live in `Ω` (proof-irrelevant), so "evaluates" = **"the kernel accepts
  the proof term"**; the concrete downstream computation (letter-frequency lookups)
  already runs — it was the *verification* that was blocked.
- **AC4 — no regression.** `cargo test --workspace` green (a change to
  surface-elaboration and/or `conv.rs` is workspace-wide blast radius —
  [[kernel-reduction-change-full-workspace-green]], validate the **full
  workspace**, never the touched crate). Every proof that checked before still
  checks (monotone: the new former only **adds** provability; `conv` only
  **accepts more** definitional equalities).
- **AC5 — spec fidelity.** The surface `J`-former's typing rule (**ONE
  `/spec/30-surface` addition**, pointing at the already-specified kernel rules in
  `15-identity`/`16-observational`) is elaborated by the spec enclave and lands
  with the build; the derived combinators get a thin `50-stdlib` prelude listing.
  **NO `/spec/1x-kernel` conv-completeness note** (no conv change). CV reviews
  `/spec`.

## Guardrails (do-not-reopen)

- **Explicit, proof-carrying transport only.** The user supplies the equality
  witness `p`; the elaborator never manufactures one. **No implicit / cross-wise
  congruence** (the unsound `K6` arm — hard NO).
- **Kernel-checked emission, never an elaborator assertion.** If a realistic goal
  fails to *compute*, the remedy is a **sound kernel completeness fix**, never an
  elaborator that routes around the kernel (the rejected K7-workaround shape).
- **Zero trust-surface growth.** No new primitive/postulate/`Decl` variant. A
  positional-`K6` conv arm, if needed, is a completeness fix argued to accept only
  definitionally-equal terms — not a trust addition.
- **Ground every mechanism claim against landed `elab.rs`/`term.rs`/`obs.rs`/
  `conv.rs`** — the descriptions here are perishable.

## Sequencing

- **Gate:** **Architect approach-review — DONE, APPROVE** (`evt_1g5bx52mdv5g6`) →
  **spec-leader elaborates the ONE `/spec/30-surface` `J`-former typing rule** (+
  the `50-stdlib` prelude listing) on this branch → merge to `main` (Integrator) →
  **Language builds** (one infer-mode `J` former ~the `absurd` pattern + the derived
  `.ken` transport prelude) → **Architect soundness** on the candidate + **Language
  QA** + **CI** + **CV reviews `/spec`**. **No Kernel lane.**
- **Lane:** Language / `ken-elaborator` (+ the `.ken` prelude). **Sibling to
  `dependent-match-nonnullary`** (Gap B) — both Language-lane, both gate
  `map-verified-laws`, but **separate WPs** (distinct mechanisms: `J`-former +
  transport prelude here; IH-slot Elim emission there). Sequence per Language
  capacity (dep-match has no spec dependency → likely first). Disjoint from Kernel's
  `sct-reconstruction-descent` (b) and the enclave's `[FS]`.
- **Downstream:** this WP is one of the **two hard gates for `map-verified-laws`**
  (the 4 comparison-dependent Map Branch-B laws need BOTH this and Gap B; the fifth,
  `toList`-ordered, needs only Gap B). It unblocks the *verified-generic-code* class
  broadly (any proof about operations branching on an abstract `Ord`/`Eq`).
