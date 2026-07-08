# NC26 - Native Trust Report and Provenance

**Owner:** Verify/Runtime-led. **Branch:**
`wp/NC26-native-trust-report-provenance`. **Size:** L. **Risk:** high.

## Objective

Extend trust reports for native executable artifacts with exact provenance,
toolchain, and evidence lanes.

## Scope

In scope:

- report fields for checked-core package, runtime IR, native artifact, backend,
  linker, runtime support, host, and toolchain identities;
- classification of tested, validated, proved, unavailable, and unsupported
  native-executable claims;
- mismatch diagnostics for stale or inconsistent artifact chains;
- integration with NC24 differential results and NC25 effect policy.

Out of scope:

- proving Cranelift or linker correctness;
- library metadata sidecars;
- supply-chain guarantees beyond recorded facts.

## Deliverables

- Native executable trust-report schema or type surface.
- Report generator attached to compiler/native emission.
- Negative fixtures for stale artifact hashes, mismatched toolchain facts, and
  overclaimed evidence lanes.

## Acceptance

- Every native executable report binds exact checked-core, runtime-IR, native,
  and toolchain identities.
- Report lanes do not overclaim beyond exact evidence from NC24 and NC25.
- Stale or mismatched provenance rejects before a successful report.
- Library and interop fields remain unavailable or absent.

## Guardrails

- Do not make a report field authoritative without a producing check.
- Do not reuse NC8/NC9 labels outside their validated subset.
- Do not treat host/toolchain capture as supply-chain proof.
