# Agent citation routing record

This record applies `DOC-W6-AGENT-EVAL` D2 to the five existing modules named
by that evaluation's D5 recommendation. Checkpoint 1 fixes the routing table
before any module is edited. It does not change D2, a pack, a task, a fixture,
an evaluation result, or any `library/agents/` module.

## D1 — claim-class routing table

Every row is conditional on a claim the answer actually makes. A row does not
direct a load merely because its module is selected. Where one answer makes
several claim classes, each applicable row fires; where it makes none, the row
causes no load.

### `core/read-ken.md`

| Claim-triggered obligation | Authority to load before asserting | Wave 6 grounding |
|---|---|---|
| If the answer asserts the public declarations, class meaning, or effect signature of a checked package, load that current `catalog/packages/**/*.ken.md` source and the applicable `spec/30-surface/33-declarations.md` or `spec/30-surface/36-effects.md` section before asserting it. | Current checked package; `spec/30-surface/33-declarations.md`; `spec/30-surface/36-effects.md` | D2's package, declaration, and effect authority classes; the cold answers that cited these correctly establish the positive boundary for the rule. |
| If the answer classifies an observation as parsing or elaboration/name resolution, load the applicable `spec/30-surface/31-lexical.md` or `spec/30-surface/39-elaboration.md` section before asserting the layer. | `spec/30-surface/31-lexical.md`; `spec/30-surface/39-elaboration.md` | `diagnose-layers` named these normative areas without loading them. |
| If the answer asserts runtime behavior, a runtime boundary, or a limit on what checking establishes about execution, load `spec/40-runtime/42-evaluation.md` before asserting it. | `spec/40-runtime/42-evaluation.md` | `explain-contract` asserted execution limits without it; the runtime observation in `diagnose-layers` also requires it. |

### `core/proof-and-trust.md`

| Claim-triggered obligation | Authority to load before asserting | Wave 6 grounding |
|---|---|---|
| If the answer asserts why a proof terminal is accepted after reduction or conversion, load the applicable sections of `spec/10-kernel/16-observational.md` and `spec/10-kernel/17-conversion.md` before asserting it. | `spec/10-kernel/16-observational.md`; `spec/10-kernel/17-conversion.md` | `write-pure-law` omitted both; `repair-proof-terminal` omitted conversion; `diagnose-layers` named the kernel area without loading it. |
| If the answer asserts that a declaration adds no trust, inherits trust, or changes the trusted base, load `spec/60-security/64-trust-model.md` before asserting it. | `spec/60-security/64-trust-model.md` | `write-pure-law` and `find-package-by-task` made trust claims without this authority. |

### `core/write-ken.md`

| Claim-triggered obligation | Authority to load before asserting | Wave 6 grounding |
|---|---|---|
| If the answer authors or repairs a `fn` or `theorem` and asserts that its declaration form is supported, load the applicable `spec/30-surface/33-declarations.md` sections, including §8.3 for theorem form, before asserting it. | `spec/30-surface/33-declarations.md` §§1 and 8.3 | `repair-proof-terminal` omitted the theorem-form authority; `write-pure-law` supplies the positive case that already loaded it. |
| If the answer authors a proof terminal and asserts that conversion justifies that terminal, load `spec/10-kernel/17-conversion.md` before asserting it. | `spec/10-kernel/17-conversion.md` | `write-pure-law` and `repair-proof-terminal` both made conversion-dependent authoring claims without this source. |

### `core/toolchain.md`

| Claim-triggered obligation | Authority to load before asserting | Wave 6 grounding |
|---|---|---|
| If the answer asserts that a Ken CLI command or spelling is implemented, or prescribes that command as available, load `crates/ken-cli/src/main.rs` or cite an exact observed command artifact before asserting it. | `crates/ken-cli/src/main.rs` or an exact observed command artifact | `write-pure-law`, `repair-proof-terminal`, `find-package-by-task`, and `diagnose-layers` asserted command availability or behavior without either form of evidence. |
| If the answer asserts what `check`, reference execution, or a host-driven run establishes, load `spec/40-runtime/42-evaluation.md` before asserting it. | `spec/40-runtime/42-evaluation.md` | `explain-contract`, `write-effectful-boundary`, and `diagnose-layers` made runtime or check-versus-run claims governed by this source. |
| If the answer asserts native ABI behavior, executable portability, or an every-target limit, load `spec/40-runtime/48-executable-artifact-contract.md` before asserting it. | `spec/40-runtime/48-executable-artifact-contract.md` | `refuse-unsupported` made substantive native-portability claims without this source. |

