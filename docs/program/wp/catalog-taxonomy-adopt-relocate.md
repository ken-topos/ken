# CAT-TAX — Adopt the 7-section catalog taxonomy + mechanically relocate (Foundation)

- **ID:** CAT-TAX · **Owner:** Team Foundation · **Size:** L · **Risk:** Medium
  (import-identity migration across the whole catalog; behavior must NOT change).
- **Objective:** Adopt the controlled 7-section catalog taxonomy and mechanically
  relocate all 37 current `catalog/packages/` entries to their canonical homes,
  rewriting every path reference + dotted import identity — **filing only, zero
  Ken behavior change** — and add the allowlist lint that prevents future drift.
- **Source of truth:** the research proposal
  `research/catalog-package-taxonomy-proposal.md` (committed on this branch), which
  reviewed all 37 entries and cross-checked the shape against Python/Hackage/Cargo.
  This frame transcribes its Phase-1/Phase-2 into deliverables + ACs.
- **⚠ Inventory currency (37, not 36):** the live catalog base is **37 entries /
  14 top-level dirs** on `origin/main @ 513955fe` — the report's first draft said
  36/13 before CC9's `catalog/packages/Test/Property.ken.md` (landed `cc2bf2ad`,
  an ancestor of main) was folded in. The report on this branch is the corrected
  37-entry version: the new entry maps `Test/Property.ken.md →
  Tooling/Testing/Property.ken.md` (Tooling now has **Verification + Testing**
  active). Inventory against the live base, not a memory of 36.
- **Depends on:** the landed path/import rule (identity = path spelling, arbitrary
  depth). **Feeds:** the catalog campaign (`06-catalog-campaign.md`),
  `07-catalog-style-guide.md`. **Gate:** catalog structural gate.

## Fixed inputs — DO NOT REOPEN

- **Operator-directed (Pat, 2026-07-15):** adopt this taxonomy and dispatch the
  relocation to Foundation. The taxonomy DIRECTION is ratified.
- **The controlled 7-section allowlist** (top-level `catalog/packages/` dirs):
  `Core`, `Data`, `Algorithm`, `Capability`, `Protocol`, `Application`, `Tooling`.
  Only sections with a mapped package are created as directories; `Algorithm` and
  `Protocol` are **reserved** (no empty dirs — documentation horizon only).
- **Semantic hierarchy:** `Section > Domain > optional Subdomain > Package`.
  Dependency direction `Core → Data → Algorithm → Capability → Protocol →
  Application → Tooling`; a package must not depend on a section to its right.
- **Classification rules** (report §"Classification rules"): Section = durable
  dependency stratum; Domain = stable subject vocabulary; Subdomain = optional,
  only when ≥2 sibling families exist/are-credible; Package = one cohesive
  importable leaf (descriptive leaf, not parent-repeat; leaf-or-namespace
  uniqueness preserved). Canonical home by what a package *does* + its dependency
  position; secondary facets are controlled metadata, NOT new top-level dirs.
- **Relocation is FILING, not editing:** move cohesive files with **Ken contents
  unchanged**; rewrite path refs + expected dotted import identities; no duplicate
  package files (redirects live in docs/tooling metadata only).
- **⛔ Targeted local builds only** (`scripts/ken-cargo`, scoped); the full
  catalog/workspace gate runs in CI. No-regression = **green in CI**.

## ⚠ Enclave-pin supersession (pin + route)

One relocation **superseded an enclave-pinned classification** in
`docs/program/wp/catalog-taxonomy-paths-imports.md`:
`Capability/Verify/ProofErasureBoundaryChecker.ken` →
`Tooling/Verification/ProofErasureBoundaryChecker.ken` (a development-time artifact
checker belongs in `Tooling`, not as an end-user `Capability`). **Enclave
concurrence is POSTED — APPROVED** (spec-leader `evt_13mrsd0ef7af`: pure taxonomy
filing, no behavior/kernel/soundness/spec change). This move is therefore **no
longer gated — land it together with the other 36 relocations.** The
`Core` → `Core.Logic`/`Core.Classes` moves are subdivision (not supersession) and
need no separate concurrence.

## Scope

**In scope (Phase 1 + Phase 2):** adopt the allowlist + classification vocabulary;
extend the campaign charter's "Sections and Domains" text with Subdomains + the
definitions; add a **catalog lint rejecting any new top-level dir outside the
allowlist**; keep future sections/domains documentation-only; mechanically
relocate all 37 entries per the report's "Proposed homes for every current entry"
table; rewrite every path/import identity; move the `ArgParse` worked example out
of the importable package corpus.

