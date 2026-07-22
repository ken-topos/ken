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
- **Validating an entry: `ken run` vs. `ken check`.** A runnable entry (its
  Definition fence ends in a nullary `proc main`) is validated with `ken run`,
  which elaborates every fence and then drives the IO. A **pure-library**
  entry (no `proc main` — the common shape for a catalog package) is validated
  with `ken check <file>` instead: it runs the identical elaboration and
  fence-role checking `ken run` does, then stops before the IO-drive step.
  `ken run` on a pure-library entry always fails with `"last definition is not
  an IO tree"` — that failure is not evidence against the entry; cite `ken
  check`'s exit code as the entry's fence-validation evidence instead.

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

## 5. Findings — RETIRED from the catalog entry (2026-07-11)

Writing real Ken is still the fleet's dogfooding instrument, but the **Findings
section is retired from the outsider-facing catalog entry.** Its function —
recording language gaps discovered while authoring — has migrated to the **live
gap-escalation flow**: an author who hits a gap escalates it in the moment, the
Architect rules, and the durable ones are captured in the campaign's
forward-candidates register and the `write-ken` technique corpus. A per-entry
Findings section is now both a **stale duplicate** of that live channel and
**inward-facing content in a reader-facing product** — the exact class §8 forbids.

- **Do not add a Findings section to new entries.**
- **Existing Findings sections are removed** in the outsider-prose sweep. Before
  removing one, **harvest any gap it records that is not already captured** in the
  live channel (route it to the Steward for the forward-candidates register); then
  delete the section. The dogfooding signal is preserved by the channel, not the
  section.

(Kernel-reduction defects, sugar candidates, and abstraction candidates are still
the high-value signals — they just route live, not through a catalog section.)

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

Use the proof keyword to record **membership**, not an informal proof role.
When a named theorem belongs to one subject's public theory, write
`proof name for S`; the declaration is bound and referenced as `S::name`,
including recursive self-references. If exactly one owning subject can be
named, attach the proof to it. Use `lemma` only when no single subject owns the
theorem:
for an interior step, or for a cross-cutting law about several definitions in
interaction. Reuse is orthogonal to membership: citing a subject-owned law from
another proof does not demote it to `lemma`. This convention refines which proof
keyword to use; it does not change the existing `fn` versus Ω-proof partition.

### 6.1 Local bindings as exposition

Use `let` to give an intermediate term a local name when that name states a
domain concept, proof endpoint, invariant, or stage that would otherwise be
visible only as nested syntax. A useful local name lets the reader describe the
remaining body at a higher level than the right-hand side. Prefer a binding for
a repeated non-atomic expression, for an important middle endpoint or item of
evidence in a proof chain, or for a single-use stage with a real domain role.

Expression length is evidence, never the decision. Keep a familiar one-step
expression inline when a binding would only rename its syntax. Small exhaustive
matches, direct structural recursion, one constructor assembly, and a one-step
`cong` are often clearer with their structure visible. A binding earns its place
only when its name states a concept the reader would otherwise have to infer.
There is no binding quota, depth threshold, or minimum count.

Bind at the narrowest scope that contains every use. Do not hoist a
branch-specific computation before a `match`, and do not move an effectful
computation across a branch or another effect. A local `let` is non-recursive;
use a top-level helper for recursion or genuine reuse. Name the role, such as
`sorted_tail`, `updated_acc`, `left_round_trip`, or `lookup_after_insert`, not
the mechanism: `tmp`, `value2`, `intermediate`, and `step_result` merely replace
visible syntax with indirection. A long preamble of unrelated bindings is a
signal to split a helper or lemma, not to build a local namespace.

When two or more sequential bindings are warranted, write one binding group
with `;` between bindings and no trailing separator before `in`. Order the
bindings by dependency: each right-hand side may use earlier names, but not its
own name or later names, and duplicate names in one group are rejected. The
formatter coalesces a maximal directly nested chain of at least two lets into
this group form. A one-binding `let` is already canonical and remains the same
production; do not manufacture a group of one
(`spec/30-surface/32-grammar.md:200-221`,
`spec/30-surface/31-lexical.md:228-231`).