### `tasks/effects-and-capabilities.md`

| Claim-triggered obligation | Authority to load before asserting | Wave 6 grounding |
|---|---|---|
| If the answer asserts an effect row or authority-supply boundary, load the current checked capability package plus the applicable `spec/30-surface/36-effects.md` and `spec/60-security/62-authority.md` sections before asserting it. | Current `catalog/packages/Capability/**/*.ken.md` source; `spec/30-surface/36-effects.md`; `spec/60-security/62-authority.md` | `write-effectful-boundary` loaded these correctly, establishing the positive boundary; the rule does not fire for answers that make no effect or authority claim. |
| If the answer asserts a host-driver or runtime entrypoint boundary for an effect, load `spec/40-runtime/42-evaluation.md` before asserting it. | `spec/40-runtime/42-evaluation.md` | `write-effectful-boundary` asserted the host-driver boundary without this source. |
| If an answer asserts an FFI support boundary, including as the reason for an honest refusal, load `spec/30-surface/38-ffi-io.md` before asserting it. | `spec/30-surface/38-ffi-io.md` | `refuse-unsupported` made substantive FFI claims without this source. |

## Seven-answer omission closure

This table proves that D1 was derived from every scored omission rather than
from a generic authority inventory. Row names refer to the routing table above.

| Recorded answer | Scored omission | Governing module and trigger |
|---|---|---|
| `explain-contract` | Runtime and execution-limit authority | `core/read-ken.md`: runtime behavior or checking-versus-execution limit. |
| `write-pure-law` | Observational reduction, conversion, trust, and CLI evidence | `core/proof-and-trust.md`: proof terminal and trust; `core/write-ken.md`: authored conversion; `core/toolchain.md`: implemented or observed command. |
| `repair-proof-terminal` | Theorem declaration form, conversion, and CLI evidence | `core/write-ken.md`: authored theorem and conversion; `core/proof-and-trust.md`: proof terminal; `core/toolchain.md`: implemented or observed command. |
| `find-package-by-task` | Trust and CLI evidence | `core/proof-and-trust.md`: trust posture; `core/toolchain.md`: implemented or observed command. |
| `write-effectful-boundary` | Runtime host-driver authority | `tasks/effects-and-capabilities.md`: host-driver boundary; `core/toolchain.md`: runtime or host-driven run. |
| `refuse-unsupported` | FFI and executable-portability authority | `tasks/effects-and-capabilities.md`: FFI support boundary; `core/toolchain.md`: native ABI or executable portability. |
| `diagnose-layers` | Loaded normative layer sources, including the unresolved judgments authority, and CLI evidence | `core/read-ken.md`: parsing, elaboration, and runtime classification; `core/proof-and-trust.md`: the kernel proof-terminal claim class is assigned here, but the exact `spec/10-kernel/18-judgments.md` pointer remains unresolved; `core/toolchain.md`: implemented or observed command. |

## Checkpoint 1 disposition

D1 assigns every scored claim class to at least one of the five existing
modules, so no claim class requires a sixth module and hard stop 2 does not
fire. For `diagnose-layers`, the proof/kernel-failure class is assigned to
`core/proof-and-trust.md`, but the exact
`spec/10-kernel/18-judgments.md` pointer remains unresolved. Every written
route has an explicit claim antecedent; none instructs an unconditional load.
Module edits, context-cost measurement, and the seven-answer paper check remain
for later checkpoints after leader acceptance of this table.

## D2 — claim-triggered module edits

The five framed modules now carry D1's pointer rules in their existing
`Authority and sources` sections. Each new obligation starts from a claim the
answer makes and points to the source that must be loaded before asserting it.
No rule directs an unconditional load, restates a normative language rule, or
adds a module or pack dependency.

