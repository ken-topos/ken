# The Ken authoring guide — how to write Ken

There is no in-model support for Ken: no model has Ken in its training data,
and won't for some time. The normative spec (`spec/30-surface`) is a
**contract** — it says what a construct means, not how to reach for it. The
catalog (`catalog/packages/`) shows **proven components** — what good Ken
looks like once it exists, not how someone got there. This guide is the
missing middle: **the practice of writing and proving Ken**, so an agent (or
person) with no Ken in memory can go from "I need a sorted-list component"
to a landed, lawful `catalog/` entry.

It is a companion deliverable to the catalog campaign
(`../../docs/program/06-catalog-campaign.md`), developed *alongside* the
packages rather than after them, because every catalog work package is
authored against this guide and every catalog retro folds improvements back
into it (`06 §"Retro discipline"`).

## The three strands

- **[Surface reference](surface-reference.ken.md)** — the practical shape of
  the language: the `const`/`fn`/`proc` purity split, `data`/`match`,
  `class`/`instance`, refinement types, effect rows, and the literate
  `.ken.md` format itself. Task-first ("how do I write X"), distinct from
  the spec's exhaustive grammar.
- **[Proof techniques](proof-techniques.ken.md)** — how to actually
  discharge a law in Ken: choosing `tt` vs. `Refl` at a proof's terminal
  step, structuring a case-split so a hypothesis stays usable, decidable
  equality via the `sound`/`complete` pattern, why `funext` is definitional,
  and the non-termination hazards a proof author needs to see coming.
- **[Decomposition & abstraction](decomposition-abstraction.ken.md)** — when
  to reach for a `class` vs. an explicit dictionary parameter,
  subsume-don't-proliferate, coexist-when-trust-differs, and the other
  reusable design moves the fleet has paid for in real build failures.

Every strand is a literate `.ken.md` document: worked examples in
`` ```ken example `` fences are elaborated against the real toolchain, and
pitfalls in `` ```ken reject `` fences are checked to actually fail — a
stale example cannot silently rot into a lie (`catalog-literate-fence-roles`,
landed). Every code example in this guide is real, landed Ken; nothing here
is aspirational syntax.

## Reading paths

The guide serves a wide range of readers at different depths
(`06-catalog-campaign.md` purpose 3). Pick the path that matches you —
none require reading a strand front-to-back before starting to write:

- **Newcomer to Ken, first catalog entry.** Surface reference §1–§3 (purity
  keywords, `data`/`match`, refinement types) → Decomposition §1 (class vs.
  dictionary) → write a small `fn` with one law, then read Proof techniques
  §1–§2 to discharge it.
- **Agent authoring a catalog package (the common case).** Skim all three
  strands' tables of contents once, then use the guide as a reference: jump
  to Proof techniques when a law won't close, to Decomposition when a design
  choice feels ad hoc, to Surface reference for exact syntax.
- **Porting a proof idiom from Lean/Agda/Idris.** Proof techniques §3
  (`funext` is *definitional*, not an axiom) and §4 (non-termination —
  Ken's kernel requires structural termination, there is no `sorry`/postpone)
  are the two places Ken's discipline differs most from what you already
  know.
- **Reviewing a catalog entry.** Decomposition's "reusable moves" table
  doubles as a review checklist: does this entry coexist rather than
  subsume across a trust boundary? Is a `class` used only where dispatch is
  genuinely needed?

## What V1 covers, and what it deliberately omits

This is a **V1** — enough to author a Core/Data-Section entry like `DS-1`
well, not an exhaustive language reference (that role stays with
`spec/30-surface`). The guide grows through the retro loop below, the same
two-phase cadence the packages themselves follow
(`docs/program/07-catalog-style-guide.md §10`).

Named omissions, so a reader isn't left guessing whether something was
forgotten or deferred on purpose:

- **Modules, imports, and the package system** (`spec/30-surface/33-declarations.md
  §3`) are not covered — today's catalog is a flat package list
  (`06-catalog-campaign.md §"Layout"`), so cross-module authoring guidance
  has no real entries to ground it against yet.
- **Effect handlers beyond `visits [Console]`/`visits [FS]`** — the row
  *type* is covered (Surface reference §5) because catalog entries declare
  rows on I/O boundaries, but handler composition, `space`, and the state
  effect are Runtime/Verify's domain, not catalog authoring's.
  `spec/30-surface/36-effects.md` is the contract.
- **GADTs, indexed families beyond a simple `Tree k v`, and Z3-assisted
  search** are out of scope — no landed catalog entry needs them yet, and
  the demand-pull discipline (`06-catalog-campaign.md §"Demand-pull"`) says
  to add guidance when a real target needs it, not speculatively.
- **Numeric refinement (`Decimal`, exact rational reasoning)** — deferred;
  the catalog's numeric Data-Section work will pull this in when it lands.

## Retro-action wiring

Catalog retros are **acted on, not archived**
(`06-catalog-campaign.md §"Retro discipline"`) — this guide is one of the
five routes a retro's Findings take:

| A catalog WP's retro surfaces… | Routes to |
|---|---|
| A clearer proof technique, a decomposition that worked, a pitfall worth naming | **This guide** — fold it into the relevant strand (or the `write-ken` skill's inlined technique) directly, or file a follow-on if the fold needs review |
| A recurring shape the surface should sugar | Ergo (triage) → Language (implement) |
| A confusing error or manual elaborator step | Ergo |
| A reusable `def`/`lemma`/`prop` | Promoted into the catalog itself (typically the Core Section), not left local to one package |
| A kernel-reduction defect | Kernel, via the enclave — the highest-value Finding |

A catalog WP is not closed until its retro's guide-directed actions are
either folded in or booked as a follow-on entry
(`07-catalog-style-guide.md §5`, Findings). The Librarian tracks guide
follow-ons the same way it tracks any as-built drift.

## Clean-room attestation

Every example in this guide is **real, landed Ken**, drawn from
`catalog/packages/`, `conformance/`, or written fresh against the elaborator
for this guide and checked (`` ```ken example ``/`` ```ken reject ``). The
proof-technique and decomposition prose distills the fleet's own hard-won
memory (`agent/memory/enclave/`, `agent/memory/build/`) — lessons paid for
in real build failures — into public guide prose written in Ken's own
terms. General dependently-typed practice (Lean/Agda/Idris) may have
sharpened an author's intuition, but no reference source is transcribed or
copied anywhere in this guide (`CLEAN-ROOM.md`). Where a strand names an
external idiom for orientation, it says so explicitly and describes it in
Ken's own vocabulary.

## Using this guide as an agent

Load the [`write-ken`](../../agent/playbooks/tools/write-ken.md) skill — it
points here and inlines the single highest-value technique so a fresh
session doesn't need to re-derive it.
