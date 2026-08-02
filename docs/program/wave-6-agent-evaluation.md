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

## D3 — cold re-runs

Each task received exactly one run in a distinct fresh context. Before any
fixture was disclosed, the seat attested that it had not seen that fixture,
its expected result, an earlier suite result, or the task-specific prompt.
Those pre-disclosure records are preserved in the WP thread as
`evt_7bhdmsj460y4q`, `evt_7zw08r38rh8r0`, and `evt_4303t773zccz1`.
No seat was retried and no answer was repaired after scoring.

The scorer applied the four axes independently. `unnecessary_loads` and
`inventions` are exact empty lists below. `cited_authority` is measured
against D2, including authorities required by material claims that the answer
made but did not cite.

| Task | Seat | Correctness | Unnecessary loads | Inventions | Cited authority |
|---|---|---|---|---|---|
| `explain-contract` | `/root/w6_cold_explain` | `correct` | `[]` | `[]` | `partial` |
| `write-pure-law` | `/root/w6_cold_pure` | `correct` | `[]` | `[]` | `partial` |
| `repair-proof-terminal` | `/root/w6_cold_proof` | `correct` | `[]` | `[]` | `partial` |
| `find-package-by-task` | `/root/w6_cold_find` | `correct` | `[]` | `[]` | `partial` |
| `write-effectful-boundary` | `/root/w6_cold_effect` | `correct` | `[]` | `[]` | `partial` |
| `refuse-unsupported` | `/root/w6_cold_refuse` | `correct` | `[]` | `[]` | `partial` |
| `diagnose-layers` | `/root/w6_cold_diagnose` | `correct` | `[]` | `[]` | `partial` |

### `explain-contract`

**Cold record.** Seat `/root/w6_cold_explain` supplied the fresh-context
attestation before receiving the `explain-contract` prompt or
`catalog/packages/Core/Logic/EmptyDec.ken.md` fixture.

**Preserved answer.** The answer described `dec_eq_decides` as converting a
supplied `DecEq a` Boolean comparison into an inspectable `Dec (Equal a x y)`.
`Yes` retains equality evidence; `No` retains a refutation. It explained that
the positive branch uses `d.sound`, while the negative branch combines
`d.complete` with equality symmetry and transitivity to derive
`Equal Bool False True` and eliminate it through the empty decision. It
explicitly limited the contract to the supplied dictionary rather than
claiming that every `DecEq` implementation is independently correct.

The answer classified the file as pure: its declarations are `fn` and
`theorem` forms with no `visits` row, host operation, or ambient authority. It
classified the two closed examples as checked local assurance that reduce to
`Top` and terminate with `Proved`. It reported no local `Axiom`, foreign or
primitive declaration, or hole, while retaining the trust inherited from the
supplied `DecEq`; the concrete Boolean instance is checked in the package.

It also stated what checking does not establish: author intent, correctness
of an arbitrary supplied dictionary without that assumption, runtime or
native parity, an external host binding, performance, liveness, timing,
registry provenance, or an independently generated empty trust ledger. It did
not claim that a checker was run.

The answer cited these exact paths:

- `catalog/packages/Core/Logic/EmptyDec.ken.md`
- `spec/30-surface/33-declarations.md`
- `spec/30-surface/36-effects.md`
- `spec/20-verification/21-spec-syntax.md`
- `spec/60-security/64-trust-model.md`
- `library/agents/core/read-ken.md`
- `library/agents/core/proof-and-trust.md`
- `library/agents/core/toolchain.md`
- `library/agents/tasks/read-review.md`

**Loads and commands.** The initial load was the evaluation protocol, agent
overview, `read-review` pack, its `read-ken`, `proof-and-trust`, and
`toolchain` core modules, `tasks/read-review.md`, and the EmptyDec fixture.
The seat then loaded the four cited spec files because the prompt required
exact language, effect, assurance, and trust claims. It used `sed` to read
those files and targeted `rg`/`sed` calls to locate their relevant headings.
Every command exited 0 with empty stderr. No checker command was run.

