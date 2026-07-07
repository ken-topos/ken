# Compiler Continuation Program

> Status: Architectural program frame after NC1-NC9. This document extends
> `07-compiler-program.md`; it is not an implementation spec. Work-package
> briefs for the first follow-on program live under `docs/program/wp/NC10-*`
> through `docs/program/wp/NC18-*`.

## 1. Checkpoint after NC9

NC1-NC9 established a conservative Rust bootstrap compiler path:

```text
CheckedCorePackage v0
  -> erased executable core
  -> RuntimeProgram
  -> Cranelift spike
  -> differential trust report
  -> bounded validator / Ken-checked boundary evidence
```

The path is intentionally narrow. It can consume checked-core packages, lower a
small supported runtime subset, compare selected executions against the
interpreter, and report bounded validation evidence. It is not yet a compiler
for arbitrary Ken packages, not a native artifact emitter for real programs, and
not a whole-compiler proof.

The next work keeps that honesty. We broaden one boundary at a time:

1. accept arbitrary Ken source/packages and lower as much as the current language
   can soundly represent;
2. emit native executables/libraries from the broader runtime artifact;
3. then broaden validation and guarantees over the larger path.

## 2. Remaining compiler campaigns

The following campaigns are deliberately similar in size to NC1-NC9, but they
do not pretend that one more campaign finishes the compiler.

| Campaign | Working range | Primary result | Boundary |
|---|---|---|---|
| General Ken-to-Runtime-IR Compiler | NC10-NC18 | Arbitrary accepted Ken packages can be compiled through checked-core into broad `RuntimeProgram` artifacts or fail loudly with stable unsupported lanes. | No native artifacts beyond existing spike. |
| Native Artifact Emitter | NC19-NC27 | Broad runtime artifacts can produce reproducible executable or library artifacts with explicit ABI, runtime support, packaging, and toolchain facts. | No stronger guarantee than evidence actually checked. |
| Broad Validation and Translation Guarantees | NC28-NC36 | Runtime and backend artifacts gain context-sensitive validators, translation validation, reproducibility checks, and richer trust reports. | Still not full self-hosting or whole-compiler proof. |
| Ken-Owned Pass Expansion / Self-Hosting | NC37+ | Selected compiler passes move from Rust-produced to Ken-authored or Ken-checked implementations. | Rust kernel and Rust bootstrap compiler remain permanent diversity paths. |

This ordering follows the trust boundary. It is cheaper and safer to make the
middle artifact broad and inspectable before making native output broad, and to
make native output honest before claiming stronger guarantees over it.

## 3. NC10-NC18: General Ken-to-Runtime-IR Compiler

### Goal

Build a compiler front door that can consume normal Ken source/packages, run the
existing elaborator and kernel admission path, emit `CheckedCorePackage v0`, and
lower selected executable targets into `RuntimeProgram`.

The user-facing input may be `.ken` source or a package manifest, but the
semantic authority remains the checked-core package:

```text
surface Ken package
  -> elaborator
  -> kernel admission
  -> CheckedCorePackage v0
  -> target closure selection
  -> erased executable core
  -> RuntimeProgram
  -> runtime-IR evaluator / interpreter comparison report
```

The first program exits when this path works for broad supported Ken code and
every unsupported construct fails at a named lane. It does not need to emit a
native binary.

### Work packages

| WP | Objective | Lead | Depends on |
|---|---|---|---|
| NC10 | Compiler driver and target-selection boundary | Language-led, Runtime/Verify support | NC9 |
| NC11 | Checked-core target closure and metadata completeness | Language-led | NC10 |
| NC12 | Runtime-IR evaluator and comparison harness | Runtime-led | NC10, NC11 |
| NC13 | Core expression lowering: variables, lets, lambdas, applications, closures | Runtime-led, Language support | NC11, NC12 |
| NC14 | Data constructor and pattern-match lowering | Runtime-led, Kernel/Language review | NC13 |
| NC15 | Records, dependent Sigma shape, projections, and proof-erasure field lowering | Runtime-led, Verify support | NC13, NC14 |
| NC16 | Primitive value lowering: integers, booleans, strings, bytes, traps | Runtime-led, Language support | NC13 |
| NC17 | Recursion, dictionaries/classes, modules, and package references | Language/Runtime joint | NC11, NC13-NC16 |
| NC18 | Effects and foreign-boundary runtime-IR representation | Runtime-led, Security/Verify review | NC12-NC17 |

### Exit criteria

- A package/file compiler entry point accepts Ken source and emits a checked-core
  package plus a target-selection report.
- Target closure selection is deterministic, content-addressed, and records
  exact package identity, target symbols, dependencies, obligations,
  assumptions, unsupported lanes, and `trusted_base_delta`.
- Supported closed executable targets lower to `RuntimeProgram`.
- A runtime-IR evaluator can execute the supported lowered subset without
  involving Cranelift.
- The runtime-IR evaluator and the landed interpreter can be compared for closed
  executable targets, with reports naming exact artifacts and target symbols.
- Unsupported constructs fail before native/backend work and name the stable
  lane that blocked lowering.
- NC8 and NC9 evidence remains bounded and honest. Broad artifacts do not reuse
  narrow validation labels unless they are actually inside the validated subset.

### Guardrails

- Do not use raw source as semantic evidence after checked-core emission.
- Do not add kernel rules, trusted primitives, or kernel dependencies on compiler
  artifacts.
- Do not make Cranelift, object layout, linker behavior, or native execution part
  of this program's success condition.
- Do not silently lower effects, foreign calls, partial primitives, impossible
  matches, dictionary holes, or unresolved obligations.
- Do not upgrade a report field from unavailable to tested, validated, or proved
  unless the exact run provides the named evidence.

## 4. NC19-NC27: Native Artifact Emitter

The second follow-on campaign turns broad `RuntimeProgram` artifacts into real
native artifacts.

Expected work areas:

- executable and library artifact model;
- public entry points and export metadata;
- ABI, symbol, and calling-convention policy;
- runtime support layer for values, closures, constructors, records, traps,
  effects, and foreign boundaries;
- object/linker packaging and reproducible build metadata;
- host/toolchain capture in trust reports;
- native smoke and differential suites over the NC10-NC18 corpus.

This campaign should not claim stronger validation merely because it emits
native files. It broadens the output boundary first; stronger guarantees remain
the following campaign.

## 5. NC28-NC36: Broad Validation and Translation Guarantees

The third campaign raises evidence over the broader compiler path.

Expected work areas:

- context-sensitive runtime-artifact validation beyond the NC8 seed subset;
- pass certificates for checked-core to runtime-IR lowering;
- Ken-checked or Ken-authored validators over canonical witness surfaces;
- runtime-IR to Cranelift translation validation;
- reproducible native artifact checks;
- richer trust reports with exact unavailable/tested/validated/proved lanes;
- adversarial negative suites for stale identities, hand-fed witnesses, and
  mismatched artifacts.

The target is not a source-to-binary theorem for all Ken programs. The target is
a disciplined evidence ladder where each reported guarantee is independently
checkable and exact-run grounded.

## 6. NC37+: Ken-Owned Pass Expansion and Self-Hosting

Once broad input, useful native output, and broad validation are stable, selected
compiler passes can move into Ken.

Good early candidates are validators, canonical witness checkers, and local
rewrites with small input/output contracts. Poor early candidates are kernel
admission, host ABI support, linker behavior, or broad optimization pipelines.

Self-hosting remains additive. The Rust kernel remains the TCB root, and the
Rust bootstrap compiler remains a permanent independent compiler path for
bootstrapping, diversity, and trusting-trust defense.
