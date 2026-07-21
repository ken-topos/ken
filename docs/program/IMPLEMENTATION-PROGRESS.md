# Implementation progress — the build backbone

**Owned by the Steward** (`agent/playbooks/federation/steward.md §2a`). This
file tracks execution **against the implementation DAG**
(`05-implementation-dag.md`), the build's analog of `spec/SPEC-PROGRESS.md`.
It **survives compaction**: on a cold start or after a compact, read this
first, then continue from the frontier (below). Update it **every synthesis
pass and on every WP state change**. The plan lives in `05`; this file
tracks *progress against it*. Run until complete, blocked, or instructed
(§2b).

**This file holds CURRENT STATE ONLY.** The full chronicle — every prior
"live state" snapshot, the detailed evidence trail for every merged WP, and
the day-by-day session logs back to project start — lives in
[`progress-archive/`](progress-archive/README.md). If you need *why* a past
call was made, or the mechanism detail behind a closed WP, start there.

**Status legend:** `not-ready` (deps unmet) · `ready` (deps met, unassigned)
· `active` (a team is building) · `in-review` (PR open / QA / CI) · `merged`
(landed + retro in). Gates: `met` / `in-progress` / `not-started`.

## Last updated / next action

**2026-07-21 10:50Z.** RT-PARITY is **CLOSED** — merged `e892777c` (PR
#800), verified by content, adversary notified, all three §10 retros in.
Cost 5 Decisions across the terminal-gate cycle; root cause of the extra
cost was the Steward's own stale grounding instruction, corrected in-flight.

One posting-outage incident this cycle, filed as a fleet lesson: the
`runtime-implementer` seat's `post_response`/`share`/`reply_to` tools went
missing while `convo-channel` itself stayed up. The seat could not post its
§10 retro, said so plainly, and refused to let the WP close on an assumed
record — the Steward posted the retro on its behalf, attributed. Lesson: a
seat can be alive and responsive yet unable to post; an absent post is not
by itself evidence of a stalled or ignoring agent.

**Next action: BUDGET-EFF.** `docs/program/wp/BUDGET-EFF-remaining-bounded-
by-effective-request.md` — an adversary-confirmed (R1) violation of
**locked** `spec/30-surface/38-ffi-io.md` (`TransferCount.remaining` must be
bounded by the *effective* request; the host clamps instead of rejecting,
and validates against the wrong bound). Confirmed by execution
(`adversary/R1-effective-request-repro @ 06bb9538`, fails at `e892777c`).
Outranks SEAL-2 — SEAL-2 closes a gate with no live defect, this is a live
contradiction of locked normative text. **This is a plumbing gap, not a
formula fix**: `effective` is discarded at validation and reaches neither
reifier, so two closures see different blast radii. **Spec erratum first**
— `38` contradicts itself as currently worded, so a code-first fix would
re-derive from a broken citation. Architect call routes together with the
enclave's erratum ruling.

**Two branches held, ready to publish** (were deliberately not pushed while
RT-PARITY was in flight and single-threaded; nothing blocks publishing them
now that RT-PARITY has landed):

| Branch | Head | Scope |
|---|---|---|
| `wp/STR-BIJ-frame` | `e135fa32` | STR-BIJ frame — A1+A2 String/List-Char "bijection" over-claim erratum (doc-only) |
| `wp/MODELS-TIER-erratum` | `55576c05` | `agent/MODELS.md` — Roles→tier column is a DEFAULT, not uniform; documents Runtime's actual (inverted) seating (doc-only, +32/−0) |

## Work-package status

Only WPs currently in the queue or closed this cycle are tracked here in
detail; the full historical catalog (NC1–27, CAT1–5, LET1–6, KTR1–2, AX1–2,
SUB1–2, PX0–PX16, SPAN-SEAL, RT-PARITY, …) is closed-and-merged and lives in
the archive.

| WP | Owning team | Status | Feeds |
|---|---|---|---|
| RT-PARITY | Runtime | **merged** (`e892777c`, PR #800; retros in) | correctness erratum (interp/native parity), no gate |
| SPAN-SEAL | Foundation | **merged** (`cd4184b8`; retros in) | `BufferSpan` producer-closure soundness, no gate |
| BUDGET-EFF | *TBD — not yet assigned* | **not-ready** (DRAFT; spec erratum must land first) | correctness fix against locked `38`, no gate |
| SEAL-2 | Foundation | **not-ready** (DRAFT, not yet framed to a branch; blocked behind BUDGET-EFF/RT-PARITY-quiescence) | closes a derived-enumeration gap in the SPAN-SEAL oracle, no gate |
| F1 (#37) | Runtime | **ready** (re-classified soundness-relevant; routes to Architect now that RT-PARITY has resolved) | K3 trusted-base promotion (indirect, via Phase-2 tranche) |
| STR-BIJ | Spec enclave | **ready** (frame committed on `wp/STR-BIJ-frame @ e135fa32`, not yet kicked) | honesty erratum on landed prose + `conformance/`, no gate (pulls a CV vote) |
| A3 (catalog-coverage walker) | *TBD* | **not-ready** (queued ahead of F4; F4 edits the catalog A3 covers) | tooling, no gate |
| F4 / PX8-F-PROOF | Foundation + Spec enclave | **not-ready** (blocked behind A3) | K3 (value model), X2 (runtime hardening) |
| F3 (#39) / F2/F3 reducer | Runtime | **not-ready** (must precede RT-SPLIT) | interpreter reducer correctness, no gate |
| RT-SPLIT | Runtime | **not-ready** (frame authored, Phase-0 decomposition ruling delivered; waits on the Runtime ring freeing up) | maintainability only, no gate |
| PX8-F-CAP (#41) | *TBD* | **not-ready** (backlog; deferred to spec-first) | — |
| RT-ESCAPE | Runtime | **filed, unsized** (pre-existing native-lowering defect surfaced by RT-PARITY; needs Architect input on which layer owns it before sizing) | correctness, no gate |
| MODELS-TIER erratum | Steward (doc) | **ready to publish** (`wp/MODELS-TIER-erratum @ 55576c05`, held) | fleet playbook accuracy, no gate |

Title-only backlog items not yet reconstructed into frames: **#38, #32,
#24, #25** — do not treat as sized or sequenced; reconstruct with the
operator before queuing.

## Active frontier — releasable now

Single-threaded fleet (operator-directed); this is a strict queue, not
parallel lanes:

1. **BUDGET-EFF** — spec erratum first (enclave), then implementation.
   Owning team TBD pending that erratum.
2. **SEAL-2** — Foundation, once framed to a branch.
3. **F1 (#37)** — ready for Architect review now (RT-PARITY, its release
   gate, has resolved).
4. **STR-BIJ** — ready to kick to the Spec enclave (Handoff Gate first).
5. **A3** (catalog-coverage walker) — before F4.
6. **F4 (PX8-F-PROOF)** then **F3 (#39)** — F3 must precede RT-SPLIT.
7. **RT-SPLIT** — decompose `cranelift_backend.rs` (22,081 lines), once the
   Runtime ring is free.
8. **PX8-F-CAP (#41)** — backlog.

`RT-ESCAPE` carries adversary finding R2 and needs an Architect layer
decision before it can be sized into this queue.

## Current blockers / escalations

- **BUDGET-EFF spec self-contradiction** — `spec/30-surface/38-ffi-io.md`
  needs an erratum before the code fix can be written (a code-first fix
  would re-derive from the currently-contradictory text). Not yet escalated
  beyond the Spec enclave; next action is routing the erratum.
- **RT-ESCAPE layer ownership** — undecided which layer (native lowering vs.
  a shared runtime frame-marker discipline) owns the defect. Needs Architect
  input before sizing; not currently blocking anything else in the queue.
- **BUDGET-EFF / A3 / PX8-F-CAP owning teams** — not yet assigned; flagged
  above as `TBD` rather than guessed. Assign at kickoff time.
- No standing infrastructure blockers: the provider content-refusal
  incident and the MODELS.md tier-table gap (both hit during RT-PARITY) are
  resolved/queued, not open blockers.

## Gate progress

The last granular G0–G8 / G-Sec / G-Ward-seam table was maintained
2026-07-14 and is now in the archive
(`progress-archive/2026-06-29-and-07-05-07-06-log.md`) — treat it as
historical, not current; it predates three weeks of landed work and should
not be read as live. In its place, here is what the current queue implies:

- The **PX-series native capability/backend build** (PX0 through PX16, plus
  the PX8 sub-series culminating in PX8-H/PX8-I/PX8-J/PX8-L) is **complete
  and merged** — this was the bulk of the G6/G7-era native-lowering work.
- The **current queue is post-PX8 hardening and correctness work**
  (BUDGET-EFF, SEAL-2, F1, F3, RT-SPLIT) plus one honesty erratum
  (STR-BIJ) and one soundness-adjacent item (F1) — most of these WPs
  explicitly feed **no G-gate** (see the table above); they are adversary-
  driven correctness/maintainability fixes on already-landed surface.
- **F1**, once ruled, is the dependency root for the ratified Phase-2
  tranche (Decimal/Char demotes, `Float.toDecimal` exactness) and is a
  named precondition for the eventual **K3** trusted-base promotion.
- **G-Sec** and **G-Ward-seam** (per `05-implementation-dag.md §Gates`) have
  no live WP against them right now; nothing in the current queue targets
  either directly.
- Full gate definitions: `docs/program/05-implementation-dag.md`
  (`## Gates (extending G0–G8)`).

## Archive

The complete build chronicle — every prior live-state snapshot, the full
evidence trail behind every merged WP back to project start (2026-06-29) —
is in [`progress-archive/`](progress-archive/README.md), split into 11
dated segments with an index. Nothing from the prior 2.23 MB tracker was
dropped in the 2026-07-21 restructuring that produced this slim file,
except the ~13 repeated "prior live state, kept for history" scaffolding
headers (pure boilerplate, collapsed) and a handful of headers that had
grown to 10,000–56,000 characters across dozens of edits (mechanically
reflowed into readable prose/bullets at their existing sentence boundaries,
not truncated — see `progress-archive/2026-07-19.md`'s editorial note).