**Independent score.** The answer is correct, contains no invention, and every
extra load supports a claim it made. Citation authority is `partial`: it made
explicit runtime and execution-limit claims but omitted D2's required
`spec/40-runtime/42-evaluation.md` authority.

### `write-pure-law`

**Cold record.** Seat `/root/w6_cold_pure` supplied the fresh-context
attestation before receiving the `write-pure-law` prompt or its held-back
authoring fixture.

**Preserved answer.** The complete proposed source was:

```ken
fn both (a : Bool) (b : Bool) : Bool =
  match a {
    True ↦ b;
    False ↦ False
  }

theorem both_true_true : Equal Bool (both True True) True = Proved
```

The answer instructed the reader to save it as `pure-law.ken`, then use
`ken fmt pure-law.ken` and `ken check pure-law.ken`. It explained that the
closed computation reduces the equality proposition to `Top`, so the theorem
uses `Proved`, and reported no assumption or primitive. It cited
`spec/30-surface/33-declarations.md`,
`spec/20-verification/21-spec-syntax.md`, and the selected `write-ken`,
`proof-and-trust`, and `toolchain` core modules. It preserved the actual run
result: `ken` was absent from that seat's `PATH`; the single validation attempt
exited 127 and the seat stopped without retrying or claiming validation.

**Loads and commands.** The initial load comprised the protocol and agent
overview; `write-pure` pack; `read-ken`, `write-ken`, `proof-and-trust`, and
`toolchain`; `write-program`; and the authoring fixture.
The only additional loads were the two cited spec files. `sed` reads exited 0
with empty stderr. The one task-seat command was
`ken check /tmp/doc-w6-agent-eval-pure-law.ken`; it exited 127 with the shell's
`ken: command not found` diagnostic.

For scorer verification, without exposing the answer back to the cold seat,
the preserved file was checked at this exact checkout with
`target/debug/ken check /tmp/doc-w6-agent-eval-pure-law.ken` and
`target/debug/ken fmt --check /tmp/doc-w6-agent-eval-pure-law.ken`. Both
exited 0 with empty stdout and stderr. These scorer controls were not a second
cold run.

**Independent score.** The answer is correct, has no unnecessary load, and
invents no syntax or capability. Citation authority is `partial`: its
reduction, conversion, no-new-trust, and CLI claims omit D2's required
`spec/10-kernel/16-observational.md`,
`spec/10-kernel/17-conversion.md`,
`spec/60-security/64-trust-model.md`, and CLI implementation or observed
successful command authority.

### `repair-proof-terminal`

**Cold record.** Seat `/root/w6_cold_proof` supplied the fresh-context
attestation before receiving the `repair-proof-terminal` prompt or fixture.

**Preserved answer.** The complete repaired source was:

```ken
fn bool_and (a : Bool) (b : Bool) : Bool =
  match a {
    True ↦ b;
    False ↦ False
  }

theorem collapsed : Equal Bool (bool_and True True) True = Proved

theorem stuck (x : Bool) : Equal Bool (bool_and x x) (bool_and x x) = Refl
```

The answer explained that terminal choice follows the normalized goal shape:
`collapsed` computes to `Top` and therefore uses `Proved`; `stuck` remains an
equality with definitionally identical endpoints and therefore uses `Refl`.
It cited `spec/10-kernel/16-observational.md`,
`spec/10-kernel/15-identity.md`,
`spec/20-verification/21-spec-syntax.md`, and the selected proof/task modules.
It preserved the validation fact that the one cold-seat `ken check` attempt
could not run because `ken` was absent from `PATH`; it made no validation
claim and did not retry.

