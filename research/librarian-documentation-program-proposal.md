# Librarian documentation program proposal

**Status:** Research proposal; no `library/` implementation is performed here.
**Date:** 2026-07-18
**Scope:** The target product-documentation corpus, its relationship to the
existing `spec/`, `catalog/`, `docs/`, and `agent/` trees, and a staged program
for the Librarian.

## Recommendation

Create `library/` as Ken's durable **product-documentation portal**. Organize
the human corpus by reader need, not by the repository's implementation teams,
and make the primary learning path **reading Ken** rather than writing it.
Create `library/agents/` as a separate, machine-consumable product-knowledge
corpus made of small task and domain modules that can be selected into a coding
agent's context.

The directory should not become a second specification or a copy of the
catalog. The authority split should remain:

- `spec/` is normative;
- `catalog/packages/` is the canonical first-party package source and its
  literate package documentation;
- `conformance/` pins observable and rejection behavior;
- `docs/adr/` records architectural decisions;
- `agent/playbooks/` and `agent/memory/` remain the Steward-owned practice
  corpus;
- `library/` explains, teaches, indexes, and presents the **product as built**.

That separation is the most important design choice in this report. A polished
duplicate that can drift is worse than an incomplete page that names its source
and current status.

The task is well-defined without further operator clarification. This proposal
assumes that Research recommends the shape and rollout, the Steward frames the
program, and the Librarian authors and maintains the resulting corpus. It does
not assume that the Librarian may change language semantics, package contracts,
or federation practice.

## What the survey says

There is no useful single ranking of programming-language documentation. A
tutorial, a language reference, and a package index solve different problems.
The defensible comparison is therefore by documentation function. Community
discussions repeatedly identify Rust, Racket, OCaml, Common Lisp, Python, and
Elixir as strong examples, while distinguishing the Rust Book from Rust's API
documentation and reference ([2019 discussion][gold-2019], [2024
discussion][gold-2024]). For Ken, the strongest directly useful exemplars are
Rust, Python, Go, Lean/Mathlib, Agda/Idris, and Dafny.

[gold-2019]: https://www.reddit.com/r/ProgrammingLanguages/comments/c39ib1/what_do_you_consider_the_gold_standards_in/
[gold-2024]: https://www.reddit.com/r/ProgrammingLanguages/comments/1flikdr/examples_of_great_programming_language_documentation/

| Exemplar | What it does well | What Ken should take |
|---|---|---|
| Rust | A visible bookshelf separates an introductory book, code-first examples, a precise reference, tool books, and generated standard-library API documentation. `rustdoc` runs documentation examples as tests, and docs.rs gives every published package a consistently generated, versioned page. | Separate learning from lookup; check every example; generate structural package facts; provide source links, search, versions, and offline output. |
| Python | One portal names a tutorial, library reference, language reference, HOWTOs, installation, and indexes as different destinations. | A one-screen portal with explicit reader routes and no expectation that the reference teaches beginners. |
| Go | The Tour, getting-started tutorial, specification, Effective Go, package docs, and runnable examples distinguish first contact, idiom, exact rules, and APIs. | Maintain a distinct idiom/review guide, and treat standard packages as both reusable code and examples. |
| Lean 4 | Separate books teach functional programming and theorem proving, while the versioned language reference explicitly says it is comprehensive reference rather than tutorial. It also integrates error explanations, release notes, supported platforms, build tools, and proof validation. | The closest language-level model: teach programming and proving separately, keep a comprehensive lookup reference, and document the whole parse–elaborate–check–compile pipeline. |
| Mathlib | Generated declaration pages are supplemented by topic indexes, naming/style guidance, tutorials, and type- or pattern-directed search such as Loogle. Its documentation overview itself uses the four Diátaxis modes. | A comprehensive catalog needs several discovery projections, not merely a package tree: topic, declaration/type, law, capability, platform, and assurance status. |
| Agda and Idris 2 | Both foreground interactive, type-driven work. Idris's tutorial says its examples are tested; Agda explicitly tells newcomers to use Getting Started rather than its incomplete language reference. | Show tool interaction and proof holes/errors as part of learning; mark incompleteness honestly and route newcomers around gaps. |
| Dafny and F* | Dafny documents specifications, proofs, compilation, tooling, and the language together; F*'s tutorial is unusually candid that its community is small and its documentation sparse. | Verified-language docs must explain the verifier workflow and assurance boundary. Research sophistication is not a substitute for complete navigation and current examples. |

