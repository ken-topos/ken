# Security conformance — seed cases

Format: `../README.md`. These pin the tier-1 security guarantees
(`../../spec/60-security/`). Many are **relational** (2-safety) — note the
`given: two runs` shape, distinct from the unary kernel/verify cases.

> **The `ifc/` cases have moved to `ifc/seed-ifc.md`** (WP Sec1, the
> elaborated `61-information-flow.md`). The four placeholders that were here —
> `secret-to-public-rejected`, `declassify-allowed-and-listed`,
> `non-interference`, `integrity-taint-rejected` — are **superseded** there
> (absorbed into the by-typing accept/reject groups + the by-proof verdict
> mapping, with the no-laundering and reduction-faithfulness guards added).
> The `supply-chain/` and `trust/` cases below are Sec3+ and remain here.

> **The `authority/` cases have moved to `capabilities/seed-capabilities.md`**
> (WP Sec2, the elaborated `62-authority.md`). The two placeholders that were
> here — `attenuation-cannot-amplify`, `no-ambient-authority` — are
> **superseded** there: `attenuation-cannot-amplify` → AC3 C4 (sharpened by the
> C1↔C2 order-dual non-degenerate pair — weaker-accepts/stronger-rejects);
> `no-ambient-authority` → AC1 A1 (+ A2 splitting capability and row halves).
> Retired here in that WP to avoid a stale-sibling contradiction.

## security/supply-chain/tampered-proof-rejected (soundness)
- spec: `spec/60-security/63-supply-chain.md §1,§3`
- given: a consumed package whose proof bundle has been tampered/fabricated
- expect: the consumer's kernel **re-check fails** → package rejected
- why: consume = re-check on *your* kernel; trust is authorship-independent.

## security/supply-chain/delta-surfaces-assumptions
- spec: `spec/60-security/63-supply-chain.md §2`, `64 §1`
- given: a package using FFI and a declassification
- expect: both appear in `trusted_base_delta`; a fully-confined verified package
  has an **empty** delta
- why: every assumption is explicit and machine-auditable before use.

## security/trust/independent-recheck (trusting-trust)
- spec: `spec/60-security/64-trust-model.md §3`
- given: output of a (hypothetically) compromised self-hosted toolchain
- expect: the independent permanent Rust kernel **re-checks** it and rejects an
  ill-typed/backdoored artifact
- why: diverse double-compilation built into the architecture.
