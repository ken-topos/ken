# NC9 - First Ken-Owned Compiler Pass

**Owner:** Language/Runtime/Verify. **Branch:** `wp/NC9-first-ken-owned-pass`.
**Size:** L. **Risk:** high.

## Objective

Implement or check one semantic compiler pass in Ken, while preserving the Rust
bootstrap compiler as a valid implementation path.

## Scope

Pick one pass whose semantics are naturally Ken-owned, such as proof erasure,
pattern-match lowering, dictionary lowering, or closure conversion.

## Deliverables

- Ken implementation or Ken checker.
- Rust/Ken cross-check path.
- Examples and mismatch diagnostics.
- Trust-report status update.

## Acceptance

- Rust and Ken outputs agree on fixtures.
- Mismatches identify the pass boundary.
- The pass can become authoritative after spec and Architect review.
- Rust implementation remains available as bootstrap or performance fallback
  unless explicitly retired.

## Guardrails

This is not full self-hosting. Do not make the Rust kernel depend on
Ken-emitted artifacts.