For proof code, name the middle endpoints and evidence that make a multi-step
chain readable, while leaving the final `trans`/`cong`/`J` skeleton visible.
This refines the proof-family organization above; it does not hide the proof
term or change its trust posture. Naming conventions in §9 apply equally to
local bindings.

A style-only `let` refactor preserves public names, result types, proof claims,
and the `trusted_base()` delta. It is still an AST change, not whitespace, and
therefore takes the normal elaboration and behavioral gates. In effectful code,
the gate must also confirm that branch placement and effect order did not move.

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

Per `docs/PRINCIPLES.md` #14 ("Nothing required lives in a comment — express
it in the language"): a comment is unchecked prose — nothing verifies it,
nothing re-checks it, it can silently drift from the code it annotates. So no
comment is ever *required* to carry proof-review information, in any entry —
package or teaching doc alike, no fence-role exception. A required fact
belongs in the entry's Ken proper, in the construct built to check it:

- A **contract or invariant** — a `requires`/`ensures` clause, or a
  refinement type on the value it constrains.
- A **proposition** — a `law`/`prop`/`lemma` declaration (or, as the catalog
  most often states one, an ordinary `fn`/`const` whose result type *is* the
  property and whose body *is* the checked proof term), discharged directly
  or via `prove`.
- A **trust boundary** — an `Axiom`, recorded in the entry's Trust &
  derivation section as a `trusted_base()` delta (§7).

When a required fact has no home in one of these, that is a signal to extend
the language (escalate the gap live, §5) — never to enshrine the fact in a
comment instead.

**Prose is written for an outsider.** The catalog is the project's public face —
a curious engineer reads it with **no visibility into the fleet's internal
process.** So all prose (motivation, narrative, `-- ` comments) reads as ordinary
standard-library documentation and carries **no insider references:** no WP IDs
or `wp/…` paths, no campaign or `DS-N` codenames, no internal decision history
("erratum", "reversed", "operator ruling", "judgment call L5"), no mention of the
agent team / moot / convo, and no crate-internal source paths (`crates/…`,
`prelude.rs`) **unless** the path is genuinely part of the public contract the
reader must know — and then name the contract, not the file. State motivation as
**timeless design rationale** the reader can use: *why* a construct exists and how
it relates to its neighbours, never the history of how the fleet arrived at it.
(§9 already bars WP-encoding in identifiers; this is the same discipline for
prose.) For two coexisting sums, *"`Result` is the error-biased sum wired into the
effect system; `Either` is the neutral one — both coexist"* is good motivation;
*"an earlier DS-3-era spec erratum subsumed `Either` into `Result`… the operator's
later L5 ruling reversed that"* is insider history to cut.

