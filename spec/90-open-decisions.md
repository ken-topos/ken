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

### OQ-int — Integer type & precision *(digest fork 1; tag OQ-1)* — **DECIDED**
- **Fork.** Is `Int` arbitrary-precision or fixed-64 by default? Is `Decimal` a
  core type? Which fixed-width integers are native? (Default overflow behaviour
  for fixed-width remains open as `OQ-1a`.)
- **Decision (operator, 2026-06-27).** `Int` is **arbitrary precision** (not
  fixed-64), with a small-int fast path. `Decimal` is a **core, essential**
  type. The **full fixed-width set is native**: signed `Int8/Int16/Int32/Int64`
  and unsigned `UInt8/UInt16/UInt32/UInt64` (everyday for bitfields, wire/byte
  layout, C-ABI FFI). Naming is the **verbose** form (`Int64`, not `I64`).
- **Still open (`OQ-1a`).** The *default* overflow behaviour on fixed-width `+`
  (checked / wrapping / obligation-generating). DRAFT: checked.
- **Affects.** `30-surface/35` (updated), `40-runtime/41`, `30-surface/38`.

### OQ-eval-strategy — conversion algorithm *(fork 2)* — **DECIDED**
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

### OQ-2 — Cumulativity *(part of digest fork 3)* — **DECIDED**
- **Fork.** Cumulative universes (`Type ℓ ≤ Type ℓ'`) vs. non-cumulative.
- **Decision (operator, 2026-06-27): non-cumulative.** Keeps a subtyping
  relation out of the trusted kernel; consistent with the small-kernel
  principle, following Lean (non-cumulative), and the observational/OTT setting.
  Ergonomics come from the untrusted elaborator: universe polymorphism + typical
  ambiguity + inserted lifts. (Coq is the lone major cumulative system — heavier
  kernel.)
- **Affects.** `10-kernel/12` (updated), `18`.

### OQ-Prop — proposition sort *(fork 3; tag OQ-3)* — **DECIDED (revised)**
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

### OQ-4 — Equality foundation *(digest fork 4)* — **DECIDED**
- **Fork.** Full cubical (interval, comp/hcomp, Glue, computing univalence,
  HITs) vs. observational TT vs. plain `Id`/`J`.
- **Decision (operator, 2026-06-27, ADR 0005): observational (OTT), not
  cubical.** After a research review (`local/`): `Eq` by recursion on type
  structure + `cast` + a strict-prop Ω (`SProp`) + native set-quotients +
  propositional truncation. `J`/`subst` compute on non-`refl` (closing the
  prototype's gap, via `cast` not the interval); funext/propext/UIP
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

---

## B. Verification

### OQ-12 — SMT integration strategy *(digest fork 12)* — **DECIDED**
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
  state/mutation model for contracts. *(digest fork 11.)*
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
  gate). **Security requirement (ADR 0004, fixed regardless of construct
  form):** capabilities MUST be **attenuable** and **revocable**, with boundary
  audit, and the effect machinery MUST host information-flow **labels** (see
  `OQ-ifc`). So OQ-8a settles the *form*, not *whether* authority is
  least/attenuable/labeled.
