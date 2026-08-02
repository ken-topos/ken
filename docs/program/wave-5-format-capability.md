# Wave 5 checked-format capability report

This report measures the 39 checked leaf packages under `catalog/packages/`
at `767bf20fe88866a98f148a34792189f1f4b48feb`. It answers whether Wave 5's
nine promised fact classes are present and mechanically extractable today. It
does not design a generator, schema, output format, or Wave 5 slice.

The population is Application 3, Capability 19, Core 5, Data 11, and Tooling
1. Every leaf is a checked literate `.ken.md` document.

## D0 and D1 — capability and disposition

| Fact class | Present in checked source | Mechanically extractable today | Smallest missing capability and owner | D1 disposition and cost |
|---|---|---|---|---|
| Subject | Yes. Every leaf has exactly one H1 title; the following prose states intent, although prose intent is not a controlled field. | **Yes.** `git grep -n '^# ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md'` emits 39 path-preserving titles. | None for the title. The prose intent remains ordinary catalog prose. | **`generated`** for the H1 subject. No transcription rot for the title; richer intent would be authored. |
| Declaration/type | Yes. All 39 leaves contain checked `ken` fences and declarations. Checked-core emission carries stable symbols and canonical declaration bytes, but `ken check` emits no inventory and the checked-core inventory is not visibility-filtered. | **No public command.** Reused Wave 4 C5: `ken check` exits 0 with empty output; `ken declarations` is an unknown command. | A public-declaration projection joining visibility to stable checked-core identity, and a command that exposes it; owner: `crates/`. | **`authored`** from checked Definition fences and Public API prose. Rot cost: every public-name or signature change requires a manual card review. |
| Law | Partly explicit but human-readable for every leaf: 26 leaves contain `law`, `proof`, or `theorem` declarations; a leaf with none can be read as declaring none. The code and Laws/proof prose are checked where fenced. | **No public command.** The checked artifact carries declarations and proof/obligation metadata, but no law-class projection or exporter. | A maintained projection that identifies public law declarations and their checked obligations; owner: `crates/`. | **`authored`** from checked declarations and Laws/proofs prose. Rot cost: adding, removing, renaming, or changing the status of a law can stale a card. |
| Effect/capability | Yes in surface signatures where applicable: 11 leaves contain effect/capability syntax; pure leaves carry no such syntax. Checked core has effects/foreign metadata, but no package-summary command. | **No public command.** Text grep cannot distinguish canonical source from explanatory or non-tangling fences, and silence is not a semantic purity report. | A package-level projection of checked effect rows and required capabilities; owner: `crates/`. | **`authored`** from checked signatures. Rot cost: an effect-row, capability index, or boundary change requires manual review, including pure-to-effectful changes. |
| Assurance | Yes, but unevenly structured. Thirty-three leaves contain trust, validation, assumption, axiom, or proof-family evidence; checked core carries obligations, assumption/trust metadata, and trusted-base delta facts. | **No public command.** Neither `ken check` nor a repository generator emits a package assurance summary. | A maintained package assurance projection over checked obligations, assumptions, and trust delta; owner: `crates/`. | **`authored`** from Trust/derivation prose plus checked declarations. Rot cost: proof closure, assumptions, and trusted-base changes can stale the stated posture. |
| Platform | The catalog campaign reserves `platform` as a controlled metadata facet (`docs/program/06-catalog-campaign.md:119–121`), but no checked leaf instantiates a per-package platform value. C1 finds platform/target words in only 2 of 39 leaves; those occurrences are not facet values. Wave 4's `TargetAbi` emitter reports the build host lane, not per-package support. | **No.** M1 finds the governing facet reservation but no instantiated per-package value or package-level projection. | Per-package instantiation of the reserved `platform` facet; owner: catalog campaign. The exact catalog convention and decision remain open; this report does not define either. | **`blocked`**. A complete package page cannot infer support from silence or host build facts. |
| Maturity | The catalog campaign reserves `maturity` as a controlled metadata facet (`docs/program/06-catalog-campaign.md:119–121`), but no checked leaf instantiates a per-package maturity value. C1 finds zero maturity, stability, experimental, or deprecated labels across the 39 leaves. | **No.** M1 finds the governing facet reservation but no instantiated per-package value or emitter. | Per-package instantiation of the reserved `maturity` facet; owner: catalog campaign. The exact catalog convention and decision remain open; this report does not define either. | **`blocked`**. Present-tense maturity prose would be invented rather than derived. |
| Dependency | Not uniformly. Catalog census C1 finds a `use`, dependency, or Consumers mention in 15 leaves, but no required package dependency declaration. Checked core carries dependency hashes and declaration references internally; `ken check` does not expose them for catalog leaves. | **No public command.** Mechanism census M1 finds internal declaration dependency indexes but no package-level catalog projection. Prose references are not a complete dependency graph. | A package-level checked dependency projection for literate catalog leaves; owner: `crates/`. | **`blocked`** for a complete index. Sparse prose can orient a human, but silence cannot honestly mean “no dependencies.” |
| Reverse-dependency | No. Catalog census C1 finds no leaf declaring reverse dependencies or dependents. Mechanism census M1 finds compiler declaration dependency indexes, but no package-level catalog relation or public command joins them across the corpus. | **No.** Producing the fact manually would require the prohibited 39-package dependency pass. | The package-level dependency projection above, plus maintained inversion over its complete population; owner: `crates/`. | **`blocked`**. It is neither a local source fact nor safely authorable from the incomplete prose relation. |

