# Open design decisions (the forks register)

> Status: **living document — for the operator.** Each entry is a genuine fork
> with materially different futures that the spec deliberately does **not**
> resolve unilaterally (per the drafting stance, `SPEC-PROGRESS.md`). Each has a
> **DRAFT recommendation** the spec currently assumes; the operator (or the Spec
> enclave) confirms or overrides. Entries are keyed by a stable **name**; the
> numeric `OQ-n` tags used inline are noted.

How to read an entry: **Fork** (the choice) · **Options** · **Recommendation**
(what the DRAFT assumes) · **Affects** (chapters) · **Why it's open**.

This register consolidates the principal design forks plus decisions surfaced
while drafting. Resolved items move to an ADR (`../docs/adr/`).

---

## A. Kernel & type theory

### OQ-int — Integer type & precision *(tag OQ-1)* — **DECIDED**
- **Fork.** Is `Int` arbitrary-precision or fixed-64 by default? Is `Decimal` a
  core type? Which fixed-width integers are native? (Default overflow behaviour
  for fixed-width resolved below as `OQ-1a` (DECIDED 2026-06-27).)
- **Decision (operator, 2026-06-27).** `Int` is **arbitrary precision** (not
  fixed-64), with a small-int fast path. `Decimal` is a **core, essential**
  type. The **full fixed-width set is native**: signed `Int8/Int16/Int32/Int64`
  and unsigned `UInt8/UInt16/UInt32/UInt64` (everyday for bitfields, wire/byte
  layout, C-ABI FFI). Naming is the **verbose** form (`Int64`, not `I64`).
- **`OQ-1a` DECIDED (2026-06-27).** Default fixed-width overflow is
  **obligation-generating** — a bare `+`/`-`/`*` emits a no-overflow proof
  obligation (`40-runtime/43 §2`); proven ⇒ total, unproven ⇒ degrades to a
  runtime check (so "checked" is *subsumed*). **Wrapping stays explicit**
  (`wrapping_add`/`+%`/`Wrapping[T]`) for intended-modular domains; never the
  silent default.
- **Affects.** `30-surface/35` (updated), `40-runtime/41`, `43`,
  `30-surface/38`.

### OQ-eval-strategy — conversion algorithm — **DECIDED**
- **Fork.** What conversion algorithm the kernel uses to decide definitional
  equality.
- **Decision (operator, 2026-06-27): follow Lean's kernel.** Operational
  algorithm = **lazy weak-head normalization + on-the-fly structural conversion
  + lazy δ-unfolding** (Lean 4's approach; consistent with Ken's Lean-style
  trusted kernel, ADR 0001), realised over an **NbE-style value domain**
  (closures + neutrals) **extended to compute the observational operations**
  (`Eq`-by-type, `cast`) and Ω proof irrelevance (ADR 0005). **NbE stays the
  declarative reference**; lazy-WHNF is the recommended implementation.
- **Deliberate divergences from Lean's *theory*** (fixed by other Ken decisions,
  ADR 0005): observational `J`-on-non-`refl` via `cast` (not
  `Eq.rec`-on-`refl`); **canonicity kept** — Ken needs **no** axioms where Lean
  postulates them (funext/propext and quotient soundness are *definitional* in
  OTT), and assumes no `choice`. Lean's **definitional proof irrelevance** Ken
  **also has**, from the predicative strict-prop Ω (`OQ-Prop`/ADR 0005), without
  impredicativity.
- **Affects.** `10-kernel/17` (updated). Interacts with `OQ-Prop`, `OQ-4`.

### OQ-2 — Cumulativity — **DECIDED**
- **Fork.** Cumulative universes (`Type ℓ ≤ Type ℓ'`) vs. non-cumulative.
- **Decision (operator, 2026-06-27): non-cumulative.** Keeps a subtyping
  relation out of the trusted kernel; consistent with the small-kernel
  principle, following Lean (non-cumulative), and the observational/OTT setting.
  Ergonomics come from the untrusted elaborator: universe polymorphism + typical
  ambiguity + inserted lifts. (Coq is the lone major cumulative system — heavier
  kernel.)
- **Affects.** `10-kernel/12` (updated), `18`.

### OQ-Prop — proposition sort *(tag OQ-3)* — **DECIDED (revised)**
- **Fork.** A primitive impredicative proof-irrelevant `Prop` vs. Ω of mere
  propositions — bundling *two* separable features: impredicativity, and
  definitional proof irrelevance.
- **Decision (operator, 2026-06-27; revised by ADR 0005).** **Impredicativity
  ruled out** (incompatible with canonicity; predicative Ω). **Definitional
  proof irrelevance:** the cubical-era call was "no `SProp`, propositional
  irrelevance"; the observational foundation (`OQ-4`/ADR 0005) **supersedes** it
  — Ω *is* a strict proof-irrelevant universe (`SProp`), so proof irrelevance is
  now **definitional and free** in the smaller OTT kernel (and *helps* agent
  proof generation: equality goals discharge definitionally). No separate
  `SProp` add-on or kernel growth.
- **Affects.** `10-kernel/12`, `16` (updated).

### OQ-4 — Equality foundation — **DECIDED**
- **Fork.** Full cubical (interval, comp/hcomp, Glue, computing univalence,
  HITs) vs. observational TT vs. plain `Id`/`J`.
- **Decision (operator, 2026-06-27, ADR 0005): observational (OTT), not
  cubical.** `Eq` by recursion on type structure + `cast` + a strict-prop Ω
  (`SProp`) + native set-quotients + propositional truncation. `J`/`subst`
  compute on non-`refl` (via `cast`, not the interval); funext/propext/UIP
  definitional; canonicity + decidable conversion proven; **no**
  univalence/higher-HITs (the mathematics features software does not use).
  Chosen for **exact fit to set-level software** and the **smallest auditable
  TCB** (tier-1) — cubical's `--safe` canonicity bugs are the adversarial
  surface agent-generated proofs probe. Blueprints: `CICobs`/`CCobs`/`TTobs`.
- **Quotients (was OQ-4a).** Set-quotients in the DRAFT; general QITs a possible
  later extension.
- **Affects.** `10-kernel/15`, `16` (rewritten), `11`, `12`, `17`, `README`,
  `18`.

### OQ-η-records — η for single-constructor inductives — **DECIDED**
- **Fork.** Extend definitional η beyond Σ to all single-constructor inductives,
  or keep it to the record/Σ class?
- **Decision (operator, 2026-06-27): η is the `record`/Σ class, not `data`.**
  Records (nested Σ) get definitional η; `data` declarations — incl.
  single-constructor — do not (declare a `record` if you want η). Rationale: one
  kernel η rule (already needed for Σ); **safe by construction** (records are
  non-recursive nested Σ, so η terminates; recursive single-ctor types are
  `data` and stay η-free, dodging recursive-η undecidability); **low-cost under
  OTT** (record `Eq` already computes componentwise, `16 §2`). Matches Agda
  `record`-vs-`data` and Lean structure-η.
- **Affects.** `10-kernel/14 §4` (updated), `13 §3`.

### OQ-decimal-eq — Lawful equality over `Decimal`'s non-canonical carrier — **OPEN**
- **Fork.** `Decimal`'s DEMOTE carrier is `Prod Int Int`
  (`MkDecimalPair coeff exp`) — **non-canonical**: many representations per
  numeric value (`(10, -1)` and `(1, 0)` both denote `1.0`). The lawful
  `DecEq`/`Num` contracts tie `eq`/ops to the kernel's **definitional `Equal`**
  (`DecEq.sound : IsTrue (eq x y) → Equal a x y`, `51 §2.2`). Value-equality
  `decimalEq` (align-then-compare) reduces `True` on structurally-distinct
  pairs, so a `sound` `Axiom` would inhabit `Bottom` (`MkDecimalPair`
  injectivity → `Equal Int 10 1 → Bottom`). So **no lawful class over `Decimal`
  that ties `eq`/ops to `Equal` is deliverable** on the current carrier — a
  decide-once basis call, not a per-instance rediscovery (`DecEq Decimal`,
  `Num Decimal` both hit the identical wall).
