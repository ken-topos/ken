# Literate transformation + L6 casing pass (operator-kicked)

**Owned by the Steward** (frame); **home: Librarian** (owns catalog-authoring
encoding — 07 §8 / `write-ken` — and the L6 bulk casing pass). **Operator-kicked**
(2026-07-10, Pat): run the `.ken → .ken.md` literate transformation now, carrying
the L6 casing rename in the same pass (one touch per file). The formatter (see
"Explicitly NOT gating" below) is a separate future discussion item and does **not**
block this.

## Two coupled, mechanical changes — applied together, one touch per file

1. **Literate transformation** `.ken → .ken.md` for the **8 remaining plain
   files** (the 6 already-`.ken.md` files skip this step):
   - `catalog/packages/Core/LawfulClasses.ken`
   - `catalog/packages/Core/LawfulFunctors.ken`
   - `catalog/packages/Core/Transport.ken`
   - `catalog/packages/Data/Sums/Sums.ken`
   - `catalog/packages/Data/Collections/Collections.ken`
   - `catalog/packages/Data/Collections/Map.ken`
   - `catalog/packages/Capability/Parsing/Parsing.ken`
   - `catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken`
   Encode per the standard (07 §8, `write-ken`, and the landed `.ken.md`
   exemplars — `EffectfulClasses.ken.md`, `OrdNat.ken.md`, `EmptyDec.ken.md`).
   The tangled code must be **identical** (modulo the casing rename below) — this
   is a re-encoding, not a rewrite.

2. **L6 casing rename** (`ds-campaign-judgment-log.md §L6`,
   `07-catalog-style-guide.md §9`) applied to **all camelCase identifiers across
   all 14 catalog files** — the 8 being transformed AND the 6 already-`.ken.md`
   (which are literate but pre-L6 camelCase). PascalCase class-like (types,
   classes, data constructors) is already correct and unchanged; camelCase
   functions / combinators / class-methods / record-fields → snake_case
   (`getOrElse→get_or_else`, `mapErr→map_err`, `concatMap→concat_map`, etc.).

## Boundary / constraints

- **Purely mechanical, ZERO semantic change.** No new declarations, no proof
  changes, no logic edits. Every tangled declaration must be equivalent to its
  pre-pass form modulo identifier names.
- **The rename must reach EVERY reference site**, or code breaks: cross-package
  imports/references, **Ken code in spec examples** that name these combinators
  (L6 scope explicitly includes spec examples), and any **acceptance tests that
  reference the identifiers by name/string**. A missed site is a build break —
  the verification bar below is what catches it.
- **Zero `crates/**/src/`, `crates/ken-kernel`, spec-normative-prose delta** — the
  only non-catalog touches allowed are (a) Ken code *inside* spec examples and
  (b) acceptance-test identifier references, both consequences of the rename. If
  anything looks like it needs a prelude/kernel/elaborator change, STOP and hand
  back — a re-encode + rename needs none.
- **Zero new `Axiom`/`postulate`/`sorry`; zero `trusted_base()` delta.**

## Explicitly NOT gating this WP (operator ruling)

- **The Ken auto-formatter is a separate future item** (logged as a discussion
  item, `ds-campaign-judgment-log.md §D1`). Do **NOT** hand-reflow the long code
  lines this pass surfaces (e.g. `EffectfulClasses.ken.md` has many 200+ column
  code lines, `decomposition-abstraction.ken.md:129` is ~295) — a mechanical
  formatter will reflow the whole catalog later, and manual reflow now is both
  wasted effort and an error risk. Keep this pass to **(encode + rename)** only;
  leave line-length to the future formatter. Follow the style guide's existing
  norms for any genuinely new prose you write, but do not chase existing long
  lines.

## Verification (the acceptance bar — proof-carrying code must keep checking)

- **Every catalog file `ken check` / `ken run` green** post-pass (the tangled
  code checks identically; no proof breaks; casing applied consistently at every
  site). This is the load-bearing check — the rename correctness lives here.
- **Full catalog acceptance suite green** (`crates/ken-elaborator/tests/` +
  whatever exercises the catalog) — catches missed rename sites in tests/spec
  examples. Zero regressions.
- **`git grep` shows no orphaned camelCase** catalog identifier and no dangling
  reference to an old name.

## Execution / ownership (Librarian's call)

Librarian **leads** (owns the encoding + the L6 pass). This is a large,
proof-carrying, multi-file mechanical effort — the Librarian decides the shape:
solo with cheap per-file subagents (Haiku, wrap-md-80-style) self-verifying each
file, or **pull Foundation** for build/QA muscle on the proof-carrying files
(Foundation has the implementer+QA ring and owns catalog authoring per P3).
Recommend a **file-at-a-time or package-at-a-time** cadence with a check after
each, not a big-bang — a per-file verify localizes any rename miss. Sequence
freely; independent files parallelize.

## Gate

Normal ring for whatever execution shape the Librarian picks → the acceptance bar
above → **@architect** (fidelity: mechanical re-encode + rename, zero semantic
change, zero `crates/src`/kernel/spec-prose delta; the tangled code equivalence)
→ **Spec vote** only if any spec-normative prose moved (it should not — Ken code
in examples is not normative prose, but flag if a boundary case arises) →
`git_request` to Steward. CI-gated (catalog + acceptance). Own retro; flag every
judgment call. Casing questions route to `§L6` / style guide `§9`; if a boundary
call (a constructor-vs-method ambiguity) needs the operator, surface it.
