---
id: SEC4-TCB
title: "Sec4's trust-model conformance seed is fully authored and nothing executes it — Sec1/Sec1ct/Sec2 each have an acceptance suite bound to their seed, Sec4 has none"
status: ready
owner: verify
size: M
gate: G5
depends_on: []
blocks: []
github: null
origin: "Measured by the Steward 2026-07-27 at origin/main d6df571e while scoping Team Verify's next WP after V4-RESIDUAL merged (PR #1117) and SEC1-IFC-R3 was ruled still-blocked. Not routed by any ring."
---

## The measurement

`conformance/security/trust-model/seed-trust-model.md` is a 21KB seed with five
AC groups and fifteen named cases, normatively grounded in
`spec/60-security/64-trust-model.md` (**Normative**). **Zero** of its case
identifiers appear anywhere in `crates/`:

| seed case | hits |
|---|---|
| `verified-term-has-empty-trusted-base` | **0** |
| `single-postulate-lists-exactly-itself` | **0** |
| `foreign-signature-surfaces-in-delta` | **0** |
| `discharged-hole-empties-delta` | **0** |
| `user-assumption-never-prelude-hidden` | **0** |
| `registered-primitive-surfaces-in-delta` | **0** |
| `kernel-has-no-ken-generated-dependency` | **0** |
| `check-signature-exposes-no-provenance-channel` | **0** |

The sibling seeds are all bound: `sec1_acceptance.rs`, `sec1ct_acceptance.rs`,
and `sec2_acceptance.rs` each open by naming their seed file and enumerating its
AC groups. **Sec4 has no counterpart.**

⇒ **The gap is a missing binding, not missing substrate.** `trusted_base()`
(`env.rs:492`), `trusted_base_delta` (`foreign.rs:256`), and the
`declare_postulate` choke point (`check.rs:1126`) are landed. Sec4's only DAG
dependency is `K-api`, which is landed.

## ⚠ This node exists because the "clean backlog" reading of Sec4 was wrong

⭐ Sec4's substrate is **already covered by scattered tests** —
`km_literal_trust_accounting.rs`, `k5_absurd_trusted_base.rs`,
`b1_acceptance.rs`, and `i8_clock_effect.rs` all exercise `trusted_base()`.
⛔ So this is **not** "Sec4 is unbuilt." It is: the substrate is built and
tested piecemeal, and the authored seed that is supposed to *name* that
coverage is unbound. The frame's `D2` requires a prior-coverage census for
exactly this reason — ⛔ the ring must cite existing coverage, not duplicate it.

## ⛔ Two halves of Sec4, and only one is in scope

`64 §6` splits it explicitly:

- ✅ **In scope** — the machine-checkable substrate: TB-Sound, TB-Complete,
  AI-Indep, Invariant TT.
- ⛔ **Out of scope** — the **external, published, independent kernel-audit
  report**. `§6` defers it as T4-class public documentation **and a governance
  call (external auditor vs. internal reviewer, and the publication decision)
  that is the operator's, not an autonomous one.**

⚠ Seed group **E** (`honest-limits-stated-normative-not-buried`) asserts a
property of documentation prose and is therefore **excluded from the executable
suite** under operator test policy (2026-07-26). It stays a seed row graded by
the conformance validator.

## ⚠ Known-stale, reported not repaired

Every code locator the seed's grounding block cites has drifted since it was
authored: `env.rs:383`→**492**, `check.rs:1055`→**1126**,
`prover.rs:367`→**494**, `env.rs:256`→**330**. ⭐ The seed's *claims* are sound;
only its *locators* moved. ⛔ The frame forbids repairing them in this WP —
`conformance/` is the enclave's, and editing a cited source moves its OID.

## Frame

`docs/program/wp/SEC4-TCB.md`, pinned by blob at `origin/main = d6df571e`.