The surveyed sources support several concrete patterns:

1. **Separate modes.** Diátaxis distinguishes tutorials, how-to guides,
   reference, and explanation because they serve study versus work and action
   versus understanding ([Diátaxis][diataxis]). Rust's Reference likewise says
   it is not an introduction and points to the Book instead
   ([Rust Reference][rust-reference]).
2. **Executable examples.** Rust runs documentation examples as tests
   ([rustdoc tests][rustdoc-tests]); Idris states that its tutorial examples are
   tested against Idris 2 ([Idris tutorial][idris-tutorial]). Ken already has a
   stronger native mechanism in checked `ken example` and `ken reject` fences.
3. **Generated reference plus authored guidance.** docs.rs automatically builds
   documentation for every published crate ([docs.rs][docs-rs]), while the Rust
   Book and Rust By Example remain authored learning works. Generated facts do
   not replace pedagogy; pedagogy should not manually reproduce generated facts.
4. **Version and status are visible.** Lean's reference identifies the exact
   Lean version it covers and includes release notes and supported platforms
   ([Lean Reference][lean-reference]). Agda labels its manual incomplete and
   directs new users elsewhere ([Agda overview][agda-overview]).
5. **Large libraries require semantic search.** Mathlib provides generated API
   pages, topic indexes, a glossary, naming conventions, and declaration-search
   tools rather than relying on its physical module tree alone
   ([Mathlib documentation][mathlib-docs]).
6. **Agent context must be selective.** Coding agents support repository-wide,
   path-specific, and task-specific instruction material. GitHub recommends
   path-specific instructions to prevent irrelevant rules from applying to the
   wrong files ([GitHub instructions][github-instructions]). Anthropic describes
   context as finite and warns that irrelevant material degrades focus
   ([context engineering][context-engineering]). Agent Skills similarly package
   instructions and optional resources as modular capabilities
   ([Agent Skills][agent-skills]).

[diataxis]: https://diataxis.fr/
[rust-reference]: https://doc.rust-lang.org/reference/
[rustdoc-tests]: https://doc.rust-lang.org/rustdoc/documentation-tests.html
[idris-tutorial]: https://idris2.readthedocs.io/en/stable/tutorial/introduction.html
[docs-rs]: https://docs.rs/about
[lean-reference]: https://lean-lang.org/doc/reference/latest/
[agda-overview]: https://agda.readthedocs.io/en/v2.7.0/overview.html
[mathlib-docs]: https://leanprover-community.github.io/documentation.html
[github-instructions]: https://docs.github.com/en/copilot/how-tos/configure-custom-instructions-in-your-ide/add-repository-instructions-in-your-ide
[context-engineering]: https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
[agent-skills]: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview

## Ken-specific design requirements

### Teach the human's real task

Ken's primary user is not a novice typist. It is a human reviewer trying to
understand agent-written software and decide whether its claims are justified.
The first learning sequence should therefore teach a repeatable reading pass:

1. identify the program's purpose, entry point, imports, and package identity;
2. read public types and refinements before bodies;
3. distinguish executable terms from propositions and proof terms;
4. find `requires`, `ensures`, laws, and named proof claims;
5. classify each claim as proved, tested, delegated, or unknown;
6. inspect effects, capabilities, authority, and information-flow boundaries;
7. audit `Axiom`, foreign declarations, declassification, and the
   `trusted_base()` delta;
8. trace the relevant data and control path only after the contract is clear;
9. distinguish reference-interpreter behavior from native-backend support;
10. follow package provenance and validation evidence.

The first substantial tutorial should be **Anatomy of a Ken program**, not
Hello World. Its exercises should ask readers to explain what a program
guarantees, what it merely tests, which authority it needs, and what would make
them reject it. A secondary authoring track can then teach a motivated human to
write the same constructs.

### Teach dependent types without requiring a type-theory course first

The guide should introduce dependent types at the point where they improve a
reviewer's reading:

- a refinement is a value plus a checked condition;
- a dependent function makes later obligations depend on earlier inputs;
- a dependent pair carries a value with evidence about it;
- an indexed family makes impossible states or branches uninhabitable;
- equality and transport explain why a term may safely cross a type boundary;
- termination is part of what makes a proof computationally trustworthy.

