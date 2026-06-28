# Implementation progress — the build backbone

**Owned by the Steward** (`agent/playbooks/federation/steward.md §2a`). This
file tracks execution **against the implementation DAG**
(`05-implementation-dag.md`), the build's analog of `spec/SPEC-PROGRESS.md`. It
**survives compaction**: on a cold start or after a compact, read this first,
then continue from the frontier (below). Update it **every synthesis pass and on
every WP state change**. The plan lives in `05`; this file tracks *progress
against it*. Run until complete, blocked, or instructed (§2b).

**Status legend:** `not-ready` (deps unmet) · `ready` (deps met, unassigned) ·
`active` (a team is building) · `in-review` (PR open / QA / CI) · `merged`
(landed + retro in). Gates: `met` / `in-progress` / `not-started`.

## Last updated / next action

- **Updated:** 2026-06-27 (seeded at design freeze; states reconcile against
  actual crate code on the Steward's first pass).
- **Next action:** decompose **Wave 1** into full WP specs in
  `03-program-of-work.md` and release the ready frontier — **F4, K1** — to the
  kernel team. Front-load the kernel observational core (K2/K2c); it is the
  critical-path, highest-risk work (§ guidance in `05`).

## Active frontier (ready now)

| WP | Why ready | Note |
|---|---|---|
| **F4** content-addr / value-model design | F1 done | feeds K3; design-only |
| **K1** Π/Σ/inductive/checked-universes | F2, F3 **done** | the critical path starts here |

Everything else is `not-ready` until its predecessors land. The two pivots to
watch (per `05`): **K2/K2c** (kernel observational core — retire feasibility
risk early) and **L5** (effects/interaction-tree — the hub WS-Sec *and* WS-B
both hang off; pull it forward once K1's API is stable).

## Work-package status (vs. the DAG)

| WP | Status | Team | Feeds gate |
|---|---|---|---|
| F1 repo / MIT / workspace / IP hygiene | active (skeleton landed) | F | G0 |
| F2 spec + conformance corpus | **merged** (spec written) | Spec | G0 |
| F3 ADRs 0001–0008 | **merged** | F/Architect | G0 |
| F4 content-addressing + value-model design | **ready** | F/Architect | G0 |
| K1 Π/Σ/inductive/universes | **ready** | K | G1 |
| K2 observational Eq/cast/Ω/quotient/truncation | not-ready (K1) | K | G1 |
| K2c conversion NbE + SCT | not-ready (K2) | K | G1 |
| K-api judgment + kernel API | not-ready (K2c) | K | G1 |
| K3 content-addressed value model | not-ready (F4) | K | G1 |
| X1 interpreter (strict CBV) | not-ready (K1,K3) | X | G1 |
| V0 minimal elaborator | not-ready (K1) | V | G1 |
| V1 spec syntax + four-way status | not-ready (V0) | V | G2–4 |
| V2 obligation generation | not-ready (K-api,V1) | V | G2–4 |
| V3 prover: Kripke + reflective cert | not-ready (V2) | V | G2–4 |
| V4 diagnostics | not-ready (V2,T1) | V | G2–4 |
| T1 diagnostic protocol | not-ready (V2) | V/T | G2–4 |
| L1 Int/Decimal/overflow | not-ready (K1) | L | G6 |
| L2 sum/match | not-ready (L1) | L | G6 |
| L3 strings/collections | not-ready (K1) | L | G6 |
| L4 modules/pkg | not-ready (K1) | L | G6 |
| L5 effects (interaction-tree) — **hub** | not-ready (K1) | L | G6 |
| L6 Bytes/IO | not-ready (K1) | L | G6 |
| L7 FFI | not-ready (L6) | L | G6 |
| L8 stdlib | not-ready (L1–L3,L-classes) | L | G6 |
| L-classes typeclass coherence | not-ready (K1,V0) | L | G6 |
| L-fmt formatter + TR39 lexer | not-ready (V0) | L/T | G6 |
| X2 runtime hardening | not-ready (K3) | X | G6 |
| Sec1 IFC by-typing | not-ready (L5) | Sec | G-Sec |
| Sec1ct @ct constant-time | not-ready (Sec1) | Sec | G-Sec |
| Sec2 capabilities | not-ready (L5) | Sec | G-Sec |
| Sec3 supply-chain re-check | not-ready (L4,K-api) | Sec | G-Sec |
| Sec4 trust-model + kernel audit | not-ready (K-api) | Sec | G5 |
| Sec5 policy-as-code | not-ready (Sec1,L4) | Sec | G-Sec |
| B1 export emitter | not-ready (V1,L5) | B | G-Ward-seam |
| B2 Temporal-as-data | not-ready (L2,B1) | B | G-Ward-seam |
| B3 trace/instrumentation contract | not-ready (B1,X1) | B | G-Ward-seam |
| B4 agentic boundary | not-ready (Sec1,Sec2,B3) | B | G-Ward-seam |
| X3 native backend | not-ready (X1,L-core) | X | G5-perf |
| X4 scale/limits | not-ready (X2,X3) | X | G5-perf |
| S1 subset compiler | not-ready (L-complete) | S | G8 |
| S2 full self-host | not-ready (S1) | S | G8 |
| T2 REPL | not-ready (V4,X1) | T | G5/G7 |
| T3 test framework | not-ready (L2) | T | G5/G7 |
| T4 pedagogy/docs | not-ready (G2) | T | G5/G7 |
| T5 ecosystem seeding | not-ready (L4,T3) | T | G5/G7 |

## Gate progress

| Gate | State | Note |
|---|---|---|
| **G0** clean-room foundations | **met (pending F4 sign-off)** | repo + spec + ADRs in place |
| **G1** vertical slice | not-started | K1→K-api + K3 + X1 + V0 |
| **G2+G3+G4** verification thesis | not-started | the differentiator |
| **G6** commercial reach | not-started | one verified component |
| **G-Sec** security tier-1 | not-started | IFC-by-typing, caps, re-check, policy |
| **G-Ward-seam** | not-started | export + trace contract a stub consumer reads |
| **G5-perf** native & scale | not-started | |
| **G5** soundness (incl. Sec4 audit) | not-started | |
| **G8** self-hosting | not-started | |
| **G7** agent loop | not-started | |

## Blockers / escalations

- *None active.* (Standing item: the **Ward** sibling project is not yet stood
  up; it is not a blocker until WS-B reaches B1–B3 — track its bring-up as a
  sibling, `05 §Ken-vs-Ward`.)
