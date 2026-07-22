# 12 — The documentation program (`library/`)

**Status:** Framed 2026-07-21. **Wave 0 active** — released to the doc team.
**Owner:** the **doc team** (§0); Steward (frame, sequencing, gates).
**Source proposal:** `research/librarian-documentation-program-proposal.md`
(Research, 2026-07-18).

Ken gets a durable product-documentation portal at `library/`, organized by
**reader need** rather than by the teams that built the repository, with the
primary learning path being **reading Ken** rather than writing it.

The research proposal is the design and I am not restating it. This document
is the **frame**: it settles the four decisions the proposal routes to the
Steward, states what binds the Librarian, and releases Wave 0.

---

## 0. The doc team (operator, 2026-07-21)

Documentation is produced by a **three-seat team on the standard build
archetype**, not by a solo Librarian:

| seat | tier | skill | role |
|---|---|---|---|
| `doc-leader` | T2 | `ken-build-leader` + `agent/teams/doc/leader.md` | scoping, sequencing, kickoffs, merge Decisions |
| `doc-author` | T2 | `ken-build-implementer` + `agent/teams/doc/implementer.md` | authoring |
| `librarian` | **T1** | `ken-librarian` | editor, fact-checker, reviewer — **the team's QA** — plus a standing as-built mandate |

**★ The judgment is concentrated on the reviewing end, not the authoring
end** (operator, 2026-07-21). Every other unit in the fleet puts its most
capable seat on production; this one puts it on review. That is deliberate:
the failure mode for documentation is not *badly written* but *confidently
wrong* — a page whose cited evidence does not carry its claim reads perfectly
and is worth less than nothing, because it still looks authoritative. Catching
that is a grounding problem, which is where T1 pays. Prose quality is not.

**Why the archetype rather than a bespoke unit.** The doc team inherits
`COORDINATION.md` wholesale — WP lifecycle, the handoff gate, review and merge
flow, retros. The Librarian had a playbook but **no place in federation law**;
routing doc work through the existing team shape closes that without writing
new law, and the overlays carry only what is doc-specific.

**Why the seats are split this way.** Scoping and verification sit in
different seats *on purpose*. The seat that reviews `library/` also edits it,
so the Librarian's approval is not the independent check a build QA's is —
**the gates are the independent oracle.** That is why §3's "prove every gate
fails on a planted violation" is an acceptance criterion and not a nicety: it
is the only check here that cannot be talked into agreeing with the party that
scoped the work.

**The doc track runs CONCURRENTLY with build work** — the one standing
exception to the fleet's single-threaded posture, granted on the basis that
doc WPs touch `library/` and `agent/` rather than `crates/`. **The exception
is contention-free-ness, not priority:** a doc WP that would touch a path a
build WP holds defers and routes to the Steward.

---

## 1. The four decisions — SETTLED

The proposal asks the Steward or operator to ratify four choices before the
Librarian migrates anything. All four follow from `docs/PRINCIPLES.md` and
from ownership I already hold, so they are settled here rather than sent up.

### D1. `library/` is explanatory and derived. `spec/` remains the sole normative authority. ✅ CONFIRMED

`library/` **must not introduce normative language.** Where a reference page
restates a rule for usability it cites the exact spec section, and a drift
gate verifies that section still exists.

This is the load-bearing decision. A polished duplicate that can drift is
worse than an incomplete page that names its source — *honesty about the
boundary*, applied to documentation.

### D2. Migration is subsumptive. `catalog/guide/` moves; it does not persist alongside. ✅ CONFIRMED

*Subsume-don't-proliferate.* Ken has no compatibility obligation to anyone
and therefore no reason to maintain parallel documentation forms during
initial development. `catalog/guide/`'s human authoring material moves into
`library/learn/`, `library/guide/`, and `library/how-to/`, leaving
**pointers, not a second maintained guide.**

The four current files (`README.md`, `decomposition-abstraction.ken.md`,
`proof-techniques.ken.md`, `surface-reference.ken.md`) are literate `.ken.md`
and are **checked**. Migration must not silently drop that checking — see §3.

### D3. `library/agents/` holds PRODUCT context only. Workflow and fleet practice stay under `agent/`, Steward-owned. ✅ CONFIRMED

