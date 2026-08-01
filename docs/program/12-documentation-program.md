# 12 — The documentation program (`library/`)

**Status:** Framed 2026-07-21. **Waves 0, 1 and 2 are LANDED** — `library/`
carries 26 documents (`library/STATUS.md`). Wave 3 and beyond remain map only;
Wave 1b and the `ffi-and-platform` task module stay deferred. Currency
re-measured at `origin/main = 6de2a099`, 2026-08-01, at the doc-track restart.
**Owner:** the **doc team** (§0); Steward (frame, sequencing, gates).
**Source proposal:** `research/librarian-documentation-program-proposal.md`
(Research, 2026-07-18).

Ken gets a durable product-documentation portal at `library/`, organized by
**reader need** rather than by the teams that built the repository, with the
primary learning path being **reading Ken** rather than writing it.

The research proposal is the design and I am not restating it. This document
is the **frame**: it settles the four decisions the proposal routes to the
Steward, states what binds the Librarian, and released Wave 0.

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
will still look authoritative.** So: **the `ken example` / `ken reject` fences
must be verified over each file before it moves.** Gate before migration, not
after. This is the one ordering constraint in the program.

**The form is migration-local, settled §4** — the migrating WP exercises the
real extractor over exactly the files it moves and pins the fence count per
file, rather than a standing CI gate. The detector's both-polarity mutation
proof already runs live in CI; the registry that once applied it to the corpus
does not run at all.

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
| 1 | The read-Ken spine, **fragment-based** — introduction, quickstart, reading curriculum taught from real checked package fragments. **Complete-program work DEFERRED to Wave 1b** | ✅ **LANDED** — `issues/DOC-W1.md`; chapters 01–06, fragments, exercises, solutions, quickstart, introduction all present and registered |
| **1b** | The whole-program reading pass: curriculum ch. 7, worked end-to-end review with an explicit verdict, on one real catalog **program** | not framed — ⛔ **gated on basic capabilities landing** (operator, 2026-07-22) |
| 2 | Agent core + task packs; refactor product facts out of `write-ken`; cold-context evals. **`ffi-and-platform` deferred** | ✅ **LANDED** — `issues/DOC-W2.md`; four agent core modules, six task modules/packs, schemas, cold-context fixtures, recorded eval results |
| 3 | Conceptual guide + how-tos; `catalog/guide/` migration (**fences verified per file before it moves, §3**) | map only — §4b · its §3 precondition is **RECONCILED**, see below — no longer a blocker |
| 4 | Complete reader-oriented reference | map only — §4b |
| 5 | Comprehensive catalog reference (**re-check D4 first**) | map only — §4b |
| 6 | Release, offline, continuous as-built operation | map only — §4b |

**Wave 0's exit condition is the one that matters:** a new page cannot land
without declaring what it is, what grounds it, and how its currency is
checked. Everything after it inherits that substrate, so it is worth getting
right before Wave 1 produces content at volume.