Each concept should have four views: ordinary-language intent, a small checked
Ken example, what the checker establishes, and what the construct does **not**
establish. Formal rules belong in `spec/`; the library supplies the reader's
mental model and links to the rule.

### Put the assurance boundary near the beginning

Most languages can defer their trust model to advanced material. Ken cannot.
The four epistemic statuses, the small kernel, the role of the elaborator and
prover, `trusted_base_delta`, interpreter/native accountability, and Ward's
delegated behavioral role belong in the introductory reading path. They are
the reason to read Ken rather than merely another functional language.

### Treat the catalog as a software catalog, not only an API reference

The physical hierarchy
`Section > Domain > optional Subdomain > Package` remains the canonical filing
system. Human and agent discovery must add independent projections over it:

- task and problem domain;
- public declaration and type shape;
- laws and proof families;
- effects and required capabilities;
- assurance status and trusted-base delta;
- platform and execution-backend availability;
- maturity and stability;
- dependencies, reverse dependencies, and examples.

Each package reference page should answer, without requiring repository search:

- What problem does this package solve?
- What is its canonical import and public API?
- Which laws does it provide, and how are they justified?
- Which effects, capabilities, platforms, and backends does it require?
- What assumptions or trusted-base additions does a consumer inherit?
- Which packages does it depend on and which examples use it?
- Where are its literate source, spec contract, conformance evidence,
  provenance, references, and license?

Structural answers should be generated from checked artifacts and package
metadata. Curated rationale and worked examples should remain in the canonical
literate package entry. The catalog reference should link or transclude those
sections rather than fork them.

## Proposed `library/` shape

The following paths are the target information architecture. A directory
should land with its first real document; the rollout need not create empty
placeholders.

| Path | Contract |
|---|---|
| `library/README.md` | One-screen portal: “read Ken,” “write Ken,” “look something up,” “find a package,” and “load agent context.” |
| `library/STATUS.md` | Generated current/partial/planned feature and documentation coverage matrix, anchored to the validated repository revision. |
| `library/manifest.toml` | Machine-readable inventory of every document's kind, audience, authority, sources, validation gate, availability, and owner. |
| `library/introduction.md` | Ken's purpose, audience, system shape, assurance thesis, and explicit non-goals. |
| `library/quickstart.md` | Install/use the current toolchain, check and run one program, format it, then perform a short trust-aware reading exercise. |
| `library/learn/reading-ken/` | Primary curriculum: program anatomy, contracts, proofs, effects/capabilities, trust, packages, and execution. |
| `library/learn/writing-ken/` | Secondary tutorial for motivated humans: small pure programs through proof-carrying and effectful packages. |
| `library/learn/exercises/` | Checked reading and writing exercises with separate solutions and explicit learning objectives. |
| `library/guide/` | Conceptual user guide: the type theory as readers need it, verification model, module/package model, effects, security, execution, and design idioms. This is Diátaxis “explanation.” |
| `library/how-to/` | Goal-directed recipes: check/run/format, read diagnostics, structure a proof, define a datatype, use a package, declare capabilities, audit trust, build for a target, and troubleshoot common failures. |
| `library/reference/language/` | Complete reader-oriented surface-language reference, cross-linked to normative `spec/30-surface/` sections and generated grammar/token tables where possible. |
| `library/reference/verification/` | Claims, obligations, proofs, diagnostics, certificates, trust statuses, and the kernel-checking boundary. |
| `library/reference/toolchain/` | CLI, files and literate fences, configuration, package/import behavior, formatter, REPL, and artifact commands. |
| `library/reference/runtime/` | Values, evaluation, effects, interpreter behavior, native compilation, erasure, traps, resources, and backend equivalence. |
| `library/reference/platform/` | Supported targets, ABI/platform differences, capabilities, and explicit unavailable lanes. |
| `library/reference/diagnostics/` | Searchable diagnostic index: message, cause, smallest reproducer, repair routes, and relevant reference pages. |
| `library/reference/glossary.md` | Ken terms, notation, Unicode/ASCII spellings, and links to definitions rather than duplicate definitions. |
| `library/catalog/` | Generated and curated software-catalog portal with section/domain pages and the discovery projections listed above. |
| `library/releases/` | Release notes and migration guides once Ken has versioned public releases; absent until then. |
| `library/agents/` | Selective product-knowledge modules and context-pack manifests for coding agents, described below. |

