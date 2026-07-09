# Catalog campaign — charter and roadmap

**Owned by the Steward.** Records what the first-party catalog *is*, who owns
it, how each entry is shaped, and the sequenced roadmap that builds it. Reads
against the operator reports in `local/` (Pat-directed, not `local/refs/`):
`core-catalog-and-agent-model-report.md`,
`native-compiler-fidelity-and-implementation-report.md`, and the Ward seam
contract (`local/ward-discharge-attestation-handoff.md`, ratified Sec6).

The first pass (roadmap in the *Roadmap* section) established the core
proof-carrying components and smoke-tested the kernel, elaborator, and language
surface — the pass that caught the early kernel-reduction defects. This charter
reframes the catalog for the phase now beginning: the same components, but
authored deliberately for the audiences and uses below, in a literate format,
under a clear home.

## What the catalog is — four purposes at once

The catalog is not only where reusable Ken code lives. It serves four purposes
simultaneously, and the charter's job is to make one artifact serve all four
rather than fracturing into four corpora.

**1. The standard components** — the verified substrate from which software in
Ken is built. *Personas:* agents building with Ken (which will **not** have Ken
in their training data in the near term) and people learning Ken by building
with it. Both read the *types and laws as the contract* — in a dependently- and
refinement-typed language, a `def` with named preconditions and a lawful
`class` instance with its proof are self-describing. This is what a
Ken-untrained agent leans on.

**2. Training data** for future models to understand Ken. *Persona:* AI labs
(their exact needs are not yet legible to us). Our working thesis: the scarcest,
highest-value form of code training data is **verified-correct,
intent-annotated, proof-carrying code that is not already on the internet** —
and the catalog is all four by construction (machine-checked, literate,
proof-carrying, novel). So we do **not** chase a spec we cannot get from labs;
we make correct, literate, proven components, and premium training data is the
byproduct.

**3. A teaching tool** for understanding Ken and programming in Ken. *Personas
(the widest set):* type-theory researchers, users of other dependently-typed or
functional languages, the type-theory-curious, experienced programmers, math or
CS students, entry-level programmers. One entry serves this whole range through
**progressive disclosure** — the same document read to different depths.

**4. (inward) The fleet's dogfooding instrument.** Writing real Ken is the only
way to surface what synthetic tests cannot: **kernel-reduction defects** (the
first pass already caught several) and **elaborator ergonomics** — recurring
implementation shapes that should be *sugared* into surface syntax, or that
should become *general-purpose `def`s, `lemma`s, or `prop`s*. These **Findings**
are a first-class output of every entry, not a side effect (routing below).

### Why one artifact serves all four

The four purposes are colinear, not competing, when the entry is a **literate
`.ken.md` document**:

- Purpose 1 needs self-describing components — the literate entry's code +
  laws + proofs.
- Purpose 3 needs progressive disclosure — the literate entry's layered
  sections and named reading paths.
- Purpose 2 is what purposes 1+3 *produce* when done well: verified + literate
  + proof-carrying + novel.
- Purpose 4 (Findings) falls out of the act of authoring, captured in a
  standing section.

The per-entry standard format that carries these layers is the subject of
`07-catalog-style-guide.md`. This charter fixes the *purpose, home, and layout*;
the style guide fixes the *shape of each entry*.

## Sections and Domains — the catalog's subject-matter architecture

The catalog's first-level division is the **Section**. Sections are dependency-
ordered: each is built from the ones before it, and the essential toolkit at the
base is the most depended-upon. A Section may be subdivided, along clear
subject-area lines, into **Domains** (e.g. the Capability Section holds a
Cryptography Domain, a Parsing Domain, …). This is the *subject-matter* spine —
orthogonal to the trust "rings" (kernel TCB vs. outer ring) and to the per-entry
format. Which Sections exist, and which package sits where, will see
**rearrangement, inclusions, and exclusions** as the catalog grows; what is
stable is the shape: a small essential core, widening toward applications.

