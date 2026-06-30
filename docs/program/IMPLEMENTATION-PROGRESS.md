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

- **Updated:** 2026-06-30 (~04:40 — **🏛️ WS-K COMPLETE** (K-api `2f1cdf8`, the
  kernel/TCB closed); G1 closed; L5 + ITree-lowering delivered; **V1 released** —
  verification spine begins. Overnight: 1 send-keys + 1 comms-relay intervention,
  both logged; the rest deep-work false alarms.)
- **Next action:** **DONE/merged:** K1, K2, K3, V0-spec (`65adf30`), K2c
  compose-erratum (`444f937`), and **K2c series-1** (`7d38b55`, 99/99 — Architect
  caught a Floyd-Warshall **union-masking** SCT bug at review, fixed `9e36918`;
  retros in, 4th SCT over-collapsing instance recorded). **V0 build DONE**
  (`158b58f`, Architect traced the shadow guard + confirmed both assertions;
  retros in → promoted the **diff-scope check** + **branch-identity/0-test guard**
  to build leader/qa playbooks). So **V0 is fully done, spec + build.**
  **L5 DONE** (`c475d6c`, spec + 22 conformance; Architect surfaced **K1.5** as a
  hard dep — `ITree.Vis` needs Π-bound/W-style recursion the kernel defers — the
  enclave declared it via the **§7.0 gate split** + merged; retros in → promoted
  **kernel-admittance-vs-staging** + **content-reconcile**).
  **X1-spec DONE** (`387227d`, pure-core `42-evaluation`; retros in → promoted
  *name-non-strict-positions* + *structural-assertion-for-non-observable-props*).
  **K1.5 DONE** (`f5b19c2`, ★★★ trust root — the **load-bearing Architect review
  caught a real metatheory defect**: the W-ι "stuck-under-binder" decidability
  story was false, decidability is by **finiteness**; reground in 1 pass; retros
  in → promoted *termination-by-well-foundedness-not-stuckness* +
  *content-reconcile-inherits-spec-bugs / internal-consistency-pass*).
  **L5-build DONE** (`13fd2bf`, 42/42; Architect caught a silent under-inference
  gap on higher-order effectful params → conservative-reject fix; retros in →
  promoted *reply_to-needs-a-thread* + *QA absent-clause-scan* +
  *conservative-guard-fails-closed*).
  **K2c series-2 DONE** (`3c6273e`, ★★★ — Architect "exemplary"; the **promotion
  ladder validated itself**: K1.5 carries caught both would-be defects
  *prophylactically at authoring*, 0 Architect conformance findings; retros in →
  promoted *absence-assertion-gating* + *verify-frame-at-pickup* +
  *perishable-frames*, the last a Steward self-lesson — my frame was a stale
  do-not-restore hazard). **So the full kernel theory is SPEC'd** (K1/K2/K2c
  s1+s2/K1.5).
  **🎉 G1 VERTICAL SLICE CLOSED** (X1-build `f4a48e1` — V0 elaborates → X1 runs).
  **Kernel trust-root COMPLETE** (K1.5-build `f037451`). **L5 FULLY DELIVERED**
  (build `13fd2bf` + denotation `8c7941f`: row-poly + ITree/handlers + §3.1
  contract + the `36 §2.1/§7.0` reconcile) → **Sec1/Sec2/B1 unblocked**.
  **ITree-lowering DONE** (`4d6f332`, Language — kernel `Term::Elim` over `ITree`
  + the `param_rows` fail-closed contract).
  **K2c-series-2-build DONE** (`ecbb279` — ★★★ obs seams; Architect caught a
  seam-3 Cast-direction bug, re-fixed `bb0b3ba`; retros → promoted *per-dimension
  discriminating cases*. **Steward comms-relay:** the fix was committed but the
  implementer's handoff *post* was failing — I verified it via tmux+git and
  relayed to kernel/spec leaders → 4-min cascade to merge). **So the KERNEL
  THEORY IS FULLY BUILT** (K1/K2/K2c-s1+s2/K1.5).
  **🏛️ K-api DONE** (`2f1cdf8`) — **WS-K COMPLETE: the entire kernel workstream
  (K1–K2c, K1.5, the TCB-boundary contract) is closed.** Its §4.6 freeze-gate
  "did its job twice over" — held the contract open just long enough for the
  Architect to catch a reversed quotient-respect `cast` direction (`16 §5.1`) in
  end-to-end re-verify before it hardened into the TCB; released the instant code
  + contract converged. (Its merge had stalled on the Architect missing the
  series-2-build merge trigger — **Steward send-keys #1**, capture-pane-confirmed
  idle, woke it to flip the hold.) Retros → promoted *the freeze-gate pattern*.
  **In flight: V1** → enclave (`wp/V1`, just released; enclave compacted) — the
  verification spine's first WP (`21-spec-syntax`: `requires`/`ensures` + four-way
  status; ★★ untrusted/kernel-re-checked); deps V0 + K-api (both done). Enclave
  then flows **V1 → V2 → V3** (the prover spine) interleaved with **X1-effects-elab
  (`42 §3`)** + **Sec1** (rides L5).
  **HELD / IDLE — both dep-blocked on the serial enclave:** **Runtime** —
  **X1-effects is NOT a clean release**; it needs a **`42 §3` effect-evaluation
  elaboration** first (the X1-spec deferred effects as out-of-scope stuck forms),
  *then* Runtime builds. **Verify** — V1 (frame primed `72fce4f`) awaits the
  enclave. **Enclave queue after K-api:** **V1** (critical path → V2/V3) → then
  **X1-effects-elab (`42 §3`)** + **Sec1** (rides L5) — author those frames as
  the enclave nears V1. **Notification defect** (operator-confirmed, intermittent
  — but all 3 kernel "stalls" tonight were deep-work / finishing / auto-compaction,
  NOT wedges): diagnose via **`tmux capture-pane -t moot-<role>`** (working vs
  idle-at-prompt) before any nudge; `send-keys` only on a confirmed wedge
  (operator GO granted). Watchdogs private `CronCreate` (§13); convo §2a on `main`.