Disposition control: the nine rows contain one `generated`, four `authored`,
and four `blocked` results. No class is assigned more than one disposition.

## D2 — operator fork

**Recommend the mixed fork, with exactly this authorable subset:** subject,
declaration/type, law, effect/capability, and assurance. Subject titles can be
generated now; the other four can be authored from checked source with the rot
costs above stated on the resulting material. Hold platform, maturity,
dependency, and reverse-dependency: their package-level facts are absent or
incomplete, so producing complete indexes now would make silence look like a
fact. This recommendation accepts manual maintenance for four grounded classes
without presenting the four blocked classes as delivered.

## D3 — three-package sample

The sample spans Core, Capability, and Data. Each package is carried through
all nine classes; no conclusion depends on a 39-package manual pass.

| Fact class | Core — `Core/Logic/EmptyDec.ken.md` | Capability — `Capability/Filesystem/Authority.ken.md` | Data — `Data/Collections/Map.ken.md` |
|---|---|---|---|
| Subject | H1: “`Empty` and `Dec` — computational falsity and decidability.” | H1: “Filesystem authority manifests.” | H1: “`Map`/`Set` — a proved, pure ordered binary search tree.” |
| Declaration/type | Checked fences define `absurd_empty`, `yes`, `no`, `DecEq`, `bool_eq`, `sym`, `trans`, and `dec_eq_decides`; the standard `Empty`/`Dec` forms are explicitly marked illustrative. | One checked fence defines `capability_read` and `full_authority_write` with their complete signatures. | Checked Definition and law fences define `Tree`, map/set operations, invariants, and the public law family. |
| Law | The Laws/proofs section checks `yes_is_true` and `no_is_false`; Definition carries the `DecEq` sound/complete contract. | No law declaration is present; the page instead records deliberately absent authority-management operations. This “none declared” result is human-read, not emitted. | Laws/proofs names five capstone laws and checked proof families, followed by checked keyed-collection laws. |
| Effect/capability | No effect row or capability appears in canonical fences; a human reads the package as pure. | Both procedures require `Cap` values and return `FS` computations with `visits [FS]`; `AFull` is load-bearing for writes. | Canonical declarations have no effect rows or capabilities; Trust/derivation calls every recursive operation pure. |
| Assurance | Trust/derivation states zero new trust category, names proof families, consumers, and validation evidence. | Checked source demonstrates authority-index separation and the prose marks the host-only complement; there is no standard Trust/derivation section. | Trust/derivation states `trusted_base()` delta zero, no `Axiom`, proof families, consumers, and `ken check` validation. |
| Platform | No package platform/support declaration. | “OS-operation denial” is named as host-only behavior, not a platform support declaration. | No package platform/support declaration; proof uses of “target” are not platform facts. |
| Maturity | No controlled maturity fact. | No controlled maturity fact. | No controlled maturity fact. |
| Dependency | Prose cites `Core/Logic/Transport.ken` and names downstream consumer classes, but no checked dependency field exists. | Uses landed filesystem producers and cites spec sections, but declares no package dependency field. | Trust/derivation describes built-ins and consumers, but declares no checked package dependency field. |
| Reverse-dependency | No local fact; consumers prose is orientation, not a complete reverse index. | No local fact. | No local fact; consumers prose is not a complete reverse index. |

## D4 — evidence and reuse ledger

### Population and directly extractable subject

