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
| `transport` | `subst`/`cong`/`sym`/`trans` (equality transport) | **Core** — pinned (no rename) |
| `collections` (`map`) | `Map`/`Set` + laws | **Data** (Collections Domain) |
| `parsing` | parser combinators | **Capability** (Parsing Domain) |
| `verify` | proof-erasure boundary checker | **Capability** (Verify Domain) — pinned |

**⚠ `06` inconsistency — RESOLVED in the Enclave pin below (P4):** current
`transport.ken` → **Core** (its content genuinely *is* equality transport, no
rename); `06 §Capability`'s "transport seeds Capability" prose is stale and is
corrected to seed with `parsing` only, reserving the Capability "transport" name
for a future wire/serialization package.

## Enclave pin (durable — Ergo builds against this)

Pinned by the enclave: **Architect** (resolver / import-scheme / design) +
**spec-author** (`06`-fidelity, the `33`-surface `import` grammar, the `07`
casing/path rule, the mapping labels). **No kernel or `trusted_base()` delta
anywhere** — `modules.rs` / the parser is a pure surface layer (no `GlobalEnv`/Σ
change); the merge takes a normal Architect soundness gate on the parser touch.

### Central principle — the path↔import map is a syntactic IDENTITY, role-blind

`import A.B.C` ⇔ `catalog/packages/A/B/C.ken`, mechanically, at **any** depth:
**N dotted components → (N−1) directories + a leaf `.ken`**. The resolver never
decodes whether a component is a Section, a Domain, or the package — that is
taxonomy metadata governing *where a file is filed*, not something the addressing
rule reads. This decoupling is what makes "no lookup table" true, and it makes P1
fall out for free.

### P1 — Path depth: VARIABLE (no mandatory Domain level)

`06` makes a Domain an **optional** subdivision, so depth varies: `Section/Pkg`
when unsubdivided, `Section/Domain/Pkg` when subdivided. Safe **because the
identity rule regularizes whatever depth exists** — the last component is always
the leaf module, all preceding are directories. Do **not** synthesize a Domain
level to force fixed depth.

- **Constraint — leaf-or-namespace, never both:** a dotted path resolves to
  exactly one of {a `.ken`/`.ken.md` file, a directory}, never ambiguous — no
  `Data/Collections.ken` beside a `Data/Collections/` directory. A name at a
  level is either a package (leaf) or a Domain (dir), so the bijection is total
  (Rust `mod.rs` / Lean discipline).

### P2 — Import/module surface + the bijection (`33`, `31`)

- **(a) Dotted `import`/`use`/`module` — accept a dotted module path.** Today each
  calls `expect_ident` (single ident, `parser.rs`); the lexer already emits `ConId
  Dot ConId …` and expression-position dotted refs already parse (`parse_dotted`).
  Add one shared `parse_dotted_module_path` (a `ConId` then zero-or-more `.ConId`)
  reused by `import` / `import…as` / selective / `use` / `module`. Grounded in `33
  §3` (qualified modules, `M.N` nesting) and the module layer already keying
  dotted `"M.N"` (`modules.rs`). No clash with expression-position projection:
  module components are `.ConId` (uppercase); `.field` projections are `.ident`
  (lowercase).
- **(b) Module = path-inferred, the single source of truth.** `33 §3.1`: "a file
  is an implicit module named by its path." The directory path **is** the module
  path — **no mandatory in-file `module A.B.C` header** (a header would be a
  second source of truth to reconcile). An *optional* checked header enforced
  `header == path` is a later ergonomic add, **out of this WP**.
- **(c) Casing = directory/file names == module identifiers VERBATIM.** Module
  components must be `conid` — **uppercase-initial** (`31 §1`/`31-lexical`:
  "`conid` — constructor / type / **module** names: uppercase-initial"). Kebab
  dirs (`lawful-classes`) are **not** valid identifiers (hyphens aren't ident
  chars), so an identity map **forces** PascalCase directory/file names — this is
  structural, not merely cleanest. The move renames e.g.
  `lawful-classes/lawful_classes.ken → Core/LawfulClasses.ken` ⇔
  `import Core.LawfulClasses`. The leaf filename minus its extension is **exactly**
  the final import component — zero transform.

### P3 — Resolution scope: bounded (addressing now, loader deferred)