- **Affects.** `30-surface/36`, `60-security/61`, `60-security/62`.

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
  later** design choice — do **not** inherit the prototype's. **Security
  requirement (ADR 0004):** the chosen model MUST carry a **stated, proven
  isolation property** (it can no longer stay "deliberate choice, not
  inherited"), since capability revocation (`60-security/62 §4`) and confinement
  rest on it.
- **Affects.** `30-surface/36`, `40-runtime/`, `60-security/62`. **Why open.** A
  significant systems design with security implications; deserves its own ADR.

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

## G. Security (tier-1; ADR 0004)

These are sub-decisions *within* committed security goals — the commitments
themselves (IFC intrinsic, least authority, re-check-on-consume, honest limits)
are **fixed** by ADR 0004; only the mechanics below are open.

### OQ-ifc — Information-flow label model
- **Fork.** The security-label model: a fixed level lattice vs. a principal-set
  decentralised label model (DLM) vs. fully user-defined lattices; labels as
  first-class values, type indices, or both; the static **discipline** giving
  non-interference *by typing* (DCC/sealing-calculus style) vs. relational
  *proof* obligations for it.
- **Recommendation.** Commit to a **lattice + upward-only flow + audited
  declassification + non-interference** (fixed); start with a **principal/level
  lattice** and a **by-typing** discipline (the scalable default), adding
  relational proof (`OQ-relational`) for bespoke/quantitative claims. Labels
  ride the indexed-effect machinery (`OQ-8`), **no kernel enlargement**.
- **Affects.** `60-security/61`, `30-surface/36`. **Why open.** Several viable
  label models; the choice trades expressiveness vs. inference/ergonomics.

### OQ-relational — Relational / 2-safety verification
- **Fork.** How relational properties (non-interference, **constant-time**) are
  generated and proved: self-composition / product programs vs. relational
  refinement types vs. a dedicated relational logic; and whether the default is
  **termination-/progress-sensitive** (does divergence or a crash leak?).
- **Recommendation.** Provide a relational mode whose certificates are
  **kernel-re-checked** like any other (no new trusted primitive); pick the
  encoding the Verify enclave finds most tractable; default to a clearly-stated
  sensitivity level. This mode also serves constant-time (a side-channel
  concern, `60-security/64 §4.2`).
- **Affects.** `60-security/61 §5`, `20-verification/`, `40-runtime/43`. **Why
  open.** Real engineering choice; relational reasoning is less settled than
  unary.

### OQ-provenance — Signing, build attestation & the package format
- **Fork.** The artifact/`​.keni` interface format; cryptographic signing
  (sigstore/cosign keyless vs. in-toto); SLSA build-attestation integration; the
  registry attestation policy.
- **Recommendation.** Define the package = `(source, artifact, .keni,
  proof-bundle, trusted_base_delta, provenance)` with **consume = re-check, not
  re-prove**; add **signing + SLSA** as the *complementary* origin/build axis
  (distinct from Ken's program-level proofs — keep the two ladders separate).
- **Affects.** `60-security/63`, `30-surface/33`. **Why open.** Ecosystem
  tooling; multiple equivalent mechanisms; sequencing after the core toolchain.

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

### OQ-export-ir — The assumption-boundary export schema
- **Fork.** The concrete schema of Ken's behavioral export (`Q` invariants, `P`
  assumptions, refinements-as-generators, effect alphabet, temporal
  obligations); ITF-compatible traces vs a Ken-native format.
- **Recommendation.** Define the schema as the first `70-behavioral/`
  deliverable; adopt **ITF** for the trace half (immediate Quint/Apalache
  interop).
- **Affects.** `70-behavioral/71`. **Why open.** First real design artifact of
  the seam; needs the effect model (`OQ-8`) settled.

### OQ-temporal — In-language temporal layer: data-only vs reasoning
- **Fork.** Keep temporal logic as exported *data* only, or add a guarded/`▷`
  modal layer so Ken can *reason* about temporal properties internally?
- **Recommendation.** **Data-only** for v1 (state + export; no kernel
  modalities), per the small-TCB principle; revisit only on a concrete need to
  prove temporal properties *inside* Ken.
- **Affects.** `70-behavioral/72`. **Why open.** Movable line; kernel-cost vs
  expressiveness.

### OQ-classical-bridge — Intuitionistic↔classical refinement mapping
- **Fork.** Ken's logic is intuitionistic/total/static; the model-checkers are
  classical/temporal. Which direction does refinement flow, and is the mapping
  itself a Ken-checked artifact?
- **Recommendation.** Treat the embedding like `OQ-12`'s — prefer a *checked*
  mapping where feasible; precedent exists (Coq+TLA refinement).
- **Affects.** `70-behavioral/`, `20-verification/23`.

### OQ-conformance — Trace conformance: gate, monitor, or both
- **Fork.** Is implementation-refines-model conformance a CI gate, a production
  monitor, or both? (The antidote to the two-artifact tax.)
- **Recommendation.** **Both**, eventually; CI gate first. Keep instrumentation
  cheap (RV overhead is instrumentation-dominated).
- **Affects.** `70-behavioral/73`.

### OQ-agentic-oracle — Oracle-free agent outputs
- **Fork.** How to assure agent outputs with no propositional oracle.
- **Recommendation.** **Metamorphic relations + RV watchdogs**; a default safety
  envelope (the verifiable FSM the agent is a nondeterministic oracle inside).
- **Affects.** `70-behavioral/74`. **Why open.** The distinctive agentic
  surface.

---

## Resolution log

| OQ | Decided | ADR |
|---|---|---|
| **OQ-int** | 2026-06-27 — arbitrary-precision `Int`; `Decimal` core; full native `Int8…Int64`/`UInt8…UInt64` (verbose names). `OQ-1a` (overflow default) still open. | — (recorded in `30-surface/35`) |
| **OQ-eval-strategy** | 2026-06-27 — follow Lean: lazy-WHNF + on-the-fly conversion + lazy δ over an NbE value domain extended to compute observational `Eq`/`cast`; NbE the reference. Diverges from Lean's theory on observational `J`/canonicity. | — (recorded in `10-kernel/17`) |
| **OQ-2** | 2026-06-27 — **non-cumulative** universes; ergonomics via universe polymorphism + typical ambiguity + elaborator lifts. | — (recorded in `10-kernel/12`) |
| **OQ-4** | 2026-06-27 — **observational equality (OTT), not cubical**: `Eq`-by-type + `cast` + strict-prop Ω + set-quotients; no univalence/HITs. Smallest auditable TCB; exact set-level-software fit. | **ADR 0005** |
| **OQ-Prop** | 2026-06-27 — predicative Ω; impredicativity ruled out. Proof irrelevance **definitional** via OTT's strict-prop Ω (`SProp`), free in the smaller kernel (revised by ADR 0005). | **ADR 0005** (recorded in `10-kernel/12`) |
| **OQ-η-records** | 2026-06-27 — definitional η is the **`record`/Σ class**, not `data`; safe-by-construction (records are non-recursive nested Σ), low-cost under OTT. | — (recorded in `10-kernel/14`) |
| **OQ-12** | 2026-06-27 — Kripke embedding primary; three-tier routing; **reflective proved-adequacy + verified checker (a) is the target** (intrinsic merits, not effort), reconstruction (b) a feasibility hedge; Z3 primary, cvc5 optional, **Coq retired**. | — (recorded in `20-verification/23`) |
| **OQ-spec** | 2026-06-27 — proof interface = **both, as one gradient**; **four-way epistemic status** (proved/tested/delegated/unknown) visible + exportable. `old`/state deferred → `OQ-Space` (lean explicit-state). | — (recorded in `20-verification/21`) |
| **OQ-behavioral** | 2026-06-27 — downstream complement is a **sibling** (`Ward`) fed by an assumption-boundary export; temporal obligations as **data, not kernel modalities**; one logic, two engines. | **ADR 0006** |

When an OQ is decided, record it here and, if architecturally significant, write
an ADR under `../docs/adr/` and update the affected chapters (replacing the OQ
tag with the decision).
