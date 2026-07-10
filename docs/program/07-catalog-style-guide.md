# Catalog style guide — the standard entry format

**Owned by:** Spec enclave for the standard; Steward/Librarian for durable
navigation. Governs first-party packages under `catalog/packages/`. It does not
change language semantics, proof obligations, or the package trust model.

The catalog is Ken's reference corpus — how Ken code should state behavior, carry
proofs, disclose assumptions, stay readable to a human reviewer, teach a
newcomer, and serve as training data (`06-catalog-campaign.md`, the four
purposes). This guide fixes the **standard entry format** that lets one artifact
serve all four, and it is the review contract for catalog refinement WPs: a
component may first land as a functional, proved artifact, then a follow-on
refinement WP raises it to this standard without changing behavior.

## 1. The entry is a literate `.ken.md` document

Each catalog package's primary artifact is a **literate `.ken.md` document** that
carries, in one file: narrative, the component's code, its laws and proofs,
worked examples, references, and reviewer navigation.

- The `` ```ken `` code blocks **tangle to a compilable module** (see
  `crates/ken-elaborator/src/literate.rs`). The tangled `.ken` is a *build
  artifact*, not the source of truth — the `.ken.md` is.
- This **subsumes the old `.ken` + `MANIFEST.md` split** (`subsume-don't-
  proliferate`): the manifest's contract — source map, derivation path,
  `trusted_base()` delta, proof-family map — becomes structured sections of the
  entry (§7). Any machine-read trust field that tooling still needs a stable
  location for is reconciled by the Librarian during migration; the reader sees
  one document.
- Purpose 1 reads the code + laws; purpose 3 reads to its chosen depth; purpose 2
  is the literate whole; purpose 4 is the Findings section. One source,
  mechanically separable (prose vs. fenced code).

## 2. The standard entry format

**Front matter.**

- An H1 title and a one-line statement of intent (the newcomer's and the
  training-signal's first anchor).
- An **index**: anchor links to the sections below.
- **Named reading paths** — the mechanism that makes progressive disclosure real
  for the catalog's wide persona range. Route each persona to its depth, e.g.:
  - *Newcomer* → Motivation → Using it
  - *Practitioner* → Using it → Laws & proofs
  - *Researcher* → Laws & proofs → Design notes
  - *Porting from Haskell/Lean/Agda* → Design notes

**Required sections, in order** (native markdown heading anchors; the index links
to them):

1. **Motivation** — what the component is, what it refines or generalizes, why it
   exists. Teaching-first prose.
2. **Definition** — the component's code in `` ```ken `` blocks (tangles to the
   module). This is the canonical source.
3. **Using it** — worked examples and common idioms (positive usage).
4. **Laws & proofs** — the stated laws and their inhabited proof terms (§6). The
   contract, machine-checked.
5. **Design notes** — why this shape; alternatives rejected; the place for
   pedagogical negative examples (§3).
6. **Findings** — what writing this taught us about Ken (§5). First-class.
7. **References** — external orientation and sources (§4).
8. **Trust & derivation** — derivation path from built-ins, `trusted_base()`
   delta with every honest exception, proof-family map, consumers (§7).

An entry is exemplary when a reviewer can answer, from the entry alone without
reverse-engineering build history: what abstraction this provides and which spec
chapter/WP owns it; which public names are stable and where their laws are
stated; which proof strategy discharges each law; the derivation path; the trust
delta; which helpers are private scaffolding and why; and which tests or
conformance seeds prove behavior and trust posture.

## 3. Code-block roles (the fence taxonomy)

