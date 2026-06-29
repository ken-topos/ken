# WP L5 — Effects, capabilities, and state (the interaction-tree hub)

> **Status:** Steward frame — **awaiting spec-leader elaboration** (3rd in the
> fan-out serial chain, after K2c + V0). spec-leader elaborates `spec/30-surface/
> 36-effects.md` (DRAFT-v0: *proposal-level for syntax, normative for the model*)
> to implementation-ready rigor — `§6` defines the precise L5 deliverable. Then
> **Team Language** builds.
>
> **Team:** Language · **Deps:** K1 (done) · **Size:** M–L · **Risk:** ★★★ (the
> **hub**: WS-Sec and WS-B ride it) · **Pulled forward** ahead of L1–L4/L6 because
> it unblocks two whole workstreams.

## Why this is pulled forward

L5 is the **interaction-tree hub**. The DAG has **Sec1** (IFC-by-typing) →
**Sec1ct** (constant-time), **Sec2** (capabilities), and **B1** (behavioral export)
all riding the effect system. Getting the effect row, the capability model, and
the pure encoding right here unblocks the entire security tier and the behavioral
seam — so it leads the Language stream rather than following L1–L4.

## Objective

A **statically-checked, transitively-inferred effect discipline** with a
**pure-kernel encoding** — effectful surface programs denote to a **pure
interaction tree**, so the kernel never loses purity — plus capabilities and the
`space` state model.

## Fixed inputs — settled; do NOT reopen

- **`OQ-8` / `OQ-8a` DECIDED** (operator, 2026-06-27): effects are **static and
  transitively inferred** (a static effect *row*, `36 §1`), not dynamic.
- **One pure kernel** (`36 §2`): the **three-layer encoding** — surface effects →
  a **pure interaction-tree denotation** → the pure kernel. Effects are a surface/
  elaboration discipline; **the kernel stays pure** (no effect machinery enters
  the TCB).
- **`OQ-9` DECIDED:** handlers are **tail-resumptive only** (`36 §5`) — no
  multi-shot / full delimited continuations.
- **Capabilities = `requires`-as-capability** (`36 §3`); **state = the `space`
  model** (`36 §4`).

## Scope

**IN (per `36 §6` — the elaboration pins the exact cut):** the static effect row +
inference; the **interaction-tree encoding/denotation** (the pure-kernel bridge);
**capabilities** (`requires`-as-capability); the **`space`** state model;
**tail-resumptive handlers**; and the `pure`/`impure` boundary **hook** that L7
(FFI) will plug into.

**OUT — other teams' WPs:** the full **FFI** (`L7` — L5 provides only the
`pure`/`impure` wiring point); the security policies that *consume* L5 — **Sec1**
(IFC-by-typing), **Sec1ct** (constant-time), **Sec2** (capability enforcement) are
**WS-Sec** WPs that ride L5, not part of it; the **behavioral export** (`B1`);
non-tail-resumptive handlers (excluded by `OQ-9`); the rest of the surface
language (L1–L4/L6).

## Acceptance (testable — the elaboration's conformance seeds cover)

1. **Effects infer + check statically** — a function's effect row is inferred
   transitively from its body; an effect escaping its declared/inferred bound is a
   **static error**.
2. **Pure-kernel encoding holds** — an effectful program **denotes to a pure
   interaction tree**; the kernel checks the denotation with no effect machinery
   (the OQ-8 "one pure kernel" invariant, demonstrated end-to-end).
3. **Capabilities gate effectful ops** — an op requiring a capability is rejected
   without it, accepted with it (`requires`-as-capability).
4. **`space` state model** behaves per `36 §4`; **tail-resumptive handlers**
   resume correctly.
5. **`pure`/`impure` boundary** is exposed for L7 to wire FFI into.

## Guardrails

- **The kernel stays pure** — the effect system is surface + elaboration + the
  interaction-tree denotation; **nothing effect-specific enters the TCB.** This is
  the load-bearing invariant the whole "one pure kernel" design rests on.
- **This is the hub — build the effect row + capability model for its consumers.**
  Sec1/Sec1ct/Sec2/B1 will build on these exact abstractions; a leaky or
  underpowered effect row forces those WPs to work around it. Coordinate the
  effect-row / capability *interface* with the Architect early (it's a
  cross-workstream contract).
- Apply COORDINATION §7 (sharpened by K2): exercise the property and **invoke
  every guard** — including the effect-escape rejection and the capability-denial
  paths, with ≥2 distinct effects/capabilities, not a single-effect happy path.

## Logistics

Branch `wp/L5-effects` cut from `origin/main`. Team Language (`language-leader` +
`language-implementer` [Sonnet, medium effort] + `language-qa`). `scripts/ken-cargo
-p <crate>`. Ring: implementer builds → QA verifies independently → merge Decision
(**Architect** always + **Spec** on `/spec`+`/conformance`) → Integrator → retros.
The effect-row / capability *interface* is a cross-workstream contract → raise
design Qs to the **Architect** early so Sec/B aren't built on a moving target;
behavioral-contract Qs → Spec.
