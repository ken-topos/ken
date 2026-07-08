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

1. accept arbitrary Ken source/packages and lower as much as the current
   language can soundly represent;
2. emit Ken-only native executables from the broader runtime artifact;
3. then broaden validation and guarantees over the executable path;
4. add native library artifacts and interop as a separate phase before
   self-hosting.

## 2. Remaining compiler campaigns

The following campaigns are deliberately similar in size to NC1-NC9, but they
do not pretend that one more campaign finishes the compiler.

| Campaign | Working range | Primary result | Boundary |
|---|---|---|---|
| General Ken-to-Runtime-IR Compiler | NC10-NC18 | Arbitrary accepted Ken packages can be compiled through checked-core into broad `RuntimeProgram` artifacts or fail loudly with stable unsupported lanes. | No native artifacts beyond existing spike. |
| Native Ken Executable Emitter | NC19-NC27 | Broad runtime artifacts can produce reproducible platform executables for Ken-only closed targets, with runtime support, packaging, and toolchain facts. | No library ABI, C ABI, Rust interop, or cross-package native linking claim. |
| Broad Validation and Translation Guarantees | NC28-NC36 | Runtime and backend artifacts gain context-sensitive validators, translation validation, reproducibility checks, and richer trust reports. | Still not full self-hosting or whole-compiler proof. |
| Native Library Artifact Generation | NC37-NC45 | Checked Ken packages can emit linkable library artifacts plus companion semantic metadata so later Ken compilations can consume them as checked dependencies. | Library artifacts are not executable proof carriers; C/Rust interop remains explicit and metadata-backed. |
| Ken-Owned Pass Expansion / Self-Hosting | NC46+ | Selected compiler passes move from Rust-produced to Ken-authored or Ken-checked implementations. | Rust kernel and Rust bootstrap compiler remain permanent diversity paths. |

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

- A package/file compiler entry point accepts Ken source and emits a
  checked-core package plus a target-selection report.
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

### Phase gate to native executable work

NC18 is the transition gate into native executable generation. Completion of
NC18 is sufficient to open NC19 only when the final NC10-NC18 report shows:

- the starter Ken-only executable subset is supported through compiler-produced
  `RuntimeProgram` artifacts;
- the runtime-IR evaluator agrees with the interpreter on that starter subset;
- unavailable comparisons are outside the starter native-codegen set, or are
  named as blockers with a prerequisite WP;
- effects and foreign-boundary facts are represented explicitly and do not
  silently enter native execution;
- reports keep unsupported, failed, unavailable, tested, validated, and proved
  lanes distinct.

If this gate is clear, the Steward should proceed directly to NC19. If it is
not clear, do not start native codegen; frame the smallest prerequisite needed
to close the reported gap first.

### Guardrails

- Do not use raw source as semantic evidence after checked-core emission.
- Do not add kernel rules, trusted primitives, or kernel dependencies on
  compiler artifacts.
- Do not make Cranelift, object layout, linker behavior, or native execution
  part of this program's success condition.
- Do not silently lower effects, foreign calls, partial primitives, impossible
  matches, dictionary holes, or unresolved obligations.
- Do not upgrade a report field from unavailable to tested, validated, or proved
  unless the exact run provides the named evidence.

## 4. NC19-NC27: Native Ken Executable Emitter

The second follow-on campaign turns broad `RuntimeProgram` artifacts into
real native executables for Ken-only closed targets.

### Goal

Emit reproducible native executables for closed Ken-only targets whose
semantics are already represented by the NC10-NC18 `RuntimeProgram` path. The
semantic authority remains the checked-core package plus runtime IR and the
exact-run reports; native execution is an implementation artifact to test and
report, not proof evidence.

### Work packages