- **Watchdog/timer hygiene (this session):** promoted to COORDINATION §13 —
  record the `schedule_call` `timer_id` + `cancel_call` on WP close (`d795966`),
  and tick on `get_space_status`/`get_mentions`, never `get_recent_context` (it
  self-nests its own fires — `e53e9c0`). Also promoted the V0 **verdict-flip
  check** for discriminating conformance cases / worked examples (`2cf1fc6`).
  Cancelled my own Steward watchdog (was the misbehaving `get_recent_context`
  one); now fully event-driven. Orphan `tmr_37rn5qdv4c800` traced to **spec-leader**
  (operator-identified) — asked it to `cancel_call`. [[orphan-watchdog-timer-record-id]]
- **Anthropic peak-hours intermittent 503s (~19:28) — NOT a fleet outage;
  K2c proceeding.** spec-leader hit a transient 503 engaging the enclave authors,
  **over-diagnosed it as a fleet-wide Anthropic stall**, and proposed a DeepSeek-
  author workaround. **Operator ground-truth: spec-author + conformance-validator
  UP and HEALTHY, no errors — never down.** Corrected spec-leader → proceed with
  K2c (retry transient 503s; held it off DeepSeek-authoring the trust-root spec
  throughout). It used the false-alarm window to write the K2c elaboration plan
  (`docs/program/wp/K2c-elaboration-plan.md`), so recovery is instant. **Lesson
  (candidate promotion): a single 503 ≠ an outage — retry + confirm a sustained,
  multi-agent failure before ever declaring the provider down or reaching for a
  workaround.** DeepSeek tier unaffected throughout.
- **FAN-OUT @ ~20:56.** K1+K2 DONE; all 6 implementers Sonnet. Tracks: **(1)
  Kernel/K2c series-1** — spec elaboration merged (`97eefe0`, Architect caught 2
  soundness bugs: SCT `↓=` too permissive, Ω-element-vs-proof PI); **Kernel team
  building** (`wp/K2c-conversion`). Series-2 (3 obs seams) deferred → Kernel after
  series-1; owe a level-discipline Ω-element-vs-proof promotion first. **(2)
  Runtime/K3** — **DONE** (`b141744`, 36/36; Architect caught a >4MiB arena
  underflow → fixed). Retros harvested → boundary-value promotion +
  compaction-obligation gate (`dec_721ey23kfp4re`). (Stalled 3× on the
  mention-mechanism — leader didn't kick off / QA prose-named / leader waited on a
  dropped Spec vote — all nudge-cleared + now fixed in the playbooks.) **(3)
  Verify/V0** — spec enclave compacted + **elaborating** (frame
  `docs/program/wp/V0-elaborator.md`); → Verify build → G1. **(4) Language/L5** —
  queued behind V0 in the spec chain.
