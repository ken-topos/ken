# WP K2c — Decidable conversion: complete the algorithm + the SCT gate

> **Status:** Steward frame — **awaiting spec-leader elaboration**. This pins the
> settled inputs, scope, and acceptance; spec-leader elaborates `spec/10-kernel/
> 17-conversion.md` to implementation-ready rigor (algorithm + SCT check to
> pseudocode, conformance seeds), then the Kernel team builds.
>
> **Team:** Kernel · **Deps:** K1 (done), K2 (in build — must merge first) ·
> **Size:** M–L · **Risk:** ★★★ (trust root; decidability is the precondition for
> the whole verification loop). **Continues the kernel spine:** K1 → K2 → **K2c**
> → K-api → G1.

## Objective

Make kernel type-checking **decidable** — always halts with yes/no. Two pieces:

1. **Complete the conversion algorithm** (`17 §1–3, §6`) to its full normative
   contract: type-directed `conv Γ A a b` with lazy weak-head normalization,
   on-the-fly structural comparison, the full η + Ω proof-irrelevance set, and
   **controlled lazy δ**. K1 + K2 already built most of this (β/Σ-β/ι/δ, Π/Σ-η,
   the obs reductions, Ω prop-arg skip); K2c **hardens it to spec 17** and closes
   any gaps the build delta surfaces.
2. **The SCT termination gate** (`17 §4`) — the net-new headline. A size-change
   termination check at **definition-admission time** that bounds δ-unfolding so
   conversion (and therefore type-checking) terminates on every well-typed input.

## Fixed inputs — settled; do NOT reopen

- **`OQ-eval-strategy` — DECIDED (operator, 2026-06-27): follow Lean's kernel.**
  The operational algorithm is **lazy WHNF + on-the-fly structural conversion +
  lazy δ-unfolding** (unfold a transparent definition only when heads differ and
  at least one is transparent; prefer *not* to unfold). **NbE is the declarative
  reference** (the meaning of "equal"), not the mandated implementation. The
  observable equality MUST be identical whichever way computed.
- **Conversion is type-directed** — `conv Γ A a b`, not `conv a b` — because η
  and proof-irrelevance depend on the type (`17 §2`).