The boundary is: **what Ken is** vs. **how this federation works.** Roles,
merge flow, model routing, WP lifecycle, the memory corpus, and the
compaction discipline are `agent/` and remain mine. How to read a Ken
program, what `tt` vs. `Refl` means, which capability an effect needs — those
are product knowledge and belong in `library/agents/`.

`agent/playbooks/tools/write-ken.md` keeps its **workflow trigger** and moves
its **product facts** into `library/agents/`. The skill then selects a pack.

### D4. Structural package reference is GENERATED from checked artifacts. Authored rationale stays in `catalog/packages/`. ✅ CONFIRMED

Signatures, dependencies, laws, effects, capabilities, platform availability,
and trusted-base deltas are generated. Curated rationale and worked examples
stay in the canonical literate package entry, and the catalog reference
**links or transcludes** them rather than forking them.

> ⚠ **D4 is a commitment the toolchain must actually be able to keep.** Wave 0
> does not assume it can. Before Wave 5 is framed, the Librarian reports which
> of those facts the checked artifact format can express **today** and which
> cannot. **A fact we cannot generate gets authored and labelled as authored —
> never generated-looking prose.** Do not approximate type-directed search with
> prose tags.

---

## 2. What binds the Librarian

- **`library/` is not a second spec.** D1 is not advice. A page that states a
  language rule on its own authority is a defect regardless of correctness.
- **Every page declares its authority class** (`derived-reference`,
  `explanatory`, `tutorial`/`how-to`, `status`, `normative-pointer`) and its
  sources, in `library/manifest.toml`.
- **A date is not evidence of currency.** Currency is a **source revision**,
  recorded by generated `STATUS.md` and build output, never hand-edited into
  pages.
- **Label capability honestly**: current / partial / planned / unavailable.
  **Planned syntax may never appear in a checked current example.**
- **The Librarian is non-blocking.** A feature merge does not wait for prose.
  The desired resting state for user-visible features is docs in the same
  change; the as-built pass is the **backstop, not the primary mechanism.**
- **Targeted builds only** — `scripts/ken-cargo -p <crate>`, never
  `--workspace` (`COORDINATION §12`).

---

## 3. Two risks the proposal does not fully close

Recording these now because both are cheap to design around and expensive to
retrofit.

**★ Checked examples are the whole value proposition, and the migration
threatens them.** `catalog/guide/`'s files are literate `.ken.md` whose fences
are checked. Moved into `library/` as prose, they silently stop being checked
and become the exact drift-prone duplicate D1 exists to prevent — **and they
will still look authoritative.** So: **the `ken example` / `ken reject` fence
gate must exist and pass before any `catalog/guide/` content moves.** Gate
before migration, not after. This is the one ordering constraint in the
program.

**A generated corpus can be confidently wrong.** Generation removes
transcription error; it does not make the generator right — R1 in
`issues/Q-CLAIM-CLOSURE.md` is a live instance from this same week, where
*both* sides of a consistency check came from one generator and agreed with
each other while pinning nothing. **Where a generated library fact matters,
it needs an anchor the generator does not produce.** Carry this into Wave 5's
frame.

---

## 4. Waves

Dependency-ordered, per the proposal. **Not time estimates.** Each wave is
framed as its own issue when its predecessor's exit condition is met — I am
not pre-committing the fleet to seven waves of work sight-unseen.

**Capacity and cadence (operator, 2026-07-22): three seats, waves run
SEQUENTIALLY.** The doc ring does not fan out across waves. This is the
program's shape, and it is why §4a below matters more than the wave list: with
one wave in flight at a time, **a wave that exits on a proxy poisons every wave
that inherits its substrate**, and there is no parallel track to catch it.

> **⛔ Scope of what is FRAMED versus what is WRITTEN DOWN** (operator,
> 2026-07-22). This document describes **all six waves** so the shape is
> visible and the dependencies are checkable. **Only Waves 1 and 2 are framed
> as executable issues.** Waves 3–6 are a map, not a commitment: each is framed
> when its predecessor's exit condition is *actually met*, re-grounded against
> the landed corpus at that moment. Do not treat a §4 subsection below as a
> release.