> ### Wave 3's §3 fence-gate precondition — RECONCILED (Steward, 2026-08-01)
>
> §3 gates the `catalog/guide/` migration on the `ken example` / `ken reject`
> fences *"remaining checked and the checker being mutation-proven"*. **The
> substantive requirement stands**: migrating prose whose examples are not
> actually checked is exactly the corpus lying to readers that this program
> exists to prevent. What needed settling was the *form*, and it is settled
> here. **Wave 3 is no longer blocked on this.**
>
> **The precondition has two halves, and they are in different states.**
>
> **Half one — the checker is mutation-proven on both polarities. Already
> discharged, and live.**
> `checked_examples_detector_rejects_invalid_example_and_stale_reject`
> (`crates/ken-cli/tests/library_documentation_gates.rs`) is one of the file's
> 26 ordinary `#[test]`s. It runs on every PR, plants an invalid `ken example`
> and a stale `ken reject`, and requires the specific diagnostic from each.
> Both polarities, in CI, today. **Nothing is owed here — do not rebuild it.**
>
> **Half two — the fences are actually exercised over the corpus. Not
> discharged, and it cannot be discharged by the registry.** Measured
> 2026-08-01 at `f31e8d94`: `VALIDATION_GATES` appears exactly twice in that
> file — its own declaration and one comment. **No test iterates it.** All
> eleven gate functions, `check_checked_examples` among them, occur exactly
> twice each: their definition and their registry row. So this is not a gap
> peculiar to checked examples — **the entire declared validation vocabulary is
> unreachable code**, and a document's `validation` list in `manifest.toml`
> currently names checks that nothing runs.
>
> **The settled form: migration-local verification at candidate time, not a
> standing gate.** The migrating WP runs the real extractor over exactly the
> files it moves and requires success. It does not reinstate global coupling —
> and given the registry is inert, "restoring the gate" would mean building new
> coupling, not restoring old, which is the operator ruling in reverse.
>
> **Why local is sufficient rather than a downgrade.** The hazard §3 names is
> created *by the act of migrating* — a fence that stops being a fence when the
> file moves — and it is fully observable *at* that act. A standing gate would
> re-check unchanged files forever to catch something that only arises when a
> file moves. Paired with the release-point re-stamp that `LIB-GATE-DECOUPLE`
> established, migration-local plus release-point is the coherent pairing.
>
> **The control must pin the fence COUNT, not just the exit status.** Running
> the checker over the migrated file and getting success is satisfied
> vacuously by a migration that turned the fences into plain code blocks —
> which is precisely the failure mode §3 is about. So the binding control is:
> the post-migration `ken example` + `ken reject` count equals the
> pre-migration count, per file, and is non-zero. **Fixed input measured at
> `f31e8d94`: 40 fences across three files** — `surface-reference.ken.md` 17+7,
> `proof-techniques.ken.md` 8+5, `decomposition-abstraction.ken.md` 3+0;
> `README.md` carries none.
>
> **Unmeasured and deliberately so:** whether those 40 fences pass *today*.
> Nothing has exercised them since the registry went inert, so a pre-existing
> failure is possible. Establishing that baseline is the migrating WP's first
> deliverable, not an assumption it may inherit — and a red baseline is a
> finding to route, never something the migration quietly repairs.
>
> ### The fences must SURVIVE the move — citation is not available here
>
> **D2 is ratified and it settles the direction: migration is subsumptive.**
> `catalog/guide/` **moves** and does not persist alongside, leaving pointers
> rather than a second maintained guide. So the shape the Wave 1 spine uses —
> prose in `library/` citing a checked file that stays in `catalog/` — is **not
> available for this material.** The spine cites `catalog/packages/`, which
> persists by design; `catalog/guide/` does not.
>
> ⇒ The obligation is the harder one §3 actually states: **the 40 fences must
> still be checked after they land in `library/`.** Not preserved in place —
> preserved *through* the move.
>
> **What that implies, and the one thing genuinely unknown.** `ken check`
> selects literate extraction **by the `.ken.md` suffix**, so a checked page in
> `library/guide/` would carry that suffix. Measured at `f31e8d94`, `library/`
> holds **zero `.ken.md` files** and **7 ken fences across 25 documents** (six
> plain, one `ken example`) — the corpus has never registered a literate
> document, and whether a `.ken.md` can be a `manifest.toml` document record
> with the generated-status machinery working over it is **unverified**. That
> is the migrating WP's question to answer early and the Librarian's call on
> corpus convention, not an assumption to inherit.
>
> **The control is therefore a conservation law, not a stay-put law:** 40
> fences before the move, 40 after, each one still exercised by the real
> extractor at candidate time — per file, 17+7 / 8+5 / 3+0. A migration that
> lands the prose while the fence total drops has done exactly the thing §3
> exists to prevent, and it will look complete.

> ### Wave 0 met that exit condition only STRUCTURALLY — the gap is now CLOSED
>
> DOC-W0 merged (`origin/main @ 6be9754b`, 2026-07-22) and **records** a
> revision, validated as a real ancestor. But **no code path read a cited
> source's bytes at `REVISION`** — so the recorded revision certified nothing
> about the corpus, which is *"a date with extra steps"*, the exact thing §121
> forbids. Found by the adversary post-merge, after nine review rounds.
>
> **Discharged by `DOC-CURRENCY-ANCHOR` (closed 2026-07-22).**
> `scripts/gen-doc-status.sh` now compares each cited source's bytes at
> `REVISION` against `HEAD` and fails on drift, so Wave 1's derived-reference
> pages no longer claim a currency nothing checks. Wave 1 was framed with
> `depends_on: [DOC-CURRENCY-ANCHOR]` and has since landed; the record is kept
> here because the banner was correct when written.

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

### Wave 1 — the read-Ken spine (LANDED)

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

**Was gated on `issues/DOC-CURRENCY-ANCHOR.md`, now discharged.** Wave 1 is
exactly where DOC-W0's unmet half bit: its derived-reference pages cite **live
spec chapters**, and nothing forced a `REVISION` bump when one moved. That gap
is closed — the content-currency check reads each cited source's bytes at
`REVISION` and fails on drift.

