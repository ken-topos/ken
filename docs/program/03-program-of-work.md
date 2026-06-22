# Program of Work

Decomposed, team-sized work packages (WPs) for **Ken** — a new, MIT-licensed,
Rust-hosted, interpreter-first verified topos language. Where a WP cites a
prototype anchor, treat it as a *spec source* — behavior to understand and
re-specify, never code to port (see `../../CLEAN-ROOM.md`).

**Conventions.** Size: S / M / L. Risk: ★ low, ★★ medium, ★★★ high (trust- or
research-critical). Every WP leaves the conformance suite green, adds its own
tests, and makes no claim a test has not confirmed. Branch per WP; merge on
green. Definition of done = acceptance criteria + docs + conformance green.

**Clean-room reminder:** any team that reads the AGPL prototype to *understand*
behavior must hand off via the spec/tests (WS-F), not by pasting code. Teams
writing Ken implementation code work from specs, not from prototype source.

---

## Work packages: definition & lifecycle

A **work package (WP)** is the unit of assignable work below: one reviewable
deliverable owned by a single team, with a stable ID (e.g. `K1`), a one-line
objective, scope, deliverables, acceptance criteria, dependencies, size
(S/M/L), and risk (★). One WP = one branch `wp/<ID>-<slug>` and one PR (a short
series for an `L`). WPs are the nodes of the dependency graph; the roadmap gates
(G0–G8, `02-roadmap.md`) are checkpoints over sets of them.

Lifecycle: **proposed** (in this catalog) → **ready** (deps merged, open
questions resolved, its gate not blocked) → **active** (pulled into a team's
ring) → **in review** (PR open, CI green, Architect/Spec reviewing) → **merged**
(by the Integrator) → **done** (acceptance criteria met; catalog + gate updated).

The **Steward** owns this catalog and cross-team sequencing — decompose, size,
sequence, track, close. The operator sets scope and priority; the Architect
advises on technical decomposition; team leaders pull *ready* WPs and run them
through the ring, proposing any newly-discovered WPs back to the Steward rather
than starting unsequenced work. See `../../agent/playbooks/federation/steward.md`.

---

## WS-F — Foundations & clean-room process (Phase 0, always-on)

### F1 — Name, license, repo, IP hygiene · S · ★
**Objective.** Stand up the new project cleanly.
**Scope.** Choose the name + check basic trademark availability; MIT `LICENSE`;
Rust workspace skeleton (kernel / elaborator / interpreter / cli crates);
contribution rules and a written **clean-room process** (who may read AGPL source,
how knowledge crosses to implementers via specs/tests only); attribution file for
permissive deps.
**Deliverables.** Repo, license, `CLEAN-ROOM.md`, `CONTRIBUTING.md`.
**Acceptance.** A new contributor can read the process and know exactly what they
may and may not look at; license is MIT; CI builds an empty workspace.
**Deps.** none.

### F2 — Spec extraction from the prototype · L · ★★
**Objective.** The legal-safe bridge: turn prototype *behavior* into a written
spec Ken is implemented against.
**Scope.** From study of the prototype + running it + its regression
*behaviors*, write: core type-theory spec (terms, types, evaluation,
conversion, universes), surface-language spec (syntax, modules, effects), and a
**conformance test corpus** (input → expected behavior) that does not embed AGPL
source. Mark every area where Ken will deliberately diverge (e.g. `Int` from day
one, checked universes, no hard slot ceiling).
**Deliverables.** `spec/` (language spec docs) + `conformance/` (black-box tests).
**Acceptance.** The spec covers the core end-to-end; conformance tests run against
the prototype binary as an oracle and pass; no AGPL source text appears in `spec/`
or `conformance/`.
**Deps.** F1. **Feeds.** all of WS-K, WS-V, WS-L.

### F3 — Architecture Decision Records · S · ★
**Objective.** Record decisions with rationale so teams don't relitigate.
**Scope.** ADRs for the locked decisions (Rust host, interpreter-first, small
permanent Rust kernel, deferred self-host) and the open ones: **content store**
(keep a hard capacity bound at all? — the prototype's 196,560 Λ₂₄ slot ceiling is
not categorically motivated; recommend an unbounded/large content-addressed store,
lattice retained only for error-correction + set-bitmap roles), concrete syntax,
effect-tracking model, and whether to keep a Space/process-isolation model.
**Deliverables.** `docs/adr/*.md`.
**Acceptance.** Every Phase-0/1 design choice has an ADR; open decisions have a
recommendation + decision owner.
**Deps.** F1. **Parallel.** F2, F4.