- **Corpus hardenings landed (the DeepSeek intent-without-mechanism pattern):**
  delegation-by-mention (`66982c2`), QA-real-mention + yon-cleanup/excluded-
  inspiration (`c05a658`), watchdog-scheduler=convo-cron + Integrator/leader
  mechanisms (`e30c066`). Steward watchdog itself now on the convo cron
  (`schedule_call`). [[playbooks-state-mechanism-not-intent]]
  Follow-on as deps clear: L1/L2/L3/L4/L6, L-classes+
  L-fmt (need V0), Sec1/2 (need L5).
- **FAN-OUT CLEARED on K2 merge (operator, 2026-06-29).** After K1 (solo) +
  K2 (in build) validated the gate (Architect review + QA + conformance caught
  every soundness issue; coordination self-correcting), the operator cleared the
  **fan-out**: on K2's build merge, release the parallel second wave (no further
  ask needed). **First wave:** K2c (Kernel, spec 17 — continues the critical
  path), V0 (Verify, spec 39 elaborator), L5 (Language, spec 36 effects — pull
  forward as the WS-Sec/WS-B hub), K3 (Runtime, spec 41/44 — **already elaborated
  by F4**, near-shovel-ready/fast-path). **Constraint:** the spec enclave is one
  team → frames queue through it serially (priority K2c→V0→L5; K3 fast-paths);
  build teams come online as their specs land (parallel builds, serialized prep).
  Steward authoring the frames during K2's build (pipelining); handoffs held
  until K2 merges. **Follow-on as deps allow:** L1/L2/L3/L4/L6, L-classes+L-fmt
  (need V0), Sec1/Sec2 (need L5), B1 (needs V1+L5), X1 (needs K1+K3).
- **(earlier) K2 solo (operator):** K1 merged (`fe1ead1`); operator chose
  measured K2-only first (not fan-out) to validate the gate before parallelizing. **K1 build retros pending:**
  kernel-implementer + kernel-qa posted (2 promotion candidates — *ground-truth-
  check-before-building* now 3×/2-team per the implementer, meets the bar; and
  *test-input-diversity* concretizing verify-the-property); awaiting kernel-leader's
  coordination retro + "retros in" handoff to finalize + promote + compact Kernel
  for K2 release.