| Wave | Content | State |
|---|---|---|
| **0** | Charter + currency substrate: `README.md`, `manifest.toml`, generated `STATUS.md`, first gates, migration ledger | ✅ **RELEASED** — `issues/DOC-W0.md` |
| 1 | The read-Ken spine, **fragment-based** — introduction, quickstart, reading curriculum taught from real checked package fragments. **Complete-program work DEFERRED to Wave 1b** | ✅ **FRAMED** — `issues/DOC-W1.md` · ⛔ gated on `issues/DOC-CURRENCY-ANCHOR.md` |
| **1b** | The whole-program reading pass: curriculum ch. 7, worked end-to-end review with an explicit verdict, on one real catalog **program** | not framed — ⛔ **gated on basic capabilities landing** (operator, 2026-07-22) |
| 2 | Agent core + task packs; refactor product facts out of `write-ken`; cold-context evals. **`ffi-and-platform` deferred** | ✅ **FRAMED** — `issues/DOC-W2.md` · ⛔ gated on Wave 1 |
| 3 | Conceptual guide + how-tos; `catalog/guide/` migration (**gated on the fence gate, §3**) | map only — §4b |
| 4 | Complete reader-oriented reference | map only — §4b |
| 5 | Comprehensive catalog reference (**re-check D4 first**) | map only — §4b |
| 6 | Release, offline, continuous as-built operation | map only — §4b |

**Wave 0's exit condition is the one that matters:** a new page cannot land
without declaring what it is, what grounds it, and how its currency is
checked. Everything after it inherits that substrate, so it is worth getting
right before Wave 1 produces content at volume.

> ### ⛔ Wave 0 met that exit condition only STRUCTURALLY. Wave 1 is gated.
>
> DOC-W0 merged (`origin/main @ 6be9754b`, 2026-07-22) and **records** a
> revision, validated as a real ancestor. But **no code path reads a cited
> source's bytes at `REVISION`** — so the recorded revision certifies nothing
> about the corpus, which is *"a date with extra steps"*, the exact thing §121
> forbids. Found by the adversary post-merge, after nine review rounds.
>
> **Whoever frames Wave 1: take `depends_on: [DOC-CURRENCY-ANCHOR]` and read
> `issues/DOC-CURRENCY-ANCHOR.md` first.** Wave 1 is where this bites — its
> derived-reference pages cite **live spec chapters**, and nothing forces a
> `REVISION` bump when one moves. The first time a cited chapter's body
> changes under a stable heading, every derived page claims a currency it does
> not have, with every gate green.

### ⛔ Wave 1 RE-SCOPED 2026-07-22 (operator): defer the complete-program work

**Operator ruling, verbatim:** *"we're still focusing on basic capabilities.
defer complete program work (revise wave 1)."*

The proposal's Wave 1 requires *"one real catalog program throughout rather
than unrelated snippets"* and exits when *"a technically experienced human …
can read one non-trivial Ken program and accurately state its contract,
assumptions, authority, and execution status."* **That is premature while the
basic capability surface is still landing** — and the survey that prompted the
ruling found the concrete reason:

- **The catalog contains exactly one actual program** —
  `catalog/examples/CommandLine/Forge.ken.md`, 55 lines. Everything else under
  `catalog/packages/` is a package.
- `Forge` is **pure spec-data with no effects**, so curriculum chapters **04**
  (effects/capabilities/authority) and **06** (execution) would have had
  nothing local to teach from — forcing exactly the *"unrelated snippets"* the
  proposal forbids.
- Neither the proposal nor this document ever named the program, and **the
  exit condition depends on the choice.** Writing the curriculum first and
  picking the program later inverts the dependency.

**So Wave 1 becomes fragment-based.** It teaches the reading discipline from
**real checked package fragments**, which exist today in volume and are
already fence-checked. It keeps: introduction, quickstart, and curriculum
chapters **01–06**. Its exit condition is correspondingly narrowed:

> **Wave 1 exit:** a technically experienced human unfamiliar with dependent
> types can read a real Ken **declaration or package fragment** and accurately
> state its contract, its assurance class, and the authority it requires —
> without yet being asked to synthesize a whole program.