The edit homes are exactly:

- `library/agents/core/read-ken.md` §9;
- `library/agents/core/proof-and-trust.md` §9;
- `library/agents/core/write-ken.md` §9;
- `library/agents/core/toolchain.md` §9; and
- `library/agents/tasks/effects-and-capabilities.md` §9.

## D3 — context-cost record

Line counts use `wc -l` on the exact checkpoint-1 tree and the checkpoint-2
working tree. Every module grew by four or five lines; none gained a new
section.

| Module | Before | After | Growth |
|---|---:|---:|---:|
| `core/read-ken` | 89 | 94 | +5 |
| `core/proof-and-trust` | 89 | 93 | +4 |
| `core/write-ken` | 114 | 119 | +5 |
| `core/toolchain` | 89 | 94 | +5 |
| `tasks/effects-and-capabilities` | 87 | 92 | +5 |
| **Five-module total** | **468** | **492** | **+24** |

Token counts use the `unicode-whitespace-v1` measurement declared by
`library/agents/manifest.toml`. They measure the exact checkpoint-1 and
checkpoint-2 module blobs.

| Module | Before | After | Growth |
|---|---:|---:|---:|
| `core/read-ken` | 549 | 591 | +42 |
| `core/write-ken` | 729 | 764 | +35 |
| `core/proof-and-trust` | 578 | 614 | +36 |
| `core/toolchain` | 557 | 598 | +41 |
| `tasks/effects-and-capabilities` | 531 | 568 | +37 |
| **Five-module total** | **2,944** | **3,135** | **+191** |

Pack closure counts sum the line counts of each pack's unique transitive module
closure, dependencies first, matching the resolution rule documented in
`library/agents/manifest.toml`. Pack manifest lines are not module context and
are not included. Unchanged task modules retain their checkpoint-1 counts.

| Pack | Unique resolved module closure | Before | After | Growth |
|---|---|---:|---:|---:|
| `read-review` | `read-ken`, `proof-and-trust`, `toolchain`, `read-review` | 349 | 363 | +14 |
| `write-pure` | `read-ken`, `write-ken`, `proof-and-trust`, `toolchain`, `write-program` | 456 | 475 | +19 |
| `write-effectful` | `write-pure` closure plus `effects-and-capabilities` | 543 | 567 | +24 |
| `author-package` | `write-pure` closure plus `author-package` | 544 | 563 | +19 |
| `repair-proof` | `write-pure` closure plus `prove-or-repair` | 534 | 553 | +19 |
| `diagnose` | `read-ken`, `proof-and-trust`, `toolchain`, `diagnose` | 345 | 359 | +14 |

The same resolved closures measured in tokens are:

| Pack | Before | After | Growth |
|---|---:|---:|---:|
| `read-review` | 2,093 | 2,212 | +119 |
| `write-pure` | 2,825 | 2,979 | +154 |
| `write-effectful` | 3,356 | 3,547 | +191 |
| `author-package` | 3,305 | 3,459 | +154 |
| `repair-proof` | 3,230 | 3,384 | +154 |
| `diagnose` | 2,120 | 2,239 | +119 |

## Checkpoint 2 disposition

D2 changes exactly the five framed authority sections, and D3 makes their full
context cost visible across all six pack closures. No run or paper check was
performed at this checkpoint; D4 and D5 remain for checkpoint 3 after leader
acceptance.

## D4 — seven-answer paper check

This is a paper check against the seven preserved Wave 6 answers. It does not
re-run a task or spend a fixture. For each answer, the check names the scored
omission, quotes the governing sentence now present in a selected module, and
tests that sentence against claims the answer actually made. Six answers are
fully reached. The seventh is partially reached and preserves one residual
authority that no new sentence would load.

### `explain-contract` — reached

- **Omitted authority.** `spec/40-runtime/42-evaluation.md` for runtime and
  execution-limit claims.