**For now there are four Sections** — Core, Data, Algorithm, Capability. That is
deliberately enough to keep the campaign busy; later subject areas (encodings and
protocols, application frameworks) become their own Sections when we reach them.

```mermaid
flowchart TB
  Core[Core Section - essential dependent-programming toolkit] --> Data[Data Section - standard datatypes and operations]
  Data --> Algorithm[Algorithm Section - general algorithms over the data]
  Algorithm --> Capability[Capability Section - focused Domains: parsing, cryptography, ...]
```

- **Core Section — the essential dependent/functional-programming toolkit.** The
  vocabulary of proof and abstraction: propositional equality and its lemmas
  (refl/sym/trans/cong/subst), decidability (`Dec`), dependent pairs and
  functions, sum/product/`Unit`/`Void`, and the lawful type-class scaffolding
  (`Eq`, `Ord`, `Semigroup`, `Monoid`, `Functor`/`Applicative`/`Monad`). This is
  what a Ken-untrained agent leans on first, and what everything above reuses.
- **Data Section — standard datatypes and their operations.** `Nat`, `Int`,
  `Bool`, `Char`, `String`, `List`, `Vector`, `Option`, `Result`/`Either`,
  tuples, `Map`, `Set` — each with its operations **and its laws proved**.
- **Algorithm Section — general-purpose algorithms over the data.** Sorting,
  searching, traversal, graph and numeric algorithms — reusable procedures that
  operate on the Data Section's structures, distinct from an
  application-facing competence.
- **Capability Section — focused competence Domains.** Parsing, cryptography, and
  the individual subject areas that build on solid data and algorithms; each is a
  **Domain** within this Section (Parsing Domain, Cryptography Domain, …). Today's
  `parsing` and `transport` packages seed this Section.

**Demand-pull (the operator's design principle).** The deeper Sections are
*clarified by building the things that ought to sit on them*. Rather than
speculate an exhaustive Core/Data Section in the abstract, we let a concrete
higher-Section target (a real parser, a real codec) surface the exact core lemma
or data operation it needs, then land that below. Build-order is therefore
**top-informed, bottom-proven**: higher targets specify the requirements; the
Sections are proven from the base up. This charter's near-term work program
(`Roadmap` → *Data-structures enrichment*) drives the catalog through the Core
and Data Sections under exactly this discipline.

The `core-catalog` report's finer Layers 0–14 (`ken.base` → `ken.verify`) slot
*within* the Sections — the Sections are the coarse spine, the report's layers
the fine sequencing inside the Algorithm and Capability Sections.

## Layout: the `catalog/` tree

The catalog gets a top-level home. Whole-catalog matter lives at the `catalog/`
root; the package tree is a light container beneath it.

```text
catalog/
  README.md            catalog index + the four purposes, one screen
  REFERENCES.md        catalog-wide reference conventions (per-entry refs live
                       in each entry — see the style guide)
  guide/               the authoring guide — "writing Ken" (see below)
  packages/            light container: a README + one subdirectory per package
    README.md          package index / navigation
    <package>/
      <package>.ken.md  the literate entry (primary artifact; tangles to .ken)
      ...
```

- `catalog/` root holds any *whole-catalog* detail (index, cross-package
  conventions, the pointer to this charter and to the style guide).
- `catalog/packages/` is **just a container** — a README and the package
  subdirectories, nothing heavier. Packages are physically **flat today**; the
  Sections and Domains below are the *conceptual* spine and build order, and a
  later rearrangement WP may group packages into Section (and, where subdivided,
  Domain) subdirectories once the set is large enough to warrant it.
- Each package is a **literate `.ken.md` entry** whose `ken` code blocks tangle
  to a compilable module; the tangled `.ken` is a build artifact, not the
  source of truth.

The migration that moved today's `packages/` to `catalog/packages/` has landed
(it touched build/tooling references — elaborator package resolution,
`crates/**` test fixtures, ~70 docs, conformance seeds — so it went through CI,
not by hand).

## Home and Findings routing (teaming)

The reframed campaign's core artifact is *proven `catalog/packages/`
components in `.ken.md`* — **Foundation's** standing mandate (Foundation
already builds the `catalog/packages/` stdlib; the first pass used Language
because it was a *surface* smoke-test, a Language concern). So the catalog is
homed in **Foundation**, and
the Findings loop is honest by construction because the *author* and the
*fixers* are different teams — the surface builder cannot grade its own
ergonomics homework.

