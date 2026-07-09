# WP — Ken reference materials + `write-ken` skill (campaign keystone)

**Owned by the Steward** (frame); **authored by the Librarian**, assembling from
the landed corpus and the fleet's own memory, with **enclave fidelity review**
(Architect on proof-technique/abstraction claims; spec-author on surface-
reference accuracy). This is the **initial WP of the catalog campaign** — it runs
*before* the campaign proper (the data-structures program), because every later
catalog WP is authored against these materials and its retros fold improvements
back into them (`06-catalog-campaign.md` → "The authoring guide" and "Retro
discipline").

## Why this is first

There is no in-model support for Ken. An agent (or person) writing Ken has the
normative spec (a contract, not a tutorial) and the catalog (proven components,
not technique). The gap is **how to actually write and prove Ken** — and that gap
must be filled before we ask the fleet to author a batch of literate catalog
entries. Standing this up first also gives the campaign's retro loop a target:
"what we learned about writing Ken" has somewhere to land.

## Deliverables

1. **`catalog/guide/` — the reference materials (V1).** Repo-visible Markdown,
   the canonical artifact (serves external readers + the training-data purpose).
   Three strands, each grounded in **landed** Ken with checked examples:
   - **Surface reference** — the practical shape of the language: the
     `const`/`fn`/`proc` purity split, `data`/`match`, `class`/`instance`,
     refinement types, effect rows, and the literate `.ken.md` format.
     Task-first ("how do I write X"), distinct from `spec/30-surface`'s contract.
   - **Proof techniques** — how to discharge laws: `refl` vs. `tt` endpoints,
     induction and motive construction, using `Dec`, funext as definitional
     pointwise equality, and the non-termination hazards to avoid.
   - **Decomposition & abstraction hints** — `class` vs. explicit dictionary,
     `subsume-don't-proliferate`, coexist-when-trust-differs, structural
     self-evidence, and the other reusable moves.
2. **`write-ken` skill** — a thin agent-facing skill
   (`agent/playbooks/tools/write-ken.md`, symlinked into `.claude/skills/` and
   `.agents/skills/` like `wrap-md-80`) that points at `catalog/guide/` and
   inlines the highest-value technique, so any agent writing Ken loads it.
3. **Retro-action wiring** — a short section (in the guide or the overlay)
   recording exactly how a catalog WP's retro folds back: guide/skill edits,
   Findings routing (surface→Ergo/Language, elaborator→Ergo, kernel→Kernel), and
   promotion of reusable `def`/`lemma`/`prop` into the Core Section.

The **Foundation catalog-authoring overlay** (charter next-action 2) may be
bundled here or follow immediately — it is the same body of knowledge pointed at
Foundation as its standing authoring skill.

## Sources and the clean-room boundary

Author from three permitted sources, in Ken's own words:

- **Landed Ken** — `spec/30-surface`, the `catalog/packages/` entries, the
  elaborator's accepted surface. Every code example must be real and elaborate.
- **The fleet's own memory** — `agent/memory/` and the Steward's operating
  memory already encode much of the proof-technique and decomposition strand as
  lessons paid for in real build failures. This is the cheapest, most authentic
  V1 draft source; distil it *outward* into public guide prose.
- **General DT practice** — Lean/Agda/Idris tactics and patterns are widely
  documented in public and may be **consulted by the enclave** to sharpen the
  guide, but the guide is written in Ken's terms and **never copies reference
  source** (`CLEAN-ROOM.md`: permissive references inform *approach*; copyleft is
  enclave-only; neither is transcribed). Implementer-tier authors work from
  landed Ken + memory, not from `local/refs/`.

## Acceptance criteria

- **AC1 — three strands present**, each with at least a few worked, **checked**
  examples drawn from landed catalog code. Positive examples in `ken example`
  fences (elaborate, don't tangle); pitfalls in `ken reject` fences (must fail) —
  the fence roles that just landed. Running the guide's `.ken.md` files passes.
- **AC2 — `write-ken` skill registered** (symlinks resolve; frontmatter
  `name`/`description`/`scope`/`model`), points at `catalog/guide/`, and inlines
  the highest-value technique. A fresh session can load it.
- **AC3 — retro-action wiring documented**, matching `06`'s Retro discipline
  (the five routes), so a catalog WP knows exactly where each Finding goes.
- **AC4 — clean-room attested**: no transcribed reference source; examples are
  Ken's own. Enclave fidelity review signs the proof-technique and surface
  claims.
- **AC5 — scoped to V1**: "enough to author `DS-1` well," not exhaustive. The
  guide grows via the retro loop (the two-phase refinement cadence applies to the
  guide as to packages). Name what V1 deliberately omits.

## Cadence and gate

Steward frame → Librarian assembles V1 from landed corpus + memory → enclave
fidelity review (Architect + spec-author) → merge Decision → publisher path.
Because the code examples are checked `.ken.md`, this touches the elaborator's
literate check path only as a *consumer* (no crate change expected); a pure-docs
merge if so, CI-gated if any fixture is added. On merge, the data-structures
program (`catalog-data-structures-program.md`) is unblocked and `DS-1` can kick.