### Initial human reading path

The first complete path should be short enough to finish and deep enough to
change how a reader reviews code:

1. `introduction.md` — why Ken exists and what it claims;
2. `quickstart.md` — check, run, format, and inspect one real program;
3. `learn/reading-ken/01-anatomy.md` — orient in a source file;
4. `02-types-contracts-and-proofs.md` — read the promise before the body;
5. `03-assurance-and-trust.md` — proved/tested/delegated/unknown and the TCB;
6. `04-effects-capabilities-and-authority.md` — what the program may do;
7. `05-packages-and-provenance.md` — what the program imports and inherits;
8. `06-execution.md` — interpreter, compiler, runtime assumptions, and traps;
9. `07-review-worked-example.md` — a complete review with an explicit verdict.

The worked example should be an existing catalog program, not a toy syntax
collage. Every chapter should end with a “reader can now answer” checklist.

### Conceptual guide chapters

The conceptual guide should eventually cover:

- programs as contract + implementation + evidence + execution;
- universes, `Type`, and proof-irrelevant `Omega`;
- functions, dependent functions, records, Sigma types, and refinements;
- inductive families, pattern matching, recursion, and termination;
- observational equality, conversion, casts, and transport;
- declarations, modules, packages, imports, exports, and identity;
- type classes, coherence, and lawful instances;
- obligations, automation, proof techniques, and failure to prove;
- effects, interaction trees, handlers, and capability supply;
- authority, information flow, constant-time claims, and foreign boundaries;
- trusted base, content identity, provenance, and supply-chain re-checking;
- strict evaluation, runtime values, capacity, resources, and controlled traps;
- interpreter semantics, proof erasure, native execution, and differential
  accountability;
- tested/delegated behavioral assurance and the Ward handoff;
- canonical formatting and the reading rationale behind Ken syntax.

This is an intended coverage map, not a request to write all chapters before
publishing the first useful path.

## `library/agents/`: a context library, not an agent manual

The agent corpus should contain **Ken product knowledge**. It must not duplicate
federation roles, merge workflow, model routing, or fleet memories; those remain
under `agent/` and are owned by the Steward. The same product modules should be
usable by an in-repo fleet agent, an external coding agent, or a future tool
that assembles prompts.

The design should follow progressive disclosure. Repository-wide instructions
point to the library and state when to load it. A task chooses one core pack,
one task module, and only the relevant domain/package modules. No ordinary task
loads the entire corpus.

### Exact agent corpus shape

| Path | Contents |
|---|---|
| `library/agents/README.md` | Selection protocol, authority rules, and the distinction between product context and workflow instructions. |
| `library/agents/manifest.toml` | Every module and pack: purpose, triggers, prerequisites, included files, source anchors, revision, validation, and measured token size. |
| `library/agents/core/read-ken.md` | Minimal syntax and semantic orientation needed to inspect any Ken source. |
| `library/agents/core/write-ken.md` | Canonical authoring forms and the probe/check/format loop; eventually subsumes the product portion of the current `write-ken` skill. |
| `library/agents/core/proof-and-trust.md` | Proof terminals, claims, `Axiom`, trusted-base accounting, and hard boundaries between proved/tested/delegated/unknown. |
| `library/agents/core/toolchain.md` | Exact current commands, file roles, expected artifacts, and fail-closed handling of unavailable features. |
| `library/agents/tasks/read-review.md` | Procedure and output contract for reviewing a Ken program for a human. |
| `library/agents/tasks/write-program.md` | From requirement to checked source, including contract-first decomposition. |
| `library/agents/tasks/author-package.md` | Package identity, literate entry shape, public API/laws, trust/derivation, examples, and validation. |
| `library/agents/tasks/prove-or-repair.md` | Goal inspection, reduction, induction/case decomposition, trusted-boundary refusal, and diagnostic routes. |
| `library/agents/tasks/diagnose.md` | Evidence-gathering order for parse, elaborate, kernel, proof, interpreter, and native failures. |
| `library/agents/tasks/effects-and-capabilities.md` | Effect rows, capability supply, handlers, authority, resources, and supported execution paths. |
| `library/agents/tasks/ffi-and-platform.md` | Foreign declarations, ABI/platform facts, target availability, and trust disclosures. |
| `library/agents/domains/` | Focused modules for kernel concepts, surface language, verification, runtime, security, behavioral assurance, and the catalog. |
| `library/agents/catalog/` | Generated compact package cards and indexes derived from the canonical catalog, not separately authored package descriptions. |
| `library/agents/packs/` | Ordered pack manifests such as `read-review`, `write-pure`, `write-effectful`, `author-package`, `repair-proof`, and `ffi-platform`. |
| `library/agents/schemas/` | The pack/manifest schema and optional structured forms used by context assembly tooling. |

