# WP B4 — the agentic boundary (WS-B capstone)

**Owner (build):** Team Kernel. **Stream / gate:** WS-B → **completes the
behavioral seam**. **Depends on:** Sec1 (IFC), Sec2 (capabilities), B3 (trace) —
all merged; also B1 (export/`P`), B2 (Temporal/RV). **Spec source:**
`spec/70-behavioral/74-agentic.md` (DRAFT v0 → impl-ready), grounded against
`61`/`62` (IFC/authority), `71` (export + assumption `P`), `72` (Temporal), `73`
(conformance/trace), `36 §3` (metamorphic), `21 §5` (four-way status).

> **Steward frame** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails. The enclave elaborates `74` to team-ready rigor +
> conformance before Team Kernel builds. **No new mechanism:** this WP
> *demonstrates* that assuring agentic outputs reduces to already-built
> machinery; the "build" composes existing pieces, it does not add a
> kernel/elaborator rule. **Perishable:** the reduction rests on the *landed*
> Sec1/Sec2/B1/B2/B3 code — pin against it, not this line.

## 1. Objective (one line)

Complete WS-B: establish, in spec + conformance, that **assuring an embedded
agent's outputs reduces to the existing seam** — IFC (`61`) + capabilities
(`62`) + export-`P` (`71`) + Temporal/RV (`72`/`73`) + metamorphic (`36 §3`),
aimed at a **maximally-nondeterministic component** — so Ken adds **zero**
agentic mechanism.

## 2. Settled inputs — FIXED, do not reopen

Per `74` (`OQ-agentic-oracle` **DECIDED**, operator 2026-06-27; ADR 0006):

1. **NO new agentic mechanism.** Assuring agent outputs *is* the existing
   machinery. This WP adds no kernel rule, no new judgment, no "agent" elaborator
   surface. **If the build finds itself writing a new mechanism, it has
   mis-scoped — stop and flag the Steward.**
2. **An embedded agent = maximal `P`** — the strongest-adversarial assumption in
   the `71` export. Ken already proves `Q` *for all values* of a nondeterministic
   input; the agent is that input at its most adversarial. The reduction *is* the
   deliverable.
3. **Three mechanisms, all already-decided:**
   - **Safety envelope / verified shield** — capabilities (`62`) + IFC (`61`) +
     contracts: the agent holds only a *propose* capability, a verified validator
     holds the *act* capability, and the system's invariant is proved
     **independent of the agent**. Lands `proved`. The agentic reading of
     least-authority (`62`): the agent plays inside a Ken-proved FSM, choosing
     only among permitted transitions.
   - **Metamorphic relations** — oracle-free *relational* checks (round-trip,
     permutation-invariance, monotonicity), `OQ-relational` (`61 §5`/`§5a`,
     `36 §3`) + L2 test-gen. No ground truth needed.
   - **RV watchdogs** — the agent's observable actions are events in `Σ`;
     monitors synthesized from `T` (`72` + `73`).
4. **Safety, NEVER quality.** The boundary assures safety / structural validity /
   relational consistency — **never** that the output is *good*. Output quality
   is the **`unknown`** quadrant (`21 §5`), explicitly outside the assurance
   boundary (honesty-about-the-boundary). A "faithful summary" obligation is
   **not dischargeable**.

## 3. Deliverable outline (enclave elaborates `74` → team-ready)

Each item ends in a concrete conformance obligation, not a survey:

1. **The reduction, formally** (`74 §1`/`§2`) — agentic-output-assurance ≡ the
   `71` export with `P` = the agent's output domain (maximal nondeterminism). Pin
   the mapping: the *propose*/*act* capability split (`62`), and the
   invariant-proved-independent-of-`P` proof shape (`71`).
2. **Envelope demonstration** — a scenario where a system embeds an agent holding
   only `propose`; a verified validator holds `act`; a safety invariant is
   `proved` **for all** agent outputs. Drives the **real** Sec2 capability
   machinery + the B1 export `P`.
3. **Metamorphic demonstration** — at least one oracle-free relational check
   (round-trip / permutation-invariance) over an agent-like nondeterministic
   producer, via the existing `36 §3` metamorphic + L2 test-gen.
4. **RV demonstration** — agent actions as `Σ`-events; a `T` obligation monitored
   via **real** B2 Temporal + B3 trace.
5. **The honesty boundary, conformance-pinned** — a case asserting output
   *quality* maps to `unknown` (never `proved`/`Q`); the safety-not-quality line
   is testable.

## 4. Acceptance criteria (testable)

- **AC1** — the reduction is stated and a conformance case routes an agentic
  scenario through the **real** `71` export as maximal `P` (not a new mechanism).
- **AC2 (envelope)** — a safety invariant is `proved` for-all agent outputs,
  driving the **real** Sec2 capabilities (propose/act split); the **verdict flips
  to reject** if the agent is instead handed the `act` capability.
- **AC3 (metamorphic)** — an oracle-free relational check passes on a
  nondeterministic producer, via the real `36 §3` machinery.
- **AC4 (RV)** — an agent-action `T` obligation is monitored end-to-end via the
  **real** B2/B3 (trace → monitor); a violating action is caught.
- **AC5 (honesty, soundness)** — a *quality* obligation maps to `unknown`, never
  `proved`/`Q`, and is **not dischargeable** (safety ≠ quality).

## 5. Guardrails — do-not-reopen

- **NO new mechanism** — compose the existing Sec1/Sec2/B1/B2/B3 machinery. If
  you are adding a kernel/elaborator rule, you have mis-scoped — stop and flag.
- **★ Producer-grep, HIGH priority** — this is a *doc/conformance* WP, so the
  **hand-feeds-the-deliverable** trap is at its most dangerous: every AC must
  drive the **real** producers (Sec2 caps, B1 export, B2 Temporal, B3 trace, the
  `36 §3` metamorphic path), never a synthetic agentic literal that re-validates
  a pre-existing consumer. The QA gate greps the real producer src, not the test.
- **Safety ≠ quality** — a quality obligation must land `unknown`, never
  `proved`. The four-way status (`21 §5`) is the discriminator.
- **Agent = maximal `P`, never a trusted oracle** — untrusted producer, verified
  boundary; the reduction fails if the agent is trusted anywhere.

## 6. Process (standard §2c)

Enclave: **spec-author** elaborates `74` (`/spec` only); **conformance-validator**
authors `conformance/behavioral/agentic/` (`/conformance` only) — independence.
3-reviewer merge Decision (**Architect** soundness + **CV** Spec on `/spec` +
**spec-author** Fidelity on `/conformance`). Then **Team Kernel** builds — light:
it *composes* existing machinery to satisfy the conformance, adding little or no
new code. Its merge Decision is **Architect-only** if crates-only; if it touches
`conformance/`, add the Spec vote. **WS-B completes on merge.**