- **Governing sentence.** `core/read-ken.md` now says: “If asserting runtime
  behavior, a runtime boundary, or a limit on what checking establishes about
  execution, load `spec/40-runtime/42-evaluation.md` first.”
- **Trigger in the preserved answer.** The answer said checking does not
  establish runtime or native parity, an external host binding, performance,
  liveness, or timing. Those are limits on what checking establishes about
  execution, so the sentence obliges the omitted load.
- **Load beyond claims made.** None. The rule fires because the answer made
  those execution-limit claims; it would not fire for the package API and
  purity claims alone.

### `write-pure-law` — reached

- **Omitted authorities.** `spec/10-kernel/16-observational.md`,
  `spec/10-kernel/17-conversion.md`, `spec/60-security/64-trust-model.md`, and
  CLI implementation or an exact observed command artifact.
- **Governing sentences.** `core/proof-and-trust.md` now says: “If asserting
  why a proof terminal is accepted after reduction or conversion, load the
  applicable sections of `spec/10-kernel/16-observational.md` and
  `spec/10-kernel/17-conversion.md` first.” It also says: “If asserting that a
  declaration adds no trust, inherits trust, or changes the trusted base, load
  `spec/60-security/64-trust-model.md` first.” `core/toolchain.md` now says:
  “If asserting that a Ken CLI command or spelling is implemented, or
  prescribing that command as available, load `crates/ken-cli/src/main.rs` or
  cite an exact observed command artifact first.”
- **Trigger in the preserved answer.** The answer justified `Proved` by closed
  reduction to `Top`, reported no assumption or primitive, and prescribed
  `ken fmt` and `ken check`. Those claims trigger all four omitted authorities.
- **Load beyond claims made.** None. Each load follows a reduction, trust, or
  available-command claim present in the answer.

### `repair-proof-terminal` — reached

- **Omitted authorities.** `spec/30-surface/33-declarations.md` §8.3,
  `spec/10-kernel/17-conversion.md`, and CLI implementation or an exact
  observed command artifact.
- **Governing sentences.** `core/write-ken.md` now says: “If asserting support
  for an authored or repaired `fn` or `theorem` form, load the applicable
  `spec/30-surface/33-declarations.md` section, including §8.3 for theorem
  form, first.” It also says: “If asserting that conversion justifies an
  authored proof terminal, load `spec/10-kernel/17-conversion.md` first.” The
  `core/toolchain.md` command sentence quoted for `write-pure-law` applies as
  well.
- **Trigger in the preserved answer.** The answer authored two `theorem`
  declarations, justified `Proved` and `Refl` from normalized goal shapes,
  and prescribed a `ken check` attempt. Those claims trigger the theorem-form,
  conversion, and command-evidence loads.
- **Load beyond claims made.** None. The loads are conditional on the exact
  authored forms, proof-terminal justification, and command prescription.

### `find-package-by-task` — reached

- **Omitted authorities.** `spec/60-security/64-trust-model.md` and CLI
  implementation or an exact observed command artifact.
- **Governing sentences.** `core/proof-and-trust.md` now says: “If asserting
  that a declaration adds no trust, inherits trust, or changes the trusted
  base, load `spec/60-security/64-trust-model.md` first.” The
  `core/toolchain.md` command sentence quoted for `write-pure-law` also
  applies.
- **Trigger in the preserved answer.** The answer said the package adds no new
  trust while inheriting the supplied `DecEq`, then recommended `ken check`.
  Those claims trigger both omitted authorities.
- **Load beyond claims made.** None. Package discovery alone would trigger
  neither rule; the trust account and command recommendation do.

### `write-effectful-boundary` — reached

- **Omitted authority.** `spec/40-runtime/42-evaluation.md` for the trusted
  runner, host-driver, and runtime-entrypoint boundary.
- **Governing sentence.** `tasks/effects-and-capabilities.md` now says: “If
  asserting a host-driver or runtime entrypoint boundary for an effect, load
  `spec/40-runtime/42-evaluation.md` first.”
- **Trigger in the preserved answer.** The answer said authority must come
  from an enclosing handler or trusted runner or host, and refused to claim
  runnability without a current runtime source or test establishing the host
  binding. That host-driver boundary triggers the omitted load.