### Contract for an agent module

Each module should answer the following in a predictable order:

1. **Use when** — positive triggers and explicit non-triggers.
2. **Prerequisites** — modules or facts that must already be present.
3. **Current capability** — what the landed toolchain supports, with no
   aspirational syntax mixed in.
4. **Canonical forms** — smallest checked examples and exact declaration or
   command shapes.
5. **Invariants and prohibitions** — rules that must not be inferred from
   examples alone.
6. **Decision procedure** — a short task sequence with observable stop
   conditions.
7. **Failure signatures** — common diagnostics, likely layer, and next source
   to inspect.
8. **Validation** — exact targeted checks for the artifact type.
9. **Authority and sources** — normative and generated sources, plus the
   revision against which the module was verified.
10. **Known unavailable or partial behavior** — fail closed rather than invite
    the agent to improvise.

Agent modules should prefer tables, signatures, checked examples, and explicit
contrasts over extended narrative. They should include **negative knowledge**:
unsupported forms, misleading near-syntax, distinctions such as `tt` versus
`Refl`, and the point at which an agent must stop instead of inventing a proof,
primitive, capability, or package.

### Pack selection and evaluation

`library/agents/manifest.toml` should make pack selection mechanical. A pack
records ordered includes, task triggers, exclusions, source revision, and its
measured size. The build should reject missing modules and circular pack
dependencies.

The first agent-library acceptance suite should use cold-context tasks:

- explain a small Ken program's contract and trust posture;
- write and check a pure function with one real law;
- distinguish and repair `tt` versus `Refl` proof endpoints;
- find and use a catalog package by task rather than guessed name;
- author an effectful boundary without omitting its capability or row;
- refuse an unsupported or unproved request honestly;
- diagnose one parse, one elaboration, one kernel, and one runtime failure.

The evaluation should record correctness, unnecessary file loads, invented
syntax or capabilities, and whether the agent cited the authority it used. The
goal is not the smallest possible token count; it is the smallest context that
reliably produces the correct, reviewable result.

## Authority, currency, and verification

### One fact, one authority

Every library page needs an explicit authority class:

- **`derived-reference`** — generated from source, checked artifacts, CLI help,
  manifests, or grammar tables;
- **`explanatory`** — authored mental model grounded in named normative and
  implementation sources;
- **`tutorial`** or **`how-to`** — checked learning/action sequence;
- **`status`** — generated availability statement tied to a repository
  revision;
- **`normative-pointer`** — navigation into `spec/`, never an independent rule.

The library itself should not introduce `normative` language. When a reference
page must restate a rule for usability, it cites the exact spec section and a
drift gate verifies the source still exists. Exact grammar, public signatures,
package dependencies, CLI forms, supported-target matrices, and trust deltas
should be generated wherever the source can express them.

### Suggested manifest record

```toml
[[document]]
path = "library/learn/reading-ken/03-assurance-and-trust.md"
kind = "tutorial"
audience = ["human-reader"]
authority = "explanatory"
availability = "current"
sources = [
  "docs/PRINCIPLES.md#1-ken-is-a-software-engineering-language-not-a-programming-language",
  "spec/60-security/64-trust-model.md",
  "spec/70-behavioral/71-assumption-boundary.md",
]
validation = ["links", "ken-fences", "source-anchors"]
owner = "librarian"
```

The validated revision should be recorded by the generated `STATUS.md` and
build output rather than hand-edited into every page. A date without a source
revision is not evidence of currency.

### Documentation gates

The program should grow these gates incrementally:

1. manifest covers every library document and every manifest path exists;
2. internal and external links are valid;
3. every source path and stable section anchor exists;
4. every Ken example elaborates, and every rejection example still rejects for
   its declared reason;
5. generated CLI, grammar, declaration, package, dependency, trust, and
   platform facts have no hand-edited duplicates;
6. every page labels current, partial, planned, or unavailable capability;
7. planned syntax may appear only in visibly planned material and never in a
   checked current example;
8. package reference coverage equals the live canonical package set;
9. agent packs resolve, have no cycles, and contain only declared modules;
10. release snapshots, when they exist, are built from the matching release
    rather than current `main`.

The Librarian remains non-blocking. A feature merge need not wait for prose,
but its diff should make the relevant manifest sources visibly stale. The
Librarian's post-merge as-built pass then repairs the docs on a doc-only branch.
For user-visible features, the desired resting state is documentation in the
same change; the observer pass is the backstop, not the only update mechanism.

## Migration from the current corpus

The repository already contains good material, but its homes reflect how the
project was built rather than how an outside reader searches.

| Current material | Proposed treatment |
|---|---|
| `README.md` | Remain the public front door; shorten over time to the thesis, current status, and links into `library/`. |
| `spec/` | Remain normative and structurally unchanged by this program. Library reference pages cite or derive from it. |
| `catalog/packages/` | Remain canonical package source and per-package literate rationale. `library/catalog/` generates discovery and structural reference around it. |
| `catalog/guide/` | Migrate its human authoring material into the appropriate `library/learn/`, `guide/`, and `how-to/` pages. Leave pointers, not two maintained guides. |
| `agent/playbooks/tools/write-ken.md` | Keep the workflow trigger thin. Move reusable Ken product facts into `library/agents/`; the skill selects the appropriate pack. |
| `docs/adr/` | Remain decision records; conceptual pages cite the accepted ADR rather than teach from decision history. |
| `docs/program/` | Remain internal program history and WP material, excluded from the public product-doc navigation. |
| `conformance/` | Remain executable contract evidence; library pages link cases and reuse checked fixtures where suitable. |
| `research/` | Remain advisory background; not part of the ordinary reader path. |

This migration should be subsumptive. Do not preserve an old and new authoring
guide indefinitely merely to reduce the move. Ken has no compatibility reason
to maintain parallel documentation forms during initial development.

## Proposed Librarian work program

The waves are dependency-ordered. They are not human-time estimates and do not
require every item in one wave to land in one branch.

### Wave 0 — charter and currency substrate

- Land `library/README.md`, `manifest.toml`, and generated `STATUS.md` with a
  small real document set.
- Ratify the authority classes and source-of-truth table.
- Add manifest, link, source-path, and availability-label gates.
- Record which existing pages will move, remain canonical, or become pointers.

**Exit:** a new page cannot land without declaring what it is, what grounds it,
and how its currency is checked.

### Wave 1 — the read-Ken spine

- Write the introduction, quickstart, and seven-part reading curriculum.
- Use one real catalog program throughout rather than unrelated snippets.
- Put assurance, capabilities, trust, and execution distinctions in the first
  path.
- Add checked exercises and a complete worked review verdict.

**Exit:** a technically experienced human unfamiliar with dependent types can
read one non-trivial Ken program and accurately state its contract, assumptions,
authority, and execution status.

### Wave 2 — agent core and task packs

- Land the agent manifest, four core modules, and the first six task modules.
- Refactor the product facts out of `write-ken` into selectable library modules.
- Add cold-context evaluations and pack integrity checks.
- Pilot the packs with the fleet, but keep federation workflow outside them.

**Exit:** a Ken-untrained coding agent can perform the core read/write/prove/
diagnose tasks without loading the entire spec, catalog guide, or fleet memory.

### Wave 3 — conceptual guide and practical how-tos

- Migrate and reconcile the current `catalog/guide/` strands.
- Fill the conceptual coverage map in demand order, starting with contracts,
  dependent data, proofs, effects, security, packages, and execution.
- Add task-focused recipes driven by actual diagnostics and recurring fleet
  failures.
- Keep explanatory pages free of internal campaign and WP history.

**Exit:** tutorials teach, how-tos direct work, and conceptual pages explain;
no single page is forced to do all three.

### Wave 4 — complete reader-oriented reference

- Build surface-language, verification, toolchain, runtime, platform, and
  diagnostic references.
