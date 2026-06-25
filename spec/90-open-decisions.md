# Open design decisions (the forks register)

> Status: **living document — for the operator.** Each entry is a genuine fork
> with materially different futures that the spec deliberately does **not**
> resolve unilaterally (per the drafting stance, `SPEC-PROGRESS.md`). Each has a
> **DRAFT recommendation** the spec currently assumes; the operator (or the Spec
> enclave, against the prototype oracle) confirms or overrides. Entries are
> keyed by a stable **name**; the numeric `OQ-n` tags used inline are noted.

How to read an entry: **Fork** (the choice) · **Options** · **Recommendation**
(what the DRAFT assumes) · **Affects** (chapters) · **Why it's open**.

This register consolidates the digest's 16 enumerated forks (§10) plus decisions
surfaced while drafting. Resolved items move to an ADR (`../docs/adr/`).

---

## A. Kernel & type theory

### OQ-int — Integer type & precision *(digest fork 1; tag OQ-1)*
- **Fork.** Is `Int` arbitrary-precision or fixed-64 by default? Is `Decimal` a
  core type? Default overflow behaviour for fixed-width (tag OQ-1a).
- **Options.** (a) arbitrary-precision `Int` default + fixed-width siblings; (b)
  fixed-64 default + a separate bignum. Overflow: checked / wrapping /
  obligation-generating.
- **Recommendation (DRAFT).** Arbitrary-precision `Int` default (correctness
  over silent overflow), small-int fast path; `Decimal` is core; fixed-width `+`
  checked by default. *The one real numeric defect the analysis found — fix it
  well.*
- **Affects.** `30-surface/35`, `40-runtime/41`. **Why open.**
  Perf/representation vs. correctness trade; the operator may want fixed-64
  default for a systems feel.

### OQ-eval-strategy — Kernel evaluation strategy *(digest fork 2)*
- **Fork.** NbE vs. another reduction strategy for conversion.
- **Recommendation.** **NbE** (the natural fit for a dependent + cubical
  kernel).
- **Affects.** `10-kernel/17`. **Why open.** Confirm against the prototype;
  small chance an alternative suits the cubical computations better.

### OQ-2 — Cumulativity *(part of digest fork 3)*
- **Fork.** Cumulative universes (`Type ℓ ≤ Type ℓ'`) vs. non-cumulative.
- **Recommendation.** **Non-cumulative** (simpler kernel; elaborator hides the
  cost via level polymorphism + lifts).
- **Affects.** `10-kernel/12`, `18`.

### OQ-Prop — Impredicative `Prop` *(part of digest fork 3; tag OQ-3)*
- **Fork.** A primitive impredicative proof-irrelevant `Prop` vs. a derived Ω of
  mere propositions.
- **Recommendation.** **Derived Ω** (no new kernel sort; uniform).
- **Affects.** `10-kernel/12`. **Why open.** The verification layer might want
  impredicativity later.

### OQ-4 — Cubical scope *(digest fork 4)*
- **Fork.** Full cubical (interval, comp/hcomp, Glue, computing univalence,
  HITs) vs. a lighter HoTT-with-`Id`/`J` core. Sub-fork OQ-4a: general user HITs
  vs. a fixed kernel menu.
