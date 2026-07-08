# NC23 - Object and Linker Packaging

**Owner:** Runtime/Integrator-led. **Branch:**
`wp/NC23-object-linker-packaging`. **Size:** L. **Risk:** high.

## Objective

Package lowered Cranelift output, runtime support, and executable metadata into
reproducible native executable artifacts.

## Scope

In scope:

- object emission for the starter platform target;
- linker invocation or packaging strategy;
- reproducible build metadata and host/toolchain capture;
- artifact hashing and file layout for executable outputs;
- CI-friendly smoke execution for supported examples.

Out of scope:

- cross-platform matrix completeness;
- library artifact formats;
- dynamic linking as a semantic dependency mechanism;
- foreign ABI promises.

## Deliverables

- Native executable packaging path for at least one starter platform.
- Deterministic artifact identity and reproducible metadata.
- Smoke fixture producing and running a closed Ken-only executable.
- Failure lanes for missing toolchain, stale artifact identity, and unsupported
  platform.

## Acceptance

- A closed starter target emits a runnable native executable artifact.
- The artifact records checked-core, runtime-IR, backend, linker, and toolchain
  identities.
- Build failures are explicit and do not masquerade as semantic failures.
- No library or interop contract is introduced.

## Guardrails

- Keep platform scope narrow and named.
- Do not rely on local machine paths as semantic identity.
- Do not make linker success a compiler correctness claim.