**Wave 1b carries what was removed:** curriculum chapter **07**
(`07-review-worked-example.md`), the complete worked review with an explicit
verdict, and the original *"read one non-trivial Ken program"* exit condition.
**It is gated on the basic capability surface being complete enough that a
real catalog program exercises effects, capabilities, and execution** — i.e.
on enough of `docs/program/10-linux-abi-completion.md` landing that such a
program exists to read. **Do not frame Wave 1b until then, and do not
substitute a purpose-built toy** — the proposal's *"an existing catalog
program, not a toy syntax collage"* constraint survives the deferral intact.

> **⚠ Framing note for Wave 1.** `library/introduction.md` **already landed in
> Wave 0** and is in `manifest.toml`. The proposal assigns "write the
> introduction" to Wave 1. Wave 1's frame must therefore say **revise**, not
> **author**, for that one file, or the ring will duplicate it.

---

## 4a. The failure mode this program is actually designed against

Waves 0 through 6 are a dependency order. **They are not the hard part.** The
hard part is that documentation's characteristic defect is invisible to every
cheap check, and DOC-W0 demonstrated it inside this very program before Wave 1
had written a line.

DOC-W0 took **nine review rounds** and produced **eight findings**, and not one
was a different kind of mistake. Every single one was **a proxy standing in for
the property**:

| # | the proxy that was checked | the property that mattered |
|---|---|---|
| 1 | the gate rejects a *fake* revision | it **accepts a real one, in CI's environment** |
| 2 | the test clones `file://{repo_root}` | an **independent** history source |
| 3 | `cat-file` says the object is present | present **AND** ancestry provable |
| 4 | the symlink was not *discovered* | the symlink is **rejected and reported** |
| 5 | the SHA was reviewed and approved | the SHA is **on `origin`** |
| 6 | the process fix was *agreed to* | the seat **can perform it** |
| 7 | `REVISION` names a real ancestor | a cited source's **bytes** were read at it |
| 8 | validation tokens are **declared** | a validation token is **consumed by a gate** |

Findings 7 and 8 were found **after merge**, by the adversary, and 7 gates
Wave 1. What finally stopped the recursion was not any individual fix: it was
**naming the predicate once** — `revision_resolved()` = *object present AND
ancestry provable* — and deriving self-heal, every deepen checkpoint, the
unshallow fallback, and all diagnostics from it.

**⇒ THE STANDING RULE FOR EVERY WAVE FRAME BELOW.** Each wave's exit condition
is stated as a **property with a named predicate**, never as a deliverable list.
A frame that says *"land these six pages and the gate passes"* has already
failed: it inherits the blind spots of whatever the gate happens to check
without anyone re-deriving them. Three specific carries:

- **State environment preconditions as named predicates BEFORE writing a
  check.** History depth, credentials, checkout topology, network reachability.
  A gate whose precondition is unwritten gets discovered one CI-red at a time,
  each round closing an instance and leaving the next layer live.
- **A completeness gate must be bidirectional.** Finding 8 was an enumeration
  checked against another enumeration of the same kind: every token in
  `KNOWN_VALIDATION_TOKENS` occurred exactly twice, both times in constants,
  and **zero** times in any gate body. Declared-set equals consumed-set, both
  directions, or the gate certifies its own vocabulary.
- **Where a generated library fact matters, it needs an anchor the generator
  does not produce.** Generation removes transcription error; it does not make
  the generator right. Both sides of a consistency check coming from one
  generator will agree with each other while pinning nothing.

---

## 4b. Wave-by-wave

Each subsection states: what the wave produces, the **property** it exits on,
what gates it, and the framing traps I have already found. Waves 1 and 2 are
framed as issues; 3 through 6 are the map.

### Wave 1 — the read-Ken spine (FRAMED · gated on `DOC-CURRENCY-ANCHOR`)

**Produces.** A revision of `library/introduction.md`; `library/quickstart.md`;
and `library/learn/reading-ken/` chapters **01–06** — anatomy, types/contracts/
proofs, assurance and trust, effects/capabilities/authority, packages and
provenance, execution. Plus the first checked exercises under
`library/learn/exercises/`.