Pedagogy needs negative examples, and negatives must never tangle into the
compiled module. The literate extractor compiles **only** an exact `` ```ken ``
fence; every other fence is prose. Roles (space form `` ```ken <role> ``, not a
colon — consistent with the landed `` ```ken ignore `` and it preserves
first-token `ken` highlighting):

| Fence | Tangles as source? | CI checks | Use |
|---|---|---|---|
| `` ```ken `` | yes | must elaborate | the component (Definition) |
| `` ```ken ignore `` | no | none | pure illustration/snippet |
| `` ```ken reject `` | no | **asserts rejection** | negative example, kept honest |
| `` ```ken example `` | no | elaborates vs. module | positive usage, checked not shipped |

- **Canonical code is THE code** (in Definition). Alternatives and anti-patterns
  live only in Design notes, marked `` ```ken ignore `` or (preferably)
  `` ```ken reject `` so a stale negative that silently starts compiling fails
  CI, and a model training on the entry is not taught an ambiguous idiom.
- The **checked** roles (`reject`, `example`) depend on the fence-roles language
  WP (`wp/catalog-literate-fence-roles`). Until it lands, negatives use
  `` ```ken ignore `` — safe (never tangled) but unchecked — and the entry notes
  the gap.

## 4. References (required)

Every entry carries a References section that orients the reader and grounds the
component in the literature. Include, where they exist:

- **Wikipedia** — for concept orientation across the persona range.
- **Papers** — arXiv, university pages, or author-owned sites. Prefer stable,
  canonical, author-owned URLs over aggregators; note paywalls.
- **Books** — title, author, edition; a stable link where one exists.
- **Source repositories** — GitHub, GitLab, Codeberg, SourceHut, and other OSS
  forges: reference implementations, the upstream project, or notably different
  designs of the same abstraction. Prefer a permalink (a pinned commit/tag), and
  name the license.

**Clean-room boundary (`CLEAN-ROOM.md`).** A source-repository reference is for
the *reader's* orientation and comparison — it is **never** an authoring source.
The entry's own code derives from Ken's spec, not from any linked repo; do not
copy or transcribe code from a referenced forge, and never cite a copyleft
(GPL/AGPL/CeCILL) source you consulted for approach — those stay enclave-only.
Cite it as "here is how others solved this," not "here is where this came from."

Format each as: title — author/venue — stable link — one line on *why* it's
relevant here. Catalog-wide reference conventions (link hygiene, preferred
sources, citation shape, license labelling) live in `catalog/REFERENCES.md`;
per-entry references live in the entry.

## 5. Findings (required — the inward purpose)

Writing real Ken is the fleet's dogfooding instrument. Every entry carries a
Findings section recording what authoring it taught us about the language, each
item routed (`06-catalog-campaign.md`, teaming):

- **Kernel-reduction defect** → Kernel (via the enclave). The highest-value
  Finding an entry can produce.
- **Sugar candidate** — a recurring implementation shape the surface should
  collapse → Ergo.
- **Abstraction candidate** — a recurring shape that should become a general
  `def`/`lemma`/`prop` → Ergo, or grown in-catalog as a shared package.

An empty Findings section is allowed (nothing surfaced); omitting the section is
not. Record the concrete shape, not a vague impression, so the receiving team can
act.

## 6. Proof presentation

Proof-carrying code should show the shape of the proof, not bury it under
incidental helpers. Preferred proof-family structure: state the public law or
class field; define the operation proved (if not already public); define the
minimum local helpers for readability; prove carrier-specific lemmas in a stable
order (base/finite cases → induction steps → bridge lemmas → final assembly);
assemble the instance or public theorem.

Every law field that claims to be proved must be inhabited by a **real proof
term**. An `Axiom` is allowed only when the entry's contract explicitly permits an
audited delta (e.g. a primitive carrier whose universally-quantified laws are not
kernel-provable). In that case: Trust & derivation lists the delta; a source
comment explains why the proof is unavailable at this layer; conformance/tests
distinguish the audited-delta case from a zero-delta lawful exemplar; and the
entry does not present the instance as zero-delta. Do not replace a hard proof
with a more literate *statement* of the law — the source must still carry the
proof term, or the entry must disclose the trusted-base delta.

## 7. Trust, derivation & navigation

The Trust & derivation section carries what the old `MANIFEST.md` did:

1. Spec catalog entry and build/refinement WP.
2. Public API (stable names).
3. Source map — a compact table from reader task to entry section/anchor.
4. Derivation path from built-ins.
5. `trusted_base()` delta, including every honest exception.
6. Proof families.
7. Consumers and compatibility notes.
8. Validation evidence, or the conformance seed that owns validation.

