# WP T1 — The machine-readable diagnostic protocol (the agent contract)

> **Status:** Steward frame — **next enclave WP** after the spine (deps met: V4 +
> V2 on `main`). spec-leader elaborates `spec/20-verification/25-protocol.md`
> (DRAFT → implementation-ready), then **Team Verify** builds it (after V4-build).
>
> **Team:** Verify · **Deps:** **V4** (`2d7a09c`, the diagnostics it serializes) +
> **V3** (`c43cdfb`, the verdicts) + **V2** · **Size:** M · **Risk:** ★★
> (**untrusted** — a serialization/UX contract; a bug is a malformed or unstable
> message, never unsoundness) · ► **Completes the agentic differentiator** — the
> verification spine's results become machine-consumable; the contract for the
> write→spec→verify→repair loop (strategy **G7**).

## Objective

Elaborate `25-protocol` (T1) — the **machine-readable diagnostic protocol**: every
**verdict** (`23`/V3: `proved`/`disproved`/`unknown`) and every **diagnostic**
(`24`/V4: Kripke countermodel, typed hole, three-region decomposition, slice
context) is emitted as **stable, schema-valid JSON** an **agent consumes without
scraping human text**. This is what makes Ken's differentiator *real for agents*:
the toolchain's output is a structured contract the write→verify→repair loop codes
against, not prose to parse.

## The framing that sets the risk level

T1 is **untrusted** — ★★. It serializes decisions already made (the kernel decided
`proved` via the cert; V4 derived the diagnostic); a T1 bug is a **malformed,
lossy, or unstable message**, an agentic-UX/contract regression, never
unsoundness. Build it **faithful + stable**. The load-bearing properties:
- **Stability** — agents code against this shape; it must declare **what is
  stable vs. versioned**, so a toolchain update doesn't silently break an agent's
  parser. The stability guarantees are the contract.
- **Fidelity** — the JSON reflects the **actual** verdict/diagnostic, never a
  lossy or relabeled one; the **`false` vs `unknown`** distinction (V4's
  load-bearing discriminator) **survives serialization** (an `unknown` must never
  serialize as `false`/refuted).

## Scope

**IN:** the **message shape** for each verdict + each of V4's four diagnostic
mechanisms (countermodel / typed hole + `unknown` propagation / three-region /
slice context) as structured JSON; the **stability guarantees** (stable fields
vs. versioned/experimental); the **agent-loop contract** (how the
write→spec→verify→repair loop `G7` reads a result and locates the actionable
signal — fix-the-spec vs supply-facts, from the `false`/`unknown` split); the
**reference JSON schema** (per `25`, finalized with the agent-team software —
specify the shape + stability, leave exact field bikeshedding to the schema).

**OUT — other WPs:** the diagnostics themselves (`24`/V4); the prover (`23`/V3);
the **Ward behavioral export** (`B1`, WS-B — a *different* sibling-export seam,
not this agent protocol); any change to verdict/diagnostic *content* (T1
serializes, it does not re-decide).

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `25` to builder rigor: the per-verdict and per-diagnostic message shape;
the **stability contract** stated explicitly (stable vs versioned); the agent-loop
consumption pattern. **Ground against the *landed* V4 (`24`, the diagnostic
values) + V3 (`23`, the verdicts) — the files, not status; serialize what they
actually emit.** Conformance (`conformance/verify/protocol/`): every verdict +
diagnostic **round-trips** to schema-valid JSON and back **without loss**
(discriminating — a `false` and an `unknown` serialize to **distinct, non-
confusable** messages, the V4 fidelity property carried to the wire); the
stability guarantee holds (a stable field's absence/rename is a conformance
failure); the agent can locate the actionable signal from the message alone.

## Acceptance (testable)

1. **Faithful + lossless:** each V3 verdict + V4 diagnostic serializes to
   schema-valid JSON and round-trips without loss; the **`false` vs `unknown`**
   distinction is preserved on the wire (never confusable).
2. **Stable contract:** the message declares stable vs. versioned fields; a
   conformance case pins the stable surface (a rename/drop of a stable field
   fails).
3. **Agent-consumable:** from a serialized result alone, an agent can read the
   verdict + the actionable signal (fix-spec vs supply-facts) **without** parsing
   human text.
4. **No regression:** a fully-`proved` result serializes to an empty/clean
   diagnostic set; V4/V3 behavior is unaffected.

## Sequencing

Next enclave WP after the spine (deps V4 + V3 + V2 landed). **Completes the
agentic differentiator** — the verification results become a stable machine
contract (the G7 loop). Build follows V4-build on Team Verify. Opens the door to
the agent-team software + differential consumers. Build queries: protocol
semantics → Spec; the agent-facing shape → Architect. Clean-room: landed
`23`/`24`/`25` + first principles; no copyleft. **(Post-spine direction: this is
the verification-completion path; broadening to WS-Sec/WS-B/language remains the
Steward's next checkpoint with the operator.)**