```console
$ git grep -n '^# ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md'
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Application/CommandLine/ArgParse.ken.md:1:# ArgParse
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Application/Configuration/Decoder.ken.md:1:# Application.Configuration.Decoder
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Application/Input/Schema.ken.md:1:# Schema
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Console/Text.ken.md:1:# `Console` — ordinary text-output helpers
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Diagnostics/Core.ken.md:1:# Capability.Diagnostics.Core
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Diagnostics/Render.ken.md:1:# Capability.Diagnostics.Render
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Filesystem/Authority.ken.md:1:# Filesystem authority manifests
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Filesystem/Errors.ken.md:1:# `FS` — file-error rendering
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Filesystem/Path/Posix.ken.md:1:# `Capability.Filesystem.Path.Posix` — byte-preserving lexical paths
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Formatting/Doc.ken.md:1:# `Capability.Formatting.Doc` — a small lawful document algebra
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Parsing/Cursor.ken.md:1:# Capability.Parsing.Cursor
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Parsing/Decoder.ken.md:1:# Capability.Parsing.Decoder
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Parsing/Numeric.ken.md:1:# `Capability.Parsing.Numeric` — located decimal parsing
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Parsing/Parsing.ken.md:1:# `parsing` — source artifacts, spans, parsers, and a Boolean grammar
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Process/Arguments.ken.md:1:# Capability.Process.Arguments
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Process/Environment.ken.md:1:# Capability.Process.Environment
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Process/Exit.ken.md:1:# Capability.Process.Exit
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Process/WorkingDirectory.ken.md:1:# Capability.Process.WorkingDirectory
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/System/Buffer.ken.md:1:# System.Buffer
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/System/IO.ken.md:1:# System.IO
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/System/Resource.ken.md:1:# System.Resource
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Capability/Time/WallClock.ken.md:1:# Capability.Time.WallClock
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Core/Classes/EffectfulClasses.ken.md:1:# `Applicative`, `Monad`, and `Traversable` — effectful constructor classes
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Core/Classes/LawfulClasses.ken.md:1:# `lawful-classes` — `Eq`, `DecEq`, `Ord`
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Core/Classes/LawfulFunctors.ken.md:1:# `lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Core/Logic/EmptyDec.ken.md:1:# `Empty` and `Dec` — computational falsity and decidability
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Core/Logic/Transport.ken.md:1:# `transport` — `subst`, `cong`, `cast`, `sym`, `trans`
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Binary/BytesKeys.ken.md:1:# `Data.Binary.BytesKeys` — lawful byte equality
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Collections/Derived.ken.md:1:# `Collections` — derived collection, string, and byte views
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Collections/Map.ken.md:1:# `Map`/`Set` — a proved, pure ordered binary search tree
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Collections/NonEmpty.ken.md:1:# `NonEmpty` — lists with a structural head
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Numeric/Nat/Arithmetic.ken.md:1:# `Nat` arithmetic — canonical operations and free algebraic laws
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Numeric/Nat/Order.ken.md:1:# `Ord Nat` — a lawful total order on `Nat`, and its operations
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Sums/Combinators.ken.md:1:# `Sums` — the `Option`/`Result`/`Either` combinator floor
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Sums/Validation.ken.md:1:# `Validation` — accumulating independent errors
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Text/Codec.ken.md:1:# `Data.Text.Codec` — safe UTF-8 and ASCII views
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Text/StringBijection.ken.md:1:# String/List-Char retraction and injectivity certificate
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Data/Text/StringKeys.ken.md:1:# `Data.Text.StringKeys` — lawful String equality and order
767bf20fe88866a98f148a34792189f1f4b48feb:catalog/packages/Tooling/Testing/Property.ken.md:1:# `Tooling.Testing.Property` — deterministic finite property checks
```

The transcript preserves every path and title. It returns 39 rows in 39 paths,
and every match is the sole H1 at line 1 of its checked leaf.

### C1 — exact-base catalog census

Each literal pattern was replayed twice at the exact base: once for matching
lines and once for matching paths.

```console
$ git grep -nEi '^# ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
39
$ git grep -liEi '^# ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
39
```

This establishes one title per leaf.

```console
$ git grep -nEi '^```ken$' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
151
$ git grep -liEi '^```ken$' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
39
```

This establishes every leaf has canonical checked source.

```console
$ git grep -nEi '^(data|record|class|instance|fn|const|proc|effect|foreign|primitive|theorem|proof|law) ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
1737
$ git grep -liEi '^(data|record|class|instance|fn|const|proc|effect|foreign|primitive|theorem|proof|law) ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
39
```

This establishes declaration/type facts exist throughout the population.

```console
$ git grep -nEi '^(law|proof|theorem) ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
831
$ git grep -liEi '^(law|proof|theorem) ' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
26
```

This establishes explicit law/proof declarations are present but not universal.

```console
$ git grep -nEi 'visits \[|program capabilities|effect |Cap [A-Za-z]|FS [A-Za-z]' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
43
$ git grep -liEi 'visits \[|program capabilities|effect |Cap [A-Za-z]|FS [A-Za-z]' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
11
```

This establishes effect facts are explicit only where applicable.

```console
$ git grep -nEi 'trusted_base\(\)|Validation evidence|Axiom|assumption|Proof families' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
109
$ git grep -liEi 'trusted_base\(\)|Validation evidence|Axiom|assumption|Proof families' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
33
```

This establishes assurance prose and declarations are present but unevenly
structured.

```console
$ git grep -nEi '(platform|target|Linux|Windows|macOS|WASI)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
40
$ git grep -liEi '(platform|target|Linux|Windows|macOS|WASI)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
2
```

This establishes words occur sparsely and do not form a package field.

```console
$ git grep -nEi '(maturity|stability|experimental|deprecated)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
0
$ git grep -liEi '(maturity|stability|experimental|deprecated)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
0
```

This establishes no checked leaf instantiates the reserved maturity facet.

```console
$ git grep -nEi '(^use |dependenc|Consumers)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
21
$ git grep -liEi '(^use |dependenc|Consumers)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
15
```

This establishes prose/import evidence is incomplete as a package graph.

```console
$ git grep -nEi '(reverse[- ]depend|dependents)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
0
$ git grep -liEi '(reverse[- ]depend|dependents)' 767bf20fe88866a98f148a34792189f1f4b48feb -- 'catalog/packages/**/*.ken.md' | wc -l
0
```

This establishes no local reverse-dependency fact exists.

The platform and dependency results are deliberately not interpreted from
counts alone. The three-package read above distinguishes proof “target” prose
and consumer orientation from the promised package facts.

### M1 — repo-wide mechanism census

The governing locus reserves the facets before the census tests whether
checked leaves instantiate them:

```console
$ git show 767bf20fe88866a98f148a34792189f1f4b48feb:docs/program/06-catalog-campaign.md | sed -n '119,121p'
Secondary classification belongs in controlled metadata facets—`platform`,
`effects`, `assurance`, `maturity`, `audience`, `security`, and
`artifact-kind`—rather than new top-level directories or duplicate packages.
```

Thus the convention is reserved, while C1 and the repo-wide searches below
show that neither facet has an instantiated per-package value or projection.
The exact catalog decision remains open.

The absence checks range over the entire exact tree, including build scripts:

```console
$ git grep -nEi 'package[-_ ]?(platform|target)|platform[-_ ]?(field|metadata|manifest)|target[-_ ]?(support|platform).*(package|metadata|manifest)' 767bf20fe88866a98f148a34792189f1f4b48feb -- .
# 4 hits in 2 paths: runtime package-target validation and one historical diary entry; no per-package catalog support field
$ git grep -nEi 'maturity[-_ ]?(field|metadata|manifest|registry|export|emit)|stability[-_ ]?(field|metadata|manifest|registry|export|emit)|(field|metadata|manifest|registry|export|emit)[-_ ]?(maturity|stability)' 767bf20fe88866a98f148a34792189f1f4b48feb -- .
# empty output
$ git grep -nEi 'reverse[-_ ]?depend|dependent[-_ ]?(index|graph)|dependency[-_ ]?(index|export|emit)|import[-_ ]?graph' 767bf20fe88866a98f148a34792189f1f4b48feb -- .
# 33 hits in 17 paths, including compiler declaration dependency indexes; no package-level catalog exporter or reverse relation
```

The checked artifact does carry internal facts:

```console
$ git grep -nE 'pub (symbols|declarations|effects_foreign_metadata|obligation_metadata|assumption_trust_metadata|dependency_semantic_hashes|dependency_declaration_refs):' 767bf20fe88866a98f148a34792189f1f4b48feb -- crates/ken-elaborator/src/checked_core.rs
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:170:    pub symbols: BTreeSet<StableSymbol>,
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:171:    pub declarations: BTreeMap<StableSymbol, Vec<u8>>,
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:178:    pub effects_foreign_metadata: BTreeMap<StableSymbol, EffectsForeignMetadata>,
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:181:    pub obligation_metadata: BTreeMap<StableSymbol, ObligationMetadata>,
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:182:    pub assumption_trust_metadata: BTreeMap<StableSymbol, AssumptionTrustMetadata>,
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:186:    pub dependency_semantic_hashes: BTreeMap<StableSymbol, String>,
767bf20fe88866a98f148a34792189f1f4b48feb:crates/ken-elaborator/src/checked_core.rs:187:    pub dependency_declaration_refs: BTreeMap<StableSymbol, StableSymbol>,
```

Those fields show why four classes are authorable rather than absent. They do
not make the facts public-command extractable, and the dependency fields do not
establish a complete package relation for the standalone catalog leaves. The
platform and maturity gap is narrower: their facet names are reserved, but
per-package values and extraction remain uninstantiated.

### Reused merged findings

- `docs/program/wave-4-generation-capability.md` C5 is reused for
  declaration/type: `ken check` validates with empty output, and
  `ken declarations` is unknown.
- `docs/program/wave-4-terminal-residual.md` §“Symbol and declaration census”
  is reused for the stable checked-core inventory versus public visibility
  boundary.
- The same report's §“Platform census” is reused for `TargetAbi`: it emits
  host-equals-target Linux build facts, not package support facts.

No new build is needed: every generated claim is discharged by the exact Git
object command above, and every other row is a source/capability finding.
