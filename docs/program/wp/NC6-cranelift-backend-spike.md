# NC6 - Cranelift Backend Spike

**Owner:** Runtime. **Branch:** `wp/NC6-cranelift-backend-spike`.
**Size:** L. **Risk:** high.

## Objective

Compile a small runtime-IR subset to native code through Cranelift.

## Scope

Build a Rust backend path from the NC5 runtime IR to Cranelift for a small total
subset. Unsupported constructs must fail loudly.

## Deliverables

- Backend crate/module or isolated backend path.
- Cranelift lowering for representative scalar and constructor cases.
- Minimal runtime shim for supported values.
- Native examples that execute and return observable Ken values.
- Unsupported-construct diagnostics.

## Acceptance

- Cranelift verifier passes.
- Native examples run.
- Interpreter/native results agree for the supported subset.
- Unsupported constructs are distinguished from backend bugs.
- Trust report marks this as F0/F1, not proved.

## Guardrails

No LLVM first target. No claim that Cranelift output is a Ken proof. No
observable pointer identity. No kernel TCB expansion.
