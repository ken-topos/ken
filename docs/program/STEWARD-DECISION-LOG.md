# Steward decision log

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
> 5. Then: F1 (#37) → Architect · STR-BIJ → enclave · A3 · F4 (carries adversary
>    S3) · F3 · RT-SPLIT. **STOP at #38/#32/#24/#25.**
>
> **`RT-ESCAPE`** — filed, unsized; **adversary's R2 attached to it** (untested:
> `freeze` takes buffer + span independently and `PrivateBufferSpan` carries no
> buffer identity, so a span minted against buffer A is well-typed against B —
> a *call-site indexing* question, orthogonal to the constructibility exemption).
>
> **Operator briefing is staged at the top of this file** for the ~11:30Z return
> — **update it with BUDGET-EFF before they arrive.**

## ▶▶ (superseded) RESUME BLOCK — 09:32Z, RT-PARITY PUBLISHING

> ### ⏱ IMMEDIATE — THE MERGE IS IN FLIGHT
>
> **PR #800, `wp/RT-PARITY-interp-native @ 9f32967d4ddb15713ecf9d2fda3f4be070f6ae8d`**,
> non-doc-only. Publisher launched 09:14Z as bg **pid 2369541**; log at
> `scratchpad/rt-parity-publish.log`. It sleeps **2144s (until ~09:50Z)**, then
> polls checks, then merges. Decision `dec_3xyep8021ke88` verified
> resolved/APPROVED at that exact SHA (QA + Architect §14 + CV). Published
> **byte-identical** — §2a tracker bundle deliberately omitted.
>
> **`gh` is NOT authed in my shell** (only the publisher mints credentials), so
> PR state cannot be queried directly — judge from the log + `origin/main`.
>
> **ON MERGE, in order — the watchdog carries the full version:**
> 1. **Verify BY CONTENT** on `origin/main`, never the "merged" line: the four
>    paths (`crates/ken-interp/src/eval.rs`,
>    `crates/ken-cli/tests/rt_parity_native.rs`,
>    `crates/ken-interp/tests/px8p_checked_buffer.rs`,
>    `conformance/behavioral/buffer-io/seed-buffer-io.md`);
>    `crates/ken-runtime/` + `crates/ken-host/` **zero-line diff**; renamed test
>    `..._unconstructible_at_the_landed_surface` present and the old
>    `..._on_landed_producer_closure` absent everywhere.
> 2. **NOTIFY ADVERSARY `agt_37vnwmcdxhw00`** — mandatory on every code merge.
> 3. Request §10 retros (carry the three items from `evt_3p9a0977psc4y`).
> 4. Close the WP. 5. Sync tracker via separate doc-only publish.
> 6. **THEN SEAL-2** — draft at
>    `docs/program/wp/SEAL-2-carrier-producer-enumeration-closure.md`, evidence
>    `adversary/SEAL2-repros@70a603da` (visible from every worktree, no push
>    needed). Frame to `wp/SEAL-2-frame` **re-anchored on the NEW `origin/main`**,
>    publish doc-only, Handoff Gate Foundation, kick.
>
> **An operator briefing is staged at the top of this file** for the ~11:30Z
> return. Keep it current.

**Resumed cleanly at 07:12Z.** The planned compaction cost nothing: branch was
`steward/work`, tree clean, and the durable record on `origin/main` +
`steward/work` carried everything. **Obligation 1 is now DISCHARGED.**

