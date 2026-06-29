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

- **Updated:** 2026-06-29 (Steward first live pass: Bug 12 cleared, frontier
  released).
- **Next action:** **F4 runs first** (operator call, 2026-06-29). Its Steward
  frame is authored (`wp/F4-content-addr-design.md`) and is being **handed to the
  spec-leader for full elaboration** before Foundation is released; Foundation is
  **held** meanwhile (re-pinged off the premature pull). While F4 elaborates,
  **frame K1's brief** (then it too goes spec-leader → team). **K1 stays held**
  from any team until F4 reaches in-review. Run the watchdog + promotion-ladder
  pass each cadence.

- **WP release pipeline (operator, 2026-06-29):** **Steward (frame) →
  spec-leader (full elaboration) → build team (execute).** The Opus enclave
  front-loads all design/spec rigor; the open-weight build teams execute, not
  design. Steward authors the frame at `docs/program/wp/<ID>.md` on the WP branch
  (scope, deliverable outline, acceptance, settled-decision pinning); the
  spec-leader elaborates it + `/spec`/`/conformance` to team-ready rigor; it
  merges to `main` via the Integrator; **then** the team is kicked off. Thin
  catalog-recap kickoffs and unsequenced team design are out. Full rule: steward
  playbook §2c.

### 2026-06-29 session log (Steward)

- **Bug 12 cleared.** The convo `/response` post path was broken federation-wide
  (it 422'd because the body used the legacy `agent_id`; the backend requires
  `participant_id`). All posting — kickoffs, handoffs, retros, merge coord — was
  blocked since 2026-06-28. The fix is a one-field rename; verified live. Posting
  now works.
- **Post-API facts (the live backend, vs. docs):** message body uses
  `participant_id` (not `agent_id`); `mentions` must be **actor_ids**, not
  display names (name-mentions are silently dropped → no notification); and
  `message_type` must be in the **backend enum** — `kickoff`/`merge_ready`/
  `blocked`/`decision` from COORDINATION §8 are **not** accepted and 400. Working
  map: kickoff→`feature`, merge_ready→`git_request`, blocked→`status_update`,
  decision→the Decision object (not a message type). Recorded as **Bug 13**
  (`local/moot-bugs.md`); flagged to the fleet in the steward cadence thread.
- **Frontier released:** **K1**→kernel-leader (thread `evt_44k4934q2nfjz`),
  **F4**→foundation-leader (thread `evt_3f87m6kcqgkg3`). Both now `active`.

## Active frontier

| WP | State | Owner | Thread |
|---|---|---|---|
| **F4** content-addr / value-model design | **frame authored → spec-leader elaborating** (Foundation held) | Foundation (via Spec) | `evt_3f87m6kcqgkg3` |
| **K1** Π/Σ/inductive/checked-universes | **ready — held until F4 in-review** | Kernel | `evt_44k4934q2nfjz` |

Next to unlock: **K2/K2c** (kernel observational core — retire feasibility risk
early) once K1's API is stable; **K3** once F4 lands; **L5**
(effects/interaction-tree — the hub WS-Sec *and* WS-B both hang off) pulled
forward once K1's API is stable.

## Work-package status (vs. the DAG)

| WP | Status | Team | Feeds gate |
|---|---|---|---|
| F1 repo / MIT / workspace / IP hygiene | active (skeleton landed) | F | G0 |
| F2 spec + conformance corpus | **merged** (spec written) | Spec | G0 |
| F3 ADRs 0001–0008 | **merged** | F/Architect | G0 |
| F4 content-addressing + value-model design | **spec-leader elaborating** (frame done 2026-06-29; Foundation held) | F (via Spec) | G0 |
| K1 Π/Σ/inductive/universes | **ready — held** until F4 in-review (2026-06-29) | K | G1 |
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

- **Bug 13 (post-API drift, recorded `local/moot-bugs.md`).** COORDINATION §8's
  message-type taxonomy diverges from the live backend enum; mentions need
  actor_ids. Mitigated by the working map above and a fleet notice. *Underlying
  fix is the operator/maintainer's:* either extend the backend enum to accept
  the §8 types, or reconcile §8 to the backend. Not blocking (work proceeds on
  the mapped types) — but flag for a moot patch so the law and the substrate
  agree.
- *No work blockers active.* (Standing item: the **Ward** sibling project is not
  yet stood
  up; it is not a blocker until WS-B reaches B1–B3 — track its bring-up as a
  sibling, `05 §Ken-vs-Ward`.)
