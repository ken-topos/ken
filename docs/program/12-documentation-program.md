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

| Wave | Content | State |
|---|---|---|
| **0** | Charter + currency substrate: `README.md`, `manifest.toml`, generated `STATUS.md`, first gates, migration ledger | ✅ **RELEASED** — `issues/DOC-W0.md` |
| 1 | The read-Ken spine, **fragment-based** — introduction, quickstart, reading curriculum taught from real checked package fragments. **Complete-program work DEFERRED to Wave 1b** | not framed — ⛔ **GATED on `issues/DOC-CURRENCY-ANCHOR.md`** |
| **1b** | The whole-program reading pass: curriculum ch. 7, worked end-to-end review with an explicit verdict, on one real catalog **program** | not framed — ⛔ **gated on basic capabilities landing** (operator, 2026-07-22) |
| 2 | Agent core + task packs; refactor product facts out of `write-ken`; cold-context evals | not framed |
| 3 | Conceptual guide + how-tos; `catalog/guide/` migration (**gated on the fence gate, §3**) | not framed |
| 4 | Complete reader-oriented reference | not framed |
| 5 | Comprehensive catalog reference (**re-check D4 first**) | not framed |
| 6 | Release, offline, continuous as-built operation | not framed |

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