Link to the spec for the contract; use this section to explain how the entry
*realizes* it — do not copy the spec chapter in. The standard-package index in
`spec/50-stdlib/README.md` stays a spec index; package-level style and navigation
live here and in `catalog/`.

## 8. Comments

Comments are required where they carry proof-review information — a contract,
invariant, proof strategy, trust posture, or non-obvious elaboration constraint.
They must not restate the syntax below them. Required classes: entry/package
header (spec chapter/WP, public abstraction, derivation path, trust summary);
section comments; law comments (the proposition in reader terms + intended proof
route); helper comments (why it exists, when not obvious); trust comments (every
`Axiom`, primitive wrapper, audited delta, or tested-not-trusted boundary);
staging comments (name the gated future capability when a natural law cannot yet
be proved). Prefer a compact source comment plus a Trust/derivation subsection
over a long inline essay.

```ken
-- Right unit for append. The base branch reduces both endpoints to `Nil`,
-- so the equality collapses to `Top` and closes with `tt`; the step branch
-- lifts the tail induction hypothesis under `Cons` with `cong`.
fn list_right_unit ...
```

Not: `-- Defines list_right_unit.`

## 9. Naming

Names tell a reviewer the abstraction, property, and role. Do not encode WP
history into durable names.

- Lowercase identifiers for term/type parameters referenced in term bodies;
  capitalized identifiers are constructor-shaped in the current surface.
- Public operations: stable domain/action names (`list_append`, `bool_and`,
  `compareChar`).
- Class fields: short and conventional when the class gives context (`map`,
  `foldr`, `assoc`, `left_unit`).
- Public law proofs: subject plus law (`list_assoc`, `list_right_unit`,
  `band_left_unit`).
- Private helpers: a role suffix, sitting near the proof family they serve.
- Avoid opaque sequence names (`lemma1`, `helper2`, `d3_case`) and overclaiming
  names (a tested function named as if it were a proof-carrying theorem).

| Suffix | Use |
|---|---|
| `_step` | One inductive or recursive step. |
| `_case` | A finite case-split branch helper. |
| `_dispatch` | Chooses among already-named cases or branches. |
| `_bridge` | Converts between equivalent statement shapes. |
| `_locality` | Proves a result is unchanged outside a focused key/path. |
| `_expected` | Computes the expected value used by a statement. |

Refinement may rename private helpers freely; public renames require a
compatibility map (old name, new name, reason, consumers checked).

## 10. Two-phase build & the refinement WP contract

Catalog work has two named phases (`06-catalog-campaign.md`): a **functional
build** (implement, run, prove the required laws; rough-but-correct source may
merge if the proofs are real, the derivation path is stated, and the trust delta
is honest) and a **catalog refinement** (raise the landed component to this
standard entry format, behavior-preserving). They have different bars; a
refinement is a real WP with its own gate, not cleanup hidden in a retro.

A refinement WP is behavior-preserving unless its kickoff says otherwise.
Definition of done:

- Public API/proof names preserved, or the compatibility map is approved.
- Existing package tests and relevant acceptance/conformance tests pass.
- `crates/ken-kernel` and `Cargo.lock` diffs are empty.
- No new `Axiom`, postulate, primitive, opaque trusted entry, raw
  proof-relevant `data ... : Omega`, or proof-surface downgrade.
- `trusted_base()` delta unchanged or narrowed.
- The entry's Trust/derivation and source map reflect the new organization.
- The diff improves at least one concrete axis: entry structure, reading paths,
  examples, references, naming, comments, proof-family grouping, or Findings.
- The retro records which guide rules were useful and which were ambiguous.

Review roles: **owning build team** (Foundation) — behavior preservation,
imports, public names, tests, compatibility map; **QA** — gates, diff hygiene,
trust-drift grep, exact-head validation; **Librarian** — durable navigation,
source map, README/spec-index pointers, findability; **Architect** — only when
the refinement crosses a proof boundary, changes a law shape or abstraction
boundary, alters a trust-delta claim, or introduces a client-observable split. A
pure naming/comment/navigation refinement whose proof terms, laws, and
abstraction boundaries are unchanged does not require the Architect.

