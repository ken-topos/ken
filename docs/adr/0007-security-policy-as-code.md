# ADR 0007 — Security policy as code (separately authored, compiler enforced)

- **Status:** Accepted
- **Date:** 2026-06-27
- **Deciders:** the operator

## Context

Ken's information-flow control (ADR 0004, `60-security/61`) makes data-flow
intrinsic and enforced by typing. But in a real organisation, classification has
**two authors with different authority**: **security/compliance** define the
policy (what is PII/secret, the principal/compartment lattice, which sinks are
cleared for what, which declassifications are permitted under what condition,
which ingestion points may classify), and **implementation teams** write code
that must conform.

Inline labels alone put classification authority in the implementer's hands — an
implementer could declare PII `Public`, or would have to re-derive the org
lattice. That is the wrong separation of concerns for compliance (GDPR/PII
boundaries, secret handling, multi-tenant data-residency). The operator, who
works in this space professionally, raised the need for a way for
security/compliance to specify policy **in source, orthogonal to and binding
on** the consuming implementation.

## Decision

**Ken has a security-policy layer: a separately-authored, mandatory, *static*
policy surface — part of Ken, not a sibling.**

1. **Role separation, not engine separation.** Unlike Ward (ADR 0006), which is
   an *engine* separation (different runtime, different time), policy is
   **static** and enforced by **Ken's own type-checker** at the same time as
   everything else — a second *input* to the one engine, not a second engine.
   What is separate is the **authoring role**: security/compliance author the
   policy; implementers are bound.
2. **A policy declares the IFC instantiation** (`60-security/65 §2`): the
   concrete lattice (DLM principals/compartments/regions), classification rules,
   channel/sink clearances, declassification edges (by whom, under what proven
   precondition), and the trusted ingestion points that may assign data-derived
   labels.
3. **Mandatory and non-weakenable.** A policy is **bound** to a build/package
   and imposed **program-wide**; relabeling against policy, writing above a
   clearance, or declassifying without the policy-granted capability are
   **compile errors**. One policy governs many modules, authored once, applied
   everywhere it is bound.
4. **No metatheory or kernel cost.** Because the IFC discipline is
   **lattice-parametric** (`OQ-ifc`: non-interference proved once for any
   bounded lattice), a policy is exactly the configuration the generic proof
   already expected — *data + binding*, not new logic. The by-typing
   non-interference theorem and the TCB are unchanged.
5. **Org-scale governance rides the supply chain.** Policy versioning,
   distribution, and *attestation that a build conformed to policy `vX`* are a
   supply-chain concern (`63`): the governing policy and any exercised
   declassification authority appear in `trusted_base_delta`/provenance. Policy
   (what is allowed) + attestation (what was used) ⇒ "this build provably
   honoured org policy."

## Consequences

- **CISO-grade policy-as-code:** classification authority held by security,
  enforcement by the compiler, no bypass — what a scanner cannot give.
- **Clean fit:** the policy *is* the lattice-parametric discipline's
  instantiation, so it adds nothing to the kernel or the soundness proof.
- **Composes with the rest:** declassification capabilities (`62`), the
  interaction-tree label denotation (`OQ-8`), distributed IFC over labeled
  messages (`OQ-Space`), and Ward's runtime monitoring of labeled events
  (`OQ-behavioral`).
- **Residual within the decided design (`OQ-policy` sub-decisions, `65 §6`):**
  the concrete policy syntax; the binding mechanism (per package/build/
  deployment); how policies **compose** (org → team → service,
  **monotone-tightening only** — a sub-policy may restrict, never relax);
  version/attestation interplay. (`OQ-policy` itself is **DECIDED**, this ADR.)
- **Federation:** security/compliance becomes an explicit authoring **role**
  alongside the implementation teams.

## Revisit if

- A use case needs policy decisions that are *not* statically enforceable (truly
  dynamic, data-dependent *policy* rather than data-dependent *labels*) — weigh
  against the "better is the enemy of good" line; dynamic *labels* at audited
  boundaries (`61 §3`) already cover the known cases (e.g. per-tenant routing).
- Policy composition across orgs/vendors needs cross-trust-domain reconciliation
  beyond monotone tightening.
