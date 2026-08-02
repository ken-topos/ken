# Wave 6 agent evaluation record

This record applies the cold-context evaluation protocol to the corpus at
`f69f6e9275b214bbb327ebe7be5ba94235ca353b`. Checkpoint 1 fixes the
drift population and citation-authority rules before any cold fixture is
shown to a seat. The run of record is
`library/agents/evaluations/results-2026-07-24.toml`, committed in
`d3b9f36c`.

This checkpoint does not run a task, consume a fixture, score an answer, or
change the historical results artifact. It does not authorize a currency
mechanism or a pack, core-module, or task edit.

## D1 — drift reconciliation

The transitive pack closure matters here. A task is at risk when its resolved
pack selects a changed core module, its fixture changed, or its prompt can
lead a cold agent to reader-facing material that did not exist in the run of
record. Merely observing that the task ID and pack manifest are unchanged
would miss those inputs.

The three changed core modules are:

| Module | Run-of-record blob | Current blob | Material change |
|---|---|---|---|
| `core/proof-and-trust` | `8d6a6d30` | `9032ad37` | Its checked-technique source moved from `catalog/guide/` to the derived `library/guide/` corpus. |
| `core/toolchain` | `8443a51f` | `f64437b0` | Its run-on-library diagnostic changed and its native comparison rule was narrowed to closure-free observations or typed projections. |
| `core/write-ken` | `3753b2cc` | `eb66dd00` | Its proof examples now use the landed `theorem` spelling, its run-on-library diagnostic changed, and its guide sources moved into `library/guide/`. |

The corpus also gained 54 paths under `library/reference/`, including 39
package cards, five catalog indexes, a platform page, and eight toolchain
pages. These are reader-facing derived references, not normative authority.

| Task | Resolved pack inputs relevant to drift | Disposition | Concrete reconciliation |
|---|---|---|---|
| `explain-contract` | `read-review` directly selects changed `proof-and-trust` and `toolchain`. | **at-risk** | The task asks for assurance, trust, execution limits, and citations. Both selected modules that govern those parts changed, and `library/reference/catalog/Core/Logic/EmptyDec.md` is a newly reachable derived account of the exact fixture. It is not not-at-risk because both the answer guidance and a plausible citation target changed. |
| `write-pure-law` | `write-pure` directly selects changed `write-ken`, `proof-and-trust`, and `toolchain`. | **at-risk** | The selected authoring example changed from `lemma` to the landed `theorem` spelling; proof-source paths and validation guidance also changed. It is not not-at-risk because the requested declaration and validation surface are among the changed selected bytes. |
| `repair-proof-terminal` | `repair-proof` depends on `write-pure`, so it selects all three changed core modules. Its fixture blob also changed from `7ecf4952` to `2b71a3f2`. | **at-risk** | Both fixture declarations changed from `lemma` to `theorem`, while the selected proof and authoring guidance changed with them. It is not not-at-risk because the cold input and its governing transitive modules are different. |
| `find-package-by-task` | `author-package` depends on `write-pure`, so it selects all three changed core modules. | **at-risk** | `library/reference/catalog/subjects.md` now answers the prompt's package-by-task discovery shape, and the derived EmptyDec card is beside the canonical checked package. Neither existed in the prior corpus. It is not not-at-risk because the prompt can now terminate on a derived index/card instead of the catalog source. |
| `write-effectful-boundary` | `write-effectful` depends on `write-pure`, so it selects all three changed core modules. | **at-risk** | Tool selection and validation guidance changed, and the corpus now contains the derived Console card plus the derived effect/capability index over the same checked package as the fixture. It is not not-at-risk because both a selected validation module and plausible authority targets changed. |
| `refuse-unsupported` | `write-effectful` depends on `write-pure`, so it selects all three changed core modules. | **at-risk** | The prompt asks for an FFI and an every-native-target promise. The newly reachable platform reference and toolchain pages discuss target and native boundaries, while the selected toolchain module changed. It is not not-at-risk because a cold agent now has plausible derived material from which it could overstate portability. |
| `diagnose-layers` | `diagnose` directly selects changed `proof-and-trust` and `toolchain`. | **at-risk** | The selected failure-signature text and native-validation boundary changed, and the new toolchain reference pages are plausible diagnostic destinations. It is not not-at-risk because the task's classification and next-check guidance are in the changed selected module. |

The at-risk set is therefore exactly all seven task IDs. The not-at-risk set
is empty; that is a measured result of the transitive pack closure, not a
default decision to re-run the whole suite.

## D2 — citation-authority statement

### Rule fixed before any run

`cited_authority = "complete"` means every material language, proof, effect,
authority, runtime, trust, CLI, package-identity, or corpus-boundary claim in
the answer is supported by the corresponding authority class below:

1. `spec/` is normative for language, proof, effect, authority, runtime, trust,
   FFI, and portability rules.
2. A current checked `catalog/packages/**/*.ken.md` source is authority for
   that package's landed declarations, checked laws, declared rows, and stated
   package posture. The catalog discovery page is authority for its current
   task-to-package routing.
