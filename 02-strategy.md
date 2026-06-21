# Strategy: a Verified Topos Language for Agentic Development

> **Ken** is a **new, distinct, MIT-licensed language** — a clean-room
> reimplementation that uses the AGPLv3 Yon fork only as a *reference prototype*,
> never as a code basis.

**Premise.** Agents can already write working — even high-quality — code. The
binding constraint on *deploying* agent-written code is **verification**: an
ethical obligation to assert that code does what it intends without putting users
at risk. Industry's answers are empirical (tests) and the shift to stricter type
systems (Rust, TypeScript). At the far end, dependent-type systems (Lean 4, F*,
Coq) offer propositional correctness but are aimed at mathematicians, not
commercial software.

**Thesis.** A topos-oriented language whose dependent kernel is exposed to the
surface, whose proof obligations are discharged automatically, and whose proof
*failures* are legible and machine-readable could occupy the empty middle: a
language an agent can write **and prove correct**, at commercial scale, under a
permissive license with a small auditable trust root.

The Yon prototype demonstrated the core ideas are buildable (a computing
dependent + cubical kernel, content-addressing, mechanical Yoneda checking). Ken
takes the validated *design* and reimplements it cleanly, free of the prototype's
license, name, soundness debt, and accreted constraints.

Read `01-reality-check.md` first (what the prototype actually does). This
document defines the goal, the clean-room ground rules, the locked architecture,
and the workstreams. Sequencing is in `03-roadmap.md`; team-sized packages in
`04-program-of-work.md`.

---

## 1. Clean-room ground rules (non-negotiable)

The MIT goal makes these load-bearing, not optional hygiene. (Not legal advice;
confirm with IP counsel before Phase 1.)

1. **No relicensing.** AGPLv3 code cannot be relicensed to MIT. Ken is a separate
   work, written from specifications — not a port.
2. **Reusable from the prototype:** language *design* and semantics, the
   topos/HoTT approach, content-addressing as identity, and all mathematics
   (Leech Λ₂₄, Golay, Co₀, hashing). Ideas, methods, and math are not
   copyrightable. Interfaces are defensible to reimplement.
3. **Not reusable:** copying or close paraphrase of AGPL source. The reality
   check's `file:line` excerpts are commentary, not implementation input.
4. **Process:** derive a written behavioral **spec + conformance test corpus**
   from the prototype (docs, observed behavior, the regression suite's
   *behaviors*), then implement from the spec. Keep AGPL source out of the
   context of whatever writes Ken's code.
5. **Dependencies:** `mmgroup` (BSD-2) is reusable with attribution, or
   reimplement; LLVM/Cranelift are permissive. Prefer permissive or
   reimplemented math.
6. **Identity:** new name and new trademark posture; do not invoke "Yon."

---

## 2. Locked architecture decisions (2026-06-21)

