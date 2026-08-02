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
| `diagnose-layers` | Loaded normative layer sources and CLI evidence | `core/read-ken.md`: parsing, elaboration, and runtime classification; `core/proof-and-trust.md`: kernel proof terminal; `core/toolchain.md`: implemented or observed command. |

## Checkpoint 1 disposition

All scored omissions route to at least one of the five existing modules, so no
claim class requires a sixth module and hard stop 2 does not fire. Every route
has an explicit claim antecedent; none instructs an unconditional load. Module
edits, context-cost measurement, and the seven-answer paper check remain for
later checkpoints after leader acceptance of this table.
