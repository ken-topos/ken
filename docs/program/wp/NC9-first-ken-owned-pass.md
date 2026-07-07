# NC9 - First Ken-Checked Compiler Boundary

**Owner:** Verify-led, with Runtime and Language support. **Branch:**
`wp/NC9-first-ken-owned-pass`. **Size:** L. **Risk:** high.

## Objective

Build the first bounded Ken-checked compiler-pass evidence: a Ken-authored
checker for proof-erasure metadata and pass-boundary facts across the exact
`CheckedCorePackage v0` to `RuntimeProgram` pair.

Rust remains the producer of the checked-core package, erased executable core,
and runtime program. NC9 checks a bounded witness about that boundary; it does
not replace the Rust bootstrap compiler.

## Scope

Implement a bounded proof-erasure metadata/pass-boundary checker.

The checked relation is limited to facts that are already represented in the
current checked-core/runtime artifact surfaces:

- package identity, `core_semantic_hash`, and `artifact_hash`;
- proof-erasure classification and metadata survival;
- runtime/proof/law field status;
- lowerability and unsupported lanes;
- obligations;
- assumptions and trust metadata;
- trusted-base delta.

The witness may introduce its own hash only if the hash is recomputed from the
concrete witness, not hand-authored.

This WP does **not** implement whole erasure in Ken and does **not** check
runtime expression or body lowering. The first checker may certify only the
facts it can read from the bounded witness.

## Deliverables

- Runtime support for emitting or exposing a bounded erasure-boundary witness
  derived from the concrete package/program pair.
- Language support only as needed to expose checked-core package fixture fields
  cleanly to the witness path.
- A Ken-authored checker for the bounded witness.
- A Rust/Ken cross-check path over the same fixture set.
- Positive and negative fixtures.
- Mismatch diagnostics that name the failing lane.
- A trust-report update recording a distinct bounded
  `KenCheckedProofErasure...`-style fact, separate from NC7 interpreter/native
  agreement and NC8 runtime-artifact validation.

## Acceptance

- **Input identity.** The checker binds the exact `package_identity`,
  `core_semantic_hash`, and `artifact_hash` from the concrete
  `CheckedCorePackage v0` and `RuntimeProgram` pair.
- **Positive fixture.** At least one bounded fixture with runtime fields plus
  erased law/proof fields, supported lowerability, and surviving
  obligations/trust metadata accepts in both Rust and Ken.
- **Negative fixtures.** The suite rejects stale identity, dropped obligation
  metadata, dropped assumption/trust metadata, changed `Runtime` vs
  `ErasedLaw`/`ErasedProof` status, dropped lowerability/unsupported lane, and a
  witness/program mismatch.
- **Agreement.** Rust and Ken checker outputs agree on the fixture set. A
  disagreement reports the pass boundary and does not emit validation success.
- **Diagnostics.** Every rejection names the failing lane.
- **Checker trust.** The Ken checker elaborates and kernel-checks cleanly with
  no open holes/postulates in its own definitions if its output is used as
  validation evidence. Any helper assumption is surfaced as trust metadata, not
  hidden behind a proved/checker-success label.
- **Report boundary.** The trust report records the bounded NC9 fact separately
  from NC7 F1 interpreter/native agreement and NC8 F2 runtime-artifact
  validation.

## Guardrails

- This is not full self-hosting.
- The Rust bootstrap compiler remains a valid implementation path and cannot
  depend on Ken-emitted artifacts.
- Do not add a Rust-kernel dependency on the Ken checker.
- Do not add a kernel rule, primitive, or trusted admission path.
- Do not use raw surface source as semantic input.
- Do not broaden NC8's runtime-artifact validator or accepted `RuntimeExpr`
  surface in this WP. Constructors, matches, records, projections, closures, or
  calls require a separate context-sensitive recomputation design and fresh
  Architect review before they can carry validation evidence.
- Do not claim Cranelift, linker, native code, object layout, or whole-compiler
  correctness from this check; those facts remain `Unavailable` or out of
  scope.
- Every report field must name an exact evidence source from the run.
  Unavailable facts stay unavailable rather than being backfilled from prose or
  maintained tables.
