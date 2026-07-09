# WP — Catalog taxonomy: file paths + imports mirror Section > Domain

**Owned by the Steward** (frame); **design pinned by the enclave** (Architect +
spec-author — the import/module scheme is a surface + resolution design call);
**built by Ergo** (elaborator/parser/`ken-cli`) + a file-move pass. **Operator
directive (2026-07-09):** "before we proceed any further" — so this **precedes
DS-2** and should land before (or be sequenced ahead of) the other pre-DS-2 work
(FR-3, guide-update), because it moves the very paths those reference.

## The problem

Two things are out of alignment with the catalog's own taxonomy:

1. **File paths are single-level and taxonomy-blind.** Today every package sits
   at `catalog/packages/<name>/<file>.ken[.md]` — a flat mix
   (`core`, `collections`, `parsing`, `verify`, `lawful-classes`,
   `lawful-functors`, `transport`) that does **not** reflect the **Section >
   Domain** hierarchy the charter defines (`06 §"Sections and Domains"`: Sections
   Core/Data/Algorithm/Capability; Domains subdivide a Section, e.g. Parsing /
   Cryptography under Capability).
2. **The import space is flat.** `import <ident>` parses a **single** identifier
   (`parser.rs:830`), and cross-package resolution is immature (DS-1 *inlined*
   `DecEq`/`sym`/`trans` rather than importing — "no cross-package import
   mechanism today," DS-1 §6). The in-session module layer already understands
   **dotted** qualified paths (`"M.N"`, `modules.rs:42`) via nested `module`
   blocks, so the hierarchy exists internally but is not surfaced as a regular
   path↔import correspondence.

**Goal (operator, principle of least confusion):** make the **file path** reflect
Section > Domain, and make the **import specification** reflect the *same*
hierarchy by a **regular, mechanical rule** — so a reader can go from an import to
a file (and back) without a lookup table.

## The design questions for the enclave

1. **Path scheme + depth.** `catalog/packages/<Section>/<Domain>/<pkg>` — is the
   **Domain level mandatory or optional**? `06` makes Domains a subdivision
   *only when a Section is subdivided*, implying variable depth
   (`<Section>/<pkg>` when unsubdivided, `<Section>/<Domain>/<pkg>` when not).
   Variable depth is fine iff the import rule mirrors it exactly. Pin the rule.
2. **Import/module surface + the path↔import bijection.** The regular form is a
   **dotted module path mirroring the file path** — `import Data.Collections.Map`
   ⇔ `catalog/packages/Data/Collections/Map.ken`. Pin: (a) does `import` accept a
   **dotted path** (parser change from the single-ident `expect_ident`)? (b) how
   is the module **declared** — an explicit `module Data.Collections.Map` header,
   or **inferred from the file path**? (c) the **name-normalization rule** — dirs
   today are kebab (`lawful-classes`) but module identifiers can't hold hyphens,
   so path components and import components must agree by construction. The
   cleanest "least confusion" candidate is **directory names == module
   identifiers verbatim** (e.g. PascalCase `Data/Collections/Map`), so the
   mapping is identity, not a transform. Pin the casing/normalization convention.
3. **Scope of the resolution change.** Does this WP *only* establish the
   convention + move files + accept dotted import syntax (leaving actual
   cross-package disk-loading as the separate DS-1-gap capability), or does it
   also build **path-based package resolution** (module path → file on disk)?
   Recommend: **establish the convention + moves + dotted surface now**, design
   it forward-compatible with disk-resolution, and treat disk-loading as a
   named follow-on (it is the DS-1 inline-vs-import gap). Pin the boundary so
   Ergo builds a bounded change.

## Proposed Section/Domain mapping (reviewable — grounds the move pass)

From `06 §Sections` + actual package **contents** (not just names):

| Package (today) | Contents | Proposed home |
|---|---|---|
| `core/empty-dec` | `Empty`, `Dec`, `absurd` | **Core** |
| `lawful-classes` | `Eq`/`Ord`/`DecEq` scaffolding | **Core** |
| `lawful-functors` | `Semigroup`/`Monoid`/`Functor`/… | **Core** |
| `transport` | `subst`/`cong`/`sym`/`trans` (equality transport) | **Core** ⚠ |
| `collections` (`map`) | `Map`/`Set` + laws | **Data** (Collections Domain) |
| `parsing` | parser combinators | **Capability** (Parsing Domain) |
| `verify` | proof-erasure boundary checker | **Capability** (Verify Domain) or Algorithm — enclave call |

**⚠ Reconcile an inconsistency in `06`.** `06 §Capability` says *"today's parsing
and transport packages seed this Section,"* but `transport.ken`'s **content** is
propositional-equality transport (`subst`/`cong`/`sym`/`trans`), which `06
§Core` explicitly lists as Core. Either `06`'s "transport" refers to a *future*
wire/serialization package (a Capability encodings Domain) and the current
`transport.ken` is misnamed for its Core role, or the prose is stale. **Resolve
this in the pin** (likely: current `transport.ken` → Core, possibly renamed to
its role e.g. `equality`/`transport`; the Capability "transport" is a future
wire package) and correct `06` to match.

## Deliverables

1. **Pinned scheme** (enclave, durable): the path template, the import/module
   surface + the path↔import rule, the naming/casing convention, and the
   resolution-scope boundary — transcribed into this WP or a brief Ergo builds
   against.
2. **`catalog/` file moves** into `<Section>/<Domain>/<pkg>` per the approved
   mapping, updating every in-tree cross-reference (the DS-1 entry's §-comments
   cite `catalog/packages/...` paths; the guide + `07` reference paths).
3. **Parser/resolver:** accept the dotted import path; module-name inference or
   header per the pin. (`crates/` delta → CI-gated.)
4. **Docs:** `06` corrected (transport reconcile); `07-catalog-style-guide.md`
   gains the path↔import convention as a normative rule; the `write-ken` guide
   references it.

## Sequencing + gate

**This precedes DS-2** and should be sequenced **ahead of / coordinated with**
FR-3 and the guide-update (both reference catalog paths — moving files after they
land means double-updating). Cadence: Steward frame → **enclave pins the scheme +
mapping** (Architect design + spec-author surface/`06`-fidelity) → durable brief
→ Ergo builds (moves + parser/resolver) → gate (Architect soundness on the
resolver touch + QA) → `git_request` to Steward → CI-gated. On merge, the catalog
addressing is regular and DS-2 (and every later entry) lands at its taxonomic
path with a mirrored import.