Comments and the surrounding Markdown prose carry only **genuine
narrative** — proof strategy, naming rationale, why a thing exists the way it
does: context a reviewer benefits from, but that nothing downstream depends
on. State that narrative in the surrounding prose, immediately before or
after the fence it concerns (`agent/playbooks/tools/write-ken.md`'s "prose is
the comment layer" rule); reserve an in-fence `-- ` comment for the rare
annotation that must point at one specific token and genuinely can't live in
prose.

The law itself is the checked type; the proof term is the checked body.
Nothing above the fence needs to restate either — the proof strategy is
narrative, so it belongs in the prose, not a comment:

The base branch reduces both endpoints to `Nil`, so the equality collapses
to `Top` and closes with `tt`; the step branch lifts the tail induction
hypothesis under `Cons` with `cong`.

```ken
fn list_right_unit ...
```

## 9. Naming

Names tell a reviewer the abstraction, property, and role. Do not encode WP
history into durable names. Local names follow the same rule: use `let` when the
name exposes meaning hidden by nested syntax, with the counter-rule and narrow
scoping requirements in §6.1.

**Casing (operator-ruled, `ds-campaign-judgment-log.md` §L6, effective now for
all NEW authoring).** Ken adopts the Python convention — class-like →
PascalCase, instance-like → snake_case — over the FP-common all-camelCase, on
the operator's readability judgment that it distinguishes class-like from
instance-like at a glance and reads better for the far-more-common instance
identifiers:

- **PascalCase**: types/type-constructors (`Either`, `Option`, `Nat`, `List`,
  `Vec`), type classes (`Functor`, `Applicative`, `Monad`, `Traversable`), and
  **data constructors** (`Left`/`Right`/`Some`/`Ok`/`Cons`/`Suc`) — these are
  class-ilk, they construct values.
- **snake_case**: functions/combinators AND class methods/record fields —
  these are instance-ilk. E.g. `getOrElse→get_or_else`, `isSome→is_some`,
  `mapErr→map_err`, `andThen→and_then`, `unwrapOr→unwrap_or`,
  `concatMap→concat_map`. A single-word name (`either`, `swap`, `zip`,
  `foldl`, `map`, `ap`, `pure`) is already fine either way.
- **Boundary calls:** data constructors follow types (Pascal); class
  *methods* follow functions (snake).

**Scope — forward directive only.** Write every NEW catalog `.ken`/`.ken.md`
identifier to this standard. Do **not** rename identifiers in already-landed
code to fix this now — the standard's own bulk renaming pass is deferred to
ride the `.ken` → `.ken.md` literate transformation (one touch per file,
casing + literate encoding together), owned by the Librarian. A camelCase
name you find in landed code (this guide's own `compareChar` example below
included) predates L6 and is not yet non-conformant-by-omission; it's queued
for that pass, not a defect to fix on sight.

- Lowercase identifiers for term/type parameters referenced in term bodies;
  capitalized identifiers are constructor-shaped in the current surface.
- Public operations: stable domain/action names (`list_append`, `bool_and`,
  `compareChar` — pre-L6, pending the casing pass; a new entry would spell
  this `compare_char`).
- Class fields: short and conventional when the class gives context (`map`,
  `foldr`, `assoc`, `left_unit`).
- Public law proofs with one owning subject: attach the law as
  `proof name for subject` and refer to it as `subject::name` (`add::assoc`,
  `leq_nat::refl`). Do not encode this membership in a flat
  `name_subject`/`subject_name` identifier.
- Ownerless interaction laws and interior proof steps: descriptive `lemma`
  names that make the several definitions or local role clear.
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
current · repeated non-atomic terms are named once where the name carries
meaning · multi-step proof chains name important endpoints or evidence · local
bindings preserve branch and effect order by staying at the narrowest scope ·
every local name improves the vocabulary rather than hiding syntax · a long
binding chain is reconsidered as a missing helper or lemma · pilot rule
respected unless a prior pilot landed · tests and trust-drift greps recorded.

**Review checklist.** Diff is docs/style/organization only for the stated scope ·
tests cover behavior preservation · no new `Axiom`/postulate/primitive/trusted
entry · any file split preserves imports and consumers · the entry connects
source order to spec chapters and acceptance criteria · a reviewer can find
operations, laws, helpers, proofs, trust posture, references, and findings
without reading the WP thread · repeated non-atomic terms carry meaningful local
names · proof-chain endpoints and evidence are reconstructable without expanding
every nested term · bindings stay narrow enough to preserve branch and effect
order · names add vocabulary rather than hiding syntax · long binding chains
trigger a helper-or-lemma review.

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
  both at once (no `Data/Sequence.ken` beside a `Data/Sequence/`
  directory).
- **Module is path-inferred.** A file is an implicit module named by its path
  (`33-declarations.md §3.1`) — there is **no mandatory in-file `module A.B.C`
  header**; the directory path is the single source of truth. (An *optional*
  checked header enforcing `header == path` is a future ergonomic add, not
  part of this rule.)
- **Import-availability honesty (must agree with the `write-ken` guide).**
  This rule fixes *addressing* — it is a separate fact from whether the
  catalog corpus exercises the resolution the addressing names. The loader
  resolves cross-file `import` and facade `export M (…)` edges lazily across
  compilation units (`spec/30-surface/33-declarations.md:147-158`); within
  one compilation unit, dotted module refs already work too. What's still
  true is narrower: no landed catalog entry yet exercises the cross-file
  case — a catalog entry that needs another package's helper today still
  inlines it (the DS-1 pattern, `Core/Logic/EmptyDec.ken.md §6`), not imports
  it. State this plainly — a corpus-coverage gap, not a loader capability
  gap — in any entry or guide passage that demonstrates the dotted syntax.