**OUT of scope (explicit — follow-on WPs):** **Phase 3 package splits** (the
report's five "Split" rows — `Collections.Collections` by carrier; `Map`→data +
relation-algorithm; `Capability.Parsing.Parsing`→source + Boolean grammar;
`Text.Numeric.Numeric`→decimal parse + format; embedded examples). **Do NOT split
any file in this WP** — a "Split" row means *relocate the combined file, intact,
to its single Phase-2 provisional home* (now explicit in the report for all four),
leaving the split to its own dependency-aware WP. The four Phase-2 provisional
homes: `Collections.Collections` → `Data/Collections/Derived.ken.md`; `Map` →
**stays** at `Data/Collections/Map.ken.md`; `Capability/Parsing/Parsing.ken.md` →
**stays** (already allowlisted); `Text/Numeric/Numeric.ken.md` →
`Capability/Parsing/Numeric.ken.md`. Also OUT: **Phase 4** drift checks beyond
the allowlist lint (path/import-at-depth, leaf-or-namespace, PascalCase,
no-empty-namespace, no-Example-without-rationale, canonical-category+facets,
no-right-dependency) — a separate hardening WP. No new empty section/domain dirs.

## Mandated deliverable outline — each ends in a concrete choice

1. **Ratify the vocabulary in the charter.** Extend `06-catalog-campaign.md`
   "Sections and Domains" with the 7-section allowlist, the Section/Domain/
   Subdomain/Package definitions, and the dependency-direction rule. Record the
   controlled metadata-facet vocabulary (`platform`/`effects`/`assurance`/
   `maturity`/`audience`/`security`/`artifact-kind`) as facets, not directories.
2. **Allowlist lint.** Add a catalog check that FAILS on any top-level
   `catalog/packages/` directory outside the 7-section allowlist. Wire it into the
   catalog gate. (Phase-4 checks are NOT in this WP.)
3. **Mechanical relocation (the 35 non-superseded entries + ArgParse example).**
   Execute the report's "Proposed homes" table exactly: move each file to its
   canonical home, Ken contents byte-unchanged; for "Split" rows, relocate the
   combined file to its **provisional** home only (no split). Move
   `ArgParse/Example.ken.md` out of `packages/` to `catalog/examples/CommandLine/`
   (or embed per the report) — it is not an importable peer package.
4. **Import-identity rewrite (full inventory).** Rewrite every path reference and
   expected dotted import identity across the WHOLE Ken-source corpus for every
   moved entry — enumerate every Ken-source root (packages, examples, tests,
   prelude/Rust-emitted references, conformance fixtures) so no consumer is
   missed. No duplicate files; redirects in docs/tooling metadata only.
5. **The superseded entry (enclave concurrence POSTED — approved).** Land
   `Capability/Verify/…` → `Tooling/Verification/…` with the other 36 relocations;
   the Steward has posted enclave concurrence (spec-leader `evt_13mrsd0ef7af`,
   approved as pure taxonomy filing). No longer gated.

## Acceptance criteria (testable)

- **AC1 — allowlist enforced.** The lint rejects a probe top-level dir outside the
  7 sections and passes the post-relocation tree; only mapped sections exist as
  dirs (`Algorithm`/`Protocol` absent — reserved).
- **AC2 — every entry relocated + import-identity updated.** Each of the 37 entries
  is at its canonical home (or the superseded one pending, AC5); every dotted
  import identity resolves to the new path; a full-corpus grep finds **no dangling
  reference** to an old path/identity. The negative sweep excludes exactly
  (1) this frame, (2) the redirect table in
  `research/catalog-package-taxonomy-proposal.md`, and
  (3) `docs/program/IMPLEMENTATION-PROGRESS.md`; the Steward-owned tracker is a
  historical operational log, not a live catalog consumer.
  (Migration-inventory discipline: enumerate every Ken-source root, including
  Rust-emitted prelude references.)
- **AC3 — filing only, behavior unchanged.** Ken contents are byte-identical across
  each move (relocation, not edit); no "Split" row split a file; the catalog
  elaborates/type-checks with identities changed but behavior provably unchanged.
- **AC4 — example demoted.** `ArgParse/Example.ken.md` is no longer an importable
  package; the importable corpus contains no `Example` package without an explicit
  library rationale.
- **AC5 — superseded move (concurrence posted).** `Capability/Verify →
  Tooling/Verification` lands with the rest; enclave concurrence is posted
  (spec-leader `evt_13mrsd0ef7af`, approved). No outcome blocks on it.
- **AC6 — CI green.** Full catalog/workspace gate green in CI (targeted local only
  via `scripts/ken-cargo`); no behavior regression.

## Do-not-reopen guards

- The report is the design source; do NOT re-derive the taxonomy. A genuinely new
  fixed boundary hard-stops to the Steward.
- Do NOT split any file (Phase 3), add Phase-4 checks, or create empty
  section/domain dirs — all explicit follow-ons.
- Do NOT change Ken contents during relocation (identity-change ≠ behavior-change;
  keep them separable so review can prove behavior held).
- Do NOT land the Capability.Verify→Tooling move before enclave concurrence.

## Grounding

Report `research/catalog-package-taxonomy-proposal.md` (this branch): §"Proposed
homes for every current entry" (the 37-row relocation map), §"Proposed section and
domain catalog" (allowlist + domains), §"Classification rules", §"Migration plan"
(Phase 1–4). Charter `docs/program/06-catalog-campaign.md`; style guide
`07-catalog-style-guide.md`; enclave path/import pin
`docs/program/wp/catalog-taxonomy-paths-imports.md` (the one superseded
classification). Catalog sources are literate `.ken.md` (not `.ken`).