```mermaid
flowchart LR
  F[Foundation<br/>authors catalog as Ken's first user] -->|kernel-reduction defect| K[Kernel<br/>via enclave]
  F -->|sugar / abstraction candidate| E[Ergo<br/>triages ergonomics]
  E -->|greenlit surface sugar| L[Language<br/>implements surface]
  F -->|abstraction candidate kept in-catalog| F
```

- **Foundation** authors entries and files Findings.
- **Kernel** (via the enclave) takes kernel-reduction defects — the
  highest-value Finding a catalog entry can produce.
- **Ergo** triages ergonomics: sugar candidates and abstraction candidates.
- **Language** implements the surface sugar Ergo greenlights.
- **Enclave** (Architect/CV) pins each abstraction boundary and gates merges,
  per the standard §2c pipeline.

The one skill no team has yet — literate-`.ken.md` pedagogy plus Findings-filing
discipline — is a **catalog-authoring overlay** attached to Foundation
(`agent/teams/foundation/` or a shared skill), not a new team. A new team would
be archetype-identical and need the same overlay anyway; minting one is
proliferation against `subsume-don't-proliferate`.

The **staffing cadence** stays demand-driven: run Foundation's cell on catalog
batches; if observed throughput later justifies a standing catalog cell,
graduate it then, informed.

## The authoring guide — "writing Ken" (parallel workstream)

