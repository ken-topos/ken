# Current briefing (live — read this first on every Steward resume)

> Extracted verbatim from the former `STEWARD-DECISION-LOG.md` head
> during the 2026-07-21 diary reorganization. This file carries the
> **live** operator briefing and resume state — it is NOT history and is
> not filed under a date. Everything else that used to live in
> `STEWARD-DECISION-LOG.md` (the historical decision narrative) now
> lives in `docs/program/diary/`, under "## Steward decisions" on the
> relevant day — see [`INDEX.md`](INDEX.md).

---

## ☕ OPERATOR BRIEFING — staged 09:30Z, for your ~11:30Z return

**Read this first; everything below is the working record.**

### Shipped while you were out (both CLOSED — merged, verified by content, retros in)
- **SPAN-SEAL** — merged `cd4184b8`, **closed**, retros in. Seals the
  `BufferSpan` producer surface: public `write_all_advance_span` let checked
  source forge a span from unrelated span/count values, violating locked
  `spec/30-surface/38-ffi-io.md:356-365`. Cost **4 Decisions**.
- **RT-PARITY** — merged `e892777c` (PR #800), **closed**, retros in.
  Interpreter ↔ native checked buffer-IO narrowing parity + six executable
  differentials. Cost **5 Decisions**.
- **Adversary, two hunts, both high-value.** On SPAN-SEAL: **the seal HOLDS** —
  it attacked the property rather than the oracle; every forgery route rejects.
  On RT-PARITY: **one CONFIRMED violation of a LOCKED spec line** (below).

### ★ The one finding you'd want to hear about first

**`TransferCount.remaining` is computed from the RAW request length, not the
CAPPED EFFECTIVE one — on both interpreter and native.** A capacity-4 buffer
ends up completely full while the reified count tells checked source 4 bytes of
budget remain. Fail-closed (nothing over-reads), but **the value checked source
reasons with is wrong against locked `38 §:404-405`**. Confirmed by *execution*:
`adversary/R1-effective-request-repro @ 06bb9538` fails at `e892777c`.

**Why RT-PARITY's own tests could never have caught it:** interp and native
*agree*. Six differentials over a shared wrong basis all pass green. **A
differential is a relative oracle — it establishes agreement, never
correctness.** Filed as **BUDGET-EFF**, now **ahead of SEAL-2**. Two things
that change its shape: it is a **plumbing gap** (the effective value is
discarded at validation and reaches neither reifier — two closures with very
different blast radii, an Architect call), and the **spec erratum must come
first** because `38` contradicts itself and code-first guarantees re-derivation.

### The one thing I'd want you to know
**Seven of those nine Decisions were spent on defects I introduced**, all one
shape: I handed over a **mechanism or a grounding** and it silently inherited
whatever that thing could not do. AC-3 named a Pi-codomain walk (missed
aliases), then named `GlobalEnv::lookup` (missed constructors), then the
RT-PARITY brief grounded a claim on a closure the adversary had **already
invalidated 5 minutes before I corrected the lesson but not the instruction**.
The rings' execution was consistently strong; the rework was mine. Four new
memories written. If you read one, make it
`correcting-a-claim-requires-sweeping-instructions-built-on-it`.

### Needs you (nothing is blocked on it right now)
1. **Provider exposure.** Only `runtime-implementer` and `adversary` are on
   Anthropic; **every other seat is on the tighter OpenAI pool.** You mentioned
   possibly moving the fleet to Anthropic — this is the argument for it.
2. **`MODELS.md` is wrong about the fleet.** A full sweep shows
   implementer + enclave = **T1**, leader + QA = **T2**, uniformly across all
   six build teams. The table reads as if build roles are uniformly T2. Not
   Runtime-specific, as I previously (wrongly) told you.
3. **Title-only issues #38/#32/#24/#25** — untouched per your instruction,
   awaiting reconstruction *with* you.
4. **`RT-ESCAPE`** — filed, unsized. Escaping a 2nd `Resource` through a bracket
   fails native lowering. Pre-existing; surfaced by RT-PARITY, correctly not
   fixed there. Now also carries the adversary's untested **R2**.
5. **Convo posting outage is recurring and it cost us twice tonight.**
   `runtime-implementer` lost `post_response`/`share`/`reply_to` **while
   convo-channel stayed up** — alive, receiving mentions, unable to answer. It
   wrote its retro to disk and I posted it verbatim on its behalf. Worth knowing
   because **an absent post is not evidence of a stalled agent**, and I mis-read
   that seat's state twice today before working this out.

### Queued and ready, in order
**SEAL-2** (drafted, Foundation — closes the producer *enumeration*: every
namespace, every result position, every source root, derived from the
elaborator's own structure so a new namespace is a build break) · F1 (#37) →
Architect · STR-BIJ → enclave · A3 · F4 (now also carries adversary S3) · F3 ·
RT-SPLIT.

---

# ▶ ACTIVE WINDOW — 2026-07-21 (operator away ~03:30 → ~11:30 UTC)

Mandate (operator, 2026-07-21): *"Keep going, route around problems if you
can't decide from PRINCIPLES.md. Keep a decision log. If you encounter another
refusal, route to the opus agent (regardless of subject area) or to a gpt-5.6
agent (if opus refuses)."*

**Standing inputs for this window (do NOT reopen):**
- **Fleet is SINGLE-THREADED** (operator, 2026-07-20) — one WP at a time; idle
  rings are correct, not stalls. Rationale: enclave context coherence.
- **Runtime seating:** `runtime-implementer` is **T1** (reseated on Anthropic
  Opus 2026-07-21 to route around a provider content refusal);
  `runtime-leader` is T2. Inverted vs the `MODELS.md` table **by design** —
  implementation is the hard part on that team. **Do not "revert" it.**
- **A single Runtime WP can legitimately run 16 HOURS.** Silence on that ring
  is not a stall.
- **Refusal routing (new this window):** on a provider content refusal, route
  the task to an **Opus** seat *regardless of subject area*; if Opus also
  refuses, route to a **gpt-5.6** seat. Protect uncommitted/untracked work
  first, never re-send the refused content verbatim.
- **STOP at the title-only issues** (#38/#32/#24/#25) — reconstruct WITH the
  operator, never invent a scope.
- Merge autonomy per the standing gate model (QA + Architect §14 + CV where
  the diff-scope pulls it + CI green). No separate operator-merge gate.

## Queue of record for the window
1. **RT-PARITY** — active; remediation in flight, 6/6 differential cases flip.
   ⚠ `dec_7p15cgqz3x0sg` OPEN, bound to **stale** `a692134e`; must be
   re-anchored, and the Architect's earlier APPROVE does **not** carry forward.
   On merge: **notify the adversary** (`agt_37vnwmcdxhw00`) — mandatory on
   every CODE merge.
2. **F1 (#37)** → Architect (soundness-relevant per research
   `evt_55tnpnhvcpptf`), released when the RT-PARITY Decision resolves.
3. **Publish** the two held doc-only branches (`wp/STR-BIJ-frame` `f030cedd`,
   `wp/MODELS-TIER-erratum` `55576c05`) — held only to avoid rebasing a
   fragile ring mid-flight.
4. **STR-BIJ** → enclave (Handoff Gate first). 5. **A3** coverage walker
   (before F4 — F4 edits the catalog A3 covers). 6. **PX8-F-PROOF (F4)** —
   risky, may hard-stop to Spec. 7. **F3 (#39)** — must precede RT-SPLIT.
   8. **RT-SPLIT**. 9. **PX8-F-CAP (#41)**.

## Operator answers at sign-off (2026-07-21 ~03:35 UTC)

1. **Credits are NOT a constraint this window.** Anthropic **46%** remaining
   with **<2 days to reset**; OpenAI **13%**. Operator: *"You should have enough
   until I get back."* Operator will *probably* move the fleet **entirely to
   Anthropic** tomorrow. ⇒ **No credit contingency needed; do not ration.**
   **★ Note the asymmetry:** OpenAI at 13% is the *tighter* pool, and most seats
   (runtime-leader, architect, CV, research, spec-*) run on it, while Anthropic
   at 46% is the looser one. **If an OpenAI seat exhausts mid-window, reseat it
   to Anthropic** — that is where the credits are, and it is the direction the
   operator already intends to move the whole fleet. Log any such reseat.
2. **A MODEL REFUSAL OVERRIDES TOPOLOGY** — operator, explicit: *"a model
   refusal overrides topology, at least when I'm not present."* ⇒ On a provider
   content refusal I may route the refused task to **any** seat that can execute
   it, **regardless of subject area or role**, Opus first, gpt-5.6 second. The
   resulting work still returns through the **normal gates** (QA, Architect §14,
   CV where the diff-scope pulls it, CI). **Scope note: this is an
   operator-absent latitude** — the standing §9 topology invariant reasserts
   when the operator is present.
3. **ADR amendments: reason from `docs/PRINCIPLES.md`. If it does not settle
   it, HOLD.** Operator: *"Use PRINCIPLES.md. It's often enough. If not, hold."*
   ⇒ Not a blanket authorization and not a blanket veto — a *method*
   instruction. See the ADR-0010 ruling below.

Operator will *probably* check in in a few hours.

## ▶▶ RESUME HERE AFTER COMPACTION (updated 10:28Z — RT-PARITY MERGED; BUDGET-EFF is next)

> ### ⏱ STATE: `origin/main = e892777c`. RT-PARITY **MERGED** (PR #800),
> verified by content. Adversary notified. **Retros: QA ✅ + leader ✅ in;
> implementer's still pending — the WP CLOSES when it lands.**
>
> ### ★★ QUEUE REORDERED — `BUDGET-EFF` NOW OUTRANKS SEAL-2
>
> The adversary's RT-PARITY hunt (`evt_1s9rt48z7bpsn`) found **R1: a CONFIRMED
> violation of a LOCKED spec line.** `TransferCount.remaining` derives from the
> **raw** request length, not the **capped effective** one, on BOTH interp and
> native. Buffer full at capacity 4 while the reified count says 4 bytes of
> budget remain. Fail-closed (not memory-unsafe, not a forgery) but **the value
> checked source reasons with is wrong against locked `38 §:404-405`/`:443-444`**.
> Full draft: `docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`.
>
> **Why it outranks SEAL-2:** SEAL-2 closes a gate with **no live defect**;
> BUDGET-EFF is a **live contradiction of locked normative text**.
>
> **★ Why RT-PARITY could not have caught it — carry this:** interp and native
> **agree**. Six differentials over a shared wrong basis all pass green. **A
> differential is a RELATIVE oracle — it establishes agreement, never
> correctness.** Where the spec is the authority, the oracle must assert against
> the **normative text**. Memory: `differential-oracle-is-blind-to-a-shared-premise`.
>
> **★★ IT IS A PLUMBING GAP, NOT A FORMULA FIX — do not size it as a two-line
> swap.** `TransferCountV1::new(read, effective)` **validates against
> `effective` then DISCARDS it**; the reply carries `span.length == count` and
> the request holds the raw length, so **neither reifier can compute the bound
> from what it is given.** Two closures with different blast radii — (a) reply
> carries the effective request (**wire/ABI**, ends host+runtime being
> byte-unchanged), or (b) host caps the request record before reification (no
> wire change). **ARCHITECT CALL, routed WITH the enclave ruling** — they
> interact. Confirmed by EXECUTION: oracle at
> `adversary/R1-effective-request-repro @ 06bb9538` (local, unpushed), fails at
> `e892777c` with all three premise assertions passing. It is **AC-3, pinned
> unchanged** — the ring makes it pass, never edits it to fit.
>
> **⛔ SEQUENCING IS FORCED: the SPEC ERRATUM COMES FIRST.** `38` contradicts
> itself — `:404-405`/`:443-444` say *effective*, `:419-420`/`:438-440` say
> *requested*. **An implementer reading only the table writes exactly what is
> landed**, so fixing code first guarantees re-derivation. Spec enclave rules
> which is normative → then the implementation, with a **spec-absolute** oracle.
>
> ### NEXT ACTIONS, in order
> 1. **Collect the implementer's §10 retro → CLOSE RT-PARITY** in the tracker.
> 2. **Doc-only publish** to sync tracker + log + both WP drafts to `main`.
> 3. **BUDGET-EFF:** route the spec question to the **Spec enclave** (idle;
>    Handoff Gate them first), then frame the implementation half.
> 4. **THEN SEAL-2** — draft complete at
>    `docs/program/wp/SEAL-2-carrier-producer-enumeration-closure.md`, evidence
>    `adversary/SEAL2-repros@70a603da`. Re-anchor on current `origin/main`.
> 5. Then: F1 (#37) → Architect · STR-BIJ → enclave · A3 · F4
>    (carries adversary S3) · F3 · RT-SPLIT. **STOP at
>    #38/#32/#24/#25.**
>
> **`RT-ESCAPE`** — filed, unsized; **adversary's R2 attached to it** (untested:
> `freeze` takes buffer + span independently and `PrivateBufferSpan` carries no
> buffer identity, so a span minted against buffer A is well-typed against B —
> a *call-site indexing* question, orthogonal to the constructibility exemption).
>
> **Operator briefing is staged at the top of this file** for the ~11:30Z return
> — **update it with BUDGET-EFF before they arrive.**