- Generate exact syntax, CLI, target, and public-declaration facts.
- Add symbol, keyword, diagnostic, and glossary indexes.
- Make each page link to its normative source and checked examples.

**Exit:** a reader who knows what they are looking for can find a complete,
current answer without reading the normative spec front to back.

### Wave 5 — comprehensive catalog reference

- Generate one reference page/card for every live package.
- Add subject, declaration/type, law, effect/capability, assurance, platform,
  maturity, dependency, and reverse-dependency indexes.
- Add type- and proposition-shaped search when the checked artifact format can
  support it; do not approximate it with prose tags.
- Expose validation, provenance, and source links consistently.

**Exit:** the catalog is discoverable both by what a reader wants to accomplish
and by the exact checked abstractions available.

### Wave 6 — release, offline, and continuous as-built operation

- Produce static searchable HTML and an offline artifact from the same sources.
- Add versioned snapshots and migration notes when public releases begin.
- Wire post-merge source changes to the Librarian's as-built queue.
- Measure dead ends, failed searches, stale-source detections, tutorial
  completion, and agent-pack evaluation results; use them to choose the next
  documentation work.

**Exit:** documentation currency is an observable product property rather than
the Librarian's memory of what changed.

## Decisions the Steward or operator should ratify

This proposal is advisory. Four choices materially affect the eventual program:

1. **Product-doc authority:** confirm that `library/` is explanatory and
   derived, while `spec/` remains the sole normative language authority.
2. **Subsumptive migration:** confirm that `catalog/guide/` moves into the new
   product library rather than remaining a second maintained guide.
3. **Agent/practice boundary:** confirm that `library/agents/` contains product
   context only and the Steward continues to own workflow and fleet-practice
   material under `agent/`.
4. **Generated catalog commitment:** confirm that structural package reference
   is generated from checked catalog artifacts and metadata, with authored
   package rationale remaining in `catalog/packages/`.

None blocks the initial Research report. They should be settled in the
Steward's program frame before the Librarian performs the migration.

## Source set

### Ken sources

- [Design principles](../docs/PRINCIPLES.md), especially the
  agents-write/humans-read asymmetry and honesty about the boundary.
- [Normative specification map](../spec/README.md).
- [Catalog campaign](../docs/program/06-catalog-campaign.md) and
  [catalog style guide](../docs/program/07-catalog-style-guide.md).
- [Current authoring guide](../catalog/guide/README.md) and
  [`write-ken` playbook](../agent/playbooks/tools/write-ken.md).
- [Librarian playbook](../agent/playbooks/federation/librarian.md).
- [Catalog taxonomy proposal](catalog-package-taxonomy-proposal.md).

### External sources

- [Rust documentation bookshelf](https://doc.rust-lang.org/),
  [Rust Book](https://doc.rust-lang.org/book/),
  [Rust By Example](https://doc.rust-lang.org/rust-by-example/), and
  [Rust Reference](https://doc.rust-lang.org/reference/).
- [Python documentation portal](https://docs.python.org/3/).
- [Go documentation portal](https://go.dev/doc/) and
  [Effective Go](https://go.dev/doc/effective_go).
- [Julia documentation](https://docs.julialang.org/).
- [Lean Language Reference](https://lean-lang.org/doc/reference/latest/) and
  [Theorem Proving in Lean](https://docs.lean-lang.org/theorem_proving_in_lean4/).
- [Mathlib documentation overview](https://leanprover-community.github.io/documentation.html).
- [Agda Getting Started](https://agda.readthedocs.io/en/stable/getting-started/)
  and [Agda overview](https://agda.readthedocs.io/en/v2.7.0/overview.html).
- [Idris 2 tutorial](https://idris2.readthedocs.io/en/stable/tutorial/introduction.html).
- [Dafny reference and user guide](https://dafny.org/dafny/DafnyRef/DafnyRef).
- [Proof-Oriented Programming in F*](https://fstar-lang.org/tutorial/book/intro.html).
- [Diátaxis documentation system](https://diataxis.fr/).
- [GitHub repository and path-specific agent instructions](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions-in-your-ide/add-repository-instructions-in-your-ide).
- [Anthropic context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents).
- [OpenAI's `AGENTS.md` description](https://openai.com/index/introducing-codex/).
