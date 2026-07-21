# 12 — The documentation program (`library/`)

**Status:** Framed 2026-07-21. Wave 0 released to the Librarian.
**Owner:** Librarian (authoring); Steward (frame, sequencing, gates).
**Source proposal:** `research/librarian-documentation-program-proposal.md`
(Research, 2026-07-18).

Ken gets a durable product-documentation portal at `library/`, organized by
**reader need** rather than by the teams that built the repository, with the
primary learning path being **reading Ken** rather than writing it.

The research proposal is the design and I am not restating it. This document
is the **frame**: it settles the four decisions the proposal routes to the
Steward, states what binds the Librarian, and releases Wave 0.

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
| 1 | The read-Ken spine: introduction, quickstart, seven-part reading curriculum on one real catalog program | not framed |
| 2 | Agent core + task packs; refactor product facts out of `write-ken`; cold-context evals | not framed |
| 3 | Conceptual guide + how-tos; `catalog/guide/` migration (**gated on the fence gate, §3**) | not framed |
| 4 | Complete reader-oriented reference | not framed |
| 5 | Comprehensive catalog reference (**re-check D4 first**) | not framed |
| 6 | Release, offline, continuous as-built operation | not framed |

**Wave 0's exit condition is the one that matters:** a new page cannot land
without declaring what it is, what grounds it, and how its currency is
checked. Everything after it inherits that substrate, so it is worth getting
right before Wave 1 produces content at volume.