**Taught from real checked package fragments**, not from one whole program and
not from invented snippets — see the re-scoping ruling above. Fragments exist
today in volume under `catalog/packages/` and are already fence-checked.

**Exit property.** *A technically experienced human unfamiliar with dependent
types can read a real Ken declaration or package fragment and accurately state
its contract, its assurance class, and the authority it requires.* Note what
this does **not** claim: nothing about synthesizing a whole program. That is
Wave 1b's exit and it is deferred.

**Gated on `issues/DOC-CURRENCY-ANCHOR.md`.** Wave 1 is exactly where DOC-W0's
unmet half bites: its derived-reference pages cite **live spec chapters**, and
nothing today forces a `REVISION` bump when one moves. The first time a cited
chapter's body changes under a stable heading, every derived page claims a
currency it does not have, **with every gate green.**

**Framing traps, both already paid for once:**
- `library/introduction.md` **already landed in Wave 0** and is in
  `manifest.toml`. The frame says **revise**, not **author**, or the ring
  duplicates it.
- **Do not name the curriculum's source fragments in the frame without
  checking they still exist and still check.** Every anchor in a frame is
  perishable; a fragment citation is an anchor.

### Wave 2 — agent core and task packs (FRAMED)

**Produces.** `library/agents/manifest.toml`; the **four core modules** —
`read-ken`, `write-ken`, `proof-and-trust`, `toolchain`; and **six task
modules** — `read-review`, `write-program`, `author-package`,
`prove-or-repair`, `diagnose`, `effects-and-capabilities`. Plus pack integrity
checks and the first cold-context evaluation suite.

**`ffi-and-platform` is the DEFERRED seventh task module.** The proposal lists
it with the other six. It cannot be written honestly yet: the FFI/platform
surface is the exact surface `docs/program/10-linux-abi-completion.md` is
still landing, and a module that documents it today would be obsolete before
the wave closed — or, worse, would document *aspirational* syntax, which §2
forbids outright. It is framed with Wave 1b or after PX8 closes, whichever is
later.

**Every module answers the proposal's ten-point contract in order** — use-when
with explicit non-triggers, prerequisites, current capability, canonical
forms, invariants and prohibitions, decision procedure, failure signatures,
validation, authority and sources, and known-unavailable behavior.

> **★ Point 10 is the load-bearing one and it is the one that will get
> shortchanged.** *Known unavailable or partial behavior — fail closed rather
> than invite the agent to improvise.* An agent module's characteristic harm
> is not being incomplete; it is being **confidently silent** about a boundary,
> which reads to the consuming agent as permission. The negative knowledge —
> unsupported forms, misleading near-syntax, `tt` versus `Refl`, the point at
> which an agent must **stop** instead of inventing a proof, primitive,
> capability, or package — is the part of these modules that pays.

**Exit property.** *A Ken-untrained coding agent can perform the core
read/write/prove/diagnose tasks without loading the entire spec, catalog
guide, or fleet memory* — and, on the tasks it cannot do, **refuses honestly
rather than improvising.**

**The seven-item cold-context eval suite**, from the proposal: explain a small
program's contract and trust posture; write and check a pure function with one
real law; distinguish and repair `tt` versus `Refl`; find and use a catalog
package **by task rather than guessed name**; author an effectful boundary
without omitting its capability or row; **refuse an unsupported or unproved
request honestly**; and diagnose one parse, one elaboration, one kernel, and
one runtime failure.

> **⚠ The eval suite records more than correctness** — it records unnecessary
> file loads, invented syntax or capabilities, and **whether the agent cited
> the authority it used**. The goal is *not* the smallest token count; it is
> the smallest context that reliably produces a correct, reviewable result.
> **Do not let the frame's ACs collapse this into a pass rate.** A run that
> passes six of seven while inventing a capability on the seventh is a worse
> outcome than one that passes five and refuses two, and a pass-rate AC cannot
> express that.

**Boundary reminder (D3, settled).** These modules carry **product knowledge
only.** Roles, merge flow, model routing, WP lifecycle, the memory corpus, and
the compaction discipline stay under `agent/` and stay mine.
`agent/playbooks/tools/write-ken.md` keeps its **workflow trigger** and moves
its **product facts** into `library/agents/core/write-ken.md`; the skill then
selects a pack. **This is a refactor with two live consumers — the fleet's own
seats and any external agent — so the frame must inventory both before moving
a fact.**