## 11. Pilot

The **first reframed `.ken.md` entry doubles as the format pilot** — it exercises
the full standard end to end (front matter, reading paths, the fence roles,
References, Findings, Trust/derivation) before the format is applied to large
proof-heavy bodies. Prefer a small, proof-strategy-rich, low-delta component for
the pilot; do not begin with a CAT-4-scale body. Pilot evidence shows readability
improved without behavior drift: before/after source map, public names checked,
tests on the exact head, trust-drift grep, and a short note on which checklist
items were exercised.

## 12. Checklists

**Functional build gate.** Component implements the behavior · required laws are
real proof terms or the trusted delta is explicit and spec-permitted · derivation
path stated · relevant conformance/acceptance tests pass · kernel/Cargo lock
invariants checked when claimed · rough organization acceptable only if a
refinement follow-on is recorded.

**Standard-entry / refinement author checklist.** No semantics, law shapes, or
proof requirements weakened · public API/proof names preserved or mapped · front
matter has index + reading paths · required sections present and in order ·
canonical code in `` ```ken ``, negatives in `` ```ken reject ``/`` ```ken
ignore `` · References section present (Wikipedia/papers/books as they exist) ·
Findings section present (empty allowed) · Trust/derivation and source map
current · pilot rule respected unless a prior pilot landed · tests and
trust-drift greps recorded.

**Review checklist.** Diff is docs/style/organization only for the stated scope ·
tests cover behavior preservation · no new `Axiom`/postulate/primitive/trusted
entry · any file split preserves imports and consumers · the entry connects
source order to spec chapters and acceptance criteria · a reviewer can find
operations, laws, helpers, proofs, trust posture, references, and findings
without reading the WP thread.

## 13. Path ⇔ import — the normative rule

(`docs/program/wp/catalog-taxonomy-paths-imports.md`, enclave-pinned, landed.)
A package's file path and its dotted import specification are the **same
thing**, spelled two ways — this is what lets a reader go from an import to a
file, and back, without a lookup table.

- **Identity map.** `import A.B.C` ⇔ `catalog/packages/A/B/C.ken[.md]`,
  mechanically, at any depth: **N dotted components → (N−1) directories + a
  leaf file.** The resolver never decodes whether a component is a Section, a
  Domain, or the package itself — that is taxonomy metadata about *where a
  file is filed* (`06-catalog-campaign.md` "Sections and Domains"), not
  something the addressing rule reads.
- **Casing.** Path/module components are **PascalCase `ConId`**
  (uppercase-initial, `31-lexical.md §1`) — the leaf filename minus its
  extension is *exactly* the final import component, zero transform. Kebab or
  snake_case directory/file names are not valid module identifiers.
- **Variable depth, no synthesized Domain level.** A Domain directory appears
  only when its Section is actually subdivided (`06`); do not insert one to
  force a fixed depth.
- **Leaf-or-namespace, never both.** A name at a given level is either a
  package (a `.ken`/`.ken.md` leaf file) or a Domain (a directory) — never
  both at once (no `Data/Collections.ken` beside a `Data/Collections/`
  directory).
- **Module is path-inferred.** A file is an implicit module named by its path
  (`33-declarations.md §3.1`) — there is **no mandatory in-file `module A.B.C`
  header**; the directory path is the single source of truth. (An *optional*
  checked header enforcing `header == path` is a future ergonomic add, not
  part of this rule.)
- **Import-availability honesty (must agree with the `write-ken` guide).**
  This rule fixes *addressing* — it does **not** mean cross-file `import`
  resolves. There is no disk loader yet (module path → file on disk is a
  named follow-on capability); within one compilation unit, dotted module
  refs already work, but a catalog entry that needs another package's helper
  today still inlines it (the DS-1 pattern, `Core/EmptyDec.ken.md §6`), not
  imports it. State this plainly in any entry or guide passage that
  demonstrates the dotted syntax — don't imply cross-file import works.
