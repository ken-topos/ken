# Diagnose

## 1. Use when

Use this module to locate a Ken failure in parsing, elaboration, kernel
checking, proof construction, reference execution, or native execution. Do not
start by rewriting the program or blaming a later layer.

## 2. Prerequisites

Load `../core/toolchain.md` and preserve the exact input, command, stdout,
stderr, exit status, and relevant generated artifact.

## 3. Current capability

The toolchain separates source parsing, elaboration, kernel admission,
reference interpretation, and native lowering. Each boundary has a narrower
command or test that can establish whether the previous layer succeeded.

## 4. Canonical forms

Use this evidence order:

```text
parse -> elaborate -> kernel/proof -> reference run -> native build/run
```

Reduce the input to the smallest case that preserves the same diagnostic and
layer. Keep the successful preceding artifact as the control.

## 5. Invariants and prohibitions

- Diagnose the earliest failing layer.
- A later failure cannot invalidate an earlier kernel proof.
- A check success cannot establish runtime or native behavior.
- Preserve the failing property when minimizing.
- Do not invent a flag, syntax form, host capability, or expected result.

## 6. Decision procedure

1. Re-run the exact command.
2. Classify the first named layer and source span.
3. Run the narrowest preceding check.
4. Minimize while preserving the same failure signature.
5. Compare reference and native paths only after both receive the same checked
   input.
6. Stop if reproduction needs an unavailable host capability or missing
   artifact.

## 7. Failure signatures

| Signature | Layer | Next action |
|---|---|---|
| token/fence error | parser | isolate syntax and fence role |
| unknown/ambiguous name or inferred-type mismatch | elaborator | inspect scope and expected type |
| proof/conversion rejection | kernel/proof | inspect reduced goal |
| `unknown`, effect, or host error after check | interpreter/boundary | inspect interaction and capability |
| reference succeeds, native differs | backend | preserve parity input and outputs |

## 8. Validation

Record a minimal reproducer, the exact failing command, the named signature,
and a nearby control that passes. After a repair, rerun the reproducer and the
targeted regression test; do not substitute a broad green count for the
original symptom.

## 9. Authority and sources

Layer boundaries come from `spec/30-surface/`, `spec/10-kernel/`, and
`spec/40-runtime/`. Current CLI routing is in `crates/ken-cli/src/main.rs`.
Revision: `library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

Environment-specific host, platform, and FFI failures may not reproduce on the
current machine. Native success does not establish all-target support. If the
required environment or capability is absent, stop at the last reproduced
layer and report the missing precondition rather than inventing a diagnosis.