- **Recommendation.** **Full cubical** (it is what makes `J`-on-non-`refl` and
  univalence reduce; a lighter core reopens the prototype's `J` gap). HITs:
  **fixed menu** first, general user HITs an extension.
- **Affects.** `10-kernel/15`, `16`. **Why open.** Kernel size vs. computational
  univalence/HITs.

### OQ-η-records — Definitional η for single-constructor inductives
- **Fork.** Extend definitional η beyond Σ to all single-constructor records.
- **Recommendation.** η for **Σ/records only** in the DRAFT.
- **Affects.** `10-kernel/14`.

---

## B. Verification

### OQ-12 — SMT integration strategy *(digest fork 12)*
- **Fork.** Kripke-embedding-as-classical-FOL vs. SMTCoq-style certificate
  reconstruction vs. Herbrand constructivization; which solvers (Z3/cvc5/both);
  keep the Coq backend?
- **Recommendation.** **Kripke embedding** primary (it is Ken's native
  semantics), with a kernel-certificate route — **(a) proved soundness
  meta-lemma** preferred, **(b) reconstruction** as fallback. Reflective
  decision for the decidable fragment. Z3 first; cvc5 optional; retire Coq.
- **Affects.** `20-verification/23`. **Why open.** (a) vs. (b) is real
  engineering; both end at a kernel-checked term.

### OQ-spec — Surface proof interface & state model
- **Fork.** Refinement types on arrows vs. a separate tactic language vs. both;
  and whether `ensures` may reference pre-state (`old(e)`), i.e. the
  state/mutation model for contracts. *(digest fork 11.)*
- **Recommendation.** **Both** refinements *and* a small tactic surface; `old`
  included only once the `space` state model (OQ-Space) settles.
- **Affects.** `20-verification/21`, `30-surface/36`.

---

## C. Surface language

### OQ-syntax — Concrete syntax
- **Fork.** Keyword set, layout-vs-braces, operator set, Unicode extent,
  visibility default — the whole concrete spelling.
- **Recommendation.** The proposal in `30-surface/31`,`32` (layout with brace
  fallback; `view`/`data`/`record`/`match`; Unicode + ASCII spellings).
  **Bikeshed, deferred — not load-bearing.**
- **Affects.** all of `30-surface/`. **Why open.** Taste + ergonomics; settle
  with the team.

### OQ-classes — Typeclass/instance coherence
- **Fork.** Instance-resolution ambiguity & coherence policy (global uniqueness?
  named instances? overlap?).
- **Recommendation.** Lawful classes-as-subobjects (`30-surface/33 §5`);
  coherence policy to be pinned — lean toward **global coherence** for
  predictability.
- **Affects.** `30-surface/33`, `39`.

### OQ-8 — Effect-system shape *(digest fork 8)*
- **Fork.** `visits`-style static+transitive rows vs. Kleisli/monadic effects
  vs. algebraic-effects-with-handlers. Sub-fork OQ-8a: capabilities a separate
  construct or just effects; static vs. runtime.
- **Recommendation.** **`visits`-style static rows** (proven, simpler), pure by
  default; capabilities **static and visible** (not the prototype's runtime
  gate).
- **Affects.** `30-surface/36`.

### OQ-9 — Continuations / handlers *(digest fork 9)*
- **Fork.** Tail-resumptive handlers only vs. reified/multishot continuations.
- **Recommendation.** **Tail-resumptive** in core; multishot is **research**
  (`02 §7`).
- **Affects.** `30-surface/36`.

### OQ-coinduction — Infinite data & productivity
- **Fork.** Coinductive types + productivity checker vs. streams-as-functions
  with a fuel/size discipline.
- **Recommendation.** **Defer**; total inductive core stands. Decide when
  streaming becomes load-bearing.
- **Affects.** `30-surface/37`, `40-runtime/43`.

---

## D. Runtime & representation

### OQ-7 — Content-addressed boundary *(digest fork 7)*
- **Fork.** Exactly which values are interned (small tuples? closures by
  code+env hash?) vs. immediate, and the per-case equality story.
- **Recommendation.** Scalars immediate; compound/identity-bearing interned; the
  small-aggregate boundary tuned in X2.
- **Affects.** `40-runtime/41`.

### OQ-hash — Addressing & hashing functions
- **Fork.** Exact in-process hash (FNV-1a vs. other), collision strategy, and
  the separate serialization/Merkle hash.
- **Recommendation.** Fast non-cryptographic hash + `memcmp` in-process (**not**
  lattice geometry); a cryptographic/Merkle hash for serialization.
- **Affects.** `40-runtime/41`, `44`.

### OQ-5 — Heap capacity bound *(digest fork 5)*
- **Fork.** Keep the Λ₂₄-derived 196,560 ceiling vs. an engineering-chosen
  bound.
- **Recommendation.** **Engineering-chosen** (wider slot field for billions),
  with the **loud-refusal** philosophy kept. The Leech number is aesthetic, not
  load-bearing.
- **Affects.** `40-runtime/44`.

### OQ-6 — Leech/Golay/Co₀ machinery *(digest fork 6)*
- **Fork.** Include the lattice math at all, and if so in which of its three
  *separate* roles (Golay EC lists; kissing-number bitmap; Co₀
  canonicalization)?
- **Recommendation.** **Not in the core**; available as optional research
  packages, never on the allocation hot path.
- **Affects.** `40-runtime/44`, `50-stdlib`.

### OQ-gc — Reclamation
- **Fork.** Manual/region reclamation only vs. adding automatic GC/refcount for
  the content heap.
- **Recommendation.** **Manual + region-scoped** (suits the immutable dedup
  model); revisit if working sets demand it.
- **Affects.** `40-runtime/44`.

### OQ-eval-order — Strictness
- **Fork.** Strictness vs. laziness for `let`/data fields (observable values
  fixed; this is space/time, not meaning).
- **Recommendation.** **Call-by-value with sharing**; branch/short-circuit
  laziness where semantically required.
- **Affects.** `40-runtime/42`.

---

## E. Concurrency, wire, process

### OQ-Space — State, concurrency & isolation model *(digest forks 13,14)*
- **Fork.** What a `space` maps to (OS process / thread / logical region); the
  transport/wire data model (the prototype's real DTO/frame stream — and the
  unimplemented LIST/MAP tags — **not** the false 4×f64 ceiling); the isolation
  guarantee. The prototype's `spawn` is `fork()`+shared memory — *not* a
  commitment.
- **Recommendation.** Encapsulated, effect-tracked, identified mutable state
  (`30-surface/36 §4`); the concrete process/transport model is a **deliberate,
  later** design choice — do **not** inherit the prototype's.
- **Affects.** `30-surface/36`, `40-runtime/`. **Why open.** A significant
  systems design with security implications; deserves its own ADR.

### OQ-witness — Surface runtime introspection *(digest fork 16)*
- **Fork.** Expose process-level heap stats / Merkle root (extensional-safe) —
  and the exact surface.
- **Recommendation.** **Yes**, process-level stats only; **never** per-value
  identity/provenance (would break referential transparency).
- **Affects.** `40-runtime/41 §7`.

---

## F. Research-track (never core; strategy WS-R)

### OQ-coalgebra — The coalgebraic layer *(digest fork 15)*
- **Fork.** Pursue Store-comonad cells/lenses, process coalgebras +
  bisimulation, profunctor wires, co-Heyting boundaries — at all?
- **Recommendation.** **Research only**; harvest pragmatic wins back as ordinary
  packages. Partly subsumed by `visits` + `space`.
- **Affects.** `02 §7`, `50-stdlib §6`.

---

## Resolution log

| OQ | Decided | ADR |
|---|---|---|
| *(none yet — all DRAFT recommendations pending operator confirmation)* | | |

When an OQ is decided, record it here and, if architecturally significant, write
an ADR under `../docs/adr/` and update the affected chapters (replacing the OQ
tag with the decision).