| Decision | Choice | Rationale |
|---|---|---|
| Host language | **Rust** | Memory-safe, good for a long-lived toolchain, strong ecosystem. |
| Initial backend | **Interpreter-first** (tree-walk/bytecode in Rust) | Fastest path to a working typecheck + prover + agent loop; prove the thesis before investing in native codegen. Native codegen (Cranelift/LLVM) added later behind the interpreter, which remains the reference semantics. |
| Trust boundary | **Small permanent Rust kernel** | The proof/type checker stays small and in Rust forever (Lean's C++-kernel model; de Bruijn criterion). Elaborator, prover, codegen build on top and may self-host later; the kernel does not. |
| Self-hosting | **Deferred** | Build a complete Rust-hosted reference first; get the write→spec→verify→repair loop credible; self-host once the language can host a compiler. Self-hosting early competes with the differentiator. |

Open design decisions (ADRs, Phase 0): the content-addressed value model
(whether to keep a hard per-store capacity bound at all — the prototype's 196,560
Λ₂₄ slot ceiling is *not* categorically motivated, per the analysis, and a clean
design may prefer an unbounded/large content store with the lattice kept only for
the roles that earn it: error-correction and set bitmaps); concrete syntax;
effect-tracking surface; the Space/process-isolation model.

---

## 3. Goal and success criteria

**North star.** An agent submits an Ken function plus a specification
(pre/postconditions, refinements, a propositional goal). The toolchain returns
*proved*, *disproved with a concrete countermodel*, or *incomplete with a typed
hole and structured next-step guidance* — actionable without reading the kernel.

Gates (objective, testable; tied to roadmap phases):

1. **G1 — Vertical slice.** parse → elaborate → kernel-check → interpret runs a
   trivial program *and* checks one trivial proof, end-to-end, in Rust.
2. **G2 — Surface proof.** A programmer writes `ensures`/refinement on a real
   function; the checker accepts a correct proof and rejects a wrong one.
3. **G3 — Automation.** A meaningful fraction of obligations discharge
   automatically, with sound handling of the intuitionistic/classical boundary
   (oracle-not-authority).
4. **G4 — Legibility.** Every failed obligation yields a structured, agent- and
   human-readable diagnostic (countermodel / hole / three-region decomposition /
   suggested actions).
5. **G5 — Soundness.** A documented kernel-soundness story holds: universe
   checking from day one (no `Type:Type`), decidable certified conversion,
   meaning-preserving evaluation. The kernel is small enough to audit.
6. **G6 — Commercial reach.** One realistic component is written and verified
   end-to-end with real types (`Int`, sum types, `Bytes`) and real I/O (FFI),
   with at least one correctness property *proved*.
7. **G7 — Agent loop.** The agent-team software drives the
   write→spec→verify→repair loop unattended on a non-trivial task via the
   machine-readable protocol.
8. **G8 — Self-hosting** (later): Ken's elaborator/compiler is rewritten in Ken
   atop the permanent Rust kernel.

**Non-goals (commercial track):** full higher-order automated proving (interactive
tactics instead); native codegen before the verification loop works; the
coalgebraic-layer research program; linear types; delimited continuations;
reproducing the prototype's f64-number-only model (Ken has `Int` from day one) or
its unchecked universes or its hard slot ceiling.

---

## 4. Workstreams

Eight workstreams. F is the always-on foundation; K→V→L→X is the build spine; T
is ergonomics/agent interface; S and R are deferred/parallel.

### WS-F — Foundations, clean-room process, governance (always on)
Name, MIT license setup, IP hygiene, repo scaffolding (Rust workspace), ADRs, and
— critically — **spec extraction**: turn the reality-check knowledge into a
written language spec + conformance test corpus that Ken is implemented against.
This is the legal-safe bridge from prototype to new code.

### WS-K — Trusted kernel (Rust) ★ trust root
The small permanent core: core dependent type theory (Pi, dependent Sigma, Id, J,
universes **with checking**), the proof checker, decidable conversion with
size-change termination, and the content-addressed value model. Designed correct
from the start — the prototype's soundness gaps (unchecked universes,
non-dependent Sigma, J-only-on-refl) are simply not reproduced. Spec'd and,
later, a candidate for formal verification of the kernel itself.

### WS-V — Verification surface (the thesis) ★ differentiator
Surface specification syntax (`ensures`, refinements, goals); obligation
generation (function-body-as-motive plumbing); the automated prover backend
(fragment classifier; direct Z3 for decidable obligations; **Kripke embedding**
for the first-order intuitionistic fragment so a classical SMT solver is used
soundly — Ken's topos semantics *are* Kripke semantics, making the embedding
native; certificates re-checked in the kernel); and proof-failure diagnostics
(countermodels, typed holes with `unknown` propagation, three-region Heyting
decomposition).

### WS-L — Language surface & stdlib
The commercial surface, which is *also* the self-hosting substrate, so it is core,
not a late bolt-on: `Int`/`Decimal` (designed in, no f64-int legacy), sum types +
`match` + exhaustiveness + `Result`/`Option`, strings/collections, modules +
package manager, effect tracking, `Bytes`/binary I/O, FFI, and a curated stdlib.

### WS-X — Execution & runtime
The interpreter (reference semantics) first; the content-addressed runtime
(structural O(1) equality, global dedup) reimplemented cleanly; later a native
backend (Cranelift or LLVM) behind the interpreter; scale/limits validation.

### WS-T — Tooling & agentic interface
The machine-readable diagnostic protocol (the agent-team contract); a REPL
(interpreter makes this natural — the *Little Prover* loop); a test/property
framework; and honest pedagogy/reference docs (also the seed training corpus for
a language with near-zero priors).

### WS-S — Self-hosting (deferred)
Once WS-L is feature-complete: rewrite elaborator/parser/codegen in Ken atop the
Rust kernel; bootstrap chain (Stage0 Rust reference → Stage1 Ken subset → full).

### WS-R — Research (parallel, never a gate)
The analysis's deep material: coalgebraic layer (Store-comonad cells, process
coalgebras, profunctor wires, co-Heyting boundaries), linear/affine types,
real delimited continuations. Design notes + prototypes; harvest pragmatic wins
back as normal packages.

---

## 5. The honest scale

Clean-room reimplementation **+** dependent-type verification **+** (eventual)
self-hosting is a *new language project* — a multi-year program, not an
enhancement. Two things make it tractable: (1) it is unusually well-suited to
agent teams — greenfield, spec-driven, with the prototype as a behavioral oracle;
and (2) the locked architecture de-risks it — an interpreter-first Rust reference
with a small kernel gets the differentiating verification loop working long
before any heavy codegen or self-hosting investment. The discipline that matters
most: ship a thin vertical slice early (G1), then widen — and never let WS-R or
the elegance of total generality displace the focused thesis (WS-V).
