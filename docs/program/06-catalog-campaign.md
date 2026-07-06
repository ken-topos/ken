# Catalog campaign — the post-core roadmap

**Owned by the Steward.** Records the roadmap decision taken after the
language-core campaign and decomposes it into sequenced work. Reads against the
operator reports in `local/` (Pat-directed, not `local/refs/`):
`core-catalog-and-agent-model-report.md`,
`native-compiler-fidelity-and-implementation-report.md`, and the Ward seam
contract (`local/ward-discharge-attestation-handoff.md`, ratified Sec6).

## The decision (2026-07-04, operator: Pat)

The language core is essentially verified (VAL2 16/0, kernel trust-root, Map
capstone, lawful classes, obs-eq termination; **effect-composition** is the one
in-flight tail). Asked which of three post-core avenues leads the next campaign,
Pat chose **catalog-led** (the Steward's recommendation). The shape:

- **Lane A — LEAD: the proof-carrying package catalog** (`core-catalog` report,
  Layers 0–14, `ken.base`→`ken.verify`). Highest readiness, lowest per-WP risk,
  most parallel; keeps the whole (idle, T2) build fleet productive now;
  and it is the enabling substrate for both other lanes.
- **Lane B — PARALLEL: Ward's ready half** — the discharge-attestation deployment
  gate + governance policy, on Ken's side of the already-**ratified** seam
  contract (Sec6). A different team's lane, so it runs alongside Lane A.
- **Lane C — DEFERRED: the native LLVM compiler** (`native-compiler` report,
  scaffold in `local/compiler/`). Biggest effort, most Rust-heavy, and *not
  capability-additive* — `ken-interp` already runs programs, so this buys speed
  and native artifacts, not new expressiveness. Sequenced as a scoped **F1/F2**
  campaign once the catalog gives it real programs to compile; **architected for
  F4/F5** from the start (the report's own warning against a hard-to-verify
  Rust island). Ward's CT-codegen ask lands here when it starts.

**Why this ordering (decide-on-merits, PRINCIPLES).** The catalog enables the
others (a compiler needs programs; Ward needs obligation-rich programs — and
catalog **Layer 12 (protocols/supply-chain)** and **Layer 14 (verify/model-check
interop)** *are literally* Ward's seam data structures). The compiler is the most
effort for capability we already have. Ward's differentiator value is real, and
its ready half parallelizes cleanly, so it runs now without leading.

## The campaign cadence (fleet fit)

The `core-catalog` report's thesis matches our fleet exactly: the **T1 enclave
pins each abstraction's boundary** (its laws, its assumptions, its exported
obligations — the hard part), then **T2 implementers fan out
mechanically** once the contract, derivation path, `trusted_base()` delta, law
propositions, and discriminating conformance cases are precise. So every catalog
WP runs the standard §2c pipeline: **Steward frame → enclave elaboration
(abstraction boundary) → merge → build team → gate**. The **first instance of
each new pattern** gets T1 design + review; siblings after it are mechanical.

Package discipline is the existing `packages/` contract (manifest, Ken source,
derivation path, declared trust delta; law fields **proved**, not postulated,
except an audited primitive-carrier delta). The catalog is a *verified
computational substrate*, not a convenience stdlib.

## Catalog quality cadence

Catalog work has two legitimate phases:

1. **Functional discovery/build.** The owning team gets the component to exist,
   run, and prove the required laws. For hard proof engineering this may leave a
   large, rough file with local helper names, sparse comments, and proof-search
   scaffolding. That is acceptable at the functional gate if the proofs are real,
   the trusted base is unchanged, and the acceptance criteria are met.
2. **Catalog refinement.** A follow-on WP raises the component to first-party
   catalog standards: organization, naming, comments, package docs,
   harmonization with sibling packages, and behavior-preserving refactor. This
   is not optional cleanup; it is how a discovered proof becomes an exemplary
   artifact.

The style standard itself is tracked as `catalog-style-guide`. Its first
application should be a small-package `catalog-refinement-pilot`, not the
largest proof-heavy body. Once the pilot proves the workflow, large components
such as maps/sets/relations get their own refinement WPs after their functional
builds land.

## Lane A — catalog WP decomposition

Sequenced against what has **landed** (Layer 0 core types + `Eq`/`DecEq`/`Ord`;
Layer 1 `collections` partial; Layer 2 **Map capstone** just merged). The first
tranche establishes the genuinely-new pattern (law-carrying classes over a type
constructor `f : Type -> Type`); later layers fan out as ready.

**First tranche (author now, elaborate after effect-composition):**

- **CAT-1 — type-constructor class pattern** *(pattern-setter, T1 design)* ·
  `Semigroup`, `Monoid` (value-level algebra) + `Functor`, `Foldable` (the first
  constructor classes). Establishes "a law-carrying class over `f : Type -> Type`"
  — the pattern every later layer leans on. Frame: `wp/CAT-1-constructor-classes`.
- **CAT-2 — Applicative / Monad / Traversable** *(depends on CAT-1)* · the harder
  constructor classes (Monad laws; Traversable's traverse/sequence coherence).
- **CAT-3 — collection laws & the `view` abstraction** *(Layer 1; depends on
  CAT-1)* · `map` length-preservation, filter membership characterization, append
  monoid + unit laws, `take`/`drop` decomposition, verified `sort`
  (sorted + permutation); and the agent-facing `view` unit (projection /
  refinement / representation / indexed / quotient-respecting / obligation-
  producing views).
- **CAT-4 — Maps / Sets / Relations laws** *(Layer 2; builds on the landed Map
  capstone)* · lookup-after-insert (same/other key), delete, union/intersection/
  difference laws, keys/values coherence, ordered invariants under `Ord`.

**Later layers — sequenced as ready** (report Layers 3–14, each a CAT-series WP
or small series, T1-pinned then fanned): parse/syntax/diagnostics (L3) ·
automata/formal-languages (L4) · graphs/dependency structures (L5) ·
statistics/probability (L6, exact/empirical/approximate tiers) · linear algebra
(L7, dimension-safe) · symbolic algebra (L8) · geometry (L9, exact-before-float)
· numerical computing (L10, error-bound refinements) · time/events/traces (L11) ·
**protocols/serialization/supply-chain (L12 — coordinates with Lane B)** ·
optimization/search (L13) · **verification/model-checker interop (L14 —
coordinates with Lane B)**. The two Ward-adjacent layers (L12, L14) are
scheduled *with* Lane B so the catalog's protocol/attestation/obligation
structures and Ward's seam stay one design.

**Fleet fan-out (Lane A).** Uses existing team ownership; the catalog is wide
enough to run several teams in parallel once patterns are blessed:
- **Language** → CAT-1/CAT-2/CAT-3 (core classes, collections, parse) — the L8
  stdlib owner.
- **Runtime** → CAT-4 (maps/sets — Map capstone was its substrate) + later
  automata/graph.
- **Foundation** → L12 protocol/supply-chain/crypto (also Lane B substrate).
- **Verify** → L14 verify-interop (also Lane B substrate) + Lane B gate.
- **Ergo** → tooling layers + the BL2/BL3 backlog (editor support, Unicode
  surface).
- **Kernel** → stays kernel/audit; assists on proof-heavy layers if a law needs
  kernel-facing review.

## Lane B — Ward's ready half (parallel)

Ken's side of the ratified discharge-attestation seam (Sec6; contract locked,
tokens pinned Ward `ffe32f2`). Build:
- the **three-check deployment gate** on the existing provenance verifier
  (signature valid + `ward.version` present; `export.hash` matches provenance =
  fail-closed revocation; each required obligation's `outcome` meets the target-
  environment requirement);
- the **governance policy** (`64`/`65`) that says what each environment demands
  (Ken owns the *requirement*; Ward specifies only the gate's *check*);
- honor the hard **I4 one-way gate** (no `outcome` promotes `T`→`proved`; no
  correctness judgment reads a Ward-internal field; no fifth outcome) with a
  discriminating conformance case.

Owner: **Foundation** (Sec3 supply-chain) + **Verify** (B-series export). *First
step is a readiness check of what B1–B4 / Sec3 / Sec6 already landed* before
framing the gate WP — sizing TBD. Ward itself (the discharge engine side) is a
separate sibling federation (`ken-topos/ward`); "focus on Ward" at the engine
level is a distinct, larger question to route back to Pat if he wants it.

## Lane C — native compiler (deferred, pre-scaffolded)

Held until the catalog gives it programs and the semantics are fully settled.
When it starts: a pragmatic **F1/F2** first campaign (executable IR +
representation model → Rust LLVM backend for a small total subset → runtime
layout/ABI → interp/native differential harness → trust-report artifact →
erasure/closure/dictionary lowering boundaries), **architected as if F4/F5 is
coming** (Ken owns semantics/IR-contracts/certificates/trust-reports; Rust owns
LLVM/ABI/linker/runtime). Scaffold already exists in `local/compiler/` (spec
01–04 + a program of work) and maps to catalog `X3 — Native backend`. Ward's
best-effort **CT-preserving codegen** obligation (`45`/`OQ-backend-target`) folds
in here.

## Sequencing / next actions

1. **Effect-composition finishes first** (in-flight enclave elaboration) — it is
   the core's tail and it exercises the Monad/effect interplay CAT-1/CAT-2 build
   on, so it lands before the catalog constructor classes.
2. **CAT-1 frame authored + queued** (`wp/CAT-1-constructor-classes`); the enclave
   picks it up for elaboration once effect-composition merges and it is compacted
   at that seam (§2c). Then Language builds it; CAT-2/CAT-3 fan behind it.
3. **SURF-1 — purity keywords + effect-row polymorphism** (`wp/purity-keywords-
   effect-polymorphism`, framed) — a surface-ergonomics WP settled by the operator
   (2026-07-04): retire `view` for `const` (zero-param pure) / `fn` (pure function)
   / `proc` (potentially impure, incl. effect-polymorphic), checked bidirectionally;
   **pin effect-row
   polymorphism** in `spec/36` (the technical core); fold BL3 (Unicode surface).
   **Sequenced after CAT-1, before CAT-2** — Traversable's `traverse` is the first
   effect-polymorphic surface definition, so the row-poly pin gates CAT-2. Enclave
   elaborates (Architect grounds the row-variable mechanism) → Language builds;
   existing `.ken` (incl. CAT-1) migrate by the checker's own purity inference.
4. **Lane B readiness check** (B1–B4 / Sec3 / Sec6 landedness) → frame the Ward
   deployment-gate WP for Foundation/Verify in parallel.
5. **Epoch push:** this plan + the tracker resync go to `main` bundled into the
   next catalog merge cycle (CAT-1 elaboration) — no lone tracker cycle.