- **Load beyond claims made.** None. Merely stating the checked effect row
  would not fire this rule; the runner and host-boundary explanation does.

### `refuse-unsupported` — reached

- **Omitted authorities.** `spec/30-surface/38-ffi-io.md` and
  `spec/40-runtime/48-executable-artifact-contract.md`.
- **Governing sentences.** `tasks/effects-and-capabilities.md` now says: “If
  asserting an FFI support boundary, including as the reason for an honest
  refusal, load `spec/30-surface/38-ffi-io.md` first.” `core/toolchain.md` now
  says: “If asserting native ABI behavior, executable portability, or an
  every-target limit, load
  `spec/40-runtime/48-executable-artifact-contract.md` first.”
- **Trigger in the preserved answer.** The answer refused a foreign-call form
  on the basis of the selected context's FFI boundary and refused an
  every-native-target promise on portability and ABI grounds. Both sentences
  therefore oblige the omitted loads.
- **Load beyond claims made.** None. The rules fire only because the refusal
  states substantive FFI and portability boundaries.

### `diagnose-layers` — partially reached; one authority not reached

- **Omitted authorities.** Loaded content from
  `spec/30-surface/31-lexical.md`, `spec/30-surface/39-elaboration.md`,
  `spec/10-kernel/18-judgments.md`,
  `spec/10-kernel/16-observational.md`, and
  `spec/40-runtime/42-evaluation.md`, plus CLI implementation or exact observed
  command artifacts.
- **Governing sentences.** `core/read-ken.md` now says: “If classifying a
  parser or elaboration/name-resolution failure, load the applicable
  `spec/30-surface/31-lexical.md` or `spec/30-surface/39-elaboration.md`
  section first.” Its runtime sentence quoted for `explain-contract` also
  applies. `core/proof-and-trust.md` now says: “If asserting why a proof
  terminal is accepted after reduction or conversion, load the applicable
  sections of `spec/10-kernel/16-observational.md` and
  `spec/10-kernel/17-conversion.md` first.” The `core/toolchain.md` command
  sentence quoted for `write-pure-law` also applies.
- **Trigger in the preserved answer.** The answer classified parser,
  elaboration/name-resolution, proof-terminal, and runtime-entrypoint failures
  and prescribed next CLI checks. The new sentences would therefore load the
  lexical, elaboration, observational, conversion, runtime, and command
  authorities.
- **Not reached.** No new sentence obliges the answer to load
  `spec/10-kernel/18-judgments.md`, even though Wave 6 D2 requires it for the
  answer's proof/kernel-failure classification. Naming the proof-terminal
  route does not silently substitute conversion for that judgments source.
- **Load beyond claims made.** None for the authorities that are reached. Each
  follows one of the four classifications or a proposed CLI check. The
  additional conversion load follows the answer's terminal-choice and
  normalized-goal explanation; it is not unconditional.

## D5 — residual findings and recommendation

The paper check reaches six answers fully and one partially, so the majority
criterion is satisfied and hard stop 4 does not fire. The one preserved
residual is a claim class, not a request for another module: an answer that
classifies a failure as a kernel judgment or proof-checking failure can still
make that claim without any revised sentence obliging it to load
`spec/10-kernel/18-judgments.md`.

The recommended follow-up is to amend the existing
`core/proof-and-trust.md` authority section with a claim-triggered pointer for
kernel-judgment classification. No new module or pack dependency is indicated,
and this work package does not apply that recommendation. The stricter Wave 6
D2 rule remains unchanged and remains the scoring authority.

## Final disposition

D1 assigns every scored claim class to the five framed modules without blanket
loading, but leaves the exact `spec/10-kernel/18-judgments.md` pointer
unresolved in the assigned `core/proof-and-trust.md` module. D2 applies the
written routes as pointers, D3 records their context cost, and D4 demonstrates
that the revised modules would fully reach six preserved answers while
exposing—not concealing—the judgments-source residual. D5 routes that residual
to a bounded future amendment. No evaluation, fixture, pack, task, result, or
normative source was changed or re-run.