**Framing traps, both already paid for once:**
- `library/introduction.md` **already landed in Wave 0** and is in
  `manifest.toml`. The frame says **revise**, not **author**, or the ring
  duplicates it.
- **Do not name the curriculum's source fragments in the frame without
  checking they still exist and still check.** Every anchor in a frame is
  perishable; a fragment citation is an anchor.

### Wave 2 — agent core and task packs (LANDED)

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

### Wave 3 — conceptual guide and how-tos (MAP · fence precondition RECONCILED)

**Produces.** `library/guide/` filled in demand order — contracts, dependent
data, proofs, effects, security, packages, execution — plus `library/how-to/`
recipes driven by **actual diagnostics and recurring fleet failures**, not by
an imagined task list. And the `catalog/guide/` migration.

**This wave carries the program's one hard ordering constraint (§3).**
`catalog/guide/`'s four files are literate `.ken.md` whose fences are
**checked**. Moved into `library/` as prose they silently stop being checked
and become the exact drift-prone duplicate D1 exists to prevent — **and they
will still look authoritative.** The `ken example` / `ken reject` fences **must
be verified over each file before it moves.** Gate before migration, not after.

The verification form is settled in §4 and is **migration-local**: exercise the
real extractor over exactly the files being moved, and pin the per-file fence
count so a migration that quietly demotes fences to plain code blocks cannot
pass on a green exit status. 40 fences across three files at `f31e8d94`.

**Exit property.** *Tutorials teach, how-tos direct work, and conceptual pages
explain; no single page is forced to do all three.* Keep explanatory pages free
of internal campaign and WP history — a reader does not care which WP landed a
feature.

> ### The seven-subject guide list — MEASURED, and six of seven are delivered
> (Steward, 2026-08-01, at `origin/main = c777d2d4`)
>
> The subject list above reads as seven chapters to write. It is not. Setting
> the seven against what `library/` holds today: **contracts, proofs, effects,
> security, packages, and execution all already carry explanatory pages**, and
> several carry two. The six `library/learn/reading-ken/` chapters and the three
> migrated `library/guide/` pages are **all classified `kind = "explanatory"`,
> `authority = "explanatory"`** in the manifest — they are conceptual pages
> filed under `learn/`, not tutorials that a conceptual page would sit beside.
> The exit property is met for those six subjects.
>
> **The residual is one subject: dependent data.** `Vec` and `Fin` occur zero
> times under `library/`, while `spec/50-stdlib/60-length-indexed-vectors.md`
> is normative with a landed family, `head`, `Fin` declaration, and `tail`.
> That is `DOC-W3-DEPDATA`, and it is an `S`.
>
> ⇒ **Do not re-derive the seven-chapter reading from this section's prose.**
> Releasing the six as fresh work would re-author correctly-classified material,
> which is the mistake this program has already made on L5 and V3. The per-
> subject coverage table is in `docs/program/issues/DOC-W3-DEPDATA.md`.

### Wave 4 — complete reader-oriented reference (MAP)

**Produces.** `library/reference/` across language, verification, toolchain,
runtime, platform, and diagnostics, plus the symbol, keyword, diagnostic, and
glossary indexes. Exact syntax, CLI, target, and public-declaration facts are
**generated**.

**Exit property.** *A reader who knows what they are looking for can find a
complete, current answer without reading the normative spec front to back.*

> ### Wave 4 has the same generation precondition Wave 5 states explicitly —
> and, until now, did not state it (Steward, 2026-08-01, at `7fa65b20`)
>
> This wave commits that syntax, CLI, target, and public-declaration facts are
> **generated**. Measured: `scripts/` holds three generators
> (`gen-doc-status.sh`, `gen-progress.sh`, `gen-source-attestations.sh`) and
> **none** extracts a declaration, keyword, syntax production, or CLI surface.
> The CLI emits **no machine-readable output**. Ken has **no diagnostic
> registry**, so a generated diagnostic index is not producible today.
>
> Wave 5 already handles exactly this case: the Librarian reports what the
> format can express *before* the wave is framed, and **a fact we cannot
> generate gets authored and labelled as authored, never generated-looking
> prose.** That rule applies here verbatim.
>
> ⇒ **The report is `DOC-W4-TOOLCHAIN`'s D0**, produced with the one Wave 4
> surface that needs no generator at all — the toolchain reference. Later Wave 4
> slices rest on its answer, so it is produced once and durably rather than
> rediscovered per slice.

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
