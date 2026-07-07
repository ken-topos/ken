# Compiler Program

> Status: Steward integration of the local compiler planning seed. This chapter
> makes the compiler campaign first-class in `docs/program/` and supersedes the
> seed files under `/workspaces/ken/local/compiler/docs/`. It is a program
> frame, not an implementation spec.

## 1. Thesis

Ken should have a durable Rust bootstrap compiler in addition to the later
self-hosted compiler.

The Rust compiler is not merely a temporary spike. It is the permanent
bootstrapping and diversity tool: it can compile checked Ken artifacts before
the self-hosted compiler is fast enough, it gives the trusting-trust defense an
independent implementation, and it remains the practical path for host/backend
work that should not be written in Ken early.

Self-hosting is additive. The self-hosted compiler eventually becomes a
Ken-authored implementation of selected passes and frontend logic, while the
small Rust kernel remains permanent and the Rust bootstrap compiler remains a
valid independent compiler path.

## 2. Boundary

The compiler consumes a stable post-elaboration artifact, not raw surface Ken:

```text
surface Ken
  -> elaborator
  -> kernel admission
  -> CheckedCorePackage v0
  -> erased executable core
  -> Ken runtime IR
  -> Cranelift
  -> native executable binary
```

The important boundary is `CheckedCorePackage v0`. Surface and elaborator work
may continue in parallel, but any change that alters checked-core artifact
meaning must preserve the artifact version, bump it, or provide an explicit
compatibility translation.

## 3. Backend Choice

The native backend target is Cranelift.

This closes the prior `OQ-backend-target` fork. The reason is not that
Cranelift is the strongest optimizer. It is the best current fit for Ken's
values: Rust-native integration, a smaller auditable surface, fast compile
times, and no large C++ toolchain in the normal backend path. Since the backend
is not in the type-soundness TCB, codegen maturity is a quality and performance
concern handled by differential testing and trust reports, not a reason to
enlarge the trusted base.

LLVM, C, and WASM remain possible future secondary targets, but they are not the
bootstrap compiler's first target.

The near-term native target is Ken-only executable generation. Native library
generation is a separate later compiler-program phase because it needs a
different artifact contract: linkable native code must travel with companion
semantic metadata so a later Ken compilation can consume the library as a
checked dependency rather than trusting the object file as proof evidence. C
interop is the eventual stable floor for foreign consumers; Rust interop is
desirable, but should be generated as Rust wrappers/crates over stable Ken or C
compatible handles, not as a promise of Rust's unstable native ABI.

## 4. Fidelity Model

Native compilation fidelity is a chain of bounded claims, checks, and reports,
not a single source-to-binary theorem on day one.

Every compiled artifact should eventually report:

- source, core, erased-core, runtime-IR, and backend artifact hashes;
- compiler, runtime, Cranelift, linker, and host toolchain versions;
- proved passes;
- validated passes;
- assumed passes and backend/runtime assumptions;
- unknown or deferred obligations;
- `trusted_base_delta`.

Fidelity tiers:

| Tier | Meaning |
|---|---|
| F0 | Emits native code for a small subset; tested manually. |
| F1 | Interpreter/native differential tests pass for supported examples. |
| F2 | Ken-owned IR invariants and pass contracts are checked by tests. |
| F3 | Selected Ken-owned passes are proved or checked by Ken. |
| F4 | Runtime-IR to Cranelift lowering is translation-validated. |
| F5 | Optimization certificates are checked. |
| F6 | Backend assumptions are minimized, audited, and reproducible. |

The first useful campaign targets F1/F2. Later work raises selected paths to
F3-F5 without pretending the whole backend is kernel-certified.

NC9's first Ken-checked pass evidence is intentionally narrower than whole
erasure or self-hosting: it checks bounded proof-erasure metadata and
pass-boundary facts across the concrete `CheckedCorePackage v0` /
`RuntimeProgram` pair. It does not certify runtime expression/body lowering,
Cranelift, linker behavior, native execution, or whole-compiler correctness.

## 5. Work Packages

The compiler work packages use `NC` IDs. They extend `X3` rather than replacing
it: `X3` is the high-level native-backend gate; `NC1` onward are the detailed
campaign that makes it buildable.

| WP | Objective | Owner | Depends on |
|---|---|---|---|
| `NC1` | Checked-core package contract | Spec enclave | K-api, V2/V4 exports |
| `NC2` | Stable symbols and canonical encoding | Language/Runtime | NC1 |
| `NC3` | Metadata coverage for primitives, data, classes, recursion, effects, trust | Language/Runtime | NC1, NC2 |
| `NC4` | Checked-core package emitter | Language | NC1-NC3 |
| `NC5` | Erasure boundary and runtime IR seed | Runtime + Architect | NC4 |
| `NC6` | Cranelift backend spike | Runtime | NC5, X1 |
| `NC7` | Differential harness and native trust report | Runtime/Verify | NC6 |
| `NC8` | First certificate validator | Verify/Kernel | NC5, NC7 |
| `NC9` | Bounded Ken-checked proof-erasure boundary checker | Verify-led | NC8 |
| `NC10` | Compiler driver and target-selection boundary | Language-led | NC9 |
| `NC11` | Checked-core target closure and metadata completeness | Language-led | NC10 |
| `NC12` | Runtime-IR evaluator and comparison harness | Runtime-led | NC10, NC11 |
| `NC13` | Core expression lowering | Runtime-led | NC11, NC12 |
| `NC14` | Data constructor and pattern-match lowering | Runtime-led | NC13 |
| `NC15` | Records, Sigma, projections, and proof-erasure field lowering | Runtime-led | NC13, NC14 |
| `NC16` | Primitive value lowering | Runtime-led | NC13 |
| `NC17` | Recursion, dictionaries/classes, modules, package refs | Language/Runtime | NC11, NC13-NC16 |
| `NC18` | Effects and foreign-boundary runtime-IR representation | Runtime-led | NC12-NC17 |

The individual WP briefs live under `docs/program/wp/`.

NC10-NC18 are the first follow-on campaign after the original NC1-NC9 compiler
program. They broaden the input path to arbitrary accepted Ken packages and
lower selected targets into broad runtime IR, but they still stop before native
artifact emission. The continuation frame is `08-compiler-continuation.md`;
that frame now sequences native Ken-only executables before native library
artifacts, and places library generation before self-hosting.

## 6. Non-Goals

Do not make the compiler consume arbitrary surface syntax directly in the
original NC1-NC9 campaign. Do not trust Cranelift output as a Ken proof. Do not
make native pointer identity observable. Do not treat a native object, static
library, or shared library as semantic authority without the checked companion
metadata. Do not block catalog development on self-hosting. Do not require the
Rust bootstrap compiler to disappear once self-hosting exists.

## 7. Review Boundaries

Spec enclave owns the checked-core artifact contract, fidelity vocabulary, and
trust-report semantics. Build teams own implementation against that contract.
Architect reviews the native boundary, erasure boundary, and any claim that a
pass moved from tested to validated or proved. Integrator reviews ordinary
diff/gate hygiene and merge readiness.