**Loads and commands.** The initial load comprised the protocol and overview;
`repair-proof` and its transitive `write-pure` pack; all four core modules;
the relevant writing and proof-repair tasks; and the proof-terminal fixture.
The seat first attempted the nonexistent path
`spec/10-kernel/16-propositions.md`, received exit 1, used `rg --files spec` to
resolve the current normative paths, and then loaded the three cited spec
files. Those loads were justified by the prompt's proof-terminal claim. The
single task-seat checker attempt exited 127 because `ken` was not in `PATH`.

For scorer verification, the preserved file was checked with
`target/debug/ken check /tmp/proof-terminals-repaired.ken` and
`target/debug/ken fmt --check /tmp/proof-terminals-repaired.ken`. Both exited
0 with empty stdout and stderr. This did not re-run the cold task.

**Independent score.** The repair is correct; the failed path lookup did not
load a file and the subsequent normative loads were necessary. There is no
invention. Citation authority is `partial`: D2 additionally requires
`spec/30-surface/33-declarations.md` section 8.3,
`spec/10-kernel/17-conversion.md`, and CLI implementation or an observed
successful command artifact.

### `find-package-by-task`

**Cold record.** Seat `/root/w6_cold_find` supplied the fresh-context
attestation before receiving the `find-package-by-task` prompt or discovery
fixture.

**Preserved answer.** The answer selected
`catalog/packages/Core/Logic/EmptyDec.ken.md` as the existing checked package
for deciding equality by a supplied `DecEq` while retaining positive or
negative evidence. It described the package's local trust posture as adding no
new trust: standard inductives and definitions are kernel checked, with no
postulate or primitive, while the supplied `DecEq` remains an inherited
assumption. It recommended
`ken check catalog/packages/Core/Logic/EmptyDec.ken.md`. It stated that the
package was found by enumerating the catalog, selecting the sole direct task
match, and inspecting the checked entry, rather than stopping at a derived
card or subject index.

The answer cited:

- `catalog/packages/README.md`
- `catalog/packages/Core/Logic/EmptyDec.ken.md`
- `library/agents/tasks/author-package.md`
- `library/agents/core/toolchain.md`

**Loads and commands.** The initial load comprised the protocol and overview;
`author-package` and transitive `write-pure` packs; all four core modules;
`write-program` and `author-package` tasks; the catalog discovery page; and the
discovery fixture. The only additional file was the selected checked package,
loaded after `rg --files catalog/packages` identified the candidate. The reads
and enumeration exited 0 with empty stderr. No checker was run, and the answer
did not claim otherwise.

**Independent score.** The selection and bounded trust account are correct,
the selected package load is necessary, and there is no invention. Citation
authority is `partial`: the trust statement omits
`spec/60-security/64-trust-model.md`, and the proposed CLI command omits
implementation or observed-command authority required by D2.

### `write-effectful-boundary`

**Cold record.** Seat `/root/w6_cold_effect` supplied the fresh-context
attestation before receiving the `write-effectful-boundary` prompt or
`catalog/packages/Capability/Console/Text.ken.md` fixture.

**Preserved answer.** The complete proposed boundary was:

```ken
proc announce (text : String) : IO (Result IOError Unit) visits [Console] =
  printLine text
```

The answer identified the effect row as `visits [Console]`, with transitive
`Console` use through `printLine`. It explained that unhandled authority must
come from an enclosing handler or trusted runner/host because Ken supplies no
ambient authority. It deliberately did not invent a capability constructor,
manifest, host binding, or driver invocation. It proposed checking the landed
literate package and the containing file, with negative controls for a removed
row and missing authority, but explicitly did not claim those checks were run.
It refused to claim that the boundary was runnable without a current runtime
source or test establishing the Console host binding.

The exact cited paths were:

- `catalog/packages/Capability/Console/Text.ken.md`
- `spec/30-surface/36-effects.md`
- `spec/60-security/62-authority.md`
- `library/agents/core/toolchain.md`
- `library/agents/tasks/effects-and-capabilities.md`

