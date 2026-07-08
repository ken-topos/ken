# NC19 - Executable Artifact Contract

**Owner:** Spec/Runtime-led, with Verify review. **Branch:**
`wp/NC19-executable-artifact-contract`. **Size:** M. **Risk:** medium-high.

## Objective

Define the first native executable artifact contract for closed Ken-only
targets after NC18 clears the runtime-IR agreement gate.

NC19 opens the native executable phase only if the NC18 final report shows that
the starter native-codegen subset is supported through compiler-produced
`RuntimeProgram` artifacts and has runtime-IR/interpreter agreement evidence.

## Scope

In scope:

- executable artifact identity and versioning;
- required checked-core, runtime-IR, native artifact, toolchain, and report
  hashes;
- distinction between executable artifact facts and semantic authority;
- go/no-go recording for the NC18 starter-subset gate;
- stable report lanes for unsupported native emission.

Out of scope:

- object emission or linker behavior;
- library ABI, C ABI, Rust interop, or cross-package native linking;
- new kernel rules, trusted primitives, or proof claims.

## Deliverables

- Executable artifact contract in program/spec docs or runtime docs.
- Gate record tying NC19 opening to the NC18 final report.
- Testable schema or Rust type surface for executable artifact metadata.
- Negative fixtures for stale checked-core/runtime-IR/native identity.

## Acceptance

- A native executable artifact cannot be described without exact checked-core
  and runtime-IR identities.
- The contract states that checked-core and runtime IR remain semantic
  authority; native bytes are tested and reported, not proof evidence.
- Library and interop claims are explicitly unavailable in this phase.
- If NC18's starter subset is not clear, this WP stops and routes the smallest
  prerequisite instead of starting native codegen.

## Guardrails

- Do not treat native object bytes as a checked dependency.
- Do not widen NC8/NC9 validators or NC18 comparison reports by prose.
- Do not define a stable foreign ABI in this WP.