| WP | Objective | Lead | Depends on |
|---|---|---|---|
| NC19 | Executable artifact contract | Spec/Runtime | NC18 |
| NC20 | Entrypoint packaging metadata | Language/Runtime | NC19 |
| NC21 | Ken-only executable runtime | Runtime | NC19, NC20 |
| NC22 | Cranelift lowering for runtime IR | Runtime | NC21 |
| NC23 | Object/linker packaging | Runtime/Integrator | NC22 |
| NC24 | Native differential suite | Runtime/Verify | NC22, NC23 |
| NC25 | Effect/foreign executable policy | Runtime/Verify | NC18, NC24 |
| NC26 | Native trust report | Verify/Runtime | NC23-NC25 |
| NC27 | Executable phase closeout | Runtime/Verify | NC19-NC26 |

### Expected work areas

- executable artifact model;
- `main`/entry-point selection and executable packaging metadata;
- runtime support layer for values, closures, constructors, records, traps,
  effects, and foreign boundaries;
- object/linker packaging and reproducible build metadata;
- host/toolchain capture in trust reports;
- native smoke and differential suites over the NC10-NC18 corpus.

This campaign should not claim stronger validation merely because it emits
native files. It broadens the output boundary first; stronger guarantees remain
the following campaign.

Library artifacts are explicitly out of this phase. In particular, NC19-NC27
should not define a stable Ken library ABI, C ABI, Rust interop surface,
cross-package native linking contract, or metadata sidecar format for imported
native libraries. A closed executable may embed or package its trust report and
semantic identities, but it does not need to be consumed later as a checked Ken
dependency.

### Exit criteria

- The compiler can emit a platform native executable for a closed Ken-only
  target in the supported NC10-NC18 subset.
- The emitted executable carries or points to exact checked-core, runtime-IR,
  native artifact, toolchain, and trust-report identities.
- Native execution is compared against runtime-IR evaluation and interpreter
  observations for the starter executable corpus.
- Unsupported effects, foreign calls, library exports, dynamic imports, or
  interop requests fail before native execution with stable lanes.
- Native executable reports do not claim library ABI, Rust interop, C interop,
  cross-package native linking, or whole-compiler proof.

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

## 6. NC37-NC45: Native Library Artifact Generation

After executable emission and broad validation are useful, Ken should add a
separate library-generation campaign. A compiled Ken library must be usable by a
later Ken compilation as if the dependency had been compiled in the same
package graph. That requires a semantic import artifact in addition to any
native object, static library, or shared library.

Expected work areas:

- library artifact modes: Ken-only library, C ABI export floor, and Rust
  binding/wrapper generation;
- companion semantic metadata carrying `CheckedCorePackage` identity, stable
  symbols, exported target closures, dependency semantic hashes, obligations,
  assumptions, `trusted_base_delta`, erased proof/law field status, and native
  artifact hashes;
- Ken-to-Ken library import semantics that consume the semantic artifact rather
  than treating native object code as proof evidence;
- ABI and calling-convention policy for exported runtime values, closures,
  constructors, records, traps, effects, and foreign boundaries;
- runtime ownership/allocation rules across library boundaries;
- C interop as the stable floor, with Rust interop preferably generated as a
  Rust crate or wrapper over stable Ken/C-compatible handles rather than a
  promise of Rust's unstable native ABI;
- trust reports that distinguish checked package facts, tested native behavior,
  validated translation facts, and unavailable proof-erasure or interop claims.

This phase must keep the native object and the semantic artifact conceptually
separate. A native library may be a cached executable realization of checked
Ken semantics; it is not itself the semantic authority. If a consumer cannot
validate the companion metadata against the package hashes and exported closure,
the import must fail before linking or execution.

## 7. NC46+: Ken-Owned Pass Expansion and Self-Hosting

Once broad input, useful native output, and broad validation are stable,
selected compiler passes can move into Ken.

Good early candidates are validators, canonical witness checkers, and local
rewrites with small input/output contracts. Poor early candidates are kernel
admission, host ABI support, linker behavior, or broad optimization pipelines.

Self-hosting remains additive. The Rust kernel remains the TCB root, and the
Rust bootstrap compiler remains a permanent independent compiler path for
bootstrapping, diversity, and trusting-trust defense.