### F4 — Math core decision · M · ★★
**Objective.** Settle the content-addressing + any lattice math.
**Scope.** Decide reuse `mmgroup` (BSD-2, attribution) vs. reimplement the pieces
Ken actually needs; design the content-addressed equality (hash + verify) and
dedup index. Resolve the F3 content-store ADR with a concrete design.
**Deliverables.** Math-core design doc + a chosen dependency or a reimplementation
plan.
**Acceptance.** Content-addressing design is specified and benchmarked at small
scale; license provenance of any reused math is clean.
**Deps.** F1. **Parallel.** F2, F3.

---

## WS-K — Trusted kernel in Rust (Phase 1) ★ TRUST ROOT

### K1 — Core dependent type theory · L · ★★★
**Objective.** The small, correct, permanent kernel core.
**Scope.** Terms/types; **dependent** Pi, **dependent** Sigma, Id, J/path
induction; universes **with stratification checking from day one** (no
`Type:Type`); the cubical machinery Ken commits to (decide scope in F2/F3 —
transport/comp/hcomp/Glue/HITs as warranted). Keep it minimal and auditable.
**Spec source (understand, don't copy).** Prototype kernel behaviors, via the
`/spec` Team Spec produces.
**Deliverables.** `kernel` crate core + property tests.
**Acceptance.** A universe loop is rejected; dependent Sigma type-checks; J
reduces over a non-`refl` path; the conformance core subset passes.
**Deps.** F2, F3. **Note.** Highest-trust WP; small surface area is a feature.

### K2 — Proof checker & decidable conversion · L · ★★★
**Objective.** The judgement that makes a proof valid.
**Scope.** Conversion/definitional equality with **size-change termination**
gating δ-unfolding (decidable, no fuel/heuristics); the closed-term proof checker
(the thing the prover backend's certificates are checked against).
**Deliverables.** Checker + conversion in `kernel`; termination tests (inverse
functions accepted, `g(x)=x+1` rejected, certified-recursive normalizes,
non-terminating not admitted as a δ-rule).
**Acceptance.** Conversion is decidable and certified; a closed proof obligation
checks; bad proofs are rejected.
**Deps.** K1. **Parallel.** K3.

### K3 — Content-addressed value model · M · ★★
**Objective.** Values with structural O(1) equality and global dedup.
**Scope.** Implement the F4 design: content hash + verify, global dedup index,
the value representation (heterogeneously typed from day one — `Int`, float,
bool, handles, structs; no uniform-number model). No hard capacity ceiling unless
F3 decides otherwise.
**Deliverables.** `value`/runtime-core module + tests.
**Acceptance.** Identical content shares one slot; equality is one comparison on
the handle; dedup is global; no precision-losing integer model.
**Deps.** F4. **Parallel.** K1.

---

## WS-V — Verification surface (Phase 2) ★ DIFFERENTIATOR

### V0 — Minimal elaborator · M · ★★
**Objective.** Surface → kernel, enough for the vertical slice.
**Scope.** Parser for a minimal surface; elaboration to kernel terms; the glue for
the G1 slice.
**Deliverables.** `elaborator` crate (minimal) + parse/elaborate tests.
**Acceptance.** A trivial program elaborates and kernel-checks.
**Deps.** K1. **Parallel.** X1.

### V1 — Surface specification syntax · M · ★★
**Objective.** Let a programmer attach propositions to functions.
**Scope.** `ensures` postconditions, refinements on arrow domain/codomain,
standalone goals over Id/Pi; desugar to kernel obligations. This is the
"becomes a distinct language" commitment — design the vocabulary deliberately.
**Deliverables.** Grammar + AST + desugaring + a doc page.
**Acceptance.** A spec annotation parses, type-checks the spec itself, and emits a
well-formed kernel obligation; no grammar ambiguity.
**Deps.** V0. **Parallel.** L1.

### V2 — Obligation generation & body-as-motive · L · ★★★
**Objective.** Route a user function's spec into the kernel checker.
**Scope.** Generate the kernel obligation from a V1 spec; feed the function body
in as a motive; manage contexts/assumptions; reuse K2 conversion.
**Deliverables.** Obligation pipeline + a worked example proven with an explicit
proof.
**Acceptance.** **G2**: a correct postcondition proof is accepted to a runnable
artifact; a wrong one rejected.
**Deps.** K2, V1.

### V3 — Automated prover backend · L · ★★★
**Objective.** Discharge most obligations automatically and soundly.
**Scope.** Fragment classifier (decidable / first-order-intuitionistic /
higher-order); direct Z3 encoding for the decidable fragment (incl. the
content-equality theory `hash(v)=hash(v') ⇔ v=v'`); **Kripke embedding**
(validity-preserving `φ ↦ φ#`) for the FO-intuitionistic fragment so Z3 is sound
without classical rechecking; certificate checking in the K2 kernel; higher-order
deferred to interactive tactics. Enforce **oracle-not-authority**.
**Deliverables.** Classifier + encoders + Kripke translation + certificate
checker; a benchmark obligation set with an automation-rate metric; an
unsoundness test (a classical-only "theorem" is rejected).
**Acceptance.** **G3**: routing correct; unsoundness test passes; automation rate
meets the Phase-2 target.
**Deps.** V2. **Parallel.** V4, L-stream. **Note.** Riskiest WP — checkpoint after
classifier + decidable path before building the Kripke layer.

### V4 — Proof-failure diagnostics · L · ★★
**Objective.** Make every failure legible to humans and agents.
**Scope.** Countermodel extraction; **typed holes with provenance** (program still
type-checks and runs, `unknown` propagates — natural here because `unknown` is a
first-class Ω value); three-region Heyting decomposition (true / false / unknown);
suggested-actions generator. Emits the data T1 serializes.
**Deliverables.** Diagnostic structures + extraction + human-readable render +
examples per failure mode.
**Acceptance.** **G4**: all four diagnostic kinds produced on representative
failures; a partially-verified program compiles and runs.
**Deps.** V2, T1. **Parallel.** V3.

---

## WS-L — Language surface & stdlib (Phase 3, overlaps Phase 2)

### L1 — `Int` / `Decimal` · M · ★
Fixed-width integers (correct, no 2⁵³ loss) and a decimal/fixed-point type, as
first-class types distinct from float. **Deps.** K1. **Parallel.** V-stream.

### L2 — Sum types, `match`, `Result`/`Option` · L · ★★
Real constructors + eliminator + tagged runtime value; `match`/`case` with
exhaustiveness; `Result`/`Option`/`Either` in stdlib. Designed in, not retrofitted.
**Deps.** L1 (recommended). **Parallel.** L3.

### L3 — Strings & collections · M · ★
First-class strings (Unicode-aware roadmap) and the core collections (list, map,
set) over the content-addressed runtime. **Deps.** K1. **Parallel.** L2.

### L4 — Modules & package manager · M · ★★
Import/module system + a registry-less, git-based package manager (the prototype's
`yon.toml`/lockfile model is a good design reference). **Deps.** K1.

### L5 — Effect tracking · M · ★★
A statically-checked, inferred effect discipline (the prototype's `visits` is a
design reference). Wire `pure`/`impure` to FFI. **Deps.** K1.

### L6 — `Bytes` & binary I/O · M · ★
Byte-sequence type (slice/concat/hex/string conversions) + binary file ops.
Prerequisite for FFI buffers and crypto. **Deps.** K1. **Parallel.** L2.

### L7 — FFI · L · ★★★
A `foreign` mechanism to call C with marshalling for scalars/`Bytes`/handles;
`pure`/`impure` via L5; document the trust boundary (FFI can violate invariants).
**Deps.** L6. **Parallel.** L2.

### L8 — Curated stdlib · M · ★
A coherent standard library (IO, file, env, time, math, collections, crypto-via-
FFI: HMAC-SHA256/SHA-2/base64). **Deps.** L1–L3.

---

## WS-X — Execution & runtime (Phases 1/4)

### X1 — Interpreter · M · ★★
Tree-walk/bytecode interpreter = the reference semantics. Powers the vertical
slice, the REPL, and later differential testing of native codegen. **Deps.** K1.
**Parallel.** V0.

### X2 — Content-addressed runtime hardening · M · ★★
Production-grade dedup index, reclamation strategy (manual/arena/epoch — decide in
ADR), introspection hooks (occupancy/dedup-rate) exposed safely. **Deps.** K3.

### X3 — Native backend · L · ★★★
Cranelift (recommended for pure-Rust simplicity) or LLVM, **behind** the
interpreter and differential-tested against it. **Deps.** X1, L-core. **Parallel.**
X4.

### X4 — Scale & limits validation · M · ★★
Characterize content-store scale, any process/Space model, and the
deliberate/loud boundaries. **Deps.** X2. **Parallel.** X3.

---

## WS-T — Tooling & agentic interface (Phases 0/2/6)

### T1 — Machine-readable diagnostic protocol · M · ★★
Versioned JSON schema for proof status, obligations, countermodels, holes,
decomposition, suggested actions — the agent-team contract. Schema designed in
Phase 0 (alongside WS-F); emission wired to V4 in Phase 2. **Deps.** schema: F2;
emit: V4.

### T2 — REPL · L · ★★
Incremental elaboration + the interpreter + ephemeral evaluation; the *Little
Prover* loop (`prove:`/`assume:` surfacing V4 diagnostics). The interpreter-first
choice makes this natural. **Deps.** V4, X1. **Parallel.** T3, T4.

### T3 — Test / property framework · M · ★
`assert`, a runner, property-based testing over generated inputs. **Deps.** L2.
**Parallel.** T2, T4.

### T4 — Pedagogy & reference ("Little Topologist") · M · ★
Socratic intro mapping Ken's concepts to REPL exercises + an honest reference;
also the seed corpus for a language with near-zero agent priors. **Deps.** G2.

### T5 — Ecosystem seeding · S · ★
Publish a test library + one verified utility via the package manager. **Deps.**
L4, T3.

---

## WS-S — Self-hosting (Phase 5, deferred)

### S1 — Stage1 Ken-subset compiler · L · ★★★
A compiler for an Ken subset, written in Ken, atop the Rust kernel; bootstrap
chain established. **Deps.** WS-L feature-complete.

### S2 — Full self-hosted toolchain · L · ★★★
Grow S1 to the full elaborator/codegen in Ken; the trusted Rust kernel remains.
Reproduce the conformance suite from the self-hosted build. **Deps.** S1.

---

## WS-R — Research (parallel, never a gate)

- **R1 Coalgebraic layer** — Store-comonad cells, process coalgebras, profunctor
  wires, co-Heyting boundaries; design note + narrow prototype.
- **R2 Linear / affine types** — use-once resources; design note + checker
  prototype.
- **R3 Real delimited continuations** — CPS in the IR; co-Heyting account as the
  long-horizon prize.

---

## Fan-out plan for agent teams

- **Team Foundation** → F1, F3, F4, then T1-schema; supports F2.
- **Team Spec** → F2 (the oracle bridge), then conformance maintenance throughout.
- **Team Kernel** → K1 → K2 (with K3 alongside); the highest-trust spine.
- **Team Verify** → V0 → V2 → V3 (the prover; riskiest), with V1/V4 from Team
  Surface.
- **Team Surface** → V1, V4, T1-emit (specs + diagnostics + protocol).
- **Team Language** → L1 → L2 → L3 → L6 → L7 → L8 (+ L4, L5 in parallel); the
  commercial + self-host substrate.
- **Team Runtime** → K3, X1, X2, later X3/X4.
- **Team Ergo** → T2/T3/T4/T5 (Phase 6).
- **Team Research** → R1/R2/R3 with spare capacity; outputs are notes, not gates.

- **Integration (the Integrator)** → owns `main`: reviews, enforces the
  clean-room and conformance gates, merges (no other team merges), and notifies
  team leaders of fresh `main`. A single agent (recommended) with the operator
  as escalation. See `04-git-and-integration.md` for the full workflow.

Each team gets its own mootup space; the Integrator's space is linked to
all of them. PRs surface as mootup Events (PR URL as artifact), merge approvals as
mootup Decisions. Synchronization is at the roadmap gates (G0–G8): no team
advances past a gate until its acceptance criteria are met and the conformance
suite is green on a fresh checkout. The clean-room boundary (Team Spec mediates
prototype knowledge; implementation teams work from specs) holds at every step,
and is enforced mechanically at the merge gate (`04-git-and-integration.md §7`).