- **K1 STARTED (2026-06-29).** Operator cleared K1 to start once the build-lock
  fix landed — it did (`4566211`), so K1 is moving. Steward frame authored
  (`ad2983a` on `wp/K1-core-type-theory`) and **handed to spec-leader for
  elaboration** (spec enclave compacted first; their F4 retro was in). On the
  elaboration merging to `main` → compact Team Kernel → release K1 to them. K1 is
  the critical path → K2 → K2c → K-api, feeding G1.
  - **Architect found 2 MORE soundness bugs — in the kernel IMPLEMENTATION
    (`dec_2hnhhdb7mrxze`, changes requested 14:42):** (1) universe-`max`
    normalization conflates distinct level variables by offset → predicativity
    break (AC-1); (2) dependent-telescope substitution doesn't weaken args →
    ι-reduct mis-indexed → subject reduction breaks (AC-6). Plus a panic on arity
    mismatch + a `shift` underflow + a W-style spec/impl divergence. The Architect
    *extracted and empirically reproduced* both from the kernel's own code. **Both
    invisible to the 45 green tests** (single level variable, closed indices) —
    **3rd validation of "verify the property, not the obvious case"** (now in
    COORDINATION §7). **3rd Architect catch overall** (F4, K1-spec-positivity,
    K1-impl ×2) — the deep impl review is the real trust-root gate; green tests ≠
    correct. kernel-implementer fixing the 2 blockers + 2 defects + 3 regression
    tests, then QA re-verify → Architect re-review. W-style reconciled in spec
    (`44bcd8b`, Architect pre-cleared).
  - **Integrator watchdog gap (operator-caught):** two green+approved PRs (#13
    typo, the W-style) sat unmerged ~25 min — the Integrator published then idled
    instead of polling GH (CI-green pushes no notification). Prompted it to poll +
    merge now; hardened its playbook (`dec_5d6jhm2vjw5tx`) — the merge-pipeline
    watchdog is a self-scheduled recurring ~5-10 min GH-poll TIMER while PRs are
    open, not wait-for-mention.
  - **Build complete (`0720eae`) → was in merge review:** kernel-implementer built
    `crates/ken-kernel` (~4.3k LOC: term/de Bruijn/grammar, env, subst, conv
    whnf/normalize + K2c seam, inductive w/ §8 positivity occurs-guards + dep
    eliminator + ι, bidirectional check), 45 tests across all 8 ACs; kernel-qa
    independently APPROVED (no ignored/tautological tests). **The read→build
    transition resolved my extended-spec-read flag** (it was deep audit, not a
    stall). Nice validation: the implementer **caught a conformance-seed typo**
    (AC-5 mislabeled a double-positive occurrence as negative) by cross-checking
    the seed against the normative §8.2 — the verify-the-property/ground-truth
    discipline applied by a GLM build agent; spec-leader fixed it (PR #13). K1
    build merge: `dec_2hnhhdb7mrxze` (PR #14), awaiting Architect (the deep impl
    review — green tests ≠ correct) + Spec + CI. On merge → compact Kernel,
    harvest build retros, mark K1 DONE, surface next-WP options to operator.
  - **Elaboration complete (`f65485f`):** spec-leader **delegated to spec-author +
    conformance-validator** (role correction held) — gap audit (7), K2-former
    tagging, K1 conversion algorithm + subject-reduction proof + strict-positivity
    algorithm (`13 §6`, `14 §7–9`), 31 K1 conformance seeds across all 8 ACs. **In
    merge review** — nudged spec-leader to open the merge Decision + the Architect
    to review (K1/K2 boundary + the load-bearing algorithm/proof additions). On
    Architect-approved + CI green + merge → compact Kernel → release K1.
  - **Architect review CAUGHT A BLOCKING SOUNDNESS BUG (`dec_5s2yv3prw09f5`):** the
    `14 §8.2` strict-positivity algorithm *dropped* subterms where a negative
    occurrence can hide → would admit `data Bad3 = mk : Pair (Bad3→Empty) Unit →
    Bad3` (inhabits `Empty`). Fix = occurs-check guards on the dropped positions +
    a conformance rejection case (AC-5 cases missed this class). 3 non-blocking
    refinements too (ι syntactic-index gate → incompleteness; dangling SCT ref;
    "common reduct" looseness). **The review gate earned its keep on the trust
    root — 2nd time (F4 closure-encoding, now K1 positivity).** Held lesson:
    Architect review is load-bearing, validated ×2.
  - **Notification caveat hit live:** spec-leader's status still said "awaiting
    Architect vote" after the vote landed — it missed the changes-requested
    notification (different thread). Re-pinged it (the §15 re-mention discipline,
    applied the same session it was written).
  - **K1-elaboration retros harvested + PROMOTED (`dec_24yhxp81eywr`):** all three
    enclave retros (spec-author, conformance-validator, spec-leader) converged on
    the lesson that already appeared in **F4-QA** — *a passing obvious-case test +
    correct prose are false signals; verify the property for every guard/case.*
    2 WPs / 3 roles → promoted to **COORDINATION §7** (verify-the-property,
    defensive pseudocode) + **architect playbook** (review kernel algorithms at
    pseudocode level). The F4 watch-list "encoder-sort / verify-property" candidate
    is now **promoted** on this recurrence. Spec enclave's K1 retros are **in** —
    can compact them for K2 when it's ready.
  - **Build-lock-fix retro harvested (WP closed 12:56):** foundation-leader missed
    the ken-cargo ship notification (compacted 11:20, ship-mentioned 11:28) — a
    3rd live hit of the notification-delivery caveat, and it independently
    re-derived the **§15 resume-mention-check** (already promoted; now validated by
    a leader too). Watch candidate (1 occurrence): *tooling-only WPs (scripts/, no
    spec crate) are a fast-path — Steward→build directly, no spec elaboration*
    (which the Steward already did for this fix). Promote to §2c on a 2nd
    occurrence/operator nod. Implementer/QA retros thin (one-line fix).
  - *Incident (operator-caught):* kernel-leader acted on my **stale original K1
    kickoff** (the pre-pipeline one) after re-onboarding — kicked the old too-broad
    scope to its implementer. Not a mentions bug (actor_ids durable + correct); my
    error was not standing Kernel down when I re-routed K1 to spec-leader. Stood
    Kernel down; the `wp/K1` branch was untouched (spec-leader holds it, correctly
    elaborating). Lesson encoded in steward §2c (a kickoff is live until retracted;
    stand down a re-routed team).
- **Build-lock fix merged (`4566211`)** — `ken-cargo` no longer wedges under
  sccache (Architect-reviewed; exit-code propagation preserved). Foundation
  collecting its retros (collect/review before any next Foundation WP). *Op
  note:* a pre-fix wedged sccache daemon may still hold the lock until
  killed/idle-respawns — kill the stale daemon once if a lingering wedge appears.
- **Earlier this session:** revised compaction protocol (Steward compacts whole
  teams, gated on retros; singletons self-compact) merged `38adb5e`; WP-branches-
  from-origin/main merged `585dbc2`; F4 + build-lock fix merged.

- **Retro harvest — F4 (reviewed 2026-06-29):**
  - **Promoted → COORDINATION §15:** resume-after-compaction ground-truth
    checklist (`orientation()` + `git reflog/status/branch -vv`) — F4 implementer
    trap; promoted on the operator-protocol coupling (Steward-compacts-every-WP
    makes post-compact resume constant fleet-wide).
  - **Watching (1-run candidates, not promoted):** QA's "verify the encoder
    *sorts*, not just that two BTreeMaps match" (build-qa testing-honesty rule —
    re-check at K3); leader's "read artifacts to coordinate, don't run builds"
    (reinforces build-leader coordinate-don't-implement; self-corrected); leader's
    `isolation:worktree` Agent mode for ring members. All node-internal (no edge
    change, §9 clean) — promote on a 2nd occurrence.
- **Skills routing validated.** orientation()→role→skill confirmed by 5/6 pilot
  agents (spec-leader/author, conformance-validator, foundation-implementer, +);
  files on `main`, file-read load works. **Caveat:** the `Skill` *tool* registers
  skills at **session start**, so mid-session rebase needs a restart for the tool
  (file `Read` works meanwhile) — CLAUDE.md now carries a Read-fallback. A fleet
  restart fully activates the Skill-tool path. Stray `origin/main` branch fixed
  by the Integrator (was `git fetch origin main:origin/main`).
- **Note on F4 authoring:** spec-leader (DeepSeek) authored the F4 elaboration
  itself (committed before the role correction landed) — clean-room intact (only
  `/spec` + the frame, no prototype). Accepted as-is on intrinsic merits; the
  Architect's merge review is the quality gate. The author=coordinator fix
  applies to *future* WPs (spec-author authors).

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
- **Frontier released, then re-routed:** initial kickoffs **K1**→kernel-leader
  (thread `evt_44k4934q2nfjz`), **F4**→foundation-leader (thread
  `evt_3f87m6kcqgkg3`). After the operator's pipeline refinements, F4 was
  re-routed through spec-leader and Foundation paused (it had jumped ahead onto
  `wp/F4-content-addr-design` with no commits — released cleanly). K1 held.
- **F4 pipeline in flight (Steward frame → spec-leader → Foundation):**
  - Planning bundle (playbooks §2c + compaction discipline, tracker, **F4 frame**)
    **MERGED to `main` as `c9c3883`** (Decision `dec_6jtjaczn5ffxc`, all CI green;
    branch swept). The F4 frame + corrected playbooks are now canonical on `main`.
    spec-leader signalled to rebase and begin elaboration.
  - **F4 elaboration** handed to **spec-leader** (compacted first) — elaborate
    `spec/40-runtime/41,44` + `conformance/runtime/` to team-ready rigor on
    `wp/F4-content-addr-design`, then merge_ready→Integrator; spec-leader pings me
    on merge → I compact + release Foundation against the on-`main` elaborated
    brief.
  - Compaction discipline applied: Integrator + spec-leader compacted before their
    handoffs (`moot compact <role>`).
  - **Role correction (operator, 2026-06-29):** the spec-leader (DeepSeek) was
    doing the F4 elaboration itself instead of delegating to spec-author (Opus).
    Corrected live (told spec-leader to hand authoring to spec-author +
    conformance-validator, compacting them first) and revised the spec-leader
    playbook to make the boundary load-bearing (coordinator ≠ author; DeepSeek
    leader must not read the prototype). Playbook routed to `main` via Decision
    `dec_43bet0wq8ayy9`. So F4 authoring now sits with **spec-author** (Opus),
    coordinated by spec-leader.

> **F4 LANDED 2026-06-29 ~06:20 — `45b62b2` on `main`** (build merge
> `dec_1darmky1az6gq`; Architect APPROVED, QA approved, CI green). **The first
> build WP is done and the WP pipeline is VALIDATED end-to-end:** Steward frame
> → spec-leader elaboration (`46b8aaf`) → Architect review (caught a real
> closure-encoding unsoundness) → implementer build (`ca6c177`) → independent QA
> → merge. Acceptance met (19/19 tests; dedup within tolerance; `CapacityExhausted`
> loud-refusal; slot-id equality depth-independent ~346ps≈359ps). Retros being
> collected by foundation-leader (COORDINATION §10). **Per operator: STOPPED here
> — K1 held, no new work.**
>
> **A QA-branch "blocker" raised by foundation-implementer was a false alarm**
> (post-compact misread; QA did verify `ca6c177` then returned home). A **real**
> blocker was surfaced — see Blockers below.

## Active frontier

| WP | State | Owner | Thread |
|---|---|---|---|
| **F4** content-addr / value-model design | **MERGED `45b62b2`** — done | Foundation | `evt_3f87m6kcqgkg3` |
| **ken-cargo build-lock fix** | **DONE** (`4566211`, retros in 12:56, Foundation quiesced) | Foundation | thread `thr_4h1a8yc4bzgqk` |
| **K1** core type theory (11–14) | **DONE** — merged `fe1ead1`; retros harvested → 3 promotions (ground-truth-before-build, exercise-the-property, branch-free-on-merge; `dec_2wwhyt9zxcrx1`) | Kernel | `evt_44k4934q2nfjz` |
| **K2** observational layer (15–16) | **DONE** — merged `832dab6`. **Architect deep-impl review caught a closed-`Empty` hole** (seam 3 `check_respect`: non-Ω quotient elim type-checked + computed without the respect proof) — implementer fixed (reject Type-codomain quotient elim; seams 1/1b made genuinely stuck; −173 lines unsound best-effort; +4 adversarial regressions), Architect re-derived the exploit dead, re-approved. 81 tests. Retros harvested → promotion `dec_46qe2bnz10fgw` (invoke every TCB guard; deferred check must gate reduction). **K2c carry-forward:** 2 `[K2c]`-tagged seeds (cast-computes-inductive, eq-inductive-dependent — now stuck, need NbE) + the 3 deferred obs seams | Kernel | `evt_63herhcztn5gw` |
| **K2c** decidable conversion (17) | **frame drafted** (`docs/program/wp/K2c-conversion.md`) — first fan-out WP; SCT gate + complete conversion algorithm. **Carry-forward from K2** (3 deferred obs seams: cast-at-inductive index rewrite, non-constant-motive J, full quotient `respect` — sound stuck fallback today). Awaiting K2 merge → spec-leader elaboration | Kernel | — |

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

- **⚠ ESCALATED TO OPERATOR (2026-06-29) — fleet build-lock wedge in
  `scripts/ken-cargo`.** It holds a machine-wide `flock` on fd 9; with
  `RUSTC_WRAPPER=sccache`, the sccache server daemonizes **inheriting fd 9** and
  holds the lock forever, so every subsequent `ken-cargo` queues to the 1800s
  timeout (foundation-implementer confirmed twice). **Fleet-wide; blocks all
  future build work** (K1 onward). One-line fix proposed: close fd 9 to cargo
  (`cargo "$@" 9>&-`) in `ken-cargo`; workaround `unset RUSTC_WRAPPER`.
  foundation-implementer offered to author `wp/ken-cargo-build-lock-cloexec`.
  **Recommend the operator authorize that fix before releasing K1.** Steward
  stopped per the F4 instruction; did not route it (would be new work).

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