### Wave 3 — conceptual guide and how-tos (MAP · gated on the fence gate)

**Produces.** `library/guide/` filled in demand order — contracts, dependent
data, proofs, effects, security, packages, execution — plus `library/how-to/`
recipes driven by **actual diagnostics and recurring fleet failures**, not by
an imagined task list. And the `catalog/guide/` migration.

**⛔ This wave carries the program's one hard ordering constraint (§3).**
`catalog/guide/`'s four files are literate `.ken.md` whose fences are
**checked**. Moved into `library/` as prose they silently stop being checked
and become the exact drift-prone duplicate D1 exists to prevent — **and they
will still look authoritative.** The `ken example` / `ken reject` fence gate
**must exist and pass before any `catalog/guide/` content moves.** Gate before
migration, not after.

**Exit property.** *Tutorials teach, how-tos direct work, and conceptual pages
explain; no single page is forced to do all three.* Keep explanatory pages free
of internal campaign and WP history — a reader does not care which WP landed a
feature.

### Wave 4 — complete reader-oriented reference (MAP)

**Produces.** `library/reference/` across language, verification, toolchain,
runtime, platform, and diagnostics, plus the symbol, keyword, diagnostic, and
glossary indexes. Exact syntax, CLI, target, and public-declaration facts are
**generated**.

**Exit property.** *A reader who knows what they are looking for can find a
complete, current answer without reading the normative spec front to back.*

> **⚠ `reference/platform/` is where D1 will be hardest to hold.** It documents
> **explicit unavailable lanes**, and cross-platform is indefinitely deferred
> (operator, L2-1). A page that describes a deferred lane in the present tense
> is aspirational syntax by another name. Label it `unavailable` and say why,
> or leave it out.

### Wave 5 — comprehensive catalog reference (MAP · re-check D4 FIRST)

**Produces.** One generated reference page or card per live package, plus
subject, declaration/type, law, effect/capability, assurance, platform,
maturity, dependency, and reverse-dependency indexes.

**⛔ D4 is a commitment the toolchain must actually be able to keep, and Wave 0
did not establish that it can.** Before this wave is framed, the Librarian
reports which of those facts the checked artifact format can express **today**
and which cannot. **A fact we cannot generate gets authored and labelled as
authored — never generated-looking prose.** Do not approximate type- and
proposition-shaped search with prose tags; the proposal is explicit and it is
right.

**Exit property.** *The catalog is discoverable both by what a reader wants to
accomplish and by the exact checked abstractions available.*

**Carry §3's second risk into this frame verbatim:** a generated corpus can be
confidently wrong, and where a generated fact matters it needs an anchor the
generator does not produce.

### Wave 6 — release, offline, continuous as-built operation (MAP)

**Produces.** Static searchable HTML and an offline artifact from the same
sources; versioned snapshots and migration notes once public releases begin;
post-merge source changes wired to the Librarian's as-built queue; and the
measurement set — dead ends, failed searches, stale-source detections,
tutorial completion, agent-pack evaluation results.

**Exit property.** *Documentation currency is an observable product property
rather than the Librarian's memory of what changed.* This is the wave that
retires the Librarian's standing as-built mandate as a **backstop** and makes
it a mechanism — §2's *"the as-built pass is the backstop, not the primary
mechanism"* only becomes true here.

> **⚠ `library/releases/` is absent until Ken has versioned public releases**
> and must stay absent. Creating it early invites migration notes for
> migrations nobody performed.

---

## 4c. What is NOT in this program, and why

- **Ken has no compatibility obligation to anyone**, so there is no
  deprecation path, no versioned doc branches, and no parallel maintained
  guide. That is D2 applied forward.
- **No `library/` page is normative.** If a reader needs the rule rather than
  the explanation, the page's job is to **name the spec section**, not to
  restate it well. A polished duplicate that can drift is worse than an
  incomplete page that names its source.
- **Nothing here documents the federation.** D3, settled.