- **Options.** **(a) Canonicalize the carrier** — a normalized representation
  (one `(coeff, exp)` per value), so `decimalEq` **is** structural `Equal` and
  `DecEq`/`Num` transport soundly (a representation-semantics WP that changes
  the already-shipped carrier). **(b) Setoid / quotient `Eq Decimal`** — target
  `Eq` (an equivalence, no `Equal`-tie) or a quotient carrier, retargeting the
  shared `DecEq`/`Num` `sound`/`complete` contract to setoid equality (blast
  radius across every `DecEq`/`Num` instance — a shared-contract design change).
  Neither is free: even honest `Eq Decimal` bottoms on missing `Int` arithmetic
  lemmas
  (`refl` needs `sub_int e e = 0`, `mul_int c 1 = c`).
- **Contrast — `Char` is unaffected.** `Char = { c : Int | isScalar c }` is
  **canonical** (one carrier value per codepoint, `proj` identity, `isScalar`
  Ω-irrelevant), so `Equal Char x y ≡ Equal Int (proj x) (proj y)`
  definitionally and `Ord Char`/`DecEq Char` transport `Int`'s **true**
  meta-theorem `Axiom` soundly. Canonical-vs-non-canonical is the whole
  discriminator.
- **Status.** **OPEN** — the single gate for the future `class Num` +
  `Decimal`-equality lane. The `Decimal` DEMOTE ships only its **computational**
  ops (exact base-10 arithmetic + primitive removal, `18a §5.6`), which are
  unaffected; no lawful-`Decimal` deliverable is in flight, so this is not
  urgent.
- **Affects.** `10-kernel/18a §5.6`/`§5.6.1` (the `Eq`-not-`DecEq` boundary),
  `50-stdlib/51` (the shared `DecEq`/`Num` contract), a future `class Num` WP.

---

## B. Verification

### OQ-12 — SMT integration strategy — **DECIDED**
- **Fork.** How to discharge obligations soundly with a classical solver under
  an intuitionistic logic, ending at a kernel-checked term.
- **Decision (operator, 2026-06-27).** **Kripke embedding** primary (Ken's
  native topos = Kripke semantics). Three-tier routing: decidable → direct Z3 +
  **reflective `decide`**; first-order intuitionistic → embedding + certificate;
  higher-order → tactics + typed holes. Certificate bridge: **route (a) — the
  reflective proved-adequacy meta-lemma + a verified certificate checker — is
  the *target*** (discharge by computation), chosen on intrinsic merits
  (permanent artifact, robust to solver drift, scales, and it yields the G5
  kernel-soundness adequacy theorem), **not** on effort. **(b) reconstruction**
  (SMTCoq-style / Herbrand) is kept **only as a feasibility hedge** + bring-up
  cross-check. Leans on OTT canonicity + definitional proof irrelevance (ADR
  0005). **Z3** primary; **cvc5** optional second oracle; **Coq retired**
  (external checker would enlarge the TCB).
- **Residual risk.** Whether the adequacy + checker-soundness metatheory
  *mechanizes cleanly* — a **feasibility** risk, retired by a thin front-loaded
  (a) slice (not an effort estimate).
- **Affects.** `20-verification/23` (updated). Interacts with `OQ-spec`.

### OQ-spec — Surface proof interface & state model — **DECIDED (interface)**
- **Fork.** Refinement types on arrows vs. a separate tactic language vs. both;
  and whether `ensures` may reference pre-state (`old(e)`), i.e. the
  state/mutation model for contracts.
- **Decision (operator, 2026-06-27): interface = both, as one gradient.**
  Declarative `requires`/`ensures` + refinement types (the human-readable
  contract) → automatic proof → typed hole → tactic/term — one smooth descent,
  not two languages; tactics produce kernel-rechecked terms (`23`). **Every
  claim carries a visible, exportable four-way epistemic status — proved /
  tested / delegated / unknown** (`20-verification/21 §5`) — the surface form of
  "prove what can be proven, state what must be tested," and the seam to the
  behavioral sibling (`OQ-behavioral`, ADR 0006).
- **Still deferred → `OQ-Space`.** `old`/pre-state and the mutation model. DRAFT
  leans **explicit state** (name pre/post as values; no implicit heap, no `old`,
  no framing/separation machinery unless forced).
- **Affects.** `20-verification/21` (updated), `70-behavioral/`,
  `30-surface/36`.

---

## C. Surface language

### OQ-syntax — Concrete syntax — **DECIDED (principles); table iterates**
- **Fork.** Keyword set, layout-vs-braces, operator set, Unicode extent — the
  whole concrete spelling (the **visibility default** sub-item is now settled,
  see below).