3. Current implementation source or an observed command artifact is authority
   for the implemented CLI and diagnostic surface where the specification
   does not define it.
4. Pack manifests and task modules are authority for what the selected agent
   context includes, excludes, or requires. They are not normative language
   sources.
5. A `library/guide/` or `library/reference/` page may guide discovery and may
   be cited as an explanation, but it cannot replace an available normative
   spec, current checked package, or implementation source. Such a replacement
   is scored `partial` or `missing`, never `complete`.

Completeness is claim-sensitive: a source need not be cited for a claim the
answer does not make. The fixture itself is an input, not automatically an
authority for the language rule it exercises.

### Per-task scoring basis

| Task | Sources required for `complete`, according to claims made | Derived substitutions that do not complete the score |
|---|---|---|
| `explain-contract` | The current `catalog/packages/Core/Logic/EmptyDec.ken.md` for the public API and declared posture; `spec/30-surface/33-declarations.md` for declaration/class meaning; `spec/20-verification/21-spec-syntax.md` and `spec/60-security/64-trust-model.md` for assurance and inherited trust; `spec/30-surface/36-effects.md` and `spec/40-runtime/42-evaluation.md` for effect and execution limits actually asserted. | `library/reference/catalog/Core/Logic/EmptyDec.md`, the catalog assurance/declaration indexes, and reading-guide prose may orient the answer but cannot replace the checked package or named spec rule. |
| `write-pure-law` | `spec/30-surface/33-declarations.md` §§1 and 8.3 for `fn` and `theorem`; `spec/10-kernel/16-observational.md` and `spec/10-kernel/17-conversion.md` for `Proved` after reduction; `spec/60-security/64-trust-model.md` for the no-new-trust claim; and `crates/ken-cli/src/main.rs` or an exact observed command artifact for `check` and `fmt --check`. | `library/guide/surface-reference.ken.md`, `library/guide/proof-techniques.ken.md`, and `library/reference/toolchain/` are explanatory or derived and cannot stand alone for those claims. |
| `repair-proof-terminal` | The current fixture supplies the two concrete goals; `spec/30-surface/33-declarations.md` §8.3 fixes the declaration form; `spec/10-kernel/16-observational.md` and `spec/10-kernel/17-conversion.md` govern `Proved`, reflexive equality, and reduction; `crates/ken-cli/src/main.rs` or an exact command artifact grounds the check command. | The proof-techniques guide is useful checked instruction, but citing it instead of the normative kernel and declaration sources is not `complete`. |
| `find-package-by-task` | `catalog/packages/README.md` for task-based discovery, the selected current `catalog/packages/Core/Logic/EmptyDec.ken.md` for exact identity and package posture, `spec/60-security/64-trust-model.md` for trust claims, and `crates/ken-cli/src/main.rs` or an observed command artifact for the check command. | `library/reference/catalog/subjects.md` and `library/reference/catalog/Core/Logic/EmptyDec.md` are derived projections. Stopping at either one, or citing it in place of the catalog source, cannot score `complete`. |
| `write-effectful-boundary` | The current `catalog/packages/Capability/Console/Text.ken.md` for landed procedures and signatures; `spec/30-surface/36-effects.md` for `proc`, `visits`, rows, and capability passing; `spec/60-security/62-authority.md` for authority supply; and `spec/40-runtime/42-evaluation.md` for the host-driver boundary. | `library/reference/catalog/Capability/Console/Text.md` and `library/reference/catalog/effects-and-capabilities.md` cannot replace the checked package or those normative sections. |
| `refuse-unsupported` | `library/agents/packs/write-effectful.toml`, its transitive pack manifests, and `library/agents/tasks/effects-and-capabilities.md` establish the selected context and its FFI/platform exclusion; `spec/30-surface/38-ffi-io.md` governs the FFI boundary; `spec/40-runtime/48-executable-artifact-contract.md` governs current native/ABI portability limits. | `library/reference/platform/README.md` and `library/reference/toolchain/native-build.md` describe derived current facts but cannot authorize an FFI task module or an every-target promise. |
| `diagnose-layers` | The current fixture supplies the observations; `spec/30-surface/31-lexical.md` and `spec/30-surface/39-elaboration.md` distinguish parsing and elaboration; `spec/10-kernel/18-judgments.md` and `spec/10-kernel/16-observational.md` ground proof/kernel failures; `spec/40-runtime/42-evaluation.md` grounds the runtime layer; and `crates/ken-cli/src/main.rs` or an observed artifact grounds each proposed CLI check. | `library/reference/toolchain/` and agent task/core prose may route the diagnosis, but they cannot replace the normative layer sources or implemented command evidence. |

No task presents an unresolved choice between two equally authoritative
sources. The derived pages have a useful but strictly subordinate role, so
hard stop 3 does not fire at checkpoint 1.

## Checkpoint 1 disposition

All seven tasks are proposed for one later cold re-run, subject to the
leader's explicit checkpoint-2 release and the protocol's per-fixture
cold-seat proof. No fixture has been spent by this reconciliation.
