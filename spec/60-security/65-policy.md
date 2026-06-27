# Security policy — authored by compliance, enforced by the compiler

> Status: **DRAFT v0**. Normative for the *shape and the binding guarantee*; the
> concrete policy syntax is `OQ-policy`. The **separately-authored, mandatory,
> static** security-policy surface (ADR 0007). It instantiates the
> lattice-parametric information-flow discipline (`61-information-flow.md`) and
> is the thing a CISO authors: **policy as code, enforced by the type-checker,
> separated from — and binding on — the implementation.**

## 1. Why a policy layer (role separation, not a new engine)

Classification has **two authors with different authority**. **Security /
compliance** define *what the policy is* — what counts as PII/secret, the
principal/compartment lattice, which sinks are cleared for what, which
declassifications are permitted under what proven condition, which ingestion
points may classify. **Implementation teams** write code that must *conform*.
Putting all of that in inline labels (`61 §3`) puts classification authority in
the wrong hands: an implementer must not be able to declare PII `Public`, and
must not have to re-derive the org lattice.

So Ken has a **policy layer**. It is **part of Ken, not a sibling** (contrast
Ward, `../70-behavioral/`): policy is **static**, enforced by **Ken's own
type-checker**, at the same time as everything else — it is not a second engine,
it is a second *input* to the one engine. What is separate is the **authoring
role**, mirroring the agent-team federation: security/compliance own the policy
as the team constraint, implementers are bound by it.

## 2. What a policy declares

A **policy module** (authored by security/compliance) supplies the concrete
instantiation of the IFC discipline (`61 §2`, lattice-parametric):

- **The lattice `ℒ`** — principals, compartments, data-residency regions, and
  any level sugar (the DLM standard, `61 §2`).
- **Classification rules** — which types/data carry which labels ("`Email` is
  `String @ Secret[user]`"; "rows of table `customers` are PII"); structural and
  source-based.
- **Channel/sink clearances** — what each `Net`/`FS`/RPC/log/`space` sink (and
  each per-compartment sink, e.g. `bucket[Tenant X]`) is cleared for (`61 §3`).
- **Declassification edges** — which downgrades are permitted, **by whom** (the
  declassification capability, `62`), under what **proven precondition** (a
  `requires`, `61 §4`).
- **Trusted classification points** — which ingestion boundaries may assign
  **data-derived** labels (the audited dual of declassification, `61 §3`).

## 3. Mandatory and orthogonal (the load-bearing property)

A policy is **bound** to a build/package and is **imposed program-wide as a
constraint the implementer cannot weaken**:

- An implementation **declares the policy that governs it**; the checker
  enforces the policy's lattice, classifications, and clearances across the
  whole program.
- **Relabeling against policy is a compile error**, not a review nit: declaring
  a policy-`Secret` value `Public`, writing to a sink above its clearance, or
  declassifying without the policy-granted capability all **fail to
  type-check**.
- Policy is **orthogonal** to implementation: one policy governs many modules;
  it is authored once and applies everywhere it is bound. Implementers write
  ordinary code; the policy is the cross-cutting constraint they cannot escape.

This is exactly what a CISO wants and a scanner cannot give: classification
authority held by security, enforcement by the compiler, no bypass.

## 4. Policy is the *instantiation*, so it costs no metatheory

Because the IFC discipline is **lattice-parametric** (`61 §2`, non-interference
proved once for any bounded `ℒ`), a policy is precisely the **configuration the
generic proof was waiting for** — the lattice + classifications + clearances +
edges. Adding the policy layer therefore **adds nothing to the metatheory and
nothing to the kernel**: the same by-typing non-interference theorem (`61 §5.1`)
holds for whatever lattice the policy supplies, and the relational/bespoke route
(`OQ-relational`) is unchanged. Policy is *data + binding*, not new logic.

## 5. Org-scale governance → the supply-chain story

The **mechanism** is in Ken (this chapter). **Governance at organisational
scale** — versioning policies, distributing them across repos, and *attesting*
that a build provably conformed to policy `vX` — is the **supply-chain** concern
(`63-supply-chain.md`): the policy a package was built under, and any
declassification authority it exercised, appear in its **`trusted_base_delta`**
and provenance (`../20-verification/25 §3`). Policy (what is allowed) +
per-package attestation (what was used) together give **"this build provably
honoured org policy."**

## 6. What is committed vs open

- **Committed (ADR 0007):** a separately-authored, mandatory, static policy
  surface, **in Ken** (not a sibling); it instantiates the lattice-parametric
  discipline; implementers are **bound and cannot weaken** it; org-scale
  governance rides the supply-chain attestation.
- **Open (`OQ-policy`):** the concrete policy **syntax**; the exact **binding**
  mechanism (per package / per build / per deployment) and how policies
  **compose** (org → team → service overrides, monotone-tightening only); how
  policy versions interact with the supply-chain attestation. Sub-decisions
  *within* the committed design (`../90-open-decisions.md`).

## 7. What WS-V / WS-L must deliver here

The policy surface (lattice, classifications, clearances, declassification
edges, ingestion points); the **binding** of a policy to a build and its
**program-wide, non-weakenable** enforcement by the checker; the integration
with declassification (`62`) and the supply-chain attestation (`63`).
Acceptance: an implementation that relabels policy-`Secret` data as `Public`, or
writes above a policy clearance, or declassifies without the policy capability,
**fails to compile**; a conformant build records its governing policy in
provenance. Conformance: `../../conformance/security/policy/`.