There is **no in-model support for Ken** — no model has Ken in its training
data, and won't for some time. The catalog shows *proven components*; it does
not, by itself, teach the **act of writing** them. So the campaign carries a
second,
parallel deliverable: an **authoring guide** — reference material that helps an
agent (ours, and hopefully others') or a person actually write Ken. It lives at
`catalog/guide/` and is developed **alongside** the packages, not after them.

It is not a fifth *purpose* — it is a deliverable that serves purposes 1
(builders lean on it), 3 (it teaches), and 2 (how-to-write-Ken reasoning is
itself premium, not-on-the-internet training data). It complements the normative
spec: **`spec/30-surface` is the contract; the guide is the practice.**

Three strands, synthesized from what we already have — the landed language
surface plus what is generally known about writing dependently-typed code —
never by copying reference source (clean-room boundary below):

- **Surface reference** — the practical shape of the language: the
  `const`/`fn`/`proc` purity split, `data`/`match`, `class`/`instance`,
  refinement types, effect rows, and the literate `.ken.md` format. Task-first
  ("how do I write X"), distinct from the spec's exhaustive contract.
- **Proof techniques** — how to actually discharge laws in Ken: `refl` vs. `tt`
  endpoints, induction and motive construction, using `Dec`, funext as a
  definitional pointwise equality, and the non-termination hazards to avoid.
- **Decomposition & abstraction hints** — when to reach for a `class` vs. an
  explicit dictionary, `subsume-don't-proliferate`, coexist-when-trust-differs,
  structural self-evidence, and the other reusable moves.

**A high-value, honest synthesis source is the fleet's own hard-won memory.**
`agent/memory/` and the Steward's operating memory already encode much of the
proof-technique and decomposition strand as lessons paid for in real build
failures — distilling those *outward* into public guide prose is both the
cheapest first draft and the most authentic. General dependently-typed practice
(Lean/Agda/Idris tactics and patterns, all widely documented in public) may be
consulted to *sharpen* the guide, but it is written in Ken's own terms and
**never copies reference source** — the same clean-room rule the catalog code
obeys (`CLEAN-ROOM.md`): permissive references inform *approach*, copyleft
references are enclave-only, and neither is transcribed. The guide is a
companion to the catalog, so its Findings and refinement cadence mirror the
packages'.

Our own agents consume the guide through a thin role skill that points at it;
the canonical artifact is the repo-visible `catalog/guide/` so it serves
external readers and the training-data purpose equally.

## Retro discipline — catalog retros are acted on

Across the rest of this project, per-WP retros are logged in the space and left
there; that is fine for the build. **Catalog WPs are the exception: their retros
are acted on**, because acting on them *is* the campaign's inward purpose
(dogfooding, purpose 4). This is an explicit part of every catalog WP's retro
instructions, and the Steward tracks the follow-through. At each catalog WP
closeout, the retro must surface and route concrete actions:

- **Into the writing skill and materials.** Anything the authoring taught about
  *how to write Ken well* — a clearer proof technique, a decomposition that
  worked, a pitfall — folds back into `catalog/guide/` and the `write-ken` skill.
  The guide improves from every entry authored against it.
- **Language surface.** A recurring shape that wants sugar → Ergo triages →
  Language implements (the Findings routing above).
- **Elaborator ergonomics.** A confusing error or a manual step the elaborator
  could do → Ergo.
- **Useful `def`s / `lemma`s / `prop`s.** A helper or law that proved reusable is
  promoted into the catalog itself as a general-purpose entry (typically the Core
  Section), not left local to one package.
- **Kernel-reduction defects** → Kernel via the enclave (the highest-value
  Finding; already in the routing above).

A catalog WP is not closed until its retro's actions are captured — filed to the
right team, or booked as a follow-on entry. The retro is a source of work, not an
archived note.

## Cadence (fleet fit)

Unchanged spine: the **T1 enclave pins each abstraction's boundary** (its laws,
assumptions, exported obligations — the hard part), then **T2 implementers fan
out** once the contract, derivation path, `trusted_base()` delta, law
propositions, and discriminating conformance cases are precise. Every catalog WP
runs the §2c pipeline: **Steward frame → enclave elaboration (abstraction
boundary) → merge → build team → gate**. The **first instance of each new
pattern** gets T1 design + review; siblings are mechanical.

Package discipline is the existing `catalog/packages/` contract (manifest, Ken
source, derivation path, declared trust delta; law fields **proved**, not
postulated, except an audited primitive-carrier delta) — now carried inside the
literate entry per the style guide. The catalog is a *verified computational
substrate*, not a convenience stdlib.

### Two-phase quality cadence

Catalog work has two legitimate, named phases, because hard proof engineering
often discovers the proof before the clearest presentation of it.

1. **Functional discovery/build.** Get the component to exist, run, and prove
   the required laws. A rough-but-correct source may merge here: local helper
   names, sparse comments, discovery-shaped organization are acceptable **if**
   the proofs are real, the derivation path is stated, the trusted-base delta is
   honest, and the WP's acceptance criteria are met.
2. **Catalog refinement.** A follow-on WP raises the landed component to the
   standard entry format: literate narrative, reading paths, examples, laws,
   References, Findings, naming, and behavior-preserving refactor. This is a
   planned step, not optional cleanup, and it does not weaken proof obligations.

The durable standard is `07-catalog-style-guide.md`. The Steward records a
refinement follow-on for any component whose entry is not yet guide-quality.

## Roadmap

Sequenced along the Sections above (the `core-catalog` report's Layers 0–14,
`ken.base` → `ken.verify`, slot within them). The **core-establishing tranche is
largely complete** — the constructor-class pattern, collections,
maps/sets/relations, parsing, lawful classes, the purity-keyword surface split,
and named-proof claims. The reframe above changes the catalog's *purpose,
format, home, and layout* for the phase now beginning.