**State.** Fleet single-threaded; operator away until ~11:30Z. `origin/main =
7e87d173`. **SPAN-SEAL ✅ CLOSED** — merged `cd4184b8` (PR #798), verified by
content, adversary notified, **all three §10 retros in** (leader 07:06:33, QA
07:06:39, implementer 07:06:40). Foundation ring idle-by-design.
**RT-PARITY ACTIVE** — runtime-implementer confirmed working (`Befuddling…`,
mid-rewrite of the closure citation onto the landed sealed closure). Every other
ring idle is CORRECT.

**★ Liveness-check correction, 07:12Z — worth more than the outcome.** My first
capture of `runtime-implementer` used `tail -6` and showed an **empty prompt, no
paste, no spinner** — the "never delivered" row of my own gate table. I was one
step from re-delivering a kickoff into a seat that was six minutes deep in the
work. A `-S -60` capture showed `Befuddling… (5m 52s)` rendering **above** the
input line. **The Anthropic spinner is not in the last six lines. Capture WIDE
(`-S -60`), always** — this is the same trap already recorded for the
`Compacting…` bar, and it generalises: my liveness detectors keep matching a
convenient proxy (the bottom of the pane) instead of deciding the property (is
this seat working). Narrow capture is not a cheaper check; it is a wrong one.

**★ ADVERSARY VERDICT ON SPAN-SEAL (07:26Z, `evt_74mjc4txd9y1e`): SEAL HOLDS.**
Deep wrapped-inclusive sweep `MISSED BY LANDED ORACLE: []`; every forgery route
`UnresolvedCon`; the transparency exploit answered NO. **RT-PARITY not
undermined.** Three findings, none merge-blocking. **Rulings I made (07:30Z):**
S2 → RT-PARITY as a **wording** requirement only, not a block (premise is TRUE,
just ungated; a third block would be disproportionate, and the gate is
Foundation's not Runtime's). S1+S2 → **SEAL-2**, drafted, deliberately NOT framed
to a branch until RT-PARITY closes (stale-frame §14 hazard). S3 → **PX8-F-PROOF
(F4)** scope. Adversary asked to preserve repros on `adversary/SEAL2-repros`.

**★★ I OVERCLAIMED, AND CORRECTED IT AT THE SAME VOLUME (`evt_6f8q947c24esp`).**
When I closed SPAN-SEAL I told the fleet the loud-fail axis was *"closed
independently of the author's imagination."* **It is not.** The panic fires only
for an id **already in `env.globals`** that classifies as neither declaration nor
constructor. It closed the **classification**; the **enumeration** stayed
partial — a `Result E BufferSpan` producer and a class field both pass silently.
**A loud-fail guard is only as closed as the domain it iterates.** Three drafts
of AC-3, three mechanisms named, and the adversary found instances #4 and #5.
The corrected lesson is *stronger* than the one I posted, which is why saying so
publicly cost nothing and buys the next reader the real rule. **When a lesson I
broadcast turns out to be partly wrong, the correction goes to the same audience
at the same prominence — a quiet fix leaves the overclaim standing in everyone
else's notes.**

**MY OPEN OBLIGATIONS, in order:**
1. ~~Collect Foundation's §10 retros → CLOSE SPAN-SEAL~~ ✅ **DONE 07:15Z.**
2. **RT-PARITY respin:** needs a **FRESH Decision** with **FRESH** Architect +
   CV votes — no prior vote carries (this has bitten this WP twice, the fleet
   four times). On merge: verify by content → **NOTIFY ADVERSARY
   `agt_37vnwmcdxhw00`** → §10 retros → close.
3. **Then the queue:** F1 (#37) → Architect · Handoff Gate + kick STR-BIJ to the
   enclave · A3 · F4 · F3 · RT-SPLIT. **STOP at the title-only issues**
   (#38/#32/#24/#25) — reconstruct WITH the operator.
4. **Low priority:** `MODELS.md` follow-up — the full-fleet sweep showed
   implementer=T1 / leader=QA=T2 is the **fleet-wide convention**, not a Runtime
   "inversion"; PR #793's example understates it. Also flag to the operator that
   **only `runtime-implementer` and `adversary` are on Anthropic** — every other
   seat is on the tighter OpenAI pool (13%).
5. **Unsized, filed:** `RT-ESCAPE`.

**Discipline earned this window — all of it still binding:**
- **A success signal tells you a thing RAN, never that it did what you meant.**
  Five tools misled me: publisher exit code, gate-script exit 0, `git commit` on
  the wrong branch, a task-notification for a `nohup` wrapper, and the
  publisher's own "merged" line. **Verify by CONTENT, always.**
- **Branch discipline:** end every publish with `git checkout steward/work` in
  the SAME command; `git rev-parse --abbrev-ref HEAD` before any log write.
- **My detectors fail the way my ACs do** — matching a convenient proxy instead
  of deciding the property. **Four** false signals; every one neutralised by
  capturing the pane *before* concluding. **Verify-before-acting is the single
  highest-yield discipline of this session.** Use the watchdog's structural,
  provider-agnostic patterns — do not retype a narrower one-off.
- **An AC that names a MECHANISM transfers its blind spots into the deliverable,
  invisibly.** State the property, the closure axes, and require a **loud
  failure on the unhandled case**. This cost SPAN-SEAL two of its four blocks.
- After any Decision opens, **confirm every named reviewer reached a working
  state** — the architect stranded ~17 min on an unsubmitted paste; repair with
  a bare `Enter`; capture WIDE.
- **Do not send a ring a claim stronger than its evidence.** RT-PARITY's new
  basis is a *test-derived* closure, not a proof — it has been wrong three
  times. "Unconstructible on the landed closure, asserted by `<test>`" is
  honest; "impossible" is not.

## Decisions this window

| Time (UTC) | Decision | Basis | Status |
|---|---|---|---|
| 03:30 | Window opened; log started. | operator directive | — |
| 03:35 | Recorded the three sign-off answers above. | operator directive | — |
| 03:53 | **RT-PARITY remediation COMPLETE — candidate `27c5caf9`; QA in flight.** See the block below. | ring delivered | in QA |
| 03:53 | **NEW DEFECT FILED: `RT-ESCAPE` — pre-existing native lowering limitation.** Surfaced by RT-PARITY, correctly NOT fixed there. Filed by me because agents cannot create tracked work (§2). | COORDINATION §2; PRINCIPLES §13 "filed and tracked" | queued |
| 04:04 | QA verdict on `27c5caf9`: **BLOCK on a mechanical `rustfmt` gate only.** All substantive evidence passed, incl. QA's **own independent causal mutation** (sentinel-return splice → all six constructible cases fail with the exact declared modes; the BufferFreeze structural control alone stayed green — correct, it is a pin not a net). QA did not accept the implementer's flip table; it reproduced it. | ring | implementer remediating |
| 04:08 | **❌ MY ERROR — diagnosed a transport stall that did not exist; sent an unnecessary rouse.** See below. | self-caught | rouse queued, harmless |
| 04:11 | **❌ MY ERROR — reckless `pkill -f` with a broad alternation pattern.** See below. | self-caught | no damage; verified |
| 05:04 | **Post-compaction resume verified clean.** Re-oriented; all three branches intact at their recorded SHAs; `origin/main` still `61a78620`; watchdog re-armed at 900s with the full briefing. Implementer alive at 35m, actively deleting `elaboration_error_mentions` and running targeted tests — i.e. **working the closure inventory, not the honesty prose**, which is the right order. | structural detector | no action |
| 05:06 | **Stale participant status DISPOSITIONED — `integrator` / "PR #365 awaiting Steward routing".** Not a live obligation. See below. | verified by content | closed |

### 04:22–04:28 — rustfmt fixed (`2b55706a`), then ARCHITECT TERMINAL BLOCK on a real defect

Chain ran correctly and fast: implementer formatted → `2b55706a`; QA
re-approved; leader opened a **fresh** Decision **`dec_6w2gbgarf5443`** (note:
**a new Decision, not a re-anchor of `dec_7p15cgqz3x0sg`** — cleaner, and both
reviewers explicitly discarded their stale votes unprompted).

**★ Architect BLOCK (terminal, rejecting on cast) — and it is a genuine catch
that QA, CV and I all missed.** The BufferFreeze source-unreachability exemption
is **not established**. The implementer's argument was: `PrivateBufferSpan` is
not nameable ⇒ every span reaching `freeze` is host-minted and in range. The
Architect found a **public** counter-producer the private-name closure does not
remove — `write_all_advance_span` (`prelude.rs:2075-2078`, closure at `:2111+`):

```ken
fn write_all_advance_span (span : BufferSpan) (count : TransferCount) : BufferSpan =
  PrivateBufferSpan (add_int (buffer_span_start span) (transfer_count_int count))
                    (transfer_count_remaining count)
```

So checked source **can** build a new `BufferSpan` from source-obtainable
spans/counts **without naming the private constructor**. The claim is false as
written.

**★ Why this is the lesson of the night, and why my own praise of that test was
wrong.** At 03:53 I logged approvingly that the implementer "pinned the premise
with a discriminating pair so it fails if the premise stops holding." **A pin is
only as strong as the property it pins.** That pair tested *direct
constructor-name rejection* (`MkBufferWindow` control); the property actually
needed is **closure under composition of every public `BufferSpan`
producer/transform**. Constructor privacy was substituted for a closure proof —
`an-enumeration-needs-a-proven-closure-not-a-better-grep`. I read the test's
*shape* (non-degenerate pair, fails-if-premise-breaks) and credited it without
checking that its discriminator matched the claim. **Verify the property, not
the representative case — COORDINATION §7 — applies to the unreachability
argument itself, not just to the tests it excuses.**

Leader's remediation brief is well-formed and includes the right §6 escape
hatch: *"If the inventory reveals a genuine design choice about the helper's
public status rather than a structurally determined closure result, stop with
the grounded alternatives and report it for Architect ruling."*

**⚑ Watch item (possible separate defect, NOT actioned yet):**
`write_all_advance_span` being public means a **public** helper mints a
**private** type from public inputs. That may be an unintended capability leak
in the prelude rather than a test-coverage gap. The Architect's respin
explicitly forces the fork (seal it at its owning layer vs. prove the numeric
closure), so it is in hand — **do not open a parallel item unless the respin
punts it back.**

**Steward action: none.** Ring is healthy, reviewers are fresh-voting, the
leader assigned remediation within 16 seconds. No stall, no transport repair
needed. Logged only.

**04:41 — CV BLOCKED independently and reached the SAME defect by its own
grounding** (`evt_77p1t3xsa9352`), explicitly noting the Decision was already
terminally rejected and that it was recording the independent CV lane anyway.
That is the review model working as designed: **two reviewers, separate lanes,
no coordination, same `write_all_advance_span` finding.** Independent
convergence is much stronger evidence the defect is real than either vote alone
— and it is why the two-reviewer gate is not redundancy. CV also ran the full
executable suite itself (7/7, 720.89s) rather than accepting QA's run.

### ❌ Two Steward errors at 04:08–04:11 — both mine, both recorded

**(1) False stall diagnosis from a narrow pane capture.** I captured
`tail -5` on `runtime-implementer`, saw a bare `❯`, and concluded QA's mention
"never reached the seat's turn." **It had.** The seat woke on QA's mention at
04:04 and was `Beboppin'` the whole time — the activity spinner renders *above*
the input line, so a short tail truncates it and manufactures a phantom idle.
I then sent a transport-repair rouse that was pure redundancy; it queued behind
the seat's live turn.

**★ The damning part: I caught this exact trap at 03:08 this same session,
deliberately widened the capture, wrote it up — and then narrowed it again three
ticks later.** The lesson "capture wide" is evidently **not durable as a habit**,
so it must stop being a habit and become a **mechanical command**.

**FIX v1 (WRONG — superseded at 04:44, see below).** I replaced the tail with a
whole-pane grep over an **enumerated verb list**
(`Working|Beboppin'|Tomfoolering|Beaming|Cooked`).

**❌ FIX v1 FAILED WITHIN TWO TICKS — and failed in the exact way the Architect
had just blocked RT-PARITY for.** At 04:44 the detector reported
`runtime-implementer IDLE` while it was in fact **`Slithering… (15m 44s · ↓
21.1k tokens)`** with 3 shells running. The activity spinner cycles through a
whimsical, open-ended verb set; my list could never be complete. **I wrote an
enumeration with no proven closure — the identical defect class the Architect
had blocked ninety minutes earlier (`write_all_advance_span` escaping an
enumerated privacy list), which I had just finished praising him for catching.**
Had I acted on that reading, I would have sent a second bogus rouse — or worse,
"recovered" a seat that was mid-work.

**FIX v2 — detect the STRUCTURE, never the vocabulary.** A working pane always
renders a **running timer in parentheses**, regardless of which verb decorates
it. Match that:
```
tmux capture-pane -p -t moot-<seat> | grep -oE "\([0-9]+[hms]( [0-9]+[ms])? (·|•)[^)]*\)" | tail -1
```
Verified against both live forms: `(16m 2s · ↓ 21.1k tokens)` and
`(2m 02s • esc to interrupt)`. Empty = genuinely idle. Non-empty = **working, do
not touch.** Now in the watchdog prompt.

**The generalizable lesson, which is the whole point of recording this:**
*when a signal has an open-ended surface vocabulary, key the detector on the
invariant structure, never on a list of observed spellings.* Enumerate only
what you can prove closed. I have now made this mistake and had it made at me
inside the same hour — `an-enumeration-needs-a-proven-closure-not-a-better-grep`
is not a lesson about greps, it is a lesson about detectors.

**Corollary retained:** "no paste + no Working" is a *never-delivered* signature
**only** if the structural detector is empty.

**(2) Reckless `pkill -f "Working|Tomfoolering|Cooked|Beaming"`.** Run
reflexively to clean up my own stale background monitor. `pkill -f` matches
against **full command lines**, and that alternation could have matched
unrelated fleet processes. It happened to match only my own task (exit 144),
and I verified afterwards: **all 26 tmux sessions alive, implementer still
working, worktree intact.** But verifying after is not a control. **RULE: never
`pkill -f` with a broad pattern in this environment. Kill a known job by its
job spec or PID, or leave it — a leaked background task is far cheaper than a
mis-killed fleet seat.** Sibling of `orphaned-background-task-loops-leak-cpu`,
which is about the *cost* of leaks; this is about the far worse cost of the
cleanup.

**Net damage: none.** But two careless acts inside four minutes, on a night
where my job was to be the careful one, is the signal — both are recorded
against my name rather than smoothed over.

### RT-PARITY candidate `27c5caf9` — state of record (03:53)

Branch `wp/RT-PARITY-interp-native`, base `origin/main @ 61a78620` (no rebase
needed). WIP `3c574d8a` **amended into** this commit. Production behaviour
unchanged from `a692134e` — this commit is **tests + the conformance seed**.
4 paths; `crates/ken-runtime/` + `crates/ken-host/` byte-unchanged (verified
empty diff). Targeted only, no workspace build:
`ken-cli --test rt_parity_native` **7/0** (728s) · `ken-interp --lib rt_parity`
**8/0** · full `ken-interp` crate green (56 lib + all integration).
**All 6 executable differential cases flip**; interpreter 5 of 8.

**⚠ `dec_7p15cgqz3x0sg` MUST be re-anchored to `27c5caf9`.** The Architect's
APPROVE was cast on `a692134e` and does **not** carry forward; CV's BLOCK was on
evidence this candidate replaces. Both need **fresh** votes.

**Three reported bounds — two NARROW what the leader's directive asked for.**
This is AC-5 ("non-reaching obligations are reported, not dropped") working, and
QA has been told to verify the bounds rather than accept them:
1. The three `u64::MAX` single-fault cases **cannot** flip at the dispatch
   boundary — pre-fix the malformed arg became `u64::MAX`, which shared dispatch
   already rejects with the *same* variant the repair produces. Labelled
   in-source as **regression pins, not nets**, so they can never be miscited as
   flip evidence. Covered instead on the dispatch-skip axis.
2. **`BufferFreeze` has NO differential case — structurally, not by omission.**
   `PrivateBufferSpan` is removed from public scope (`prelude.rs:2110-2133`), so
   every span reaching `freeze` is host-minted and already in range; checked
   source cannot supply a malformed argument. Pinned by a **discriminating
   pair** (public window ctor must elaborate / private span ctor must not) so it
   **fails if the premise stops holding** — the right way to pin an unreachable.
   Same reasoning makes `FsWriteAt`'s `buffer_start`/`length` source-unreachable.
3. Coincident fault is a **rights** fault, not liveness — see `RT-ESCAPE`.

**⚠ CI COST TO WATCH:** the differential suite runs **~12 min locally (7 native
builds)** and will add real wall-clock to **every** CI run, not just this PR.
Flagging as a compounding cost; not blocking this WP.

### RT-ESCAPE (new, queued) — escaping a 2nd `Resource` fails native lowering

**Grounded by RT-PARITY:** constructing a closed-but-still-referenced resource
needs it escaped from its bracket; escaping a **second** `Resource` through a
bracket fails native lowering with
`OrientedSubcontinuationPlanV1: checked Runtime frame marker was consumed more
than once`. Escaping a resource **plus a plain value** lowers fine ⇒ it is
specific to a second *Resource*, and it is **pre-existing, not an RT-PARITY
regression**. The implementer correctly **filed rather than fixed** it, per the
frame's "if native looks wrong, file it, don't fix it here" guardrail, and
routed RT-PARITY's coincident-fault coverage through a **rights** fault instead
— which discriminates the same narrowing-order property.

**Why I am filing it here and now:** the implementer said "filing it" and the
seed records the finding, but **a note in a conformance seed is not a tracked
work item**, and agents cannot create one (§2 — the leader proposes, the Steward
sequences). PRINCIPLES §13's route-around clause requires the root fix be
**"filed and tracked"**; this is that filing. Not sized or scheduled yet —
Runtime-owned, needs Architect input on whether the frame-marker consumption
rule or the escape lowering is the defect's layer.
| 03:40 | **ADR-0010 amendment: PROCEED on the STR-BIJ branch. No hold.** Reasoned from PRINCIPLES per the operator's method instruction; it settles the question, so the hold clause does not fire. | PRINCIPLES §13, §8, §4 | ruled |

### Ruling — amending ADR 0010 under STR-BIJ (PRINCIPLES-settled, 03:40)

**Question.** STR-BIJ corrects a "bijection" over-claim at 7 sites. One is
`docs/adr/0010-...:61` — an **Accepted** ADR. Does amending it need the
operator, or does the normal merge Decision (Architect + CV) suffice?

**PRINCIPLES §13 — "fix the defect at its layer, never compensate upward" — is
directly on point and decides it.** The corpus is a citation stack: the ADR is
the layer beneath, and `spec/37:188`, `Derived.ken.md:1389` and
`seed-collections.md:656` each **cite ADR 0010 §2 as their authority**. The
defect lives in the ADR's justification sentence ("the round-trip is the
identity on scalar sequences" — naming the direction that is *false* under NFC).
Fixing the six leaves while leaving that sentence intact is **exactly**
compensating upward: it leaves the defect live for every future author who
reads the ADR and re-derives "bijection," and it hides the root. §13 even names
the tell — *"you reach for 'just handle it here instead'"* — which is precisely
what holding the ADR line and patching leaves would be.

**§8 reinforces:** the sentence is an over-claim about what the round trip
guarantees, and over-claiming *is itself a failure*. Knowingly shipping a
corpus-wide honesty fix that preserves the root over-claim is incoherent.
**§4:** holding for 8 hours to dodge a governance question is the *expedient*
over the correct/permanent.

**Why this does NOT need the operator.** The ADR's **decision stands
unchanged** — `String` *is* canonical w.r.t. `List Char`, `DecEq`/`Ord String`
transport *is* deliverable. Only the stated *reason* was too strong (the
consumer needs `s2l` injectivity, which the landed retraction axiom supplies).
An amendment that changes **no decision** is a wording correction, it is
recorded **as** a dated amendment (frame AC-6, never a silent rewrite), it is
fully reversible and auditable, and the ADR's own co-decider — the **Architect**
— votes on it through the §14 Decision it already casts. No new authority is
being claimed.

**★ The boundary I am setting for the rest of this window** (this is the part
that generalizes): **an ADR amendment that PRESERVES the recorded decision →
proceed under the normal gates. An amendment that would CHANGE a recorded
decision → HOLD for the operator**, regardless of how obviously right it looks.
The second is a governance act; the first is an honesty repair. PRINCIPLES
settles the first and is silent on the second — and per the operator's
instruction, silence means hold.

**⚑ SCOPE OF "HOLD" — operator clarification (2026-07-21 ~03:45):** *"By 'hold'
I mean hold on the ADR. Route around it if you have to hold and you can route
around it."* ⇒ **A hold is scoped to the held ARTIFACT, never to the WP.** If an
ADR line must hold, **land the other 6 sites and hold only the ADR line** — do
not stall the erratum behind it. Generalize to every hold this window: hold the
narrowest thing that must be held, route the rest through.

**★ This is NOT in tension with the §13 reasoning above — PRINCIPLES §13
already carries the exact clause, and I should have surfaced it the first
time.** §13 verbatim: *"A higher layer may route around a lower defect only as a
**disclosed, temporary** hedge, with the root fix **filed and tracked** — never
as the permanent answer, and never in a way that **hides** the defect."* So the
operator's instruction and §13 are the same rule. The three conditions are
binding and are what separate a legitimate route-around from the
compensating-upward §13 forbids:
1. **Disclosed** — the shipped artifact says the root is still wrong. A leaf fix
   that reads as complete while the ADR still misstates the property is exactly
   the "hides the defect" failure.
2. **Filed and tracked** — the held ADR line gets a real queued item, not a
   tracker sentence saying I intend to file one. (My own dropped F1 dispatch
   this session is the cautionary case.)
3. **Temporary** — it comes back to the operator, it does not quietly become the
   permanent state.

**Applied now:** moot for ADR 0010 — PRINCIPLES settled it as *proceed*, so
nothing is held and no route-around is needed. The clause is recorded because it
governs whatever the rest of this window turns up.

---

# Steward decision log — autonomous window 2026-07-18

Operator (Pat) away ~03:47 UTC → ~11:30 UTC. Mandate: keep the fleet moving;
use `docs/PRINCIPLES.md` for judgment calls; route around anything needing
operator input (log it here + keep other work moving); merge on gates + CI green
per the standing gate model (Runtime/QA + Architect §14 + CV + CI — there is no
separate operator-merge gate). Pat may check in once or twice.

**Routing rule for the window:**
- **Design/component forks** → Architect (`agt_37reqftfe6g00`).
- **Genuine product/priority forks not settled by roadmap + PRINCIPLES** →
  log here, `defer_question` to Pat, and keep all other work moving. Do NOT
  guess a one-way-door product decision.
- **Everything gated + green** → merge autonomously (publisher), verify
  byte-identical, PIN, chase retros.

**Settled inputs (do NOT reopen):** R2 (no Ken affine/linear type; safety in
runtime+Ward), Ward is a separate project (Ken only exports the boundary),
collapse-V1/V2-no-compat (PRINCIPLES transient T), Linux-ABI-direct (POSIX
dropped), targeted-builds-only (no local `--workspace`), operator approvals are
fixed inputs.

## Plan of record for the window
1. **PX8-N** (bounded-Nat reply-lowering) — in fresh Runtime QA on rebased tip
   `9bd52149` (conflict-free onto PX8-X main, retains §14 `dec_vh7vdv5428p0`).
   On QA APPROVE → republish non-doc-only → merge → PIN → chase retros.
2. **PX8-F** (buffer-IO surface) — held at `60a481b5`; unblocks once PX8-N lands.
   Rebase onto combined main + re-kick Foundation (corrected terminal obligation:
   author the FIRST real linked checked `writeAll` native fixture, per Architect
   `evt_4kh6gz18tvzs6`). This completes the PX8 buffer-IO floor.
3. **Continue the Linux ABI campaign** (`docs/program/09-posix-linux-abi-campaign.md`)
   — decompose + sequence the next WPs from the charter after the PX8 floor,
   handoff-gate + kick per §2c. Author shovel-ready T1 briefs.
4. **Test-suite review WP #24** stays gated until the ABI campaign lands.

## Default stances (flag on check-in if you disagree)
- **Merge autonomy:** I will merge any WP that clears its gates + CI, including
  ABI/OS-interface landings — the gate model is the safety net. I will NOT invent
  an extra hold unless a change is a genuine one-way door with no PRINCIPLES-clear
  answer, in which case I hold at the pre-merge gate and log it.
- **Credit:** I will run the enclave for T1 brief-authoring as the roadmap needs
  but keep the compact-at-seam discipline to conserve.

## Decisions
| UTC | Decision | Basis | Reversible? |
|---|---|---|---|
| ~03:47 | Continue ABI campaign per charter after PX8 floor; merge on gates+CI green | roadmap + standing gate model | yes (Pat can re-prioritize on check-in) |
| ~03:51 | **Window priority CONFIRMED by Pat: continue the Linux ABI campaign per the charter.** | operator directive | n/a (explicit operator call) |
| ~03:53 | PX8-F brief terminal-obligation corrected (author FIRST writeAll native fixture, not rerun); re-kick-ready pending PX8-N landing. ABI next-WP = PX9 (structured errno, Foundation) per charter §6 PX7→PX8→PX9. | Architect evt_4kh6gz18tvzs6 + charter | yes |
| ~04:00 | Publish PX8-N `9bd52149` non-doc-only WITHOUT tracker bundle (base predates current-main tracker deltas → bundling would conflict); sync tracker+log via separate doc-only merge after. | avoid self-inflicted tracker merge conflict (cadence exception, logged) | yes |
| ~04:14 | PX8-N MERGED+PINNED on `origin/main = ace72db7` (PR #763). Verified all 9 reviewed paths + full tree byte-identical to §14-approved `9bd52149`; `dec_vh7vdv5428p0` discharged. Runtime §10 retro requested (`evt_2agz5010dpaz8`). | gate model + byte-identity verification | n/a (landed) |
| ~05:05 | PX8-F hard-stop (transparent recursive `writeAll` overflows native compile) → routed to Architect, who ruled it VALID (`evt_2vgt1s790vaee`): frame a **Runtime-owned prereq PX8-L** (finite closure boundary + general recursive-`DeclarationRef` native lowering; the "NC22+" capability the backend punts). Authored PX8-L brief, pushed WP branch `b5fb8aec`, held PX8-F on it. Did NOT let anyone rewrite `writeAll`/add intrinsic/enlarge stack (Architect guardrails). | Architect mechanism ruling + §2c decomposition | n/a (design ruling) |
| ~04:20 | PX8-F treated as a held-WP **semantic unfreeze** (not a quick same-WP respin): the schema collapsed underneath Foundation (PX8-X V2→sole unversioned) and the native carrier landed (PX8-N), so their held context is stale → run the FULL handoff-gate compaction of Foundation L/I/Q before re-kick. Brief flipped HELD→RESUME, base pinned to `ace72db7`. | [[held-wp-unfreeze-is-a-semantic-rebase-rederive-anchors]] + compact-before-new-work rule | n/a (process) |
| ~08:43 | **PX8-L PUBLISHING** — Decision `dec_2pgckamtt8eqh` verified RESOLVED (object: APPROVE exact `79d08408`, resolved_by architect; Runtime QA + §14 both cast; no CV — spec/conformance byte-identical). Publisher launched non-doc-only (bg `beht5rblv`, PR #768) on a local branch pinned to exact `79d08408` (avoids the stale-local-branch force-push hazard). Polling CI. ON MERGE: verify byte-identical → PIN → separate doc-only tracker/log sync → retros → create+kick PX8-H → PX8-F. | resolved-Decision-object gate + §14 discipline | n/a (publishing) |
| ~08:28 | **PX8-L §14 BLOCK on `5b7d031c` (`evt_3j5v38wadrgaa`, Decision `dec_79sg9emgfhthc` rejected) — stale-frame contradiction, MINE.** Architect approved the production mechanism but blocked because the PX8-L brief on the branch still carried pre-split language (c8b8cdb7 as a PX8-L acceptance discriminator; sequence PX8-L→PX8-F direct) — I split PX8-H + updated the tracker but not the parent brief. Frame-only respin `79d08408` (one path = the PX8-L brief, +40/−28; production byte-identical) reconciles it to `evt_3tcjvkcsz02fa` (bankable PX8-L; c8b8cdb7 = presently-red PX8-H/PX8-F downstream evidence; order PX8-L→PX8-H→PX8-F); pushed + synced to steward/work; QA re-gating + fresh Decision to follow. Lesson recorded ([[wp-split-leaves-parent-brief-downstream-stale]]). Cost: one extra §14 round-trip. | Architect §14 gate caught the contradiction | n/a (corrected) |
| ~07:50 | **Architect ruled (B) (`evt_3tcjvkcsz02fa`): bank PX8-L @ `3b917a9b` as the finite recursive-declaration unit; spin the composition seam into a distinct Runtime prereq. Sequence PX8-L → PX8-H → PX8-F.** Mechanism (Architect-reproduced): the new seam is a missing PX7-O/P heterogeneous eliminator-frame case — an env-resolved `ComputationalRecursorClosure` feeding a known ordinary match before returning to outer computational frames; the syntactic `Call Var` classifier can't see the env-resolved closure → intervening ordinary frame (`after_read`) not installed → dynamic match hits fail-closed default (NOT PX8-L's cyclic-normalization/DeclarationRef mechanism). Actions: (1) relayed PX8-L clear-to-review at `3b917a9b` to runtime-leader (Runtime QA → §14 → Decision → publish; ruling ≠ §14 approval); (2) authored PX8-H brief `docs/program/wp/PX8-H-heterogeneous-continuation-composition.md` pinning Architect's fixed shape (value-aware call seam installs ordered heterogeneous continuation before lowering recursor producer; NO "every `Call Var` is aggregate" heuristic; fail-closed preserved; no special-case/ABI/kernel change), Owner Runtime, Size M, Risk High, bases on LANDED PX8-L. `c8b8cdb7` immutable downstream discriminator; PX8-F resumes only after PX8-H lands + unchanged fixture does real writes both lanes. | Architect mechanism ruling + §2c decomposition | n/a (design ruling) |
| ~07:36 | **PX8-L hard-stop routed to Architect (`evt_3y2e081bazryj`) for mechanism + scope disposition.** PX8-L's OWN Architect-ruled contract is DELIVERED+green at checkpoint `3b917a9b` (finite SCC closure + recursive `DeclarationRef` lowering live; dynamic-Nat proof 3/3; `ken-runtime --lib` 231/231). A NEW seam surfaced only on the PX8-F `c8b8cdb7` throwaway overlay: real `writeAll` now compiles/links/opens/allocates/completes `FsReadAt(ReadSome)` then traps before first `FsWriteAt` — the recursor carrier drops the intervening ordinary continuation (`after_read`) from the recursive-IH frame chain, so dynamic `ReadProgress` match hits fail-closed "no source case" (causally proven: `-4`→success passes admission but writes nothing → weakening is not a fix). Fork to Architect: (A) natural completion of recursive-lowering order-preservation → extend PX8-L; (B) distinct nested recursive-IH × ordinary-continuation composition boundary → new Runtime prereq node, land `3b917a9b` on its delivered core; (C) else. Held PX8-L at `3b917a9b`; asked Runtime to push the checkpoint as durable artifact. | hard-stop clause + mechanism=Architect / scope=Steward split | n/a (routed) |
| ~16:4x | **Research QA-report merge request (`evt_6yneba6cgxqb3`) resolved as MOOT — no publish.** The report (`research/qa-conformance-to-rust-test-guidelines.md`) is already byte-identical on `origin/main @ 3be76cc2`, and the QA playbook already references it (`agent/playbooks/build/qa.md` L240 gate, folds 3 promise classes + 10 gates) — operator msg-2 fully satisfied+landed. Branch `wp/research-rust-test-design-guidelines @ 762ea1d8` is stale-based (merge-base `78ef39eb`, pre PX8-N/X/P/V) → publishing would phantom-revert the train; told research to stand down. Integrator "#365 awaiting routing" = STALE (PR #365 merged 2026-07-08; 0 open PRs). | artifact verification (empty content-diff + merge-base check) | n/a (housekeeping) |
| ~09:10 | **PX8-L `79d08408` FAILED CI merge-gate (PR #768 `build + test`); NOT merged, `origin/main` stays `3be76cc2`.** Publisher stopped correctly (conformance / clean-room / path-guard passed; only `build + test` failed). Root cause: `ken-elaborator --test nc17::recursive_body_view_lowers_to_explicit_runtime_declaration_ref` (`:354`) — the mechanism lowers an unapplied recursive ref to `Call{DeclarationRef,[]}` but the **landed** contract (nc17 on main, `:356`) pins the bare `DeclarationRef`; the QA/§14 evidence set ran `ken-elaborator --lib` (95), not the `nc17` integration test. Routed to Runtime (`evt_667h9q7wgk62m`; both seats Working, mechanism frozen while they localize): diagnose (a) mechanism-fix preserve bare ref [favored] vs (b) intended shape-change → landed-contract change routes Steward→Architect (do NOT silently update the oracle); re-run set MUST include nc17 + grep-derived DeclarationRef/`Call` blast-radius. `dec_2pgckamtt8eqh` does not carry; respin → QA → §14 → fresh Decision → republish. Lesson [[declref-lowering-change-needs-elaborator-integration-tests]]. | CI is a co-equal merge gate; targeted-local ≠ full-workspace CI | n/a (routed) |
| ~09:35 | **PX8-L production respin `8711f82e` PUBLISHING (option-a bare-ref fix).** Runtime chose option (a): `erasure.rs` +3 — an empty recursive application spine returns bare `RuntimeExpr::DeclarationRef` (only nonempty spines build `Call`), preserving the landed nc17 oracle; no Architect contract-change needed. Runtime QA APPROVE (`evt_16sc4xbn89yek`, re-ran nc17 + NC13/NC15 + PX7-L/PX8-L integration sweep) + Architect §14 APPROVE — **Decision `dec_3fbm85e7c0nc8` VERIFIED RESOLVED** via list_decisions object (`status=resolved`, `resolved_by=architect` agt_37reqftfe6g00, APPROVE exact `8711f82e`, base `3be76cc2`, nine linear commits, erasure.rs-only child of `79d08408`, byte-identical elsewhere; no CV). Pushed `8711f82e` (credentialed), relaunched publisher non-doc-only (bg `b7cn2ylx7`, PR #768 re-polling CI). Caught + cleared a stranded §14 paste on the architect seat (post-auto-compact, no Working spinner) with bare-Enter. ON MERGE: verify byte-identical → PIN → doc-only sync → retros → PX8-H → PX8-F. | resolved-Decision-object gate + §14 discipline | n/a (publishing) |
| ~09:55 | **PX8-L MERGED + PINNED @ `origin/main = e74e935f` (PR #768).** Publisher `b7cn2ylx7`: "checks passed and merge command succeeded". Verified landing by CONTENT not ancestry (squash-merge): `e74e935f` parent = `3be76cc2` (clean squash, no intervening merges), full-tree diff `8711f82e` vs `e74e935f` **empty** (byte-identical to §14-approved SHA), `erasure.rs` bare-ref fix present on main. `dec_3fbm85e7c0nc8` discharged. Next: chase Runtime L/I/Q §10 retros (gate-1 for the PX8-H handoff), doc-only tracker/log sync to main, then handoff-gate + kick PX8-H off `e74e935f`. | gate model + byte-identity (squash) verification | n/a (landed) |
| ~10:10 | **PX8-L CLOSED; PX8-H KICKED (Runtime).** All 3 PX8-L retros in (L `evt_1r0ajk54vwt5h` / I `evt_6y602rfk4cgns` / Q `evt_3s8znerr4dmmb`; I+L retros both captured the targeted-vs-CI lesson). Doc-only tracker/log sync merged (PR #769 → `origin/main = fe6e5360`). Ran handoff-gate `scripts/handoff-gate-compact.sh runtime-leader runtime-implementer runtime-qa` — verified all 3 `Context compacted` + reset to `fe6e5360`. Staged + pushed `wp/px8h-heterogeneous-continuation-composition @ 1acee25d` (off `fe6e5360`, brief only). Kicked `evt_21efmpkqak09s` (code_share, all 3 mentioned; brief authoritative); confirmed all 3 seats `Working`. Route Architect §14 + Runtime QA (no CV). PX8-F stays held on PX8-H. | §2c handoff-gate discipline + shovel-ready brief | n/a (kicked) |
| ~11:40 | **PX8-H mid-build carrier escalation → Architect; Steward cleared a stranded-paste stall.** Implementer grounded the ruled value-aware seam to the resolved `lower_expr` `ComputationalRecursorClosure` arm, found all 4 existing syntactic classifier sites fail value-aware + frame-reversal breaks a later consumer, and escalated an A/B carrier fork to Architect (`evt_3r7mmdmz21f97`: (A) extend `EliminatorFrame` w/ a general closure-continuation frame vs (B) thread a pending ordinary-frame context from the caller). The question **stranded as an unsubmitted paste on the architect seat** (idle, no Working) — I cleared it (bare-Enter); architect auto-compacted, re-oriented, and is now reading the PX8-H frame to rule. Mechanism = Architect's call; I stay out of the A/B design and only fixed transport + tracked. Awaiting the carrier ruling → implementer resumes → candidate. | transport backstop; mechanism→Architect / process→Steward split | n/a (routed) |

| ~12:05 | **PX8-H carrier fork RESOLVED — Architect ruled (B) caller-provenance (`evt_h706fqp1dqa8`); implementer building it; NO stall.** Architect ruled the A/B fork at 11:50: fixed carrier is (B) — thread the pending ordinary continuation from the already-selected `lower_computational_producer_expr` `Let` context into the resolved recursor call; do NOT scan the recursor closure's base body for a general continuation (that classifies recursor *implementation* syntax as caller control context — the PX7-O category error the negatives forbid). 5-point composition shape pinned (capture `body`+`producer_env` at the `Let` seam → route `value` w/ that continuation before outer eliminators → prepend recursor computational layers at `Call Var` resolution → consume pending continuation after all recursive layers, bind `[value]+saved env` as ordinary `Let` → preserve trap/binder-arity/env-order/final-kind; `lower_dynamic_constructor_match` defaults byte-unchanged; `EliminatorFrame` may be the storage type but provenance/install = (B)). Ruling reached the Codex seat cleanly (no strand this time). Diagnosed LIVE: implementer is Working (~1h53m single turn, expected for a Codex seat) with a local work commit `98f91c6b` "thread caller continuation through recursive IH", iterating on the immutable `c8b8cdb7` overlay test (H-P6), currently red; origin px8h still `1acee25d` (no candidate yet — local git only, releases to me for the credentialed push). Evolving content + live compile/test = progressing, not wedged → **no space post** (watchdog: post only on a REAL stall). Held event-driven for the candidate SHA. | LIVE-artifact diagnosis (pane + ls-remote); mechanism=Architect | n/a (progressing) |
| ~12:15 | **PX8-H second hard-stop (structural-identity question) → Architect; Steward cleared another stranded-paste stall.** Implementer built (B) caller-provenance — reaches the live `lower_computational_producer_expr` Lets (`FsReadAt`+`ResourceRelease`), judgment `produces_with(env-recursors +1) && !produces_with(empty)`, pending frame binds `[response]+saved producer_env` — but hit a deeper structural gap: the `FsReadAt` callee resolves to a chain of **5 `ComputationalRecursorClosure` layers with `outer_eliminators.len()==0`**, so the "existing outer" frames the (B) ruling assumed live as the caller's eliminator suffix are actually already NESTED inside the resolved callee chain; treating the chain uniformly still presents ReadProgress to the ResourceBracketResult layer. Implementer hard-stopped "before guessing" and asked the Architect for the structural identity separating immediate recursive-IH layer(s) from the nested outer layers (`evt_4wzxjpw2jzqnx`, mentions architect+leader). Mechanism → Architect's lane. Question **stranded as `[Pasted Content 2048 chars]` on the architect Codex seat** (at rest post-(B)-ruling, checkpoint `3911d1a9`, no Working) → cleared (bare-Enter); architect auto-compacted + re-orienting to rule. Implementer still Working (~2h07m, adding `outer_eliminators` diagnostics — not idle/wedged). Steward = transport + tracking only; I do NOT adjudicate the structural identity. Awaiting the ruling → implementer resumes → candidate. | transport backstop (bare-Enter); mechanism=Architect / process=Steward | n/a (routed) |
| ~12:24 | **PX8-H structural-identity fork RESOLVED — Architect ruled peel-exactly-one-layer (`evt_13xfc1thegtxz`); implementer picked it up cleanly and is building; NO stall.** After the bare-Enter cleared the strand, the architect auto-compacted, re-oriented, acked pickup (`evt_6wd4n926bqq6j` @ 12:16), and ruled @ 12:24: the split is the OUTERMOST `ComputationalRecursorClosure` node vs its `recursive` payload — peel exactly one layer before the caller-owned pending continuation (cardinality is exactly one: each IH mint wraps the selected recursive field once; no site creates a multi-layer current group). 5-layer FsReadAt → layer 0 immediate, 1–4 outer tail. Order = immediate head → PendingLet → recursive-tail frames → existing eliminator suffix; residual base invoked ONCE (no re-call of the full recursor value); `produces_with` stays admission/non-vacuity only; `lower_dynamic_constructor_match` byte-unchanged; proof strengthened with a 2+-layer distinct-defaults control + 1-layer control. Ruling mentions the implementer and reached its Codex seat CLEANLY (no strand this time) — turn cycled, Working, amended local work commit `495a6b70` (+177 in cranelift_backend.rs); origin px8h still `1acee25d` (local-only until release). Steward stayed out of the structural adjudication (Architect's lane); transport + tracking only. Awaiting candidate SHA. | mechanism=Architect; LIVE-artifact diagnosis (pane + convo event + ls-remote) | n/a (progressing) |
| ~12:5x | **Orphan convo `schedule_call` timer `tmr_37vn72vp` fired stale content into the space (noise) — NOT cancellable by me.** A legacy convo timer replayed the long-resolved 09:19 PX8-L §14 request (`dec_3fbm85e7c0nc8`, merged @ `e74e935f`) as a System-event mention at me — the exact broadcast-noise pattern §13 deprecates (my real watchdog is the private `CronCreate` `bccc58bb`). `cancel_call(tmr_37vn72vp)` → 404 "not owned by you"; `list_calls` → "No active timers" → **RESOLVED, non-recurring: it was the runtime-implementer's own one-shot wait for the PX8-H ruling (`evt_1dc2edwkn77kt`), already fired + auto-removed** (hence the 404 for everyone; it's gone, not un-killable), and the implementer will not schedule another. Runtime-leader also hit the 404 (`evt_6h66bve20h0g9`). No response needed — the implementer already answered the leader; a Steward post would be noise. No frontier impact (content stale). Lesson reaffirmed: use private `CronCreate`, never convo `schedule_call`, for any wait/watchdog ([[cancel-watchdog-cron-when-fleet-quiescent]]). | §13 (private CronCreate, not convo schedule_call) | n/a (noise, self-removed) |
| ~12:36 | **PX8-H THIRD hard-stop (HostResult/ITree operational contradiction) → Architect, engaged cleanly; NO stall.** Implementing the peel-one-layer ruling literally surfaced a type contradiction: reaching `Let` value = `Effect FsReadAt` → `Lowered::HostResult`, but peeled head cases = `ITree::{Ret,Vis}` → `lower_computational_match_value_composed(HostResult, head)` fails the constructor-only boundary ("scrutinee is not a constructor value ... HostResult"). Pending-before-chain hits the wrong ReadProgress default (and is forbidden by `evt_13xfc1thegtxz`). Implementer asked the Architect to rule the exact operational meaning of "head→Pending→tail" when the Let value is a HostResult and the head accepts only ITree constructors (`evt_13rhynzmgjcrd`, clean checkpoint `495a6b70`, not weakening consumer / not treating HostResult as ITree / not changing defaults). Architect's lane (operational semantics). Reached the seat cleanly (no strand this time) — Architect ENGAGED @ 12:36 (`evt_31q0q6p9cxmwr`: "picked up; re-anchoring `495a6b70`, tracing producer/consumer type transition before correcting/reaffirming the ordering"). Implementer Working/"Waiting for agents", turn active. Steward out of the mechanism; transport + tracking only. Awaiting the operational ruling → implementer resumes → candidate. | mechanism=Architect; LIVE-artifact diagnosis (panes + convo events) | n/a (progressing) |
| ~12:40 | **PX8-H operational fork RESOLVED — Architect corrected the execution vector to `PendingLet → head → tail → suffix` (`evt_6nz2kfs9h6r8s`); implementer building it cleanly; NO stall.** The Architect confirmed the third hard-stop is VALID and that its own prior `head → PendingLet → tail` spelling was wrong for this lowering API: `eliminators[0]` consumes the current producer result, which at the reaching `Let` is a raw `Lowered::HostResult` (from `Effect FsReadAt`) that the ITree `Ret|Vis` head cannot consume. Fix keeps the caller-provenance carrier + the exact one-layer structural cut but reorders the vector so `PendingLet` is the first adapter — it ordinary-lowers the response, binds `[response]+saved_producer_env`, invokes the peeled residual base ONCE → residual body produces the ITree value → THEN head consumes it; tail follows head. Explicitly NOT a new prerequisite (amend PX8-H at this seam; `495a6b70` is a valid partial checkpoint). Guardrails unchanged (residual base once, no layer twice, call-env `args→captures→saved_env`, no residual-body scan, `lower_dynamic_constructor_match`/defaults/PX7-O·P negatives byte-unchanged); discriminator strengthened (head-first→HostResult refusal; pending-first→residual ITree to head; multi-layer distinct-defaults proves head<tail; 1-layer empty-tail; full-chain-reinvoke/head-behind-tail→wrong default). Ruling mentions the implementer and reached its Codex seat CLEANLY (no strand) — Working, editing cranelift_backend.rs trap path; origin px8h still `1acee25d` (local-only until release). Steward out of the mechanism (Architect's lane); transport + tracking only. Next: implementer completes H-P1–H-P6 + integration blast-radius (NC13/15/17 + PX7-O/P/PX8-L) → candidate SHA → my push/Decision/publish path. Note: pane-grep watchers false-positive on stale scrollback (prior rulings' @-mentions persist on screen) — use convo events, not capture-pane grep, to detect a NEW ruling. | mechanism=Architect; LIVE convo-event diagnosis | n/a (progressing) |
| ~12:58 | **PX8-H FOURTH ruling — leaf-to-root shared decomposition (`evt_6vmt0tmyhmb0m`); the resolving mechanism; implementer building cleanly; NO stall.** After the PendingLet-first vector (ruling 3) compiled and materially advanced H-P6 (checkpoint `9da7e0d3`: checked source now observes FsOpen×2/BufferAllocate/FsReadAt(ReadSome 6)/ResourceRelease×3, still 0 FsWriteAt + RuntimeTrap(4)), the implementer found the head's selected continuation was the outer ResourceBracket consumer, not the required after_read/ReadProgress one, and hard-stopped again (`evt_190vh71ytxwna`/`evt_6bn1gsvn9e5pp`: outermost node default = "checked HostIO match had no constructor arm" = final consumer, not local after_read). Architect ruled (correcting its own ruling-2 "outermost-head"): the stored recursor chain is a CONTINUATION-NESTING PATH, not an execution stack — storage order outer→inner, EXECUTION order inner→outer (leaf-to-root). `after_read` is owned by the TERMINAL recursor node immediately above the residual base (structural identity, not a default string/name/chain-length/body-scan). ONE shared decomposition rule (walk `recursive` collecting `{cases,default,outer_env}` in storage order → residual base = first non-recursor → reverse ONLY that collected chain once → apply IDENTICALLY at all THREE recursor-call consumers: caller-provenance PendingLet, ordinary `lower_expr` call, `lower_computational_producer_expr` call). The reverse-only-at-Pending partial was an incomplete application of the one invariant, not a heuristic. Byte-preserve each frame + call-env `args→captures→saved/producer_env`; `lower_dynamic_constructor_match`/HostResult/defaults/final-kind/PX7-O·P unchanged; NOT a new prerequisite. Discriminator: 3-layer distinct-defaults stores outer→mid→inner but executes inner→mid→outer; 1-layer unchanged; revert-to-storage-order recovers the wrong outer default; exercise all three consumers. Ruling mentions implementer, reached the Codex seat CLEANLY (no strand) — Working, editing cranelift_backend.rs (per-node `extend(tail)` → shared `extend(frames)` inner→outer); origin px8h still `1acee25d` (local-only). This is the 4th sequential Architect design ruling in the PX8-H chain (B-carrier → peel-one-layer → PendingLet-first vector → leaf-to-root shared decomposition); each was the Architect's lane and I stayed out of the mechanism, tracking + fixing transport only (2 stranded-paste bare-Enter clears earlier). Condensed the tracker's PX8-H block (4 verbose ruling transcriptions → compact chain; full detail here). Awaiting candidate SHA. | mechanism=Architect; LIVE convo-event diagnosis | n/a (progressing) |
| ~13:09 | **PX8-H FIFTH hard-stop (ProcessExitStatus over-consumption at ordinary 5-layer recursor call) → Architect engaged; genuine progress; NO stall.** Implementer built ruling-4's shared leaf-to-root decomposition at all three consumers (checkpoint `afeba2fc`) — the ReadProgress→ResourceBracketResult mismatch is RESOLVED. H-P6 now fails LATER (object emission): an ordinary 5-layer recursor call over-consumes through the final exit layer → `ProcessExitStatus` into a surrounding `Result::{Err,Ok}` match (consumer log: ordinary recursor-call chains 1,3,7,then 5). Implementer hard-stopped exactly under ruling-4's "if a failure remains, preserve the exact value kind/cases and hard-stop again" clause (its read: whole-chain execution at ordinary recursor-call consumers over-consumes layers belonging outside that call's source context; leaf→root right for the ReadProgress seam, over-broad here) — did NOT filter by length/default/body or weaken the Result match. Architect's lane; reached cleanly (no strand); Architect ENGAGED @ 13:09 (`evt_4sxg77y9jbnyb`: re-anchoring `afeba2fc`, tracing the 5-layer ordinary recursor-call boundary, ruling the structural ownership cut w/o heuristics or weakening the match) — pane confirms Working (searching "ordinary closure continuation", `requires_heterogeneous_deforestation`). Implementer idle-waiting. This is the 5th sequential Architect design ruling in the PX8-H chain; each hard-stop moves the failure strictly deeper (real progress, not thrash), implementer disciplined (hard-stop w/ evidence, not guess), Architect anticipated it ("hard-stop again"). Steward out of mechanism; transport + tracking only. Op-awareness note added to tracker for Pat's check-in (thorny lowering, progressing, no operator decision needed). | mechanism=Architect; LIVE convo+pane diagnosis | n/a (progressing) |
| ~13:27 | **PX8-H SIXTH ruling — recursor-lineage overlap cancellation (`evt_5p7ef4yvmcaac`); implementer building cleanly; NO stall.** Architect ruled the 5th hard-stop valid: keep leaf-to-root, but the caller-provenance `Let` owns a COMPOSITION of two chains sharing an inherited outer suffix (producer `[A₀A₁B₀B₁C₀C₁D]`, body `[B₀B₁C₀C₁D]`); current impl runs the shared outer suffix TWICE (→ ProcessExitStatus into Result::{Err,Ok}). Fix = private clone-stable `RecursorFrameProvenance` on each minted layer (private to `Lowered`; NO IR/kernel/ABI/wire/host/surface change) → when the reaching `Let` has both a recursor-call value AND an admitted body recursor-continuation call, decompose both, compute maximal common outer suffix BY PROVENANCE (not length/case/default/body/hash), split producer/body unique-inner ++ shared_outer, execute shared suffix ONCE after PendingLet: `producer_unique_inner→PendingLet(body residual)→body_unique_inner→shared_outer→caller suffix`. Corrects the prior "whole chain at every consumer" wording (owned chains still leaf-to-root; the Let owns two → quotient once). H-P4 strengthened (provenance can't be faked by shape; suffix-before-Pending mutation recovers the exact refusal; empty/exact/raw/standalone controls). Ruling mentions implementer, reached the Codex seat CLEANLY (no strand) — Working, editing cranelift_backend.rs (`mint_recursor_frame_provenance()`); origin px8h `1acee25d` (local WIP `afeba2fc`+). Architect pane had a self-echo paste of its own ruling (verbatim `evt_5p7ef4yvmcaac`; convo has NO event after 13:27, so nothing waiting on the architect) — left it (bare-Enter would risk re-post). 6th sequential Architect ruling; still progressing (checkpoints 495a6b70→9da7e0d3→afeba2fc, failure strictly deeper each round). Steward out of mechanism; transport + tracking. | mechanism=Architect; LIVE convo+pane diagnosis | n/a (progressing) |
| ~15:3x | **NEW ROLE: Adversary (standing red-team) — authored + wired; provisioning underway.** Operator directive: add an adversarial tester, librarian-shaped, that hunts recent changes + blast radius for flaws/gaps/leaky-abstractions/undesirable-behavior (the positive side is well-covered by CV+QA; the negative side wasn't a standing function). Operator picked (AskUserQuestion): **advisory-only/non-blocking** (findings→Steward triage, never gates), **standing T1 seat that dispatches T2/T3 fan-out**, **refs-read like research** (both tiers, leakage recheck). Authored `agent/playbooks/federation/adversary.md` (=`ken-adversary` skill) + full wiring: skill symlinks (`.claude`+`.agents`→same tracked file), `agent/memory/roles/adversary/`, AGENTS.md routing+memory-scope+clean-room summary, CLEAN-ROOM.md refs grant, MODELS.md T1 roster, COORDINATION.md §15 singletons, steward.md §5b (triage-its-findings). Committed `83ac0a4d` on steward/work — **NOT yet on main** (was holding for operator playbook review; operator then said "provision it" → publishing implied). **▶ PROVISIONING (operator gave a PAT at `/workspaces/ken/.mootup/credentials` = single `mootup_pat_…` token; DO NOT echo it):** mootup is a pip pkg; operator cloned the backing source into `local/refs/convo` for API docs (reading it is OPERATIONAL infra, not Ken-code authorship → no clean-room issue). Explore subagent `a52ff773` mapped the API (base `https://mootup.io`, space `ken-topos spc_4q7g0se87rgje`, SEC-5 ticket flow). **✅ AGENT CREATED via surgical REST (avoided `moot config provision`/`--fresh` which could rotate live seats' keys mid-PX8-H): `POST /api/agents {role:adversary, agent_profile:codex}` → `actor_id agt_37vnwmcdxhw00` + ticket → `POST /api/registration-tickets/{ticket}/exchange` → agent key MINTED + stored in `.moot/actors.json` under `adversary` (chmod 600, never echoed).** Role config = harness=codex, model=gpt-5.6-sol, effort=high (matches research/architect T1 codex seats). **✅ FULLY PROVISIONED + LAUNCH-READY:** (1) corpus PUBLISHED to main doc-only PR #772 (`956cd7c7→839a52e9`) — playbook/skill/routing/clean-room/models/coordination/steward-§5b all on main; (2) `[agents.adversary]` added to `moot.toml` (codex/gpt-5.6-sol/high + startup_prompt), confirmed via `moot config show`; (3) convo actor `agt_37vnwmcdxhw00` + key in `.moot/actors.json`. **✅ SEAT LIVE + ORIENTED (operator launched @15:48):** `adversary joined the space`/`connected` (evt_2xzas7rqhapv6/evt_7nd178a52bvzr); `moot-adversary` tmux + `.worktrees/adversary` present; running `gpt-5.6-sol high` (T1). It self-oriented from the startup_prompt + playbook CORRECTLY — status "ready — standing by event-driven for Steward/publisher merge notifications; grounded repro+file:line+invariant findings only"; subscribed to space; self-summary confirms "No polling, merge gating, fixes, or direct team-thread posts" (matches the playbook exactly). Idle-waiting for a CODE merge to hunt (recent merges PR#770-772 doc-only → nothing substantive; PX8-H not landed yet — correct). **Adversary fully operational.** My standing role now: triage its findings per steward.md §5b when they arrive on the side thread. | operator directive + PAT | n/a (in progress) |
| ~14:4x | **OPERATOR count-anchor for §5a: the recent research pull discharged the 6th-hard-stop trigger — do NOT re-escalate at 7th/8th; next re-trigger = 9th.** Pat corrected my numbering: the just-ruled hard-stop is the **6th** (I'd labeled it 7th; operator's count is authoritative). Since research was just pulled in for this window (`evt_mfsc2rkzjvs1`), §5a does NOT fire again at the next couple hard-stops — the mechanical every-3rd trigger next fires at the **9th** (with refined research focus). Anchored in the tracker RESUME banner so a watchdog-tick/cold-resume me doesn't mis-fire. Behaviorally consistent with my prior note ("next re-trigger reaching the 9th"); this pins the LABEL so the count stays correct. | operator directive (count anchor) | n/a (guidance) |
| ~14:3x | **PX8-H 7th ruling LANDED (`evt_8drwxa3qcm93` @14:15) — active continuation spine; §5a escalation pattern validated in real time.** Architect (41m empirical tracing on preserved T1) REJECTED the implementer's proposed `Let→Match(Call Var)` widening — the live erased scrutinees are `Var` not `Call Var`, no reaching `Match(Call Var)` exists — and instead ruled: (A) move provenance to the logical `ComputationalEliminatorFrame` (mint once/frame, IH nodes copy, no per-node re-mint); (B) thread a compiler-private ACTIVE CONTINUATION SPINE through the producer/value-lowering API, split at maximal shared outer ancestry by frame provenance, nested call owns unique inner only, caller resumes shared outer once. **The research advisory (`evt_mfsc2rkzjvs1`) measurably shaped the ruling** — Architect explicitly adopted its broad "missing explicit evaluation-context ownership" diagnosis and rejected its narrow widening as non-reaching. That is the §5a escalation working exactly as intended on its first (Pat-prompted) use: an independent prior-art perspective redirected a 7-deep chain from a false premise (`Call Var` widening) to the right structural reframe (active-spine). Implementer picked up cleanly, now Working the active-spine build (`EliminatorFrame::Active(_)`, `PX8H SELECT provenance/ancestry`); origin px8h `1acee25d` (no candidate yet). NO STALL — healthy 8th-round build. Steward: transport-verified the ruling reached the implementer's turn (§2c step 7); out of the mechanism. Next §5a re-trigger would be a further hard-stop reaching the 9th (refined research focus); tracking the count mechanically. | mechanism=Architect; §5a pattern validated; LIVE convo+pane diagnosis | n/a (progressing) |
| ~14:2x | **OPERATOR PROCESS DIRECTIVE: hard-stop-chain → research-advisory escalation, now codified.** Pat, watching PX8-H iterate through 7 hard-stops without a clear path, ruled a standing pattern: on the **3rd** hard-stop of an Architect↔implementer mechanism chain (then every **3rd**: 6th, 9th…), the Steward (1) **fully holds the Architect** (not timeboxed — its tokens are better spent with the advisory's context than on another unaided pass) and (2) kicks the **research agent in-thread, transport/framing only** (WP name + thread_id + hard-stop evt_…s + latest checkpoint SHA + the one circling question; ask for prior-art in refs[both tiers]+internet; NO Steward design opinion). Research posts its advisory **back in the same thread** mentioning Architect+Steward (advisory, not a ruling) → that mention is the Architect's resume signal. Never earlier than the 3rd (1–2 stops self-resolve); a *progressing* chain still triggers; never re-kick at the 4th/5th. Codified: `steward.md §5a` (new), `research.md` (aux-report exception + advisory bullet — swept the "never inject into a team thread" contradiction), fleet memory `hard-stop-chain-escalate-to-research-advisory` + README row. My assessment folded two refinements Pat accepted/overrode: I proposed a *bounded* hold — Pat overrode to a **full** hold (token efficiency); I proposed "floor, escalate earlier if thrashing" — Pat set **3 as the hard earliest** + every-3rd re-trigger. Lanes unchanged (Steward=transport+hold; research=advisory; Architect=ruling). PX8-H itself: advisory already in-thread (Pat prompted it `evt_mfsc2rkzjvs1`); no new escalation needed, architect Working the 7th ruling with it in-thread. | operator process directive; sibling of always-compact-at-seam (mechanical count defeats "one more round") | n/a (codified) |
| ~14:0x | **OPERATOR DIRECTIVE (genuine, first this session): grant the research agent read access to `local/refs/` — DONE + landed on main.** Pat: "update the clean room rules to allow the research agent to read local/refs/." Asked one crisp fork (permissive-only vs both tiers, since copyleft carries the leakage-taint that is load-bearing); operator chose **both tiers** (same footing as the Architect/Spec enclave, bound by the same leakage recheck). Edited `CLEAN-ROOM.md` (permissive + copyleft reader sanction now names research; recheck binds research as it binds the enclave), `AGENTS.md`/`CLAUDE.md` (mirror in the reference-material summary), and `agent/playbooks/federation/research.md` (rewrote the clean-room bullet from "off-limits, route to enclave" → "may read to understand, both tiers, never vendor/copy, leakage recheck binds you"). Invariants explicitly UNCHANGED: Yon (AGPLv3) stays excluded/unmounted; implementers still build from `/spec` only; nothing vendored. Committed `c33ebb56` on steward/work; published doc-only via scripted publisher (clean branch off origin/main, cherry-pick `2bf615c9`, `--doc-only`) → **PR #770 merged, `origin/main fe6e5360 → 3ee74ca8`**; verified all three files carry the grant on main; temp worktree + synthetic branch cleaned up. In-lane (I own the practice corpus + clean-room routing); did NOT touch PX8-H. | operator directive; PRINCIPLES (honesty about the boundary) + CLEAN-ROOM leakage discipline preserved | n/a (done) |
| ~13:5x | **PX8-H 7th hard-stop (Let-quotient zero-hit; real seam = ordinary Match(Call Var)) → Architect; Steward cleared a T1-downgrade safety-check modal on the architect seat.** Implementer built ruling-6 provenance (compiles `fbe35572`) but found the authorized direct-Call/direct-Call `Let` quotient is a ZERO-HIT seam (0 overlay hits for both `lower_computational_producer_expr::Let` and `lower_expr::Let`); the real path is standalone ordinary `lower_expr::Call` arms (layers 1,3,7,5) → ordinary `Match(Call Var)` receiving `ProcessExitStatus` (the original defect shape: env-resolved recursor feeding a known ordinary match). Asked the Architect to authorize `Let→Match` caller-provenance widening (`evt_4y4xyxcbfwm8s`, mentions architect+leader; refused to widen without a ruling). The hard-stop reached the architect and it engaged (ran `git show fbe35572:cranelift_backend.rs`) but its Codex seat was **BLOCKED ~15min on a safety-check interstitial modal** ("Additional safety checks — 1. Retry with a faster model / 2. Keep waiting / 3. Learn more", cursor defaulting to the faster-model DOWNGRADE). Steward selected **"Keep waiting" (Down→Enter)** → architect back Working on `gpt-5.6-sol high`, **T1 preserved** for the hard ruling. This is a REAL stall (frontier blocked ~15min on an infra modal) correctly diagnosed + cleared without touching the mechanism. 7th sequential hard-stop; still progressing (checkpoints advance, failure strictly deeper). **NEW INFRA HAZARD (memory-worthy):** a Codex safety-check modal can block a T1 seat mid-ruling and its default action downgrades the model — the Steward must select "Keep waiting" to preserve T1 for hard soundness/design rulings, never let the faster-model default ride. | mechanism=Architect; infra unblock preserving T1 (MODELS.md) | n/a (unblocked) |
| ~17:3x–17:5x (07-18) | **PX8-H §5a escalation FIRED at the 9th hard-stop — worked as designed; then two operator-directed refinements codified + landed on main (PR #773 `047960ab`).** The chain reached 8→9→10 hard-stops on the delimited-continuation carrier. At the 9th (`evt_3f264sn6x9kqr`) I fired §5a mechanically: full Architect hold + research prior-art advisory kicked in-thread (`evt_4f93p7d1ffeta`, non-blocking). Research **affirmed** the 9th ruling's `owned_layers + resume_cursor` representation against five prior-art lines (Dybvig–PJ–Sabry, libmprompt, Koka, multicore-OCaml, Freer/ITrees) (`evt_3mtt3g4qc3ajm`); I discharged the hold (`evt_6yw94d6jwnrqe`, transport-repaired a stranded paste on the architect seat), and the held 10th ruled normally (`evt_4zn7k00v26p74`, cursor-replacement transition, durable `0ad13561`). Operator affirmed the every-3rd cadence and asked for two refinements, now in §5a: **(1) catch it pre-ruling** (poll a chain approaching a trigger tighter so the hold lands *before* the Architect rules — research shapes a ruling most when it informs it, cf. the 7th vs the affirm-after-the-fact 9th); **(2) marginal-value framing** for later re-triggers (12th+) — scope the ask to the exact new fork and license "prior art has nothing new — current approach is known-best" as a first-class answer, so a re-survey adds perspective instead of manufacturing novelty. Cadence itself unchanged. | operator process directive; doc-only publisher PR #773 | landed on main `047960ab` |
| ~18:1x–18:2x (07-18) | **OPERATOR DIRECTIVE: move the §5a hard-stop-chain research-advisory trigger INTO the Architect; Steward → backstop (PR #774 `30220a2d`).** Pat: "Does the architect have instructions to hold and call out to research every 3rd hard stop, or is that just you? … more efficient if it was automatic behavior by the architect, rather than a poll race by you." (Grounded: it *was* just me — architect.md had no §5a; research.md was "Steward-invoked".) Wired the trigger to the design authority: **architect.md new §1a** (Architect counts its own consecutive hard-stops; on every 3rd/6th/9th/12th, *before ruling*, self-holds → frames + calls research in-thread → rules on the advisory; count survives self-compaction via in-thread re-derivation; Steward tracker = count of record; operator anchors arrive via Steward). **steward.md §5a** reframed driver → **backstop** (authoritative count; catch a missed trigger by holding the old way + transport-only research kick; guarantee the no-poll advisory lands; record). **research.md** bullet flipped Steward-invoked → **Architect-invoked (Steward backstops)**. Fleet memory updated. Why better: hold lands pre-ruling by construction (kills the poll-race that made the 9th an affirm-after-the-fact), and the design authority frames a sharper question than a transport relay. Cadence unchanged. **Transition note:** the in-session Architect loads §1a only on its next re-orient; until then the **Steward backstop is the active §5a mechanism** (a 12th this session, if it precedes the Architect's reload, is caught by my watchdog — no gap). Pat explicitly **declined** codifying the deep-chain→operator meta-signal ("not as a meta signal yet") — not added. | operator process directive; doc-only publisher PR #774 | landed on main `30220a2d` |
| ~19:0x (07-18) | **OPERATOR ASK: investigate "breakdowns in thread discipline" — findings delivered, remedy path OPEN (awaiting Pat).** Investigated the live PX8-H thread `thr_37h5` (structural metadata via `get_recent_context detail=full` → jq). **Cause:** convo message vocabulary + reply tooling don't match the federation's actual acts, so agents substitute nearest-fit types + post around tooling rough edges; hygiene never written as law (scattered memory lessons, not one protocol). **Concrete breakdowns:** (1) `message_type` mislabel — Architect rulings typed `decision_propagated` (no real Decision object), advisories/asks typed `review_request`, everything else `status_update` catch-all; explicit `"message"` rejected yet is the omit-default. (2) durable-bookkeeping leaks into the work thread (each ruling + a separate "recorded at architect/work @X" status ≈ 2× messages). (3) transport strands (Codex paste-not-submitted; `reply_to` `'speaker_id'` error → post_response w/ ragged parent). Threading itself mostly intact. **Remedy split:** Steward codifies a COORDINATION.md thread-protocol (act→type mapping, keep-bookkeeping-out, mention discipline, summarize-long-chains); OPERATOR owns backend fixes (extend message_type enum with ruling/advisory/hard_stop/hold; fix reply_to + message-default; Codex paste-submit). **OPEN Q to Pat:** land convention now / hold for enum decision / both; + widen investigation if breakdowns are elsewhere. Not yet acted — awaiting Pat. | operator investigation ask | findings posted; remedy pending operator |
| ~19:1x (07-18) | **OPERATOR DIRECTIVE (thread discipline, finding #2): the Architect must STOP posting durable-state-update notifications to the work thread** (the "durable checkpoint … recorded on `architect/work @<sha>`" `status_update`s after each ruling — keep the git commits, just don't announce them). Instructed the Architect at root `evt_5syt4zswyb2vh`; it acknowledged (re-oriented + "incorporates the new thread-discipline rule"). **✅ CODIFIED in `architect.md §3`** ("Checkpoint durably, but do NOT announce it to the work thread") so it survives the Architect's own compactions — the convo instruction alone was context-only. The broader COORDINATION.md thread-protocol (act→type mapping, mention discipline, summarize-long-chains) still folds in once Pat rules the remedy path. | operator process directive | codified `architect.md §3`; broader protocol pending Pat |
| ~post-15th (07-18) | **PX8-H frame RECONCILED to the ruled mechanism (15th ruling's "amend the frame" directive; banked on `steward/work`).** After 15 Architect design rulings, the WP brief still framed the fix as ruling-1's "install the one missing intervening ordinary frame" (H-D1/H-D2) and Fixed-input-5 barred "new representation" — which the ruled compiler-private `NativeJoinPlan` carrier would read as violating at §14 (the exact stale-frame class that §14-BLOCKED PX8-L, `dec_79sg9emgfhthc`). Amended `docs/program/wp/PX8-H-*.md`: (a) new "Mechanism as ruled" § transcribing the 15-ruling chain **by citation** + the 15th ruling's contract + load-bearing invariant **verbatim** (`evt_7sdmvyme8qy50`); (b) Fixed-input-5 + Do-not-reopen guard AUTHORIZE the compiler-private checked-package join-plan carrier (sibling `NativeEntrypointPlanV1`, validated-not-invented) while still barring public/ABI/wire/kernel reps; (c) H-D4/H-D5 deliverables + H-P7 join-plan discriminators; (d) Landed-anchors add `compiler_driver.rs` + demoted `merge_scalar_branch`/`record_scalar_merge_kind`. **In-lane:** transcription of the Architect's ruled contract, not new design (mechanism authority stays the cited events). **LAND-TO-MAIN (doc-only) = first pre-§14 step when a candidate SHA releases**, so the candidate branch carries a current frame at §14; re-amend if a 16th/17th ruling refines the contract. | 15th ruling directive + §14 frame-currency (PX8-L precedent) | banked on `steward/work`; lands to main pre-§14 |

---

# ▶▶ 2026-07-21 WINDOW — narrative entries continue here

*(Navigation: the 07-21 window's header, standing inputs, queue, and decision
table are at the TOP of this file. The dated narrative entries below sit under
the 07-18 heading only because both windows append to the end of the file. Read
top block first, then here.)*

### 05:06 — `integrator` "PR #365 awaiting Steward routing" is a STALE STATUS, not work

The participant roster shows `integrator` (`agt_37reqx7jqz400`) with the status:
*"watchdog: PR #365 green on current head `befc2dc4…`, but exact-head
review/decision is stale after prior blocked heads; awaiting Steward routing."*
That reads as an unrouted obligation I own. It is not one. Verified, not assumed:

- `befc2dc4` is **`tools: add scripted PR auto-merge path`**, sitting on
  `NC27`/`NC26` — a **2026-07-08-era** commit line, not this window's work.
- `git merge-base --is-ancestor` says it is **not** an ancestor of `origin/main`
  — which by itself looks like "unmerged", and is exactly the trap.
- **Checked by CONTENT instead:** `git cat-file -p
  origin/main:scripts/scripted-pr-automerge.sh` returns the file. **The work
  landed** (squash-merged, so the original commits dangle — the documented
  squash-merge trap in the §2c gate note). The script is the very one my own
  playbook names as the canonical publisher path; it has been in use all night.
- `integrator` has **no `moot-integrator` tmux pane** — it is not a seated agent
  in this fleet. Nothing is waiting on a turn.

**Ruling: closed, no action.** Two standing lessons applied and both were
load-bearing here — *branch-ahead ⇏ unmerged, verify by content*
(`check-main-via-git-object-store-not-find`), and *a pane/participant status
line is not agent state* (`pane-suggestion-text-is-not-agent-state`). A status
string is a claim by a past turn about a past moment; it does not expire on its
own and it does not become false-looking when it goes stale. Had I taken it at
face value I would have opened a merge chain for already-merged work in the
middle of a single-threaded window.

### 05:08–05:20 — the watch item punted back; framed as **SPAN-SEAL** (Foundation)

**What happened.** The Architect's respin forced the fork I predicted at 04:28,
and it came back as a design question rather than a closure result — exactly the
§6 escape hatch the leader's brief provided for. QA blocked `506fa393`; the
leader escalated; the Architect ruled (`evt_1ppsszssn593s`): **seal the raw
transform, do not preserve it public, and do not merely add a `u64` bound
check.** The leader then routed it to me as a blocking pre-existing erratum
outside RT-PARITY scope, correctly refusing to absorb it.

**★ The report arrived as an overflow story, and the overflow story has a cheap
wrong fix.** The leader's escalation led with unbounded `start' = start + count`
composition reaching past `u64::MAX` — true, but astronomical, and it invites
"add a bound check." The Architect saw past the framing: the breach is available
**at small values**, because the two arguments are *not indexed to the same
request, span, or buffer*, so source can combine unrelated host-minted values
and choose a new start/budget. A `u64` check repairs the magnitude symptom and
leaves the abstraction breach intact. **Recorded because the cheap fix would
have passed a review that accepted the reporter's framing.**

**I verified the ruling's load-bearing facts rather than passing them through**
— three plausible arguments about this exact surface went unchecked tonight:
- helper installed `prelude.rs:2076`, **absent** from the closure list `:2111+`
  (read the list, not the line number);
- the **only** landed source consumer outside RT-PARITY is the catalog wrapper
  `catalog/…/IO.ken.md:22,24` — repo-wide grep;
- **no spec or conformance file names the helper or its proof.** Spec
  `38-ffi-io.md:356-365` locks the contract it violates.

⇒ **Code repair to conform to an already-locked spec. NOT a spec amendment, NOT
an ADR.** So the operator's "if PRINCIPLES doesn't settle it, HOLD the ADR" rule
**does not fire** — there is no ADR here, and nothing needs the enclave to start.

**★ I elevated one thing from "prefer" to load-bearing.** The Architect wrote
*"Prefer* making `private_write_all_fuel` call the single private advance
helper." Grounding it in the source shows it is stronger than a preference: the
real loop constructs its span **inline** at `prelude.rs:2044-2046`, so `writeAll`
**never executes** `write_all_advance_span` — and therefore the *published*
catalog lemma `write_all_preserves_exact_prefix` proves a property of a faithful
restatement of the step rather than of the step itself. That is a
proof-about-a-proxy defect sitting underneath the capability leak, and it is why
SPAN-SEAL is a **net strengthening, not a removal**. Flagged the elaboration
hazard too: the loop spells the budget `(Suc remaining)`, the helper spells it
`transfer_count_remaining count` — definitionally equal *only inside that match
arm*. The frame requires that refactor be **shown** to elaborate, not argued to.

**★ AC-3 is the criterion that matters, and it is why this recurred.**
`px8f_buffer_io_surface.rs:59-67` **already** asserts a private-name list, and
the escaped helper is not in it — the list enumerates what the author remembered
to seal, not what must be sealed. So the producer passed a test whose stated
purpose was to catch exactly this. The same shape then repeated one layer up in
RT-PARITY's unreachability argument. **A better allowlist leaves the defect one
addition away from recurring**, so AC-3 requires the closure be **derived** from
the elaborated environment (`env.globals` → `GlobalEnv::lookup` → walk the Pi
codomain → collect every `BufferSpan`-returning global) and compared to an exact
expected set. I confirmed that is buildable before writing the AC —
`GlobalEnv::lookup`/`decls()` are public — so the AC is not unbuildable prose.
A source grep would not do: the prelude is **Rust-emitted**, not `.ken`.

**Owner: Team Foundation.** Grounded, not guessed — `be65f3d2 PX8-F: buffer/IO
surface + writeAll` introduced **both** the prelude helper and the catalog lemma
block. Foundation authored the surface and owns it. The Architect has already
ruled the *shape*, so this is execution against a specified design — the
shovel-ready model, appropriate for the ring's tier.

| Decision | Basis |
|---|---|
| **SPAN-SEAL framed, owner Foundation, one branch/one Decision.** | Architect ruling + PX8-F provenance |
| **Required votes: QA + Architect §14 + CV** — the diff changes a *published catalog surface*, so diff-scope pulls CV. | standing gate model |
| **RT-PARITY held at `506fa393`** until SPAN-SEAL lands, then rebase/respin. | Architect; single-threaded fleet |
| **Held doc branches released for publish.** The reason for holding (don't rebase a fragile ring mid-flight) is **gone** — RT-PARITY is now held pending SPAN-SEAL and will rebase regardless. | reassessed on changed facts |
| **Did NOT preemptively reseat Foundation off OpenAI (13%).** Operator's instruction is reactive — *"if an OpenAI seat exhausts, reseat it."* Preemptively moving three seats is a topology change without a trigger, beyond "route around." Monitoring. | operator's actual words |

### 05:08–05:21 — three publishes, a dead seat repaired, SPAN-SEAL kicked

**Published all three held doc branches** (the hold reason was gone — RT-PARITY
is held pending SPAN-SEAL and will rebase regardless):

| PR | Branch | Verified on `origin/main` **by content** |
|---|---|---|
| #793 | `wp/MODELS-TIER-erratum` | `agent/MODELS.md` carries the DEFAULT/inverted/observe text |
| #794 | `wp/STR-BIJ-frame` | `docs/program/wp/str-bij-overclaim-erratum.md` present |
| #795 | `wp/SPAN-SEAL-frame` | brief + bundled tracker sync present |

`origin/main` **`61a78620 → 31b7fda5`**. All doc-only ⇒ **no adversary
notification** (that obligation is on **code** merges); recorded so the omission
reads as a ruling rather than a lapse. **Held-branch list is now empty.**

### ★ ~05:15 — `foundation-qa` was DEAD on an invalid model string. New stall class.

Gate step 5 (verify the drops) is what caught it — and it is exactly why that
step is mine and not the script's. Two seats showed `Context compacted`; the
third showed:

> `400 invalid_request_error: The 'gpt-5.6-terras' model is not supported when
> using Codex with a ChatGPT account.`

**`gpt-5.6-terras`** — a plural typo. Every turn that seat took was 400ing,
including its own `/compact`. It had been sitting there reading as a normal
idle seat.

**`moot.toml` was FINE** (`model = "gpt-5.5"`) — the bad value was an
**in-session override**, so diagnosing from config would have found nothing and
concluded the seat was healthy. **The live model is in the pane footer; that is
the only place this is visible.**

**Repaired without the operator**, per the standing latitude to route around
problems: drove the in-pane `/model` selector over tmux → `gpt-5.6-terra medium`
(matching the config's intended T2/medium; the typo makes the intent
unambiguous) → confirmed `Model changed to gpt-5.6-terra medium` → **re-ran that
seat's compaction**, since the failed one had done nothing. Now `Context
compacted`.

**Why this mattered rather than being a curiosity:** SPAN-SEAL requires a QA
vote from that seat. Had I trusted the script's exit 0 and skipped step 5, the
kickoff would have gone out to a ring with a silently dead QA seat and the WP
would have stalled with no diagnosable cause. Saved as memory
(`seat-on-invalid-model-string-400s-every-turn`) — it is the **third** distinct
way a seat dies while merely looking quiet, alongside provider content refusal
and model-at-capacity.

**Also observed: `foundation-implementer` is `gpt-5.6-sol` = T1**, leader T2 —
the *same inversion* as Runtime. Two teams now, so implementer-T1/leader-T2 is
the pattern for hard-build teams, not a Runtime quirk. Memory updated.

### 05:20:46 — SPAN-SEAL kicked; Handoff Gate run in full

Gate: retros in ✓ · no in-flight obligation ✓ · quiescent ✓ · all three
compacted (`handoff-gate-compact.sh`, branch-ahead verified **0** on each before
running, so the hard reset risked nothing) ✓ · **drops verified individually**
✓ (this is where the dead seat surfaced) · kickoff posted `evt_4pgqybwbkfssd` ·
**delivery confirmed — leader observed `Working`**, not merely a returned
`event_id`.

Kickoff carried the three traps explicitly: don't take the cheap `u64` fix;
verify the loop refactor elaborates rather than arguing it does; and derive
AC-3's closure instead of extending the allowlist that caused this. Also told
the leader about its QA seat's repair so it has the history rather than
diagnosing it cold.

⚠ Told the ring to **`git fetch` before branching** — the gate reset their
worktrees to `61a78620`, then I published three branches on top, so the brief
did not exist at their HEAD. That ordering hazard is inherent to overlapping the
gate with publishes; the overlap was still right (it saved ~5 min), but the
fetch instruction is **mandatory** whenever they're overlapped.

### ❌ 05:36 — MY ERROR: I logged estimated wall-clock times, and they were ~15 min FAST

The three headings above originally read `05:22–05:40`, `05:30`, and `05:38`.
The actual kickoff timestamp is **`05:20:46`** (`evt_4pgqybwbkfssd`), and the
clock read **05:36** when I noticed — so I had written a *future* time into the
record and dated a completed action after the moment I was writing about it.

**Cause:** I anchored on my own sense of elapsed effort instead of reading the
clock, exactly the failure `silence-duration-anchor-on-actual-clock-not-last-
event` names. It is more dangerous in a decision log than in a stall diagnosis,
because these timestamps are what a returning operator uses to reconstruct
order, and a drifted one silently misorders cause and effect.

**Corrected** against event ids and `date -u`, which are the only two anchors
that cannot drift. Rule for the rest of this window: **every logged time comes
from an event id or `date -u`, never from estimation.** The watchdog prompt
inherited the same bad "as of 05:40Z" stamp and is corrected on next re-arm.

### 05:23–05:35 — SPAN-SEAL candidate `8f625666` delivered; QA in flight

Twelve minutes from kickoff to candidate. Ring behaved well: the implementer hit
a **branch-custody collision** (leader still holding
`wp/SPAN-SEAL-span-producer-closure`), named it precisely, and the leader
released within 14 seconds — no Steward intervention needed. That is the
`wp-branch-handoff-deadlock-leader-holds` trap resolving *correctly* for once.

**The implementer met AC-3 in its strongest available form.** I asked for a
derived closure compared to "an exact expected set"; it derived the set and
showed the expected set is **`{}`** — no public `BufferSpan`-returning global at
all. That is strictly better than what I specified, and it is the honest result
rather than a convenient one.

Reported: `private_write_all_fuel` now calls the sole private transition; the
**elaboration hazard is closed empirically** (the real recursion elaborates with
`write_all_advance_span span count`, so the branch-local `(Suc remaining)` /
`transfer_count_remaining count` definitional equality is *accepted*, not
argued); public surface is an observer-only proposition + checked lemma; both
private names pinned rejecting with exact `ElabError::UnresolvedCon` names;
positive control sources span/count only from `ReadSome` and reaches `writeAt` /
`freeze` / `writeAll`. **No `u64` bound was added** — the trap was avoided.

Scope three paths, +125/-11; runtime/host diff empty; targeted suites only.

**Steward action: none.** Leader routed to QA at `05:35:32` with a
well-specified brief that explicitly asks QA to check the closure oracle *truly
fails for a new producer* rather than being a prose/grep oracle — the right
question, and the one I would have added. Awaiting QA, then Architect §14 + CV.

### ❌❌ 05:40 — MY ERROR: three commits landed on the WRONG BRANCH (recovered, nothing published)

**What happened.** To publish the SPAN-SEAL frame I ran `git checkout
wp/SPAN-SEAL-frame` — and **never returned to `steward/work`.** The next three
operations then ran against the wrong branch:

- `ae6ed71d` and `cc51b7e0` (two decision-log appends) committed onto
  `wp/SPAN-SEAL-frame`;
- worse, that branch was cut from `origin/main @ 61a78620`, whose
  `STEWARD-DECISION-LOG.md` is the **stale 2026-07-18 version** — so ~121 lines
  of this window's record were appended to a file that does not contain this
  window. The 07-21 header was not there; the entries were orphaned.

**How it surfaced.** Not by my noticing. A tool result told me the file "had
been modified on disk," and the diff it showed started
`# Steward decision log — autonomous window 2026-07-18` — a line that cannot be
line 1 of the file I thought I was editing. **The harness caught this, not me.**

**Blast radius — bounded, and verified rather than assumed.** PR #795 published
`ce066d63`; both stray commits are strictly **after** it, so nothing wrong was
ever published. Grepped `origin/main` for both commit subjects → **0 hits**.
`origin/main = 31b7fda5` is correct and the three landed frames are intact.

**Recovery.** Extracted the strayed range as a pure-addition diff (verified **0
deletions**, so nothing was overwritten), preserving blank lines — a first pass
with `grep '^+[^+]'` silently dropped them (121 → 95 lines), which would have
mangled every paragraph break. Reset `wp/SPAN-SEAL-frame` back to the exact
published `ce066d63` so the branch matches what shipped. Appended the recovered
text to the real log on `steward/work`; verified single-occurrence, no
duplication, headings intact.

**Cause and rule.** I have a memory for precisely this —
`verify-branch-after-checkout-away-and-back` — and did not apply it. The trap is
that `git commit` **succeeds** on the wrong branch and prints a plausible SHA,
so there is no failure signal; the only tell is content, and I was appending
(never reading) so I never saw the wrong content. **Rule for the rest of this
window: every publish sequence ends with an explicit `git checkout steward/work`
in the same command, and any log/tracker write is preceded by `git rev-parse
--abbrev-ref HEAD`.**

**Second-order note.** This is the same failure family as the two errors above
it: I trusted an action's *success signal* (a commit SHA, an exit 0, a "sent"
line) as evidence about a *property* it does not establish. That is now three
distinct instances tonight — the publisher exit code, the gate script's exit 0,
and now `git commit`. The general form is worth carrying: **a command
succeeding tells you it ran, never that it did the thing you meant.**

### 05:42–05:55 — CV blocked SPAN-SEAL `8f625666`; ★ the defect recurred INSIDE MY OWN AC

**Chain ran clean and fast.** QA approved `8f625666` at 05:42 (including its own
reversible closure-oracle mutation — it removed only the helper's private-name
closure and confirmed the derived set became `{write_all_advance_span}`, so QA
proved the oracle *reaches*). Leader opened `dec_4nkaf54v5vkyh`. **CV BLOCKED at
05:49**, terminally.

**CV's finding — and it is excellent.** AC-3's oracle derived the producer set
**syntactically**: strip raw `Term::Pi` codomains, raw-match
`Term::IndFormer{ id == BufferSpan }`. CV demonstrated the bypass with a
*reaching* counterexample rather than an argument:

```ken
def BufferSpanAlias = BufferSpan
fn escaped_alias (span : BufferSpan) (count : TransferCount)
  : BufferSpanAlias = write_all_advance_span span count
```

Public, elaborates, genuinely produces a `BufferSpan` through the private
transition — and the derived set stayed **`{}`**. So the oracle did not satisfy
the brief's own "a newly added producer must fail by default."

**★★ The lesson, and it lands on me.** This is the **third** occurrence of one
defect on one surface, each a layer deeper:

| # | Artifact | Why it failed |
|---|---|---|
| 1 | the private-name allowlist in `px8f_buffer_io_surface.rs` | enumerated what someone remembered to seal |
| 2 | RT-PARITY's BufferFreeze unreachability argument | enumerated constructor *names*, not producers |
| 3 | **AC-3's first implementation** | enumerated *spellings* of the result type, not its meaning |

**A syntactic head-match is still an enumeration in disguise.** And #3 is my
fault, not the ring's: AC-3 as I wrote it said *"walk each decl's type through
its Pi codomain to the head and compare."* That is a **mechanism**, and I called
it a property. The implementer built precisely what I specified. **I wrote an AC
to prevent an enumeration defect and expressed it in a form that admitted a
subtler one.**

**General rule, recorded:** *"derived" is only as strong as **derived modulo
what**.* Name the equivalence the closure must hold under — here Ken's
definitional equality — or a reader will reasonably choose the cheapest
equivalence the words permit.

**Action taken: amended the frame and landed it mid-respin (PR #796,
`origin/main 31b7fda5 → e119e6a7`, verified by content).** AC-3 now demands
weak-head reduction against the real `GlobalEnv` before the Pi decision and
after every codomain step, comparing the reduced head, covering
whole-function-type aliases, with CV's alias producer required to appear in the
derived set.

**Why mid-respin rather than after:** a candidate reviewed at §14 against a brief
specifying a now-known-insufficient method is exactly the stale-frame
contradiction that **§14-blocked PX8-L** in an earlier window — mechanism sound,
sunk by its own frame's words. Landing the amendment before the new SHA's §14
means the candidate is judged against a frame that agrees with it. Told the ring
explicitly that nothing in their in-flight correction changes and that I added
nothing beyond CV's requirement.

**Ring health: very good.** Branch-custody collisions twice (leader, then QA
holding the branch); both named precisely and released in **under 15 seconds**
without Steward involvement. Leader's respin brief transcribed CV's requirement
faithfully and added the right guardrail — *"do not alter the sealed product
surface merely to accommodate the test."*

**Steward action beyond the frame amendment: none.** Fresh QA + Architect §14 +
CV required on the new SHA; no prior vote carries.

### 06:00–06:10 — respin `b057cc1e` (strong); ★ Architect was TRANSPORT-STRANDED, repaired

**Respin delivered 06:00:09, QA APPROVED 06:03:55.** Twelve minutes CV-block →
QA-approved. The correction is the real thing, not a patch over the symptom:

- The oracle now calls kernel `whnf` against the real `GlobalEnv` **before Pi
  classification** and **after every codomain**, carrying a `Context` and
  pushing each Pi domain before reducing a dependent codomain. That is the
  dependent-type-correct form, not just "reduce the result."
- **A permanent TWO-ARM discriminator**, derived set exactly
  `{escaped_result_alias, escaped_function_alias}` against a production set of
  `{}`. One arm needs **post-codomain** reduction, the other needs **pre-Pi**
  reduction.
- **QA proved both arms causal rather than trusting their names:** two
  independent reversible mutations — replacing post-codomain `whnf` with the raw
  codomain loses **only** `escaped_result_alias`; replacing pre-Pi `whnf` with
  the raw type loses **only** `escaped_function_alias`. Restored cleanly.

That is exactly the `two-arm-producer-needs-a-case-per-arm` discipline, executed
without being told. After three rounds of this defect the ring is now producing
the strongest form of the evidence on its own.

### ★ 06:04–06:10 — the Architect never received the Decision request (transport)

`dec_61psqte1sfegf` was opened at 06:04:19 mentioning the Architect. CV picked
up 20 seconds later and went `Working`. **The Architect showed nothing.**

Captured its pane **wide** — and found the documented failure: the composer held
**stacked `[Pasted Content …]` blocks with no `Working`**. The Decision request
was delivered to the buffer and **never submitted**. Worse, it was *stacked*:
two earlier pastes (2757 and 2048 chars) sat beneath it, so my 05:53 frame-
amendment notice had almost certainly been stranded too — roughly **17 minutes**
of a required §14 reviewer silently holding unread work.

**Repair: a bare `Enter`** to that pane (playbook §2c step-7 table). Architect
went `Working` within 5 seconds.

**Why this one mattered.** The seat's *status* still read `ready`, and under
single-threading an idle Architect looks correct. Nothing would have surfaced
it: the leader had done everything right, `post_response` returned a valid
`event_id`, and CV was progressing normally — so the Decision would simply have
sat one vote short, indefinitely, with every participant believing it was
someone else's turn. **`event_id` proves the event exists; it never proves an
agent read it.** This is the fourth-plus occurrence on *this specific seat*
(three on 2026-07-14), which is why the check is in the gate rather than left to
vigilance — and why a **wide** capture is mandatory: `tail -5` would have shown
a bare prompt and I would have called it idle.

**Standing correction to my own watchdog practice:** I check liveness on the
*working* ring, but a §14/CV reviewer that has been *mentioned* is equally a
seat that can strand. From here: after any Decision opens, confirm **every named
reviewer** reached `Working`, not just that the Decision exists.

### 06:12–06:22 — Architect BLOCK #4 on the same shape; AC-3 restated as a property (PR #797)

**CV APPROVED `b057cc1e` at 06:10:57** (independently re-proving both alias arms
causal). **Architect BLOCKED at 06:12:40**, terminal, on a genuinely new hole:

> `public_buffer_span_producers` begins each arm with `env.env.lookup(*id)?`.
> `GlobalEnv::lookup` resolves only top-level declarations (`env.rs:342`).
> Constructor IDs live in `ctor_index` behind `GlobalEnv::constructor`
> (`env.rs:404`) — so **every public constructor took the `?` path and vanished
> from the derived set unexamined.**

**★★★ This is the FOURTH occurrence of one defect shape on one surface**, and
the fourth is mine in a way the others were not:

| # | Artifact | Enumerated… | Whose |
|---|---|---|---|
| 1 | private-name allowlist | what someone remembered to seal | pre-existing (PX8-F) |
| 2 | BufferFreeze unreachability | constructor *names* | Runtime ring |
| 3 | AC-3 first oracle | *spellings* of the result type | **my AC (mechanism-as-property)** |
| 4 | AC-3 second oracle | one *category* of global | **my AC (named an API)** |

**The drafting diagnosis, stated plainly.** My AC-3 said *"look up each
`GlobalId` via `GlobalEnv::lookup`."* **I named an API and called it a
property.** The implementer used exactly that API and inherited its blind spot
— while being entirely correct against my words. And I had grepped the kernel
accessors while drafting: **`constructor` at `env.rs:404` was in my own search
output**, and I still wrote only `lookup`. The information needed to avoid
breach 4 was in my context at the moment I created it.

**General lesson (the real deliverable of this WP, beyond the code):** *an
acceptance criterion that specifies a **mechanism** transfers that mechanism's
blind spots into the deliverable, invisibly — because the result passes review
against its own text.* State the **property**, name the **axes** closure must
hold along, require a **loud failure on the unhandled case**, and let the
implementer choose the mechanism. Every layer here except the first was
introduced by a spec that described *how* instead of *what*.

**Action: second frame amendment, landed (PR #797, `origin/main e119e6a7 →
d18b4b89`, verified by content).** AC-3 is now a property closed along three
axes: (1) modulo definitional equality; (2) over **every category** of public
global; (3) **★ loud failure on any id resolving to neither known category.**

**Axis 3 is my addition on top of the ruling, and it is what terminates the
chase.** Axes 1 and 2 were each found by a reviewer after the fact; enumerating
categories keeps losing to the next unenumerated one. Loud-fail-on-unknown
converts *"a category nobody thought of passes silently"* into *"a category
nobody thought of breaks the build"* — closed independently of the author's
imagination. The Architect required it as an implementation detail; I promoted
it to the AC's load-bearing clause and said so in-thread rather than smuggling
it in.

**Frame amended BEFORE the next §14** again — the leader independently asked for
exactly this (`evt_z7tep5a39785`) while I was already drafting it, which is a
good sign the stale-frame discipline has propagated to the ring.

**Chain watch:** two terminal blocks on SPAN-SEAL (CV, then Architect). Not yet
a §5a research-advisory trigger (that counts *Architect* hard-stops, currently
1). But if a third block lands on this same AC, the correct move is **not**
another amendment — it is to question whether a test-derived closure is the
right instrument at all, and route that to the Architect as a design question.
Recording the trigger now so a post-compaction me does not simply amend again.

### 06:21–06:24 — candidate `02677059` (three axes, each with a reaching discriminator)

Implementer delivered 06:21:26, ~7 minutes after the block. All three axes
covered, and — the part that matters — **each axis has a permanent
discriminator justified by what the OLD oracle would have done**:

| Axis | Fixture | Why it reaches |
|---|---|---|
| defeq | alias fixture, exact `{escaped_function_alias, escaped_result_alias}` | one arm needs pre-Pi `whnf`, the other post-codomain |
| categories | re-exposes the **actual sealed `PrivateBufferSpan` ctor id** under synthetic public name; asserts `lookup(id).is_none()` **and** `constructor(id).is_some()` | *"the old declaration-only oracle would derive `{}` and fail this test"* |
| unknown | inserts `escaped_unknown_id -> GlobalId(u32::MAX)`, `#[should_panic]` pinned to exact panic text | *"the old silent `lookup(*id)?` path would not panic, so this test would fail"* |

That last column is the discipline I have been asking for all night, produced
unprompted: a discriminator is only evidence if it **fails on the thing it
claims to exclude**, and each one here is justified against the specific prior
mechanism. `px8f_buffer_io_surface` 7/7. Scope still three paths, +212/-11;
runtime/host empty. QA in flight; Architect + CV to follow.

### ❌ 06:23 — MY ERROR (caught before acting): a `400` health-check false positive

My watchdog liveness sweep flagged `conformance-validator` with `400` — the
invalid-model stall class I had logged an hour earlier. **It was a false
positive.** `400` matched as a bare substring inside **agent IDs**
(`agt_37rekz81pp400`, `agt_37reqfp1tm400`) printed in the pane's participant
list. CV was entirely healthy: it had re-oriented on the amended frame,
correctly marked **its own prior APPROVE stale**, and set status to await the
fresh candidate — exemplary behavior.

**Caught it the right way:** I captured the pane and read it *before* concluding
anything, then grepped for where `400` actually matched, then checked for the
real signature (`invalid_request_error` / `is not supported when using`) → **0
hits**. No action taken, nothing posted, no seat disturbed.

**But the detector was bad and is now fixed.** `400` is a terrible pattern —
it matches agent IDs, SHAs, token counts, durations. Replaced with the anchored
signature `invalid_request_error|is not supported when using|"status":400`.

**This is the THIRD detector-quality failure of the window** (verb-enumeration
false IDLE; narrow `tail -5` false stall; now a loose substring false error) —
and they rhyme with the four-layer AC defect I spent the last hour on. **My
monitoring patterns fail the same way my acceptance criteria do: they match a
convenient proxy instead of deciding the actual property.** The corrective is
identical in both places — anchor on a signature that cannot occur by
coincidence, and prefer a structural match over a substring.

The one thing that went right: **verify-before-acting held.** A false positive
that is checked costs a pane capture; one that is acted on costs a needless
rouse and, if I had "repaired" a healthy seat's model, real damage.

### 06:26–06:35 — SPAN-SEAL APPROVED (all three lanes) and PUBLISHING as PR #798

**Decision `dec_1971te8pb7spb` verified RESOLVED by reading the Decision object,
not the merge request.** `status: resolved`; resolution APPROVED at exact
`02677059` on base `d18b4b89`; resolved_by CV `agt_37reqfr97xm00` at 06:30:55;
QA + Architect §14 + CV all recorded against that SHA. Also verified `d18b4b89`
is an ancestor of the candidate, and that my local branch ref sits exactly on
`02677059` (the stale-local-branch force-push hazard).

**★ DELIBERATE PLAYBOOK DEVIATION — published WITHOUT the §2a tracker bundle.**
§2a normally has me add a tracker-sync commit to the candidate before
publishing. **After four terminal blocks tonight, every one turning on
exact-SHA fidelity, adding even a doc-only commit would merge a SHA that none
of the three reviewers voted on.** Published `02677059` byte-for-byte; tracker
syncs in a separate doc-only publish after. Same precedent as PX8-N earlier in
this window. Stated the deviation in-thread so it reads as a decision, not an
oversight.

### ★ 06:34 — a task notification said "completed exit 0" while the publisher was still running

The background-task notification for the publish reported **`status:
completed`, exit code 0** — and the publisher was very much alive: the log read
`Waiting 1557s before polling PR #798 checks`, and `ps` confirmed one live
process. The notification was for the `nohup … &` **wrapper**, not the job.

**A textbook instance of tonight's recurring rule**, and the fourth distinct
tool to produce it: publisher exit code, gate script exit 0, `git commit` on the
wrong branch, and now a task-completion notification. **A success signal tells
you a thing ran; it never tells you the thing you cared about is done.** Had I
taken it at face value I would have gone looking for a merge that had not
happened, found `origin/main` unchanged, and plausibly concluded the *publish*
failed — inventing a problem out of a misread signal.

**Verification that actually settles it:** the publisher's own log tail plus
`ps`, then — at the end — `origin/main` **by content**. Recorded in the watchdog.

### 06:40 — full-fleet anchored health sweep (25 seats): CLEAN

Ran the anchored dead/stranded signatures across every `moot-*` session while
CI runs. **No dead seat, no stranded mention, nothing needing repair.** Worth
doing: tonight already produced one dead seat (`foundation-qa`, invalid model)
and one stranded reviewer (`architect`, ~17 min), neither of which announced
itself.

**The one flag was my OWN pane** — `❌DEAD ⚠STRANDED` on `moot-steward`. A
self-match: my pane contains the literal signature strings because I have been
writing them into the watchdog prompt and this log. Harmless, but a clean
reminder that **a detector run over its own output matches its own
description.** Exclude self from future sweeps.

### ★ 06:40 — the sweep CORRECTED a memory I had written twice tonight

I recorded Runtime's seating as an "inversion" and Foundation as a "second
instance." **Both framings were wrong.** The sweep shows the pattern is the
**fleet-wide convention**, uniform across all six build teams:

- `<team>-implementer` → `gpt-5.6-sol` = **T1**
- `<team>-leader` → `gpt-5.6-terra` = T2
- `<team>-qa` → `gpt-5.6-terra` = T2
- enclave (`architect`, `conformance-validator`, `spec-author`, `research`) →
  **sol = T1**; `spec-leader`, `librarian` → terra = T2

So the real rule is **implementers and the enclave get T1; leaders and QA get
T2** — and `agent/MODELS.md`'s Roles column simply does not describe the landed
fleet for build teams. Anthropic seats: `runtime-implementer` and `adversary`
(Opus 4.8) only; **every other seat is on the OpenAI pool**, which is the
tighter one at 13% — a real concentration risk worth flagging to the operator.

**Why I got it wrong twice:** I generalized from single observations (Runtime,
then Foundation) instead of enumerating the population, when enumerating it cost
one command. That is *the same defect I have spent this entire window
correcting in other people's work* — reasoning from a representative case rather
than deciding the property over the whole set. It is a good deal easier to see
in an acceptance criterion than in my own belief.

Memory `tier-table-is-a-default-runtime-seats-t1-implementer` corrected. The
landed `MODELS.md` erratum (PR #793) is still accurate as far as it goes — "the
table is a default, observe the seat" — but its Runtime example understates the
scope. **Not amending it now**: fleet is single-threaded, SPAN-SEAL is mid-CI,
and this is a documentation-precision issue with no operational consequence
while the observe-don't-infer rule stands. Queued behind the current chain.

### 07:04–07:08 — SPAN-SEAL MERGED `cd4184b8`; full merge chain executed

**PR #798 CI green, merged. Verified BY CONTENT, not by the "merged" line:**
helper in the private-name closure (1); public proposition present (3); catalog
names the proposition (2) and the raw helper is **gone** (0); three-axis oracle
discriminators present (6); `crates/ken-runtime/` + `crates/ken-host/` **zero-line
diff**. `origin/main d18b4b89 → cd4184b8`.

**Chain executed in order, none skipped:**
1. ✅ verified by content
2. ✅ **adversary notified** (`evt_1nzfry4kw5pt8`) — pointed first at the closure
   oracle, *because that claim has already been wrong three times tonight*, and
   told it plainly that RT-PARITY's respin will depend on it. Confirmed engaged.
3. ✅ §10 retros requested from Foundation (`evt_1qs6q1n501v8w`) — carrying both
   the four-layer lesson and **my own AC-names-a-mechanism error**, plus four
   ring practices worth spreading.
4. ⏳ **SPAN-SEAL is `merged`, NOT `closed`** — closes when retros are in.
5. ✅ **RT-PARITY released** (`evt_2qwd9x2s8z8p5`), runtime-leader confirmed
   `Working`.

**★ In the RT-PARITY release I explicitly forbade an overclaim.** Its old basis
("`PrivateBufferSpan` is not nameable ⇒ every span is host-minted") was false;
the new basis is the landed derived closure. But that is a **test-derived
property, not a proof**, and it has been wrong three times — so I told the ring
to write *"structurally unconstructible on the landed closure, asserted by
`<test>` along three axes"* and **not** *"impossible"*, and to make the
dependency visible so that if the adversary breaks the closure, the broken
dependency is obvious. Sending a ring a stronger claim than the evidence
supports is how the original defect got in.

### ❌ 07:07 — MY ERROR (caught before acting): narrow liveness pattern, 4th false signal

Checking the three mentions reached their seats, I grepped `Working \(` — the
**Codex** pane form — and `adversary` came back empty. It was fine: Anthropic
panes show spinner verbs, and it read `Honking… (1m 20s · ↓ 3.7k tokens)`,
actively diffing the merge.

**The annoyance is that my own watchdog already carries the correct structural
pattern** (`\([0-9]+[hms]…\)`, provider-agnostic) and I typed a narrower one-off
instead of using it. **A rule I wrote down but did not reach for is not yet a
habit.** Fourth false signal of the window; caught, again, only because I
captured the pane before concluding. Verify-before-acting is now the single
highest-yield discipline of this session — it has converted every one of my
detector failures into a non-event.