This WP delivers **(i)** the path convention, **(ii)** file moves + PascalCase
renames + in-tree cross-ref rewrites, **(iii)** the parser accepting dotted
`import`/`use`/`module`. It does **NOT** build the disk loader (module-path →
file-on-disk): there is no cross-package loader today (`modules.rs` is
single-compilation-unit bookkeeping), and a real loader (catalog-root anchor,
file discovery, cycle detection, caching) is a distinct capability that must not
ride a file-move WP.

- **Honesty pin (load-bearing).** With the loader deferred, a dotted
  `import Data.Collections.Map` **across separate files does not resolve yet** —
  **DS-1's "inline the small helper, don't import" pattern stays** until the
  loader lands. Within one compilation unit, dotted module refs work as today.
  This WP makes the *addressing* regular (what DS-2 + the loader build against);
  it must **not** claim cross-file import works. The loader is the named follow-on
  (= the DS-1 §6 inline-vs-import gap); the identity rule is already
  forward-compatible with it.

### P4 — Section/Domain mapping + the `06` reconcile (`06`-fidelity)

Resolved homes, by **content** (`06 §Sections`):

| Package (today) | Content | Home (pinned) | Import (representative leaf) |
|---|---|---|---|
| `core/empty-dec` | `Empty`/`Dec`/`absurd` | **Core** | `Core.EmptyDec` |
| `lawful-classes` | `Eq`/`Ord`/`DecEq` | **Core** | `Core.LawfulClasses` |
| `lawful-functors` | `Semigroup`/`Monoid`/`Functor` | **Core** | `Core.LawfulFunctors` |
| `transport` | `subst`/`cong`/`cast`/`sym`/`trans` over `J` | **Core** | `Core.Transport` |
| `collections` | `Map` (+`Set`, …) + laws | **Data / Collections** | `Data.Collections.Map` |
| `parsing` | parser combinators | **Capability / Parsing** | `Capability.Parsing.…` |
| `verify` | proof-erasure boundary checker | **Capability / Verify** | `Capability.Verify.…` |

(Exact leaf-package names are finalized in the move pass; the imports show the
identity for representative leaves.)

- **`transport` → Core; `Core/Transport.ken` is honest, no rename.** Content is
  the five equality-transport combinators, thin non-recursive wrappers over the
  surface former `J` (`53-transport.md`) — Core proof-utility, zero
  `trusted_base()` delta. **`06`-fix:** the `06 §Capability` line "today's parsing
  **and transport** packages seed this Section" is stale for `transport`; correct
  it to seed Capability with **`parsing` only**, and **reserve the Capability
  "transport" name for a future wire/serialization (encodings Domain) package**
  distinct from this Core one.
- **`verify` → Capability (Verify Domain)** (spec-author's `06`-label call,
  concurring with the Architect's lean and the Steward's non-binding lean): it
  enforces a **trust boundary** (proof-erasure correctness over checked-core
  packages), a focused *competence* — not a general-purpose algorithm over the
  Data Section's structures, which is what `06 §Algorithm` scopes (sort-like
  operations).
- **`collections` → a `Data/Collections/` Domain dir of per-structure leaf
  packages** (`Data/Collections/Map.ken`, later `Set.ken`, `Vector.ken`, … ⇔
  `import Data.Collections.Map`) — matching Data Section → Collections Domain →
  per-structure packages, and extensible as the Domain grows. **Defer the final
  split to content-inspection in the move pass:** if `collections.ken`'s content
  is genuinely one cohesive unit rather than a Map/Set split, keep it whole
  (`Data/Collections.ken` ⇔ `import Data.Collections`, honoring leaf-or-namespace).
  spec-author / Ergo resolve per content.

### P5 — The `06` / `07` / guide doc pins (apply in the build, per this text)

- **`06`:** correct `§Capability` per P4 (drop today's `transport` from the seed
  list, keep `parsing`, reserve the future wire "transport"); no other `06`
  change.
- **`07-catalog-style-guide.md`:** add a **normative "Path ⇔ import" rule** — the
  identity map (central principle), PascalCase `ConId` path/module components
  (P2c), variable depth + leaf-or-namespace (P1), path-inferred module (P2b).
- **Import-availability honesty (must agree with the in-flight DS-1 guide
  update).** Both `07` and the `write-ken` guide must state that **cross-file
  `import` is not available yet** — the current pattern is DS-1's
  inline-a-small-helper; the dotted-import syntax is "the addressing you will use
  **once the loader lands**," not a working cross-file mechanism today. The `07`
  rule text (spec-author) and the Librarian's DS-1 guide prose must **agree** on
  this current-state wording (the Steward coordinates the two so they don't
  diverge).

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