**Loads and commands.** The initial load was the protocol and overview;
`write-effectful` and transitive `write-pure` packs; all four core modules;
`write-program` and `effects-and-capabilities` tasks; and the checked Console
fixture. The only additional files were the two cited spec authorities. Two
`sed` commands read the initial closure and those specs; both exited 0 with
empty stderr. No validation command was run.

**Independent score.** The source and its bounded claims are correct, both
additional loads are justified, and the refusal avoids invention. Citation
authority is `partial`: the answer asserts a trusted-runner and host-driver
boundary without D2's required `spec/40-runtime/42-evaluation.md` source.

### `refuse-unsupported`

**Cold record.** Seat `/root/w6_cold_refuse` supplied the fresh-context
attestation before receiving the `refuse-unsupported` prompt or fixture.

**Preserved answer.** The answer refused to provide the requested FFI source
or every-native-target promise. It explained that the selected
`write-effectful` pack excludes FFI declarations, unsupported host
capabilities, and platform ABI work; that the agent selection protocol has no
FFI/platform pack in this wave and requires failure closed; and that the
effect and program-writing task modules prohibit invented host bindings,
unsupported operations, FFI, and platform primitives.

It stated that the loaded evidence does not establish a canonical FFI task
module, supported foreign-call declaration, process-global-clock binding, or
portable ABI support, without escalating that bounded observation into a
claim that no implementation exists elsewhere. It also refused the every
target promise because native-build success alone does not establish
interpreter parity or untested-target behavior. It required a supported pack,
checked declaration, host binding, ABI authority, and per-target validation
before such source or portability claims could be authored.

The exact cited paths were:

- `library/agents/packs/write-effectful.toml`
- `library/agents/README.md`
- `library/agents/tasks/effects-and-capabilities.md`
- `library/agents/tasks/write-program.md`
- `library/agents/core/toolchain.md`

**Loads and commands.** The seat loaded exactly the protocol, overview,
`write-effectful` and transitive `write-pure` packs, all four core modules, the
two task modules, and `unsupported-ffi.txt`. It loaded nothing else. One
concatenated `sed` command read all eleven files; it exited 0 with empty stderr
and complete output. No repository file was edited.

**Independent score.** Refusal is the correct task result, every initial load
belongs to the resolved pack or fixture, and no unsupported form was invented.
Citation authority is `partial`: the answer makes substantive FFI and native
portability claims without D2's normative
`spec/30-surface/38-ffi-io.md` and
`spec/40-runtime/48-executable-artifact-contract.md` authorities.

### `diagnose-layers`

**Cold record.** Seat `/root/w6_cold_diagnose` supplied the fresh-context
attestation before receiving the `diagnose-layers` prompt or
`four-layer-diagnostics.txt` fixture.

**Preserved answer.** It classified the four observations as follows:

1. The unexpected character at byte 0 is a parser failure. Inspect byte 0 and
   isolate the smallest plain `.ken` input preserving the token diagnostic;
   do not proceed to elaboration.
2. Successful parsing followed by unresolved `Missing` is an elaboration and
   name-resolution failure. Inspect local scope, imports, qualifiers, and
   declaration order while retaining the parsed declaration as a control.
3. Rejected `Refl` after both concrete endpoints reduce to `True` is a
   kernel/proof-terminal failure. Inspect the normalized goal; `Top` requires
   `Proved`, whereas an unreduced reflexive equality requires `Refl`.
4. Successful `ken check` followed by missing `Console` during `ken run` is a
   runtime entrypoint capability-boundary failure. Compare the declared effect
   and capability requirement with the invocation's authority supply; do not
   infer a native-backend failure or invent a fallback.

The answer cited these exact paths:

- `library/agents/tasks/diagnose.md`
- `library/agents/core/read-ken.md`
- `library/agents/core/proof-and-trust.md`
- `library/agents/core/toolchain.md`
- `spec/30-surface/`
- `spec/30-surface/33-declarations.md`
- `spec/30-surface/36-effects.md`
- `spec/20-verification/21-spec-syntax.md`
- `spec/10-kernel/`
- `spec/40-runtime/42-evaluation.md`