- **η + proof-irrelevance set** (`17 §2`): Π-η, Σ-η, Unit-η/record-η, and **Ω
  proof-irrelevance** — at `P : Ω` any two terms are equal, the checker **skips
  propositional arguments** (already begun in K2's `conv_struct`).
- **SCT principle** (`17 §4`): call graph → size-change matrices (`↓` / `↓=` /
  `?` on the structural subterm order) → idempotent closure → **accept iff every
  idempotent loop strictly decreases some parameter.** Strictly more permissive
  than "structural recursion on one fixed arg" (handles permuted/lexicographic +
  mutual recursion). Eliminator recursion (`14 §3`) is already total — SCT gates
  only **general recursive δ definitions**.
- **No coinduction** (`OQ-coinduction`) — coinductive values do not arise.
- **The obs conversions** (`Eq`-by-type, `cast`-refl + cast-by-type, quotient
  equality) are **K2's** and land with it; K2c consumes them, does not re-derive.

## Scope

**IN:** the complete type-directed conversion algorithm (`17 §3`) hardened to
spec; the **SCT admission check** (`17 §4`) with the call-graph / size-change /
idempotent-closure / strict-decrease machinery; decidable level equality and Ω
proof-irrelevance in the conversion path; termination on all well-typed inputs
(`17 §5`); the `17 §6` conformance set.

**OUT:** the **content-hash O(1) fast path** (`17 §3`, non-normative perf — a
later optimization; must never change the yes/no result, so it is not on the
decidability-critical path); **full NbE normalization as the implementation** (the
recommended impl is lazy-WHNF; read-back/quote only where a syntactic normal form
is genuinely needed); the **surface-level totality policy** (offering opaque
admission for an SCT-failing def is `spec/30-surface/`, not the kernel — the
*kernel* simply refuses transparent admission it cannot certify).

## One open item for elaboration to resolve

`17 §4` carries an **(oracle)** on the exact size order: how primitives count and
how `cast` under recursion is treated. The spec's resolution path —
**primitives neutral (no strict-decrease), `cast`-under-recursion conservatively
`?`** pending validation against the reference interpreter — is the default;
spec-leader + Architect either pin it or scope the validation explicitly. The
*principle* and accept-condition are already committed and are not in question.

## Carry-forward from K2 (elaboration to scope: fold here or spin a K2-completion WP)

K2's build merged with **three obs-reduction completeness seams** deferred —
**sound today** (each falls back to a stuck/neutral term, never a wrong result),
but **incomplete** (some cast/J/quotient cases get stuck instead of reducing).
spec-leader + Architect decide whether these belong in K2c (they are
conversion-adjacent — the obs reductions feed conversion) or a separate
K2-completion WP:

1. **`cast`-at-inductive index rewrite** — `cast_at_inductive` rebuilds the
   constructor but keeps the family-index *value*, wrapping in `Cast` when the
   index changes (the suc-injectivity / index-equality seam); casting
   `Vec A n → Vec A m` of `vcons n a xs` leaves the index `n`.
2. **Non-constant-motive `J`-on-non-refl** — reduces for constant motives (the
   headline), leaves `J` neutral otherwise (the `cong`/`sym` sub-equality
   construction — the hard OTT core — is unfinished).
3. **Full quotient `respect`** — `check_respect` raw-well-forms the respect proof
   for non-Ω targets rather than verifying the full `cong`/`cast` schema (inline
   "soundness TODO"); Ω-target cases are respect-free per `16 §5` and are correct.

## Acceptance (testable — the spec-leader's conformance seeds must cover)

1. **SCT-accept:** a **lexicographic**-descent definition and a **mutually
   recursive** pair are both admitted transparent and δ-reduce. (Not just
   single-argument structural — exercise the permuted/lexicographic/mutual paths
   the criterion exists for.)
2. **SCT-reject:** a non-terminating transparent definition is **rejected** at
   admission (the kernel never admits uncertified transparent recursion).
3. **δ-heavy convertibility terminates:** a convertibility query that forces
   substantial controlled δ-unfolding halts with the correct yes/no.
4. **Full η + proof-irrelevance:** Π-η, Σ-η, Unit-η, and Ω proof-irrelevance each
   decide equality at the relevant type without comparing skipped content.
5. **Obs conversions decide:** `cast A A refl a ≡ a`, an `Eq`-by-type, and a
   quotient equality all convert correctly through the unified algorithm.
6. **Decidability:** type-checking **halts** (yes/no) on every well-typed input
   in the corpus — no loop, no semi-decision.

## Guardrails

- **The kernel is total on raw-wf input** — yes/no/typed-error, **never loops,
  never panics.** SCT is what guarantees the "never loops" half.
- **The obs equality is identical** whichever way it is computed (lazy-WHNF vs
  NbE read-back). A fast path that ever reports unequal-as-equal or vice versa is
  a soundness break.
- **Small TCB / de Bruijn criterion** (`docs/PRINCIPLES.md`): this is the trust
  root; minimize surface, justify every addition, build to the spec precisely.

## The K1 lesson — load-bearing here (COORDINATION §7)

K1 hid two soundness bugs behind a green suite that exercised only single-variable
/ closed / β-discarding paths. K2c's two highest-risk surfaces are exactly the
"degrees of freedom" kind:
- **SCT:** test **mutual** + **permuted/lexicographic** descent and a genuine
  **idempotent-loop** that decreases on a *non-first* parameter — not just
  single-argument structural recursion (which is the obvious case that hides a
  too-weak check).
- **Conversion:** exercise controlled δ where heads differ with **≥2 distinct
  transparent constants**, and the obs/η paths with **open terms + ≥2 distinct
  type/level variables**. The Architect will review the conversion + SCT
  **algorithms at pseudocode level** (it has caught soundness bugs 4× — the SCT
  accept-condition and the δ-unfold trigger are precisely where an as-implemented
  read pays off).

## Logistics

Branch `wp/K2c-conversion` cut from `origin/main` (after K2 merges). M–L — split
into a series if it helps (e.g. conversion-hardening → SCT). `scripts/ken-cargo
-p ken-kernel`. Ring: implementer builds → QA verifies independently (vary every
degree of freedom) → merge Decision (**Architect** always + **Spec**) → Integrator
→ retros. K2c/K-api boundary or SCT-design Qs → Architect; behavioral-contract Qs
→ Spec.