- **Decision (operator, 2026-06-27): the *principles* are decided; the concrete
  *token table* iterates with the team under them.** Because agents write and
  humans read, the canonical form is **optimized for reading** (the typability
  tax doesn't bind agent-written code): **(1)** rich notation that **matches
  established CS/Math convention, never invents**; **(2)** a **total ASCII
  transliteration** (write either; glyph carries zero extra info); **(3)** a
  **single mandated formatter** canonicalizing ASCII → Unicode + layout (no
  style latitude); **(4)** **keywords stay ASCII words**, notation reserved for
  operators; **(5)** a **bounded, confusable-resistant** blessed set with lexer
  normalization/rejection of Unicode look-alikes (TR39) — a *security* property
  (the reviewer reads exactly what the kernel checks; no homoglyph backdoor,
  `60-security/64`). A starter glyph↔ASCII table is in `30-surface/31 §1b`.
- **Still iterating (team, under the principles):** the exact glyph for each
  construct (notably `≡`-vs-`==` equality and the lattice-op/`ℓ` ASCII),
  layout-vs-brace details, the keyword set — **except** the **definition
  keywords**, now fixed: `view` is **retired**, split into `const`/`fn`/`proc`
  (operator, SURF-1; `30-surface/36 §1.6`, `31 §4`), whose spellings are **not**
  iterating.
- **Resolved sub-item (SURF-1 D3, 2026-07-04):** the **Unicode surface = lexer
  *and* formatter** — the lexer accepts a curated Unicode glyph and its ASCII
  transliteration as the **same token** (ASCII stays accepted forever), the
  formatter **emits canonical Unicode** on save, and keywords stay ASCII words
  (`30-surface/31 §1c`). A direct consequence of principles 2–4, pinned for the
  BL3 build; not a new fork.
- **Resolved sub-item (ES3, 2026-07-01):** the **visibility default =
  module-private-by-default + `pub`** (`30-surface/33 §4`) — the least-surface /
  information-hiding-forward choice that matches abstract export. Not iterating.
- **Deferred follow-on (CAT-2 Fork A, 2026-07-04):** **implicit superclass
  coercion** — auto-inserting a superclass projection so a bare `map` resolves
  in an `Applicative`/`Monad` context (the Haskell `Functor f ⇒ Applicative f`
  reading) instead of the explicit `d.functor.map`. CAT-2 ships **explicit**
  superclass wiring (`50-stdlib/56 §2`); the sugar **would** need a new
  elaborator capability (resolution walking the superclass edge, a `55 §6`
  guardrail re-fork) — **not** taken now, re-forked to Steward if/when wanted.
  Purely ergonomic; nothing depends on it.
- **Affects.** `30-surface/31 §1a/§1b` (updated), all of `30-surface/`;
  `30-surface/33 §4` (visibility default resolved).

### OQ-classes — Typeclass/instance coherence — **DECIDED**
- **Fork.** Instance-resolution ambiguity & coherence policy (global uniqueness?
  named instances? overlap?).
- **Decision (operator, 2026-06-27, ADR 0008).** Split by where the dictionary
  lives. **Property classes** (Ω-valued) get coherence **free** from proof
  irrelevance. **Structure classes** (`Type`-valued): **one canonical instance
  per (class, head-type)** in implicit search (resolution = a function of the
  type, which the law-carrying prover relies on); **orphan instances a hard
  error** (instance with class or head-type); **no overlap**; **ambiguity is a
  compile error**, never a silent pick; search terminates (SCT-family bound).
  **Named instances are first-class values passed explicitly** — the
  dependent-types escape hatch (no `newtype` gymnastics): implicit search stays
  canonical, explicit passing is unrestricted.
- **Affects.** `30-surface/33 §5`, `39 §2`. **Recorded.**

### OQ-8 — Effect-system shape — **DECIDED**
- **Fork.** `visits`-style static+transitive rows vs. Kleisli/monadic effects
  vs. algebraic-effects-with-handlers; and the **kernel encoding**. Sub-fork
  `OQ-8a`: capabilities a separate construct or just effects; static vs.
  runtime.
- **Decision (operator, 2026-06-27).** **Static effect rows** (`visits`), pure
  by default, transitively inferred. **Encoding = three layers into a pure
  kernel** (`30-surface/36 §2`): **authority** = capability-passing (Π over
  tokens); **denotation** = an **interaction-tree / free-monad** pure data
  structure (one denotation powers handlers-as-folds, Ward's event alphabet, IFC
  labels, and verification); **specification** = WP/Hoare predicates over the
  denotation. The kernel never gains an effect primitive (small-TCB invariant).
  Handlers **tail-resumptive only**; multishot → `OQ-9`/research.
- **`OQ-8a` DECIDED.** Capabilities are **first-class value tokens** — static,
  visible, **attenuable + revocable + audited** (ADR 0004), handler-or-row
  supplied — distinct from logical `requires`. Not a runtime gate.
- **Stateful-effect verification methodology → `OQ-Space`.** Decided here:
  effect *shape/encoding*; deferred there: how stateful effects' pre/post is
  reasoned.
- **`SURF-1` child pins — DECIDED, no reopened fork** (consistent with the
  DECIDED model, kernel-untouched, `ken-elaborator`-only, no new `Term`/`Decl`):
  - **(1) Row-variable surface** (`30-surface/36 §1.5`, Architect-grounded, D1).
    The effect-polymorphic **row variable** `[e]` (and open-row tail `[E | e]`),
    bound as an **implicit parameter**, made *surface-writable* — the row
    variable was already *implied by* the tree denotation (Koka rows,
    the cited precedent) and its internal machinery already landed; D1 adds the
    surface + a bounded recursive-fixpoint lift (`36 §1.3`/`§1.5`). Every
    concrete instantiation is **statically closed** (AC3, structural).
  - **(2) Purity keywords `const`/`fn`/`proc`** (`30-surface/36 §1.6`, operator
    ruling Pat 2026-07-04). `view` **retired**; the definition keyword is a
    **checked static-purity signal** — `const` = pure value (0 explicit value
    params), `fn` = pure function (≥1), `proc` = at-least-potentially-impure
    (concrete row, a **row variable**, or a `space` op) at any arity;
    **effect-polymorphic ≠ pure**. Verified **bidirectionally** (keyword vs.
    signature + inferred effects); a mismatch is a **hard error** (`§1.6.3`).
- **Unblocks.** `OQ-export-ir` (event alphabet = interaction-tree nodes),
  `OQ-ifc` (labels ride the row); **`SURF-1` pin (1) unblocks `CAT-2`**
  (Traversable's `traverse` is the first surface effect-polymorphic definition).
- **Affects.** `30-surface/36` (updated), `30-surface/32`/`33`/`31`
  (SURF-1 grammar/keywords/lexer), `60-security/61`, `60-security/62`,
  `70-behavioral/`.

### OQ-9 — Continuations / handlers — **DECIDED (excluded)**
- **Fork.** Tail-resumptive handlers only vs. reified/multishot continuations.
- **Decision (operator, 2026-06-27): tail-resumptive only; multishot excluded —
  a positive design choice.** The expressiveness multishot is reached for is
  **already subsumed**: generators (`visits [Yield]`),
  nondeterminism/backtracking as **search-as-data** folded by total recursion,
  async/concurrency via the **seam** (`OQ-Space`), and delimited control
  captured *denotationally* by the interaction tree (`OQ-8`). What it *uniquely*
  adds — first-class dynamic `call/cc` — is unpredictable, **complicates**
  proofs (breaks single-consumption WP; single-shot is a proof
  *simplification*), and is costly to compile — cutting against every Ken
  commitment. Not "research someday"; a footnote only if a concrete
  *unsubsumable* need appears.
- **Affects.** `30-surface/36 §5` (updated).

### OQ-coinduction — Infinite data & productivity — **DECIDED (deferred)**
- **Fork.** Coinductive types + productivity checker vs. streams-as-functions
  with a fuel/size discipline.
- **Decision (operator, 2026-06-27): inductive/total core — no coinductive
  types, no productivity checker; deferred.** A productivity/guardedness checker
  is real TCB growth (dual of SCT, the guarded-modal machinery `OQ-temporal`
  declined). **Infinitude is routed away from the value layer:** a *total*
  program's interaction tree is **finite** (reaches `Ret`); forever-running
  processes are total per-message handlers + the runtime loop + Ward's temporal
  model (`OQ-Space`, `OQ-temporal`). Streaming is served by **generators**,
  **`Lazy` streams (fuel/depth-bounded)**, or the **seam** — finite-by-
  construction, totality unchanged. "Defer" ≠ "cannot stream".
- **Re-open trigger.** A concrete recurring need for first-class infinite values
  or coinductive/bisimulation proofs the three idioms cannot serve → a
  **contained sized-types/guardedness** discipline or a **deep-embedded
  coinductive layer with reflective productivity**, never clock-modal kernel
  growth.
- **Affects.** `30-surface/37 §3` (updated), `40-runtime/43 §4` (updated).
  **Recorded.**

---

## D. Runtime & representation

### OQ-7 — Content-addressed boundary — **DECIDED**
- **Fork.** Exactly which values are interned (small tuples? closures by
  code+env hash?) vs. immediate, and the per-case equality story.
- **Decision (operator, 2026-06-27).** **Scalars immediate; compound/identity-
  bearing interned**, equality per case (slot-equality interned, native
  immediate). The principle is fixed; the **small-aggregate boundary is an
  empirical X2 tuning**, not semantics.
- **Affects.** `40-runtime/41 §5` (updated).

### OQ-hash — Addressing & hashing functions — **DECIDED**
- **Fork.** Exact in-process hash (FNV-1a vs. other), collision strategy, and
  the separate serialization/Merkle hash.
- **Decision (operator, 2026-06-27).** **Two hashes, two jobs:** a fast
  **non-cryptographic hash + `memcmp`** in-process (**not** lattice geometry); a
  **cryptographic/Merkle** hash for serialization (`63`). The exact functions
  are an X2 constant.
- **Affects.** `40-runtime/41 §3` (updated), `44`.

### OQ-5 — Heap capacity bound — **DECIDED**
- **Fork.** Keep the Λ₂₄-derived 196,560 ceiling vs. an engineering-chosen
  bound.
- **Decision (operator, 2026-06-27).** **Engineering-chosen, no practical
  ceiling** (wide 48-/64-bit handles for billions+), **loud refusal** kept as a
  permanent stance; the Leech number is aesthetic. Exact width is an X2/X4
  constant.
- **Affects.** `40-runtime/44 §2` (updated).

### OQ-6 — Leech/Golay/Co₀ machinery — **DECIDED**
- **Fork.** Include the lattice math at all, and if so in which of its three
  *separate* roles (Golay EC lists; kissing-number bitmap; Co₀
  canonicalization)?
- **Decision (operator, 2026-06-27).** **Not in the core** — optional research
  packages only (WS-R), never on the allocation hot path. The lattice math is an
  aesthetic flourish, kept out of the load-bearing runtime.
- **Affects.** `40-runtime/44 §4` (updated), `50-stdlib`.

### OQ-gc — Reclamation — **DECIDED (deferred impl detail)**
- **Fork.** Manual/region reclamation only vs. adding automatic GC/refcount for
  the content heap.
- **Decision (operator, 2026-06-27).** **Manual + region-scoped** now. Automatic
  GC is a **well-demonstrated benefit at modest cost** and **invisible to the
  language surface and semantics** (immutable values + content identity ⇒
  reclaiming an unreachable slot changes nothing observable), so it **collapses
  to a deferred implementation detail** the runtime may adopt when working sets
  demand — **no language fork**.
- **Affects.** `40-runtime/44 §3` (updated).

### OQ-eval-order — Strictness — **DECIDED**
- **Fork.** Strictness vs. laziness for `let`/data fields (observable values
  fixed; this is space/time, not meaning).
- **Decision (operator, 2026-06-27).** **Call-by-value (strict) with sharing,
  strict by default.** Totality makes eval-order **meaning-preserving**, so pick
  the most **predictable/legible** order — strict wins (reason-able cost model,
  reading order, no thunk/space-leak footguns, composes with content-addressed
  sharing). **Predictability is a precondition for the time/space reasoning
  security needs** (`@ct`/`61 §5a`, worst-case bounds). Laziness only **where
  required** (`if`/`match` taken-arm, `&&`/`||` short-circuit) or **by explicit
  annotation** — an opt-in **`Lazy a`** thunk (forced-on-demand, memoized;
  laziness *visible in the type*, never implicit). Distinct from the kernel's
  lazy-WHNF conversion (`OQ-eval-strategy`): runtime executes CBV, kernel
  decides defeq lazily; they agree only on final values. Coinductive fragment,
  if added (`OQ-coinduction`), brings its own local guarded laziness.
- **Affects.** `40-runtime/42` (updated), `41` (sharing). **Recorded.**

### OQ-domain — Ken's intended domain / positioning — **DECIDED**
- **Fork.** How broad — and how bounded — is Ken's intended domain: a bare-metal
  systems language, or a verified software-engineering language over a wider
  application range?
- **Decision (operator, 2026-07-02, in-session ruling).** **Broad but bounded,
  and asymmetric between the two bounds.** The **lower bound is
  systems-*adjacent*** — one notch above true systems programming — and is
  **settled and substantiated**: the content-addressed managed heap with
  optional, semantics-invisible reclamation is the *right* substrate for it, not
  a compromise (`40-runtime/44 §3`). The **upper bound — reaching application,
  edge, web, and mobile targets — is directional, not delivered**: an
  aspirational reach realized via **native codegen**, itself **as-yet-unexplored
  design space** (`40-runtime/45`, the X-series; the target choice is
  `OQ-backend-target`, **OPEN**), so it is **not** a claimed capability. Across
  that range Ken is a **verified software-engineering language**, not a
  bare-metal systems language. `OQ-domain` fixes the *domain*; it does not
  settle the *toolchain* (`OQ-backend-target`), nor claim the upper-bound
  targets as built.
- **Affects.** `40-runtime/44 §3` (memory-model rationale, the settled lower
  bound), `40-runtime/45` + `OQ-backend-target` (the forthcoming upper-bound
  reach), `docs/PRINCIPLES.md` §I.1 (mission domain-qualification).
  **Recorded.**

### OQ-backend-target — Native codegen target/toolchain — **OPEN (operator-ratifiable)**
- **Fork.** Which native-code target/toolchain the backend (`X3`, `45`) lowers
  core terms to. The **one irreversible** architectural decision in the backend
  (long-term dependency + TCB-surface + toolchain implications), so it is
  surfaced for operator ratification, not locked by the design ring.
- **Options (tradeoffs in `45 §5`).** **Cranelift** (Rust-native, in-tree, small
  surface, fast compile / younger codegen, fewer targets); **LLVM** (mature,
  best codegen, many targets / large C++ toolchain, big surface); **C source**
  (maximally portable / needs a C toolchain, UB surface); **WASM** (sandboxed,
  portable / different execution model, not "native" per se).
- **Design-ring lean (not a lock).** **Cranelift** — Rust-native keeps the
  toolchain in-tree and the auditable surface small (small-auditable-TCB); and
  because the backend is **not** in the soundness TCB (`45 §2`), codegen
  maturity is a **quality/perf** concern the differential corpus catches,
  **not** a trust concern. Weigh against the operator's perf/target-breadth
  priorities.
- **Status.** **OPEN** — an **operator-ratifiable Decision**. No target is
  written into normative prose as decided; **`X3-build` (`ken-codegen`) does not
  start until the operator ratifies** the target.
- **Affects.** `40-runtime/45` (frames the model + tradeoffs, target-agnostic).
  **Open.**

---

## E. Concurrency, wire, process

### OQ-Space — State, concurrency & isolation — **DECIDED**
- **Fork.** What a `space` maps to (OS process / thread / logical region); the
  transport/wire data model; the isolation guarantee; and (handed from `OQ-8`)
  the stateful-effect verification methodology + `old`.
- **Decision (operator, 2026-06-27).** **State:** `space` = **encapsulated,
  non-aliased** cells with identity; mutation `becomes` denotes to a
  state-passing fold of the interaction tree (`OQ-8`). Because state is
  partitioned per-space with **no cross-space aliasing**, reasoning is **bounded
  per-space Hoare — no separation logic**. **`old(e)` admitted, scoped to a
  `space` operation's `ensures`** (a cell's pre-call value); no global
  `\old`/heap — this resolves the `OQ-spec` deferral. **Concurrency:**
  **shared-nothing, message-passing** (actor-style) for *in-Ken* communication —
  isolation is a **guarantee** (no shared mutable memory ⇒ no data races),
  pairing with capabilities (a space handle is authority), effects
  (send/receive), IFC labels (on messages), and Ward (message events = the
  behavioral alphabet). Distribution-ready; the **runtime realization**
  (process/thread/green/distributed) is deferred to `40-runtime`. **Transport:**
  content-addressed **immutable value passing** (cross-space dedup by hash;
  composes with K3); labels ride; typed/session channels a later refinement.
  **Division of labor:** Ken proves **local/sequential/per-space** correctness;
  **global/concurrent/distributed/temporal** correctness is **delegated to
  Ward** (Quint/Apalache/P model-check the message protocol).
- **FFI caveat (ancillary).** The shared-nothing guarantee is for *in-Ken*
  communication. **FFI** may use **shared memory** at the foreign boundary — but
  that boundary is already explicitly *unsafe/untrusted* (`30-surface/38 §3`),
  so it does not weaken the in-Ken isolation property.
- **Isolation property (ADR 0004).** Shared-nothing gives a *stated* isolation
  guarantee on which capability revocation (`60-security/62 §4`) and confinement
  rest; the runtime realization MUST preserve it. Warrants an ADR when
  `40-runtime` settles the realization.
- **Affects.** `30-surface/36 §4` (updated), `20-verification/21 §4` (`old`),
  `40-runtime/`, `60-security/62`, `70-behavioral/`.

### OQ-witness — Surface runtime introspection — **DECIDED**
- **Fork.** Expose process-level heap stats / Merkle root (extensional-safe) —
  and the exact surface.
- **Decision (operator, 2026-06-27).** **Process-level stats only** (slots,
  dedup rate, arena bytes, Merkle root) as an extensional-safe `witness`
  facility; **never** per-value identity/provenance/address — that would let a
  program observe sharing and break **referential transparency**.
- **Affects.** `40-runtime/41 §7` (updated).

---

## F. Research-track (never core; strategy WS-R)

### OQ-coalgebra — The coalgebraic layer — **DECIDED (excluded)**
- **Fork.** Pursue Store-comonad cells/lenses, process coalgebras +
  bisimulation, profunctor wires, co-Heyting boundaries — at all?
- **Decision (operator, 2026-06-27): not in the core — same reasoning as
  `OQ-9`.** The pragmatic wins are **subsumed** by `visits` + `space` (and
  reactive/temporal concerns are Ward's, `OQ-temporal`/`OQ-coinduction`); the
  rest is a mathematical probe, not a software-engineering need. Harvest any
  concrete pragmatic win back as an ordinary package; no core/kernel layer.
- **Affects.** `00-overview.md §5`, `50-stdlib §6`.

---

## G. Security (tier-1; ADR 0004)

These are sub-decisions *within* committed security goals — the commitments
themselves (IFC intrinsic, least authority, re-check-on-consume, honest limits)
are **fixed** by ADR 0004; the mechanics below are now **DECIDED** (see each
entry and the Resolution log).

### OQ-ifc — Information-flow label model — **DECIDED**
- **Fork.** Fixed level lattice vs. principal-set DLM vs. user-defined; labels
  as values vs. type indices vs. both; non-interference by typing vs. by proof.
- **Decision (operator, 2026-06-27).** The discipline is **lattice-parametric**:
  non-interference proved **once for any bounded lattice** (so the fixed/DLM/
  user-defined split dissolves — all are instances). **Standard lattice = DLM**
  (confidentiality = readers, integrity = endorsers; levels as sugar,
  compartments as products). Labels **static type indices by default**
  (erasable; annotate the interaction-tree `perform` nodes, `OQ-8`), with
  **first-class labels at audited boundaries** for **data-derived
  classification** — tag at ingestion (a trusted, capability-gated, audited
  point), carry as `∃ℓ. A@ℓ` statically, one runtime check at the sink (covers
  per-tenant routing); **full dynamic/faceted IFC excluded** ("better is the
  enemy of good"). **By-typing** non-interference is the default;
  bespoke/quantitative → `OQ-relational`. The concrete lattice/
  classifications/clearances/edges come from a **separately-authored policy**
  (`OQ-policy`). **No kernel enlargement.**
- **Affects.** `60-security/61` (updated), `65`, `30-surface/36`,
  `70-behavioral/`.

### OQ-policy — Security policy as code *(new; ADR 0007)* — **DECIDED**
- **Fork.** Does Ken need a separately-authored policy surface so security/
  compliance specify enforced policy orthogonal to implementation — and is it in
  Ken or a sibling?
- **Decision (operator, 2026-06-27, ADR 0007).** **Yes — a mandatory, static,
  separately-authored policy surface *in Ken* (not a sibling).** Role
  separation, not engine separation: enforced by Ken's own checker. A policy
  supplies the
  lattice/classifications/clearances/declassification-edges/ingestion-points
  (`65 §2`), **bound program-wide and non-weakenable** (relabel/over-clear =
  compile error). It is the **instantiation** of the lattice-parametric
  discipline → **no metatheory/kernel cost**. Org-scale governance rides
  supply-chain attestation (`63`).
- **Still open (within `OQ-policy`).** Concrete policy **syntax**; **binding**
  (per package/build/deploy) and **composition** (org→team→service,
  **monotone-tightening only**); version/attestation interplay.
- **Affects.** `60-security/65` (new), `63`, `61`. **ADR 0007.**

### OQ-relational — Relational / 2-safety verification — **DECIDED**
- **Fork.** How relational properties (non-interference, **constant-time**) are
  generated and proved: self-composition / product programs vs. relational
  refinement types vs. a dedicated relational logic; and whether the default is
  **termination-/progress-sensitive** (does divergence or a crash leak?).
- **Decision (operator, 2026-06-27).** **By-proof relational mode reduces to
  unary obligations the kernel re-checks** — **product programs** preferred over
  naive self-composition (lock-step keeps invariants solver-tractable); a
  first-class relational logic, if ever needed, is a **reflective deep
  embedding**, never a kernel primitive (reflect-don't-extend). **Default is
  progress-sensitive** (a crash / non-termination *is* an observable leak);
  termination-insensitive only by **explicit annotation** (shows in the four-way
  status / delta). The heavy product-program machinery is **deferred** until a
  concrete value-dependent declassification case needs it — the **by-typing
  taint** path covers the load-bearing security work.
- **Constant-time — split out, not a relational proof.** CT is a distinct
  **opt-in `@ct` (timing-sensitive) label** (separate from `Secret`); its values
  may never reach a **leakage-relevant effect sink** (secret-dependent branch /
  index / var-time op), enforced **by typing** — a unary taint discipline that
  **soundly enforces the source-level 2-safety property** (FaCT/ct-verif), so
  **no product programs**. The sensitive **range = the `@ct` label's live span**
  (intro → `declassify`), so **no `constant_time { }` region**. The *timing
  guarantee* (codegen/hardware-relative — cache lines, `cmov`-vs-branch) is
  **delegated to `Ward` + the toolchain** under a stated **leakage
  model**/platform, recorded in the discharge attestation (`63 §5a`); a
  **policy** may require `@ct` for a data class (`65`). Ken's static part is a
  *necessary precondition*, honestly not the whole guarantee.
- **Affects.** `60-security/61 §5/§5a`, `30-surface/36 §3`, `60-security/64
  §4.2`, `65`, `63 §5a`; product-program engine deferred to `20-verification/`.
  **Recorded.**

### OQ-provenance — signing, attestation & package format — **DECIDED**
- **Fork.** Signing mechanism; SLSA integration; the `.keni` format; registry
  attestation policy.
- **Decision (operator, 2026-06-27).** Package = `(source, artifact, .keni,
  proof-bundle, trusted_base_delta, provenance)`; **consume = re-check, not
  re-prove**. Signing = **keyless sigstore/cosign + in-toto/SLSA** attestation
  content (keyless suits an agent ecosystem — no keys to leak); **aim high on
  SLSA** (reproducible builds make it natural). **Two ladders kept distinct:**
  program-proof (re-checked, zero-trust) vs build-provenance (trusted, origin).
  **+ Policy attestation** (new, ties `OQ-policy`): the governing policy
  hash/version travels in provenance; consume checks a **monotone-compatible**
  policy ⇒ "provably honoured org policy `vX`" across the dep graph. Registry =
  namespace ownership + mandatory provenance. **Implementation deferred** to
  post-core-toolchain; this fixes the shape + standards.
- **Affects.** `60-security/63` (updated), `65`, `30-surface/33`.

---

## H. Behavioral assurance (downstream; ADR 0006)

The sibling (**`Ward`**) and the seam to it. *One logic, two engines:* Ken
states what it cannot prove; the sibling models/tests/monitors it.

### OQ-behavioral — The downstream complement's shape — **DECIDED**
- **Fork.** Extend Ken's kernel with temporal/modal types vs. a separate
  behavioral sibling consuming Ken's export.
- **Decision (operator, 2026-06-27, ADR 0006): tightly-coupled *sibling***
  (`Ward`) fed by an **assumption-boundary export**; Ken stays the small static
  core. Temporal obligations are **stated as deeply-embedded data**
  (LTL/μ-calculus `Temporal` type), **not kernel modalities** — TCB untouched.
  One logic, two engines.
- **Affects.** `70-behavioral/`, `20-verification/21`. **ADR 0006.**

### OQ-export-ir — The assumption-boundary export schema — **DECIDED**
- **Fork.** The concrete schema of Ken's behavioral export (`Q` invariants, `P`
  assumptions, refinements-as-generators, effect alphabet, temporal
  obligations); ITF-compatible traces vs a Ken-native format.
- **Decision (operator, 2026-06-27, ADR 0006).** The export is an
  **assume-guarantee contract**, **generated** from verified content (never
  hand-authored, so it cannot overclaim — a projection of the four-way status).
  Five parts: `guarantees Q` (proved), `assumptions P` (the `trusted_base_delta`
  + explicit `assume`s + boundary labels), `alphabet Σ` (**= the
  interaction-tree perform-node signatures**, `OQ-8` — reuse, not reinvention),
  `obligations T` (the `Temporal` data, delegated), `generators G`. **Two
  layers:** Ken-native for the propositional contract, **ITF** for traces.
  Versioned + content-addressed + travels in provenance (`63`). **`G` carries
  support structure only** — the equivalence-class partition, boundaries, and
  case decomposition (derivable) — **never a sampling measure**;
  invariant-exclusions ride `P`/`Q`, judgment-exclusions go to the sampling
  policy (`OQ-sampling-policy`).
- **Affects.** `70-behavioral/71` (drafted), `README`. **Recorded.**

### OQ-sampling-policy — The test-sampling measure (Ward-side) — *deferred*
- **Fork.** The *measure* over the valid state space for empirical testing
  (likelihood / risk-cost weighting / soft exclusions) is human + environmental
  judgment, not derivable. Where does it live and how is it authored?
- **Decision (operator, 2026-06-27): outside Ken source, durably.** It is
  **per-deployment** (the same Ken component needs a different measure as an
  internal API vs. an external endpoint), so it belongs in the class of
  deployment-adjacent artifacts (`Dockerfile`/Terraform), **not** in source. It
  is a separately-authored **sampling policy** on `Ward`'s side, governed like
  the security policy (`60-security/65`): distinct authoring role (QA/risk/
  domain), versioned, attested in provenance ("tested under sampling policy
  `vX`, coverage `Y`"). Ken's exported `G` partition is the **vocabulary** it
  indexes. **Open (deferred):** the policy *language* + attestation interplay —
  needs `Ward`'s sampler design (downstream).
- **Affects.** `Ward` (sibling); `70-behavioral/71 §4` notes the seam.

### OQ-temporal — In-language temporal layer — **DECIDED**
- **Fork.** Keep temporal logic as exported *data* only, or add a guarded/`▷`
  modal layer so Ken can *reason* about temporal properties internally?
- **Decision (operator, 2026-06-27, ADR 0006): data-only, durably.** No temporal
  modalities in the kernel (no `▷`/clocks/Löb — that grows the TCB, the surface
  ADR 0005 rejected cubical to avoid). **Durable, not a v1 hedge:** it is the
  consistent application of Ken's *reflect-don't-extend* principle (OTT over
  cubical; reflective `decide` over trusting Z3; `Temporal`-as-data here) and
  the topos-fragment decomposition that justifies the seam (Ken = static/
  propositional fragment, Ward = temporal/modal fragment of one logic).
- **The boundary (load-bearing).** Ken reasons **about** temporal formulas (they
  are inductive data — transformations, normalizations, well-formedness, all
  ordinary static proofs) but **not with** temporal modalities (no `▷` in the
  judgment); discharging the obligations is Ward's.
- **Revisit-trigger.** Unbounded liveness ("no deadlock for *all* `N`", which
  model-checking only covers for `N ≤ k`) — handled, if it bites, by a
  **contained reflective model** (prove in the deep-embedded semantics),
  **never** kernel modalities.
- **Affects.** `70-behavioral/72` (drafted), `README`. **Recorded.**

### OQ-classical-bridge — Intuitionistic↔classical seam — **DECIDED**
- **Fork.** Ken's logic is intuitionistic/total/static; the model-checkers are
  classical/temporal. Which direction does refinement flow, and is the mapping
  itself a Ken-checked artifact?
- **Decision (operator, 2026-06-27, ADR 0006).** **Strictly one-way (Ken →
  Ward).** Ken exports obligations + assumptions; Ward discharges them; results
  **never re-enter Ken as proofs** (never promoted to `proved`) — for human
  legibility as much as soundness. **Sound by assume-guarantee construction:**
  Ken proves `Q ⊣ P`, kernel-checked, intuitionistically valid however `P` is
  later discharged; no classical strength leaks in (and on
  decidable/finite-state obligations the gap vanishes anyway).
- **Translation faithfulness (the Ken-checked half).** `τ` splits: **property
  translation** `compile : Temporal Σ → WardFormula` is proved
  semantics-preserving **once, at the compiler level** (amortized to zero per
  obligation; the analog of the Kripke-adequacy lemma, `20-verification/23 §4`);
  **model translation** is structural — the model is *generated* from code (no
  authoring drift) + conformance (`OQ-conformance`) + an honest assumption. The
  one trust edge (Ward implements the axiomatized semantics) is **pinned as the
  Ward version in the discharge attestation** (`OQ-discharge-attestation`).
- **Affects.** `70-behavioral/71 §5`, `20-verification/23 §7`. **Recorded.**

### OQ-discharge-attestation — Post-build validation artifact — **DECIDED**
- **Fork.** How to represent "the delegated obligations were discharged by Ward"
  as a first-class compliance artifact (vs. text logs + coverage XML).
- **Decision (operator, 2026-06-27): a signed, runtime-checkable discharge
  attestation** (`60-security/63 §5a`), governed on the policy-attestation
  ladder (`65`), enabling a per-target-environment **deployment gate**.
- **Ratified (Sec6, 2026-07-01) — `Ward` finalized its half (ward `f33276b`);
  Ken's half pinned in `63 §5a`:**
  - **Ken-visible field set** (the ratified contract surface, all already B1-
    emitted, `70-behavioral/71 §2.1`): `export.hash`, `export.contractVersion`,
    `ward.version` (the one trust edge), `obligations[].id`/`.field`/`.outcome`,
    `signature`. Literal tokens + the `predicateType` URI are `Ward`'s wire
    spelling, oracle-tagged under `OQ-export-wire`.
  - **Narrowing from the 2026-06-27 sketch:** the `Ward` policy (hash+version)
    and the sampling choices/coverage are **`Ward`-internal**, not Ken-visible —
    Ken classifies **epistemic status**, never `Ward`'s **mechanism**. No Ken
    correctness judgment may read a `Ward`-internal field (`policy`/`bound`/
    `evidence`/`ct.method`/`regression`).
  - **Outcome vocabulary:** four-way **total** `discharged / bounded / monitored
    / failed` — `bounded-to-`k`` **widened to `bounded`** (covers model-check
    depth **and** sampled coverage; the bound is `Ward`-internal).
  - **Hard invariant (I4):** no `outcome` promotes a `T` to `proved`; a
    discharge projects to `P`/`tested`, never `Q` (`70-behavioral/71 §5.1`). The
    CT-
    validation *method* stays `Ward`-side (ward §13); Ken carries the verdict.
- **Build follow-on (named, sequenced-behind):** the three-check deployment gate
  on Ken's provenance verifier (Team Verify, WS-Sec) — the runtime face of the
  static invariants pinned in `63 §5a`.
- **Affects.** `60-security/63 §5a`, `65`; `70-behavioral/71 §5`.

### OQ-conformance — Ken's observability contract — **DECIDED**
- **Fork.** Is implementation-refines-model conformance a CI gate, a production
  monitor, or both? (The antidote to the two-artifact tax.)
- **Decision (operator, 2026-06-27, ADR 0006): reframed to Ken's half.** The
  spec question is **not** gate/monitor/both (that is a downstream engine's
  policy) but **what Ken emits to make the running system observable in the
  model's vocabulary**. Ken provides a **trace/instrumentation contract** (a
  companion to the `71` export, generated): concrete **`Σ`-event schema** at the
  effect boundary (cheap — instrumentation-dominated), **correlation/identity**
  keys for multi-space traces (`OQ-Space`), runtime forms of `Q`/`P`, and the
  **monitor spec synthesized from `T`** (LTL→Büchi). The export is a **broadcast
  contract** to a *family* of engines (verifier, test-gen, runtime monitor); the
  runtime monitor is likely a **distinct engine** (a k8s sidecar) with a
  *conformance* policy, not Ward's *discharge* policy.
- **Out of scope (downstream).** Gate vs. monitor vs. both, the engine, the
  failure response (halt/alert/degrade/rollback) — per-deployment policy,
  recorded in the discharge attestation (`OQ-discharge-attestation`).
- **Affects.** `70-behavioral/73` (drafted), `71 §1`, `README`. **Recorded.**

### OQ-agentic-oracle — Oracle-free agent outputs — **DECIDED (scoped out)**
- **Fork.** How to assure agent outputs with no propositional oracle.
- **Decision (operator, 2026-06-27): not a Ken mechanism — it reduces to
  already-decided machinery.** An embedded agent is a
  **maximally-nondeterministic input** (the strongest `P`); assurance =
  **constrain** (verified envelope = `62` capabilities + `61` IFC + contracts;
  `proved`), **relate** (metamorphic = `OQ-relational`, oracle-free; test-gen
  L2), **watch** (RV monitors = `73`/`72`). **Honesty boundary:**
  safety/structural/relational assured, **quality never** — the envelope is
  `proved`; agent output is `tested`/`delegated`/`unknown`. Ken builds nothing
  new.
- **Affects.** `70-behavioral/74` (drafted, boundary-statement). **Recorded.**

---

## Resolution log

| OQ | Decided | ADR |
|---|---|---|
| **OQ-int** | 2026-06-27 — arbitrary-precision `Int`; `Decimal` core; full native `Int8…Int64`/`UInt8…UInt64` (verbose names). `OQ-1a` also DECIDED (overflow obligation-generating; wrapping explicit). | — (recorded in `30-surface/35`) |
| **OQ-eval-strategy** | 2026-06-27 — follow Lean: lazy-WHNF + on-the-fly conversion + lazy δ over an NbE value domain extended to compute observational `Eq`/`cast`; NbE the reference. Diverges from Lean's theory on observational `J`/canonicity. | — (recorded in `10-kernel/17`) |
| **OQ-2** | 2026-06-27 — **non-cumulative** universes; ergonomics via universe polymorphism + typical ambiguity + elaborator lifts. | — (recorded in `10-kernel/12`) |
| **OQ-4** | 2026-06-27 — **observational equality (OTT), not cubical**: `Eq`-by-type + `cast` + strict-prop Ω + set-quotients; no univalence/HITs. Smallest auditable TCB; exact set-level-software fit. | **ADR 0005** |
| **OQ-Prop** | 2026-06-27 — predicative Ω; impredicativity ruled out. Proof irrelevance **definitional** via OTT's strict-prop Ω (`SProp`), free in the smaller kernel (revised by ADR 0005). | **ADR 0005** (recorded in `10-kernel/12`) |
| **OQ-η-records** | 2026-06-27 — definitional η is the **`record`/Σ class**, not `data`; safe-by-construction (records are non-recursive nested Σ), low-cost under OTT. | — (recorded in `10-kernel/14`) |
| **OQ-12** | 2026-06-27 — Kripke embedding primary; three-tier routing; **reflective proved-adequacy + verified checker (a) is the target** (intrinsic merits, not effort), reconstruction (b) a feasibility hedge; Z3 primary, cvc5 optional, **Coq retired**. | — (recorded in `20-verification/23`) |
| **OQ-spec** | 2026-06-27 — proof interface = **both, as one gradient**; **four-way epistemic status** (proved/tested/delegated/unknown) visible + exportable. `old`/state deferred → `OQ-Space` (lean explicit-state). | — (recorded in `20-verification/21`) |
| **OQ-behavioral** | 2026-06-27 — downstream complement is a **sibling** (`Ward`) fed by an assumption-boundary export; temporal obligations as **data, not kernel modalities**; one logic, two engines. | **ADR 0006** |
| **OQ-8 / OQ-8a** | 2026-06-27 — static effect **rows** (`visits`), pure by default; **layered encoding** authority(tokens)/denotation(interaction-tree)/spec(WP) into a pure kernel; handlers tail-resumptive only; capabilities = static value tokens (attenuable/revocable/audited). Stateful verification → `OQ-Space`. | — (recorded in `30-surface/36`) |
| **`OQ-8` child · SURF-1** | 2026-07-04 — **row-variable surface** `[e]`/`[E | e]` as an implicit param (effect polymorphism made surface-writable, statically closed at every instantiation; unblocks CAT-2) + **purity keywords** `const`/`fn`/`proc` (`view` retired; checked static-purity signal, bidirectional, mismatch a hard error; effect-polymorphic ≠ pure) + **Unicode surface** = lexer(both spellings, same token)+formatter(emits Unicode), keywords stay ASCII. Kernel-untouched. | — (recorded in `30-surface/36 §1.5`/`§1.6`, `31 §1c`/`§4`, `32`/`33 §1`) |
| **OQ-Space** | 2026-06-27 — encapsulated non-aliased `space` cells → **bounded per-space Hoare, no separation logic**; **`old` scoped to space ops** (resolves `OQ-spec` deferral); **shared-nothing message-passing** (in-Ken), content-addressed transport; runtime realization → `40-runtime`; concurrent/temporal correctness **delegated to Ward**; FFI shared memory is the (unsafe) exception. | — (recorded in `30-surface/36 §4`); ADR when `40-runtime` settles |
| **OQ-ifc** | 2026-06-27 — **lattice-parametric** non-interference (proved once, any lattice); **DLM** standard; static type-index labels + first-class **boundary** labels for data-derived classification (per-tenant), no full dynamic IFC; by-typing default; lattice supplied by policy. | — (recorded in `60-security/61`) |
| **OQ-policy** | 2026-06-27 — **policy as code**: a mandatory, static, separately-authored security-policy surface **in Ken** (role separation, not a sibling); the lattice-parametric *instantiation*; non-weakenable; governance via supply-chain. | **ADR 0007** (recorded in `60-security/65`) |
| **OQ-provenance** | 2026-06-27 — package = (source, artifact, .keni, proof-bundle, delta, provenance); consume = **re-check**; **keyless sigstore + in-toto/SLSA**; two ladders distinct; **+ policy attestation** (governing policy in provenance, monotone-compatible consume check). Impl deferred. | — (recorded in `60-security/63`) |
| **OQ-classes** | 2026-06-27 — **property classes** (Ω) coherent for free; **structure classes** = **one canonical instance per (class, head-type)**, **orphans a hard error**, no overlap, ambiguity is an error; named instances are first-class values passed **explicitly** (the dependent escape hatch); search terminates. | **ADR 0008** (recorded in `30-surface/33`, `39`) |
| **OQ-export-ir** | 2026-06-27 — export = **assume-guarantee contract**, **generated** from verified content (can't overclaim); five parts `Q`/`P`/`Σ`(=interaction-tree alphabet)/`T`/`G`; **Ken-native contract + ITF traces**; versioned/content-addressed/in provenance; **`G` = support structure only, never a measure**. | **ADR 0006** (recorded in `70-behavioral/71`) |
| **OQ-sampling-policy** | 2026-06-27 — the test-sampling **measure** lives **outside Ken source, durably** (per-deployment; `Dockerfile`/Terraform class); a Ward-side **sampling policy** governed like the security policy; Ken's `G` partition is its vocabulary. Policy *language* deferred (needs Ward's sampler). | — (deferred; `70-behavioral/71 §4`) |
| **OQ-temporal** | 2026-06-27 — **data-only, durably**: no kernel temporal modalities; `Temporal` is inert inductive data, stated + exported + delegated to Ward. Boundary: Ken reasons **about** formulas, **not with** modalities. Unbounded liveness → contained reflective model, never kernel modalities. | **ADR 0006** (recorded in `70-behavioral/72`) |
| **OQ-classical-bridge** | 2026-06-27 — **strictly one-way (Ken → Ward)**; Ward results never promoted to `proved`; sound by **assume-guarantee** (`Q ⊣ P`); **translation faithfulness** Ken-checked **once at the compiler level** (analog of Kripke adequacy) + generated-model + conformance; trust edge pinned as Ward version in the discharge attestation. | **ADR 0006** (recorded in `70-behavioral/71 §5`, `20-verification/23`) |
| **OQ-discharge-attestation** | 2026-06-27 — post-build validation = a **signed, runtime-checkable discharge attestation**; a **deployment gate** enforces per-target-environment validation; policy-attestation ladder. **DECIDED (Sec6, 2026-07-01, ward `f33276b`):** Ken-visible field set ratified (`export.hash`/`.contractVersion`, `ward.version`, `obligations[].id`/`.field`/`.outcome`, `signature` — all B1-emitted); `Ward` policy + sampling **reclassified `Ward`-internal** (Ken classifies epistemic status, not mechanism); outcome four-way `discharged/bounded/monitored/failed` (`bounded-to-`k`` widened to `bounded`); I4 hard (no `outcome` → `proved`). | **DECIDED** (`60-security/63 §5a`) |
| **OQ-conformance** | 2026-06-27 — **reframed to Ken's half**: Ken emits a **trace/instrumentation contract** (concrete `Σ`-event schema at the effect boundary + correlation/identity for multi-space + runtime `Q`/`P`/`T` monitors), making the running system observable in the model's vocabulary. Export = **broadcast contract** to a family of engines; runtime monitor likely a **distinct sidecar**. Gate/monitor/both + response = downstream policy. | **ADR 0006** (recorded in `70-behavioral/73`) |
| **OQ-relational** | 2026-06-27 — by-proof relational = **re-checked unary obligations** (product programs; reflective embedding if ever first-class), **progress-sensitive** default, heavy machinery **deferred**. **Constant-time split out**: a distinct **opt-in `@ct` label** enforced **by typing** (taint to leakage-effect sinks; sound 2-safety enforcement, no product programs); timing guarantee **delegated to Ward** under a leakage model; policy may require `@ct` per data class. | — (recorded in `60-security/61 §5a`, `30-surface/36`, `64`, `65`) |
| **OQ-eval-order** | 2026-06-27 — **CBV (strict) with sharing, strict by default**; totality makes eval-order meaning-preserving so pick the predictable order (cost model, reading order, no space leaks; precondition for `@ct`/bounds). Laziness only where required (branches/short-circuit) or by explicit **`Lazy a`** thunk. Distinct from kernel lazy-WHNF conversion. | — (recorded in `40-runtime/42`) |
| **OQ-coinduction** | 2026-06-27 — **inductive/total core, deferred**: no coinductive types/productivity checker (TCB growth `OQ-temporal` declined). Infinitude routed away from the value layer (total ITree is finite; forever = per-message handler + runtime loop + Ward). Streaming via generators / `Lazy` (fuel-bounded) / the seam — finite-by-construction. Re-open → contained sized-types or reflective deep embedding. | — (recorded in `30-surface/37`, `40-runtime/43`) |
| **OQ-7** | 2026-06-27 — scalars **immediate**, compound/identity-bearing **interned** (equality per case); small-aggregate boundary an **X2 tuning**, not semantics. | — (recorded in `40-runtime/41`) |
| **OQ-hash** | 2026-06-27 — **two hashes**: fast non-crypto + `memcmp` in-process; crypto/Merkle for serialization (`63`). Exact functions an X2 constant. | — (recorded in `40-runtime/41`) |
| **OQ-5** | 2026-06-27 — **engineering-chosen capacity, no practical ceiling** (wide handles), **loud refusal** permanent; Leech number aesthetic. | — (recorded in `40-runtime/44`) |
| **OQ-6** | 2026-06-27 — Leech/Golay/Co₀ machinery **out of the core**, optional research packages only, never the hot path — the lattice math kept out of the load-bearing runtime. | — (recorded in `40-runtime/44`) |
| **OQ-gc** | 2026-06-27 — manual + region-scoped now; **automatic GC a deferred implementation detail** (semantics-invisible — addable later with no language fork). | — (recorded in `40-runtime/44`) |
| **OQ-domain** | 2026-07-02 — **broad but bounded (asymmetric)**: lower bound systems-adjacent, *settled & substantiated* (managed content-heap is the right substrate, `44 §3`); upper bound application/edge/web/mobile *directional, not delivered* — an aspirational reach via native codegen, itself unexplored (`45`, `OQ-backend-target` OPEN). Verified software-engineering across the range, not bare-metal systems. | — (recorded in `40-runtime/44 §3`, `PRINCIPLES §I.1`) |
| **OQ-witness** | 2026-06-27 — **process-level stats only** (extensional-safe `witness`); **never** per-value identity/provenance (referential transparency). | — (recorded in `40-runtime/41`) |
| **OQ-9** | 2026-06-27 — **tail-resumptive only; multishot excluded** (positive choice). Expressiveness subsumed (generators / search-as-data / seam / interaction-tree denotation); `call/cc` uniquely adds unpredictability + breaks single-consumption WP (single-shot *simplifies* proofs). Footnote only if an unsubsumable need appears. | — (recorded in `30-surface/36 §5`) |
| **OQ-1a** | 2026-06-27 — fixed-width overflow **obligation-generating by default** (proven ⇒ total; unproven ⇒ runtime check, so "checked" subsumed); **wrapping explicit** (`+%`/`Wrapping[T]`) for intended-modular domains, never silent. | — (recorded in `30-surface/35 §3`, `40-runtime/43`) |
| **OQ-coalgebra** | 2026-06-27 — **excluded from core** (same reasoning as `OQ-9`); pragmatic wins subsumed by `visits` + `space`, reactive concerns are Ward's; harvest any win as a package, no core layer. | — (recorded in register; `00-overview.md §5`) |
| **OQ-agentic-oracle** | 2026-06-27 — **scoped out (not a Ken mechanism)**: agent = maximal `P`; constrain (envelope = `61`/`62`) + relate (metamorphic = `OQ-relational`) + watch (RV = `73`). Safety assured, **quality never** (envelope `proved`; output `tested`/`delegated`/`unknown`). | **ADR 0006** (recorded in `70-behavioral/74`) |
| **OQ-syntax** | 2026-06-27 — **principles decided** (canonical form optimized for *reading* since agents write / humans read): rich notation matching CS/Math convention + total ASCII transliteration + one mandated formatter + keywords-as-words + a bounded **confusable-resistant** set (TR39, a security property). Concrete token **table iterates** with the team. | — (recorded in `30-surface/31 §1a/§1b`) |

When an OQ is decided, record it here and, if architecturally significant, write
an ADR under `../docs/adr/` and update the affected chapters (replacing the OQ
tag with the decision).