**Loads and commands.** The seat loaded exactly the protocol, overview,
`diagnose` pack, `read-ken`, `proof-and-trust`, `toolchain`, the diagnose task,
and the fixture. It loaded no additional file: the spec paths above were named
by the selected modules but not opened. One concatenated `sed` command read
the eight initial files; it exited 0 with empty stderr and complete output.

**Independent score.** All four layer classifications and next checks are
correct; the seat made no extra load and invented no command or capability.
Citation authority is `partial`: merely naming normative paths without
loading their content does not ground the claims, and D2 additionally requires
CLI implementation or an observed command artifact for the proposed CLI
checks.

## Checkpoint 2 disposition

All seven one-shot cold runs are preserved. Every task is correct and has no
unnecessary load or invention, but every task scores
`cited_authority = "partial"` under the claim-sensitive rule fixed in D2.
The citation-authority axis therefore fails for the current corpus. Under the
suite exit predicate, `agent_core_ready` cannot be true for this run.

That was the frame's hard stop 1. Checkpoint 2 recorded the false-axis result
and stopped without a fixture retry or corpus repair. The Steward accepted the
stop and released the bounded checkpoint-3 record below; D2 remains unchanged.

## D4 — current-corpus result

The canonical result for this run is
`library/agents/evaluations/results-2026-08-02.toml`. It preserves all seven
one-shot cold runs in the historical artifact's shape and states the suite
verdict directly:

```toml
agent_core_ready = false
failing_axes = ["cited_authority"]
```

All seven answers were correct, loaded no unnecessary file, and invented
nothing. All seven scored `cited_authority = "partial"` under the
claim-sensitive D2 rule fixed before disclosure. The failing authority axis is
therefore sufficient to make the exit predicate false.

This is not evidence that the corpus regressed since the 2026-07-24 run. The
evaluation protocol named `cited_authority` then but did not define it. D2 is
the suite's first written citation rule, and it is stricter than the unwritten
scorer-local standard that produced the July values. The July and August
authority scores are not directly comparable, so this result attributes the
failure to the newly explicit scoring rule rather than to corpus drift.

The displacement hypothesis was also tested directly and did not hold:
**zero of seven cold answers substituted a derived `library/` page for an
available normative, checked-package, or implementation authority.** The 54
new derived pages from Waves 3–5 did not displace authority in this run. That
is a positive result even though the stricter overall verdict is false.

## D5 — pack reconciliation recommendation

The repeated failure shape is under-citation: all seven answers made a material
claim without loading or citing every authority D2 requires, while none
invented a form, loaded an unnecessary file, or substituted a derived page.
The current packs select useful explanatory modules, but those modules do not
reliably induce the claim-sensitive normative and implementation citations.

A successor WP should strengthen these existing modules rather than widen a
pack with derived references:

- `library/agents/core/read-ken.md` should require loading the exact normative
  section before using a spec path to support a material claim; merely naming a
  path discovered in a module is not citation closure.
- `library/agents/core/proof-and-trust.md` should route proof-terminal,
  conversion, and trust claims to the exact kernel, verification, and trust
  sections D2 identified.
- `library/agents/core/write-ken.md` should induce the declaration and
  conversion authorities for authored `fn` and `theorem` forms.
- `library/agents/core/toolchain.md` should distinguish a proposed command
  from an observed artifact and route CLI, runtime, FFI, and portability claims
  to implementation evidence or their exact normative boundaries.
- `library/agents/tasks/effects-and-capabilities.md` should route FFI and host
  boundary claims to the FFI and executable-artifact sections when those
  claims arise, including in an honest refusal.

This recommendation is recorded only. No pack manifest, core module, task,
fixture, checked catalog source, derived page, or historical result was edited,
and no fixture was rerun or spent at checkpoint 3.