**Near-term: the data-structures enrichment program.** The first program of the
reframed phase drives the catalog deliberately through the **Core Section**
(essential toolkit) and **Data Section** (standard datatypes + operations) under
the demand-pull discipline — detailed in its own program doc
(`docs/program/wp/catalog-data-structures-program.md`). Beyond the Data Section,
the remaining Sections/layers sequence as ready:

parse/syntax/diagnostics · automata/formal-languages · graphs/dependency
structures · statistics/probability (exact/empirical/approximate tiers) · linear
algebra (dimension-safe) · symbolic algebra · geometry (exact-before-float) ·
numerical computing (error-bound refinements) · time/events/traces ·
**protocols/serialization/supply-chain (coordinates with Lane B)** ·
optimization/search · **verification/model-checker interop (coordinates with
Lane B)**. The two Ward-adjacent layers are scheduled *with* Lane B so the
catalog's protocol/attestation/obligation structures and Ward's seam stay one
design.

### Deferred Z3 evaluation gate

Z3 remains an optional proof-search accelerator, not a trusted checker and not a
dependency for current builds. Defer until the catalog contains enough large,
proof-heavy packages that an enabled/disabled comparison is meaningful, then run
the two-step program in `03-program-of-work.md` under V3 (integrate an
off-by-default Z3-backed search whose results the kernel still re-checks; then
characterize throughput). Output is a keep/opt-in/remove decision report. Do not
default Z3 unless catalog-scale measurement shows a clear benefit.

### Lanes B and C (unchanged)

- **Lane B — Ward's ready half (parallel).** Ken's side of the ratified
  discharge-attestation seam (Sec6; tokens pinned Ward `ffe32f2`): the
  three-check deployment gate on the provenance verifier, the `64`/`65`
  governance policy (Ken owns the *requirement*, Ward the *check*), honoring the
  I4 one-way gate with a discriminating conformance case. Owner: **Foundation**
  (Sec3) + **Verify** (B-series). First step is a readiness check of what
  B1–B4/Sec3/Sec6 already landed before framing the gate WP.
- **Lane C — native compiler (deferred, pre-scaffolded).** Held until the
  catalog gives it programs and semantics are settled. A pragmatic F1/F2 first
  campaign (executable IR → Rust LLVM backend for a small total subset →
  layout/ABI → interp/native differential harness → trust-report), architected
  as if F4/F5 is coming (Ken owns semantics/IR/certificates; Rust owns
  LLVM/ABI/runtime). Scaffold in `local/compiler/`. Ward's CT-preserving codegen
  obligation folds in here.

## Sequenced next actions

The reframe itself has **landed**: charter (`06`) + standard entry format
(`07`), the `packages/` → `catalog/packages/` migration, and the checked
literate fence roles (`ken reject`/`ken example`) are all on `main`. The phase
now beginning:

1. **Initial WP — Ken reference materials + writing skill** (before the campaign
   proper). Stand up the first version of `catalog/guide/` and the `write-ken`
   skill, so every later catalog WP is authored against a real guide and its
   retros have somewhere to fold improvements back into. Frame in
   `docs/program/wp/ken-authoring-guide.md`. This is the keystone: the campaign
   both *uses* it (to write well) and *feeds* it (via the retro discipline).
2. **Foundation catalog-authoring overlay** — the literate-`.ken.md` pedagogy +
   Findings-filing skill attached to Foundation; the precondition for authoring
   the first batch to guide quality (may bundle with the initial WP).
3. **Data-structures enrichment program** — the near-term program of WPs driving
   the catalog through the Core and Data Sections under demand-pull; sequence and
   rationale in `docs/program/wp/catalog-data-structures-program.md`. Its first
   WP (`DS-1`, `Empty`+`Dec`) doubles as the **`.ken.md` format pilot** — no
   literate entry exists yet.
