# Current briefing (live — read this first on every Steward resume)

> **This file is LIVE STATE ONLY.** When something here stops being true,
> move it to `diary/YYYY/Mon/DD.md` — do not append a newer block above it.
> Appending is what grew the old tracker to 2.23 MB.
> History: [`INDEX.md`](INDEX.md) · Work items: `docs/program/issues/*.md`

**As of 2026-07-24 ~18:0xZ. OPERATOR IS PRESENT.**

> ## ⚡ OPERATOR PRIORITY (2026-07-24): **LAND RT-NATIVE-FNSPLIT.**
> *"I did not mean to abandon the RT-NATIVE-FNSPLIT effort. Continue that work.
> Your primary goal now should be to land RT-NATIVE-FNSPLIT."*
>
> ⛔ **Nothing was abandoned** — the Architect's viability ruling explicitly
> retains bounded-function partitioning and every proved semantic from #24–#33.
> It replaces the **representation**, not the effort.
>
> ✅ **THE OWED ACT IS DISCHARGED.** The recut frame is **on `main`**
> (`docs/program/wp/RT-NATIVE-FNSPLIT-recut.md`, landed `6964b053`), the handoff
> gate ran and was drop-verified, and **the Runtime ring is KICKED on Phase 1**
> — kickoff `evt_2kgfmmeeh2x7w`, `runtime-leader` confirmed `Working`.
> Runtime now owns the move; I am the backstop, not the driver.
>
> ★ **The frame's load-bearing choice:** Phase 1 measures the **HELD** (unchanged)
> representation at n=3..7 *before* any rewrite — because that is precisely the
> Architect's own falsifier for the hold. It either **kills the rewrite as
> unnecessary** (cheapest possible exit) or yields the baseline every later claim
> is measured against. Do not let Phase 1 be skipped as a formality.

### ✅ Landed this window
- **`origin/main = bf00f1a9`** — the four operator directives are ON MAIN
  (verified by content, not by the publisher's report): COORDINATION §8a + §10⁻a,
  steward playbook §2d + ledger axis, plus the §5a armed trigger.
- **`PUB-VERIFY` CLOSED** as landed-by-other-work (operator-directed).
- **`DOC-VALIDATION-BINDING` PUBLISHING** @ `f1eb7408`, branch FROZEN
  (`evt_6ewc3t460tvn7`). ⚠ My corpus publish moved main `8ebe370a → bf00f1a9`
  under its Decision; **intersection tested EMPTY**, so immaterial — no rebase.
  **Retros still owed before it closes.**


> # ⇢ LIVE STATE — 2026-07-24, Opus 5.0 session, post-restart
>
> **The successor handoff is DISCHARGED and has been retired from this block.**
> Everything it owed is done: drops verified, Runtime kicked, cadence re-armed,
> watchdog re-armed.
>
> ## Where the frontier is — PHASE 1 IS CLOSED; BOUNDARY A IS THE LIVE UNIT
>
> **`origin/main = 510de0e3`.** The recut frame was **AMENDED** after Architect
> hard-stop #1 (`evt_6dpb96kn1583f`) and handed back: `evt_30a344an210g`,
> `runtime-leader` pane-confirmed `Working`. Ring was compacted first (drops
> verified) **without** `handoff-gate-compact.sh`, deliberately — see below.
>
> **⛔ Phase 1 is CLOSED at `could_not_determine`. Do not re-run it.** My frame's
> central premise was **false against the landed code**: unchanged `b077eb7a`
> cannot complete even the checkpoint's own depth-2 public control (fails
> `NativeExitScopeTransitionV1: scope body return lost its parent producer tail`),
> and it has no pre-existing planner boundary to measure at. **AC1.1's fail-closed
> third outcome caught it** — it refused to return the permissive answer. The
> ring escalated instead of building around it, exactly as the perishability
> clause asks.
>
> ⛔ **What that falsifies is the BASELINE PREMISE, not the representation hold.**
> The hold stands **structurally** (variable-width composite identity, no
> fixed-K/key-width invariants) — it never rested on curve-fitting. Do not read
> `could_not_determine` as "the rewrite is unnecessary."
>
> ### ✅ BOUNDARY A REPORTED AND I HAVE RULED MY HALF (`evt_7wgktr1dk4qvk`)
>
> Candidate `92cac774` on `wp/RT-NATIVE-FNSPLIT-recut-boundary-a-planner`.
> **Sequence APPROVED with ONE required fold.** Runtime stays stopped on the
> semantic port until the fold lands **and** the Architect returns its half
> (the frame requires Steward **+** Architect).
>
> **Verified by me, not taken from the report:** all four series affine — second
> differences exactly zero (nodes Δ28, edges Δ34, helpers Δ62, persistent Δ37);
> scope genuinely pre-emission (4 `ken-runtime` files, no CLIF/wall/RSS);
> depths correctly reported as **affine, not forced constant** (honors the
> corrected width metric); the exact-mutation guard test is real (drives
> production `validate()`, asserts exact error strings).
>
> ★ **The construction proof is stronger than AC2.1 asked for:**
> `StaticHelperKey{node,transition}` derives **`Copy`**, and `Copy` is
> incompatible with `Vec`/`String`/`Box` — so "no variable-width member in
> identity" is **compile-enforced**. ⚠ `size_of` alone would NOT have proved
> this (a `Vec` field has constant `size_of`, variable content).
>
> **⛔ The fold:** `helper_identity_excludes_dynamic_activation_…` is **vacuous** —
> `assert_eq!(wrapper.key, wrapper.key)` is a tautology and `assert_ne!(frame,
> changed)` tests its own setup; `changed` is never fed back through the planner.
> **Phase-2 point 2 has no guard at all**, while the report cited this test as
> evidence for it. The invariant *does* hold by construction — this is an
> unguarded invariant + an over-cited test. Required fix is the real property:
> two distinct activations through one static node share **one** key and do
> **not** grow planned-helper count.
>
> **Cadence: hard-stop count STAYS 1.** A planned stop for a required read is not
> a hard-stop, and neither is a review fold. Next pull still #3.
>
> **Live unit = Boundary A (planner census) ONLY**, on a fresh branch off current
> `origin/main`: the factored static transition graph for n=3..7, reporting static
> nodes/edges/planned helpers, persistent-store nodes, out-of-line evidence
> records, fixed K per static node, and fixed key/frame/store schemas. **It STOPS
> for a Steward + Architect read before any semantic body emission (AC1.4′) — I
> owe that read.** ⛔ CLIF bytes and wall/RSS belong to Boundary B and must not be
> demanded of A.
>
> ⛔ **WIDTH METRIC — my original was WRONG and would have rejected a correct
> design.** Constant applies to **inline identity/frame/store-node width**;
> logical persistent-chain depth MAY be Θ(n) because the frame carries one
> constant-width ID. Never demand constant chain length.
>
> **Runtime WPs legitimately run for HOURS** (T1 `sol` implementer, 16-hour
> single-WP sessions on record). That is not a stall — do not tune thresholds to
> tens of minutes on this ring.
>
> ### ⛔ DO NOT run `handoff-gate-compact.sh` on `runtime-implementer`
>
> It hard-resets each worktree branch to `origin/main`.
> `wp/RT-NATIVE-FNSPLIT-recut-phase-1-census` is **30 commits of genuinely
> unmerged work** (the held representation + the census WIP), and the Architect
> ruled it **stays held** — Phase 3 ports from it. Compact that ring with a plain
> `tmux send-keys -l '/compact'` instead, which touches no refs. Verified: the
> branch tip was still `82bd1f43` after compaction.
>
> ## Research cadence — ARMED (this is the count of record)
>
> ```text
> RECUT CHAIN: hard-stop count = 1 · NEXT RESEARCH PULL = #3, then #6, #9, …
>   #1 = Architect amendment ruling evt_6dpb96kn1583f (frame amended in response)
> HELD CHAIN:  frozen at 33, closed, does NOT carry forward
> ```
>
> The Architect **explicitly deferred to this count** on #1 ("the Steward's
> tracker remains the count of record, so no Research pull is due before #3").
>
> Written into `issues/RT-NATIVE-FNSPLIT.md` as an armed line, not prose. **A
> deep chain with zero advisories is the tell that BOTH the Architect's
> self-trigger and my backstop have lapsed** — that already happened once here
> (10 hard-stops dry). Catch-up rule: if a trigger is missed, fire at the **very
> next** hard-stop, don't wait for the next clean multiple.
>
> ## ✅ Checkpoint `b077eb7a` is DURABLE — and this nearly went wrong
>
> `git tag rt-native-fnsplit-checkpoint-b077eb7a`, **pushed to origin** (verified
> by `ls-remote`). When found it lived on **one local branch with zero off-box
> copies**; `handoff-gate-compact.sh` then **hard-reset that exact branch within
> ten minutes** (local fallback:
> `preserved/wp-RT-NATIVE-FNSPLIT-native-partition-b077eb7a`).
> ⛔ **Do not delete the tag.** Phase 1 measures it; Phases 2–3 port from it.
>
> ## 🚨 IMMEDIATE — `DOC-GATE-RECORD-AXIS`: ALL GATES PASS, publish blocked by a GITHUB WRITE OUTAGE
>
> **This is the first thing to retry on resume.** Everything is done except the
> publish itself.
>
> - **Branch is on `origin` at the exact approved SHA** —
>   `wp/DOC-GATE-RECORD-AXIS-gate-record-axis` @ `b3afd48b`, confirmed by
>   `ls-remote`. ⛔ **FROZEN — do not rebase/amend/re-anchor.**
> - `dec_7htr8nc7076x` **resolved** (the `proposed` list is empty); QA
>   `evt_5qstakpep4njx` + Architect `evt_1qtygnzf4a67q` on that SHA; scope one
>   file ⇒ Architect-only correct; merge-base == `757ce46a`; intersection empty.
> - **AC1 positive control was genuinely run** — QA added a real second
>   `kind = "status"` record and watched the assertion FIRE.
>
> **The blocker: GitHub fails every WRITE to the pulls endpoint while READS
> succeed.** 5 attempts / 2 mechanisms (`gh pr create` GraphQL; REST
> `POST /pulls` → empty body), each a distinct server-side incident ID (last:
> `C2B2:1E7A6B:120E2A5:124A855:6A63BBE2`). **Positive control:** `GET` PR #921
> returns fine, rate limit untouched at 5000. ⇒ **not** auth, **not** rate limit,
> **not** branch protection, **not** a gate.
>
> **No PR exists ⇒ no CI has run.** On recovery: open the PR on the frozen SHA,
> let CI gate it, merge on green, then verify **by content on `origin/main`** and
> post to the ring. Ring owes nothing; I hold it. Retros owed when it lands.
>
> ⚠ **This is mutable external state (§7a) — TEST it, do not cite this block.**
> It may be fixed by the time you read this.
>
> **Outage still live at 19:34Z** (6 attempts, latest incident
> `DA54:39E685:12AC979:12EB77B:6A63BE4E`). Watchdog retries it as **step 0** of
> every tick, so recovery is picked up automatically. ⛔ No PR ⇒ no CI has run.
> ⛔ Do **not** try to route around branch protection — §14 requires the path to
> stop and route the fact, not to improvise.
>
> ### ✅ Boundary A: FOLD DISCHARGED on `e70bb2a5` (supersedes `92cac774`)
>
> Steward half **APPROVED** (`evt_4g5qe6tz3s4k0`). The repair is the strong form:
> `helper_key_for_activation(node, frameA)` vs `(node, frameB)` — same static
> node, two **real closed** frames, key **recomputed** each time, asserted to
> collapse to one; plus `planned_helpers` unchanged. **And it is enforced in
> `validate()` over every node**, so the invariant is structural, not merely
> tested. ★ The old `u32::MAX` frame would now be rejected as *unclosed* — the
> original approach could not have worked even with a correct assertion.
> ✅ **Census table restated for `e70bb2a5`** (`evt_1favdzgj04y02`) — and the
> counts are **identical** to `92cac774`: nodes 87/115/143/171/199 (Δ28),
> edges 103→239 (Δ34), helpers 190→438 (Δ62), persistent 128→276 (Δ37), all
> Δ²=0; K=8 flat; **helper-key 12 B** (the honest fixed width of the typed
> `PlannedHelperKey::{Node,Edge}` inventory), frame 32 B, store 16 B, all flat.
> Asking was still right — the inventory *had* changed; the counts happened not
> to.
>
> ### ⛔ ARCHITECT BLOCKED `e70bb2a5` (`evt_2km3wm7h9ckgp`) — a THIRD candidate is due
>
> It **confirms my fold is real and may stand**, and confirms the eight-slot
> frame + `{kind,local,aux,child}` store are expressive **without a ninth slot**
> (the expressibility question I routed to it). But it reproduced **two
> production closure holes** with exact mutations:
>
> 1. **Alternate callable edges bypass W.** `validate_source_return_topology`
>    proves the R→W/W→T/T→CompletedTail edges *exist* and makes W's incoming
>    exclusive, but **not** T's or CompletedTail's; the CompletedTail check
>    counts only edges already labelled `CompleteProducerTail`, so a second
>    incoming edge of another kind is **invisible**. Fix: close the quartet by
>    **set equality, not existence**.
> 2. **Entry closure inferred from CARDINALITY.** Reachability compares
>    `reachable.len()` with `nodes.len()` and never proves each entry is a real
>    node ID — so an out-of-range root `StaticNodeId(u32::MAX)` balances the
>    count and `validate()` returns `Ok(())` while a real node is unreachable.
>    Fix: range+uniqueness before traversal, **exact set equality** after.
>
> `runtime-implementer` picked it up 71s later and is working the fold.
> **Cadence: still hard-stop count 1** — the Architect explicitly said this is a
> review fold, not a hard-stop, and that the count remains mine.
>
> ★★ **PROMOTION CANDIDATE — all THREE Boundary-A findings are one shape:** a
> check that compares a **proxy** instead of the **property**. Mine
> (`assert_eq!(x,x)` + a mutation that never re-enters the mechanism), the
> Architect's #1 (counting only the *labelled* subset), and #2 (**cardinality**
> standing in for **set equality**). Three independent instances in one WP, found
> by two different seats ⇒ this clears the §10 bar. Write it up.
>
> ## (resolved) The Decision was `proposed` — held, then unblocked in seconds
>
> **A `git_request` arrived (`evt_517mjv8rr6kb`) and I did NOT merge.**
> `dec_7htr8nc7076x` is **`status: "proposed"`** — `resolved_by`/`resolved_at`/
> `resolution` all `null`. §14: merge only on a **resolved** Decision, never on
> `merge_ready` prose. Held at `evt_6ja7bqqawx3pr`.
>
> **Everything else is verified clean, so this is a one-step unblock — publish the
> instant it resolves:** tip == approved SHA `b3afd48b` ✅ · scope is exactly
> `crates/ken-cli/tests/library_documentation_gates.rs` so Architect-only review
> is correct ✅ · merge-base == current `origin/main` == `757ce46a` ✅ ·
> intersection **empty**, no rebase wanted ✅.
>
> ⚠ **I am not disputing the votes happened** — both were cited with real event
> IDs (QA `evt_5qstakpep4njx`, Architect `evt_1qtygnzf4a67q`). What is missing is
> the *resolution*. Also asked them to confirm **AC1's positive control** (that
> the F1 assertion was proved to fire by actually adding a second
> `kind = "status"` record, not inferred).
>
> ★ **Probe discipline that mattered here:** `list_decisions(resolved)` did not
> contain it — but I only trusted that after a **positive control** (a decision I
> knew was resolved *was* in the same dump, 1 of 400). Then confirmed directly on
> the `proposed` list. Don't conclude absence from one negative lookup.
>
> ## ▶ Track 2 — `DOC-GATE-RECORD-AXIS` with the Verify ring
>
> Kicked `evt_4sv449b1e4tcy`; ring compacted + reset to `510de0e3` (drops
> verified); leader pane-confirmed `Working`; tracker flipped to `active`.
> Adversary F1+F2, both **confirmed preventive**, both cheap. **Deadline is a
> precondition, not a date: close before the next `library/` record is added.**
> Reviewer is **Architect only** (`crates/`-only diff, §8a).
>
> ⚠ **F1 is `DOC-VALIDATION-BINDING`'s own defect class on a different axis** —
> token→runner *existence* is bound, *coverage* is not. I verified all three
> adversary anchors against current `main` and they hold; the harness is
> unchanged since `96ab2b4b`.
>
> ## Open loops I own (in priority order, and NONE outranks Runtime)
>
> 1. **Harvest + promote the `DOC-VALIDATION-BINDING` retros** (`evt_78qng91927xvj`;
>    merged `96ab2b4b`). One carry is strong and was **independently validated
>    twice in one day** — the implementer hit it, and so did my own verification
>    probe — so it clears the §10 bar: *after deleting or renaming a load-bearing
>    symbol, repo-wide grep its old spelling and classify EVERY survivor as
>    live-to-update or intentionally historical before handoff.* A second carry
>    worth promoting: the **reverse-dependency proof** (rename the runner, show
>    the build fails *at the registry line*, restore byte-for-byte) — it
>    demonstrates a binding instead of arguing for one.
> 2. **Compact the Verify ring** at its WP seam (retros are in, so it is unblocked).
> 3. **`STR-BIJ` stays HELD** until I re-derive the `library/` ledger consumer
>    population — DOC-VALIDATION-BINDING landed and **changed it**, which is the
>    point. Contention has a LEDGER axis, not just a file axis.
>
> ⛔ **§10⁻ ceiling:** retro/lesson artifacts **batch**; they do not each earn a
> merge, and no process merge happens while a ring holds finished unmerged work.
> The tell is one command: `git log --since=<3h ago> origin/main` with paths —
> if nothing in the window touched product, a ring is stalled and something is
> reporting it as fine.
>
> ## Fleet state
>
> `origin/main = 6964b053` · **Runtime**: Phase 1 active, leader `Working` ·
> **Verify**: idle, WP merged, retros in, awaiting my compaction · **Architect /
> research / adversary / everyone else**: idle, no obligation.
> ⚠ **Two T1 seats died on `Selected model is at capacity` today. That state is
> indistinguishable from healthy idle except in a WIDE capture** (`tail -20+`) —
> the `Compacting…` bar and the work spinner both render ABOVE the input line, so
> a narrow `tail -5` false-read idle on me twice. Rouse clears it.
> ⚠ **Unpublished on `steward/work`:** this briefing + the armed cadence line +
> the checkpoint-tag record. Bundle into the next publish (§2a); do **not**
> publish `steward/work` itself.


## Standing state

- **`origin/main = 8ebe370a`** (`PX8-F-CAP-41 Phase 1` — sealed checked buffer-
  capacity handle, §38 fold). Green. Recent landings behind it: `cbf6a298`
  PX8-SPAN-PROV Phase 2 → `b64ad9f3` Phase 1 → `4ac9141e` SEAL-2 →
  `238a5c5d` RT-ESCAPE.
- **⚡ THE BOX LOST POWER at ~16:19Z, and again at ~16:33Z** (two disconnect
  waves in the space feed; operator-confirmed). **Every seat re-oriented from
  cold.** Consequences that outlived the restart are in the next block — read
  it before diagnosing anything as a stall.

### ⛔ THE ONE LIVE CHAIN — RT-NATIVE-FNSPLIT, hard-stop #33

**The entire runtime frontier is serialized behind this single chain**, and
everything else downstream of it is blocked by construction:

```text
RT-NATIVE-FNSPLIT (active, runtime)
  └─ NATIVE-HANDLE-CARRIER (draft)  ← blocked on it
       └─ PX8-F-CAP-41 Phase 2      ← blocked on that
```

| seat | state @ 17:2xZ |
|---|---|
| `runtime-implementer` | **implementing the #33 ruling** off `d43b6933` (ack `evt_61n2zpnj5rgvm`) |
| `architect` | **#33 RULED** `evt_55c62m0anfyyk`; now working the **viability review** |
| `research` | #33 advisory delivered `evt_138ty4vycfgr5`; idle/ready |

**#33 ruled: choose A** — compose the selected head into the exact final
producer cursor *before* `reserve_partition_source_return`, giving
`producer_head = W(selected, successor=T)` / `live_producer_tail = T`, with STOP
kept strict and the consumed cursor admitted only as a **pointer-free spent
receipt**. Normal execution is exactly `W once → T once → CompletedTail(T)`.

### ⛔⛔ VIABILITY RULED 2026-07-24 — HOLD + RECUT. THE CADENCE IS OVER.

**`evt_3m1g3v4m2bj51`. Runtime is HELD at clean `b077eb7a` until I author the
recut frame — that is the Steward's owed act and nothing moves without it.**

- **Single-`Function` inlining is DEAD AND ALREADY REPLACED.** The root cause
  this WP was filed on is **stale**; `b077eb7a` already emits one function per
  `PartitionWorkItem`. The issue file's title + origin section are now marked
  superseded inline.
- **The mechanism family is VIABLE (Θ(n) reachable); this REPRESENTATION is
  not.** Helper identity is a Cartesian tuple of variable-width dimensions, so
  **Θ(n) states × Θ(n)-wide data ⇒ Θ(n²)** descriptor/comparison/frame work.
  ★ **Hash-consing children cannot save it** — it shares equal subterms, it
  cannot merge distinct tuples whose components merely overlap.
- **More per-hard-stop sealing has NO route to the gate.** Sealing is linear
  only in the graph it *receives*.
- **Count FROZEN at 33; research cadence SUSPENDED** with the machine it
  counted (#34 is evidence, not a ruled stop). Re-arm against the recut chain.
- ★ **Honest reading of n=4 preserved:** one point cannot establish an exponent
  — `370n`, `93n²` and a threshold-at-n=5 all pass through it. **The hold rests
  on code inspection rejecting an O(n) proof, not on curve-fitting.**

**Next unit is a planner/census recut — NOT #35:** generate n=3…7 *before*
lowering bodies; acceptance needs bounded first differences **and** structural
invariants (fixed K helpers per static node, constant max key/frame width).

★ **This is what the operator's viability call bought:** 33 hard-stops of
correct, converging semantic work were being spent on a representation that
provably could not reach the gate. **Every individual ruling was right; the
thing they were accumulating into was not.**

### ⚡ The escalation that produced it (for method, not status)

Operator ruled 2026-07-24 that 33 hard-stops with SP-A still scaling-red
warrants a step back. Escalated to the Architect as `evt_98j3z2n49bpg`:

> Can single-`Function` inlining + defunctionalized continuation sealing reach
> the O(n) scaling gate **at all**, or is 1,482 states at n=4 structural
> super-linearity further sealing cannot remove?

I framed the question only — the ruling is the Architect's. **Sequencing call I
made:** the implementer *continues* #33 meanwhile, because that ownership fix
moves toward the constant-width form under any outcome; the Architect can hold
it with one word. Told it to pull Research immediately rather than wait for #36.

★ **Best grounding came from the Architect's own #33 ruling:**
`PartitionProducerKontSiteActionKey::ApplyActiveEliminators` still carries
**vector-shaped** `pending`/scope/capture data (`partition.rs:5700–5777`) — it
called this *"not the scalable form of this ruling."* A measured defect, not a
forecast.

**Thread for everything on this chain: `thr_7vdcfhxgfw128`.** The open question
is *selected-head ownership at the STOP predicate* — candidates A (precompose
selected-head work into the exact final producer cursor) vs B (selected
ancestry/scope as typed dynamic activation subordinate to the producer cursor).
⚠ **That is the Architect's lane. Do not form or transmit a view on it.**

**Two stacked transport failures cost this chain a restart's worth of latency —
both repaired 2026-07-24 ~16:5xZ:**

1. The Architect's advisory request `evt_3vr382mrv99pe` posted with an **empty
   `mentions` array** ("mentioning Architect only" in its prose refers to
   *research's reply*, not its own routing). Research is a **no-poll seat**, so
   it was never notified and re-oriented to *"awaiting dispatch"* while the
   Architect sat `blocked-on-Research` **indefinitely**. Repaired by pointing
   research at the event verbatim (`evt_d2b3vahe7khj`) — **transport only, the
   question was not restated.**
2. **`architect` and `librarian` both died on `⚠ Selected model is at
   capacity`** mid-re-orientation. Research runs the *same* model and was fine,
   so **capacity is transient per-request, not a model-wide block** — a rouse
   clears it (architect roused → `Working`). `librarian` re-oriented fully and
   holds no obligation, so its idle is correct.

★ **The generalizable tell: a seat whose turn dies on a capacity error is
indistinguishable from a healthy idle seat.** Neither posts, neither pushes.
Only a **wide** `capture-pane` shows the `at capacity` line above the composer.

- **§5a research trigger is now ARMED in the issue file** (it was not — that is
  why this chain ran **10 hard-stops dry**): `docs/program/issues/`
  `RT-NATIVE-FNSPLIT.md` carries `hard-stop count = 33` /
  `NEXT RESEARCH PULL = #36`, cadence every 3rd. **My tracker is the count of
  record**; the Architect re-derives its own across compactions and loses.
- **⛔ RT-NATIVE-FNSPLIT DOES NOT MERGE ON "the tests pass"** — the operator's
  scaling gate (`evt_4btfhwqhah1ye`) binds: empirical n=3..7 harness +
  research-grounded analytical growth order + a verdict. **SP-A is
  independently scaling-red at 1,482 states / 1,525 edges**; no advisory in this
  chain has cured that, and each one says so explicitly.
- **Build side is capped at TWO implementation tracks** (operator). Doc is the
  standing exception; the enclave is not a build team, so it does not count
  against the cap. **Track 1 = runtime/FNSPLIT. Track 2 is currently EMPTY.**
- **⛔ IDLE BUILD RINGS ARE CORRECT — DO NOT "FIX" IT.** Operator ruling
  2026-07-22: *"just doc plus two implementation tracks."* `kernel`, `language`,
  `ergo` idle with zero ready items is the **intended** state, not a stall.
  **There is NO WP-authoring obligation for them.**

### ⛔ Retraction: the stale-base "near-miss" was FALSE (2026-07-22 ~13:43Z)

I held ABI-REVOKE claiming a stale base *"would have silently DELETED"* two
DOC-W1-2 chapters, and wrote it up as a general rule. **Disproved by
measurement:** BUDGET-EFF was on the **same stale base**, **also lacked both
chapters entirely**, and after merging **both chapters survived** — `214bf4de`
has one parent (squash). **GitHub squash-merge applies `merge-base → branch`,
not `main → branch`.** Absence from a stale candidate is not a deletion.

★ **`git diff --name-status origin/main <sha>` is NOT a staleness detector** —
it fired identically on the safe candidate and the suspect one. **The correct
test is the INTERSECTION:**

```sh
BASE=$(git merge-base <sha> origin/main)
comm -12 <(git diff --name-only $BASE <sha> | sort) \
         <(git diff --name-only $BASE origin/main | sort)
```

Empty ⇒ staleness immaterial, publish. Non-empty ⇒ inspect those files, take
the union deliberately. The ABI-REVOKE rebase was still the **right call** —
its `library/REVISION` + `manifest.toml` bump was a genuine non-empty
intersection — **but for a reason I stated wrongly.** Retracted publicly in
`evt_54t014vnef2xr`.

### ★ QA bound-verdict attestations are now on `origin` (2026-07-22 ~12:53Z)

The commit-your-verdict workaround (used when a QA seat's convo outbound dies)
left verdicts on **one local ref in one clone** — `a4473ab0` had **no second
copy anywhere off this box**, and `handoff-gate-compact.sh` has already
hard-reset that exact branch once (`preserved/runtime-qa-work-7c86db36`).
Pushed to durable refs, each verified by `ls-remote`:

```
attest/runtime-qa-verdicts   a4473ab0   (53501ffe is an ancestor — both carried)
attest/ergo-qa-verdicts      cf791c7f
attest/verify-qa-verdicts    04efa001
```

**This fixes durability, NOT discoverability** — a branch name nobody watches
is still a pointer someone must deliver. **Transcribing these into the repo
proper is the actual close and needs a WP.** ⛔ Do not reset, clean, or
re-anchor `runtime-qa/work`, `ergo-qa/work`, or `verify-qa/work`.

★ **The transferable lesson:** *when a workaround relocates a failure mode, the
new location inherits none of the scrutiny the old one had.* The pattern fixed
a real selection error and quietly moved the fragility from the message layer
to the storage layer, where nobody was looking.

> ### 🚨 INFRA ESCALATION FOR THE OPERATOR — `runtime-qa` convo outbound is DEAD
>
> ⚠⚠ **UNVERIFIED SINCE THE 2026-07-24 POWER CYCLE — DO NOT CITE THIS, TEST IT.**
> This is a claim about **mutable external state** (COORDINATION §7a): it can
> become false without any file changing, and the box has since power-cycled
> twice, which restarts the MCP client this diagnosis was about. **The next time
> `runtime-qa` needs to post, have it try — do not pre-emptively route it to the
> commit-a-verdict workaround on the strength of this block, and do not escalate
> it to the operator again without a fresh failed attempt.** Everything below is
> evidence about 2026-07-22, not about now.
>
> **Inbound works** (it receives mentions and reviews normally); **it cannot
> POST.** Unchanged across 4 watchdog ticks and 2 `/mcp` reconnect attempts.
> Server-side is healthy — every other seat posts normally.
>
> ⛔ **Do NOT relay its self-posts to close a provenance gate.** The gate exists
> precisely so an attestation is *not* Steward-sourced; a relayed self-post is
> a contradiction and banks the gap permanently.
>
> **The working path, and it is better than a relay:** QA **commits its verdict
> to its own branch** through the shared clone, and reviewers read it by object.
> Slice 5 closed this way — `53501ffe`,
> `docs/program/qa-triage/RT-SPLIT-slice5-runtime-qa-verdict.md`, binding
> `APPROVE` on `744bda14` / base `1f70a71b`. Precedent existed already
> (`ergo-qa @ cf791c7f`, `verify-qa @ 04efa001`). **A relay verifies the
> selection; a commit eliminates the selection.**

### ▶ Track 1 — RT-NATIVE-FNSPLIT (Runtime ring) — THE ONLY LIVE BUILD TRACK

Full state in **Standing state** above. One line here so this section is not a
second source of truth: **held at hard-stop #33 / `d43b6933`; research advisory
in flight; Architect rules after it lands; operator scaling gate still unmet.**

### ▶ Track 2 — `DOC-VALIDATION-BINDING` (Verify ring), kicked 2026-07-24

Operator-directed fill of the empty second slot. Kicked `evt_3fv5d1men6r9y`;
ring compact-verified at `8ebe370a`; branch
`wp/DOC-VALIDATION-BINDING-gate-token-binding` off current `origin/main`.

⚠ **Re-homed doc → verify, and the reason matters.** I first offered this to the
operator as *"scoped to `library/`, doc-only, Architect does not vote"* — **that
was wrong.** The mechanism is `crates/ken-cli/tests/library_documentation_gates.rs`,
a 106 KB Rust gate harness. Consequences: the doc ring cannot own it (its
concurrency licence *is* path-disjointness from `crates/`), and **the
COORDINATION §14a doc-only exemption does not apply — the Architect must vote.**
Corrected to the operator in the same breath as acting on it.

★ **Transferable:** *a WP's tracked `owner:` field is a routing claim, not a
grounded one.* This one said `doc` and had said so since it was filed. **Grep
where the mechanism actually lives before you route on the field.**

`PX8-F-CAP-41` Phase 2 was the natural Track 2 but is **blocked** behind
`NATIVE-HANDLE-CARRIER` → `RT-NATIVE-FNSPLIT`, so the PX8 spine cannot fill this
slot while #33 is open.

### ⛔ FOUR OPERATOR DIRECTIVES ARE LAW AND ARE **NOT ON `main`**

Found 2026-07-24 while checking `steward/work` drift. Verified **by content**
(`git grep <phrase> origin/main`), not by branch-ahead, so the squash-merge trap
is excluded:

| item | status |
|---|---|
| `COORDINATION §8a` — Architect/Librarian review in PARALLEL over disjoint domains | **not on main** |
| `COORDINATION §10⁻a` — adversary channel report-only, scoped to `crates/`+catalog | **not on main** |
| steward playbook §2d — separate judgment from action (OODA) | **not on main** |
| steward playbook — contention has a LEDGER axis | **not on main** |

⛔ **Why this is not bookkeeping.** Every seat reads `agent/COORDINATION.md`
**from its own worktree at `origin/main`.** After the power cycle the whole
fleet re-oriented against a COORDINATION that is **missing two operator
directives**. I hold them only because I read from `steward/work`. So the fleet
is currently reviewing in series where §8a says parallel, and the adversary is
operating without its §10⁻a scope fence.

⚠ **`steward/work` is 70+ commits ahead of `origin/main`**, against §6a's *"at
most the current unpublished tracker delta."* Most is the squash-merge trap —
**do not treat branch-ahead as unmerged.** The correct route is §6a step 2: cut
`wp/steward-<slug>` from **current** `origin/main` and apply only the intended
changes; never publish `steward/work` itself.

**Awaiting the operator's call on publishing this** (§10⁻: process work is
subordinate to product flow — but these are the operator's own directives, and
their absence is actively changing how the fleet behaves).

### ▶ Doc track — IDLE

`DOC-W0`/`DOC-W1` closed, `DOC-W1-*` slices merged. `DOC-W2` is `draft` (a MAP,
framed only when Wave 1's exit condition is met). `DOC-VALIDATION-BINDING` is
`ready` and unassigned — the cheapest genuinely-parallel item on the board.

### ⏭ Releasable frontier (generated — `IMPLEMENTATION-PROGRESS.md`)

All deps met, nothing blocking a kickoff. **Ownership shown is the tracked
owner, not an assignment.**

| item | owner | note |
|---|---|---|
| `DOC-VALIDATION-BINDING` | doc | validation vocabulary claims a 1:1 gate binding nothing enforces |
| `STR-BIJ` | spec-enclave | String/`List Char` bijection over-claim; ⚠ carries a **ledger-axis** sequencing constraint |
| `KW-THEOREM` | spec | rename surface keyword `lemma` → `theorem` |
| `F1-37` | runtime | bignum `Int` soundness review for K3 trusted-base promotion — ⛔ *same ring as FNSPLIT, so not concurrent* |
| `MODELS-TIER` | steward | MODELS.md — Runtime seating is the norm, not an exception |
| `PUB-VERIFY` | steward | `scripted-pr-automerge.sh` exits 0 on a failed push |

★ **`PUB-VERIFY` is the one with teeth:** a publisher that exits 0 on a failed
push means a "published" report can be false. It is *my* tooling, so §10⁻
applies — it waits behind product unless it actually blocks a landing.

### ⏭ Durable rules kept from the closed RT-SPLIT / BUDGET-EFF tracks

The track narrative moved to [`2026/Jul/22.md`](2026/Jul/22.md). These outlived
it:

- ★ **Path-disjointness is re-derived BY THE RECEIVING RING at pickup and
  reported before implementation starts — never asserted by me in a kickoff.**
  I once asserted it *from a string literal* (a constructor **name** inside
  quotes) and the ring caught it.
- ★ **Classify by COMPILATION REACH, never by a `production`/`cfg(test)`
  binary.** A predicate satisfiable with `test = false` is production-reachable,
  and a feature name containing `test-support` does not change that. Every
  independent sweep of that file — mine, the adversary's, the implementer's —
  partitioned on the two-cell binary and was blind to the third cell **the same
  way**.
- ★ **A gate can be incapable of seeing its own subject.** A `default-off`
  feature's rustdoc dump reads identically whether a re-export is right, wrong,
  or **missing** — *incapable*, not weak. Local gates stay green while another
  crate is broken; only CI catches it. ⛔ Never `--workspace` locally to chase
  this — name the one cross-crate test instead.
- ⏭ **Queued small fix:** rustfmt drift at `crates/ken-runtime/src/store.rs:602`,
  **pre-existing on `origin/main`** (verified there, not introduced by RT-SPLIT).
  Touches `crates/` ⇒ normal ring gates; a standalone item, **never a rider**.
- ⏭ Sweep ~20 stale `/tmp/ken-*` worktrees. **ASK before touching any with
  tracked changes.**


### ⛔ PUBLISH DISCIPLINE — tightened 2026-07-22 after invalidating a Decision

I moved `origin/main` under RT-SPLIT slice 2's merge Decision **22 seconds**
after it opened, with a docs-only publish. Third occurrence. **`list_decisions`
run at the top of the work answers a question about the wrong moment**, and no
adjacent check catches a 22-second race. **So the trigger is earlier: hold
Steward publishes whenever a build ring holds a QA-APPROVED CANDIDATE, not
merely an open Decision.** Re-check immediately before the publisher call; if a
publish is urgent, **announce the window in-channel first.** Path disjointness
is irrelevant — §14 is about identity, not conflict.

## Operator rulings — 2026-07-21 ~12:45Z. SETTLED, do not reopen.

**On Linux ABI II** (`research/linux-abi-ii-work-program-proposal.md`):

- **No "ratification."** The charter is a **planning document, not a
  commitment**. Nobody outside the project is watching, and nothing depends
  on our timelines or stated intentions. **I had imported a governance ritual
  that does not apply — do not re-raise status-correction as a decision.**
- **Where there is a gap between what was anticipated and what was done, fill
  the gap first.** Hence `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated by the operator 2026-07-21 after I re-raised it). A very late
  feature, behind a long line of other work. Manifest v2 = family-scoped and
  generated, **not** cross-target.

  ⛔ **This ruling ALREADY ANSWERS any non-linux finding — do not route one
  back as a scoping question.** I did exactly that with "`ken-host` has never
  compiled on any non-linux target" (28 `cfg(not(target_os = "linux"))`
  fail-closed sites, never built, `abi_v1.rs:747`). **Under this ruling that
  finding is inert**: the lane is deferred, nothing builds it, and the defect
  cannot bite. It is dead code for a deferred target, not an open decision.
  Record such findings as *observations against a deferred lane* and stop —
  a settled ruling is a fixed input, never a question to re-ask.
- **L2-0: all desirable, nothing deferred.** All nine
  `RepresentedUnavailable` operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** They monitor
  and adjust. **Do not reason about schedule or cost.**
- ★ **My lane is token efficiency in terms of delivered work.** That is the
  axis to optimize and the one to report on.

**Still genuinely open (lower stakes, no re-ask):**

- **Provider concentration** — only `runtime-implementer` and `adversary` are
  on the Anthropic pool.

**CI-TRACKER-GATE is RESOLVED.** The operator granted the app `workflows:
write`; verified present in the installation's permission set, and a
workflow-bearing push was accepted. Close the issue once PR #804 lands.

> ★ **Diagnosing a `workflows`-permission rejection.** A freshly minted token
> is NOT enough — but neither is assuming staleness. `mint-gh-token.sh` with
> its final extraction changed from `['token']` to `['permissions']` prints
> the installation's **actual** grants. That converts "the push failed, why?"
> into a direct answer. Note the publisher only mints a new token when `gh`
> is not already authenticated, so a cached ~1h token keeps its old scopes;
> force a fresh mint before concluding anything.

## The completion program — written, NOT started · COVERAGE VERIFIED

`docs/program/10-linux-abi-completion.md` — **on `main`**. Four tracks:
**ABI-R** reconcile, **ABI-A** availability promotion, **ABI-M** manifest
(native-target only), **ABI-S** synchronous floor, plus **Track T** the
committed exit (PX10/PX11/PX12).

> **⚠ IDs RENAMED 2026-07-22 (operator).** Tracks R/A/M/S now carry an
> **`ABI-` prefix** — the bare `A3` collided with `issues/A3.md`
> (catalog-coverage walker) and `R1`-`R3` collided with the adversary's
> finding labels. **`PX9`-`PX12` keep their charter IDs.** `L` was rejected
> as a prefix: `L1`-`L7` are existing WPs.

> **★ COVERAGE ANSWER (operator asked 2026-07-21; verified file-by-file):
> 0 of 18 items have an issue.** The only live node of §5's graph is
> **PX8**, its *root*. Everything downstream of `PX8 -> ABI-R3` and
> `PX8 -> PX9` is unframed. §9 of that document is the record.
>
> **AND the document had a hole:** the charter's **runtime revocation
> membrane** (`09` §5) is absent from it. `RevocationHandle { revoked: bool }`
> (`ken-elaborator/src/capabilities.rs:256`) is still the static contract —
> **its own doc comment says the runtime membrane is DEFERRED** — and there is
> **zero** revocation code in `ken-host`/`ken-runtime`/`ken-interp`/`catalog`.
> PX7's generation-checked handle table is a *different* property
> (use-after-close, not withdrawal of delegated authority). **L2 assumes it**
> (§8.1 gate 9). **AWAITING OPERATOR: fold into Track ABI-S / split as its own
> WP / accept as a known limitation.**

Verified against `main`, not taken from the advisory: 22 ops, 13
`NativeTested` / 9 `RepresentedUnavailable`, no process/socket/poll family in
any state, PX7 landed — and **PX9 (cross-domain `System.Error`) is chartered
but undelivered**, which the advisory's "good filesystem floor" phrasing
obscured. PX9 gates most of Track T.

**Not started.** Next step when the operator says go: decompose the tracks
into `docs/program/issues/` entries.

## ✅ TRACK Q IS COMPLETE — nothing outstanding (2026-07-21 ~21:55Z)

**Q-RESIDUE merged: `origin/main @ 64337192` (PR #818)**, from
`wp/Q-RESIDUE-test-rework @ 3f752451`. Verified on main **by content** (the
crate diff against the approved SHA is empty), not by the publisher's exit
code. Issue closed, all three retros in, adversary notified
(`evt_4g7qasxqdy5s8`). Q1 and Q2 merged earlier the same day.

**The fleet is idle and home-clean, and that is CORRECT** — the queue below
is unstarted pending the operator's direction. Do not kick any of it
without their go.

> ### ⚠ THE STALL THAT LOOKED LIKE QUIESCENCE — expect this again
>
> Track Q sat still for ~20 minutes not because it was done but because
> **the architect's composer had two Q-RESIDUE vote requests stacked as
> `[Pasted Content …]` and never submitted.** No `Working`, no turn, no
> vote, and nothing in the system surfaces it. A bare `Enter` released it.
>
> **"The fleet is quiescent" is an observation with at least two causes.**
> Single-threading makes idle rings normal, which is exactly what
> camouflages a wedged pane. On any quiet tick, `capture-pane -S -200` the
> architect specifically and look for stacked pastes — the composer strands
> there repeatedly. See `sweep-wedged-panes-misses-stacked-paste-form`.

> ### ⚠ AND A GREP THAT NEARLY MIS-REPORTED THE MERGE
>
> Verifying the merge, `grep -c 'examples.len() == 5'` on main returned
> **1** — reading as "the frozen count survived." It had not: the match was
> inside a **comment documenting its removal**; the live assertion is
> `!examples.is_empty()`. **Grepping a name instead of the mechanism** —
> the precise failure class this WP existed to remove, committed while
> verifying the fix for it. Read the context, never the count.

> ### ★★ THE RESULT WORTH KEEPING FROM THIS WP
>
> **AC-2's mutation proof caught a bad test before it shipped, on its first
> application.** A first draft of the settlement-ordering test
> *"only hand-sequenced the two helper functions instead of invoking the
> real `unsafe extern "C"` entrypoint, so it wouldn't have caught a real
> regression."* It would have sat **green through an actual defect** — a
> test exercising a **proxy** instead of the **mechanism**, exactly the
> class this WP existed to remove.
>
> The final discriminator is confirmed **three times independently**:
> implementer's authoring run, QA's independent re-execution on the branch,
> and the captured panic (`abi_v1.rs:1590`, `left: 0, right: 1`).
>
> **Require the mutation proof on every future test-rework WP.** It is the
> only step that distinguishes a grounded assertion from a green one.
>
> ⚠ **But do not over-read the three runs.** They are three confirmations
> of **one** discriminator, not three discriminators — all three flipped
> the same seam. A wrong seam produces exactly this agreement. Same shape
> as `differential-oracle-is-blind-to-a-shared-premise`. Raised with the
> adversary at merge; unresolved by design, it is theirs to attack.

> ### ⚠ I MIS-READ THIS WP'S DIFF AND TOLD THE FLEET
>
> I flagged `abi_v1.rs` (+71) as a **production-surface ABI change** in a
> test-rework WP. It is **entirely inside `mod tests`.** I inferred it from
> the file path and line count without opening the diff — same shape as the
> §1 CI misdiagnosis. I corrected it in the flag itself and told the ring to
> **vote on the branch, not on my summary.** They did, and grounded their
> review against real code (`abi_v1.rs:824-841` + the C call site).

## ✅ TRACK Q — DONE (2026-07-21). Only Q-RESIDUE remains, and it is an S.

**Q1 landed. Q2 complete: 428 triaged, 100% classified, six rings in
parallel.** Result: `docs/program/qa-triage/FINDINGS.md`.

| class | count | share |
|---|---:|---:|
| durable-invariant | 392 | **91.6%** |
| compat-vector | 19 | 4.4% |
| transition-sentinel | 7 | 1.6% |
| UNCLASSIFIABLE | 10 | 2.3% |

**Q3–Q7 folded by the operator into ONE S** — `docs/program/issues/Q-RESIDUE.md`
(status `ready`, owner `runtime`). **Not kicked; awaiting operator go.**

> ### ★★ Q4 AND Q7 WERE EMPTY — THE LESSON, NOT JUST THE RESULT
>
> 147 tests flagged for asserting an outcome without naming the variant:
> **every one sound.** All 27 wall-clock flags: **sound.** Both tracks had
> been sized from **scan hit counts**, and hit counts carried almost no
> signal about defects. Authorizing Q3–Q7 off the totals would have reworked
> ~300 correct tests.
>
> **⛔ Do not re-derive Q-RESIDUE's scope from `scripts/qa-risk-scan.py`.**
> It emits a **review queue, not a defect list**. The inventory in the issue
> is the whole of the work.

> ### ★★ THREE DEFECTS IN MY OWN INSTRUMENTS — ALL FOUND BY OTHERS
>
> 1. **The scanner fabricated a test.** An unanchored `#[test]` matched the
>    attribute *in prose* (`rt_parity_native.rs:3` is a doc comment). The
>    phantom swallowed 480 lines of helpers. **Foundation found it by
>    reading source.** `--self-test` passed throughout — it only checked
>    files that HAVE the patterns, never one that would INVENT a row. It now
>    has a negative arm (`NEVER_A_TEST`).
> 2. **"Two counts agreed" was an echo, not corroboration.** I cited the
>    scanner reproducing the documented 1909 as proof it wasn't dropping
>    tests. Both used the same naive match, so both counted prose mentions.
>    True total **1905**. A differential oracle is blind to a shared premise.
> 3. **The aggregator read the wrong file** — Ergo parsed 23 vs a reported
>    71 (glob matched QA's partial share, not the leader's assembly). Caught
>    only because the leader's own count disagreed.
>
> **Every one was caught by an INDEPENDENT source, none by my own checks —
> which were built on the same premises as the things they checked.**

> ### ★ TRANSPORT: a mention proves the EVENT exists, never that it was READ
>
> **Five of six Q2 kickoffs silently failed to deliver.** Repair: `tmux
> send-keys` a pointer to the `evt_…` (point at it, never restate it).
>
> **It reproduces INSIDE a ring.** Kernel stalled at 50/72 — its leader
> delegated, the implementer never got it, and the leader went idle
> believing it had handed off. **Silence and done look identical from here.**
>
> **⛔ Do NOT detect "working" by grepping a spinner word** — the verb is
> randomized ("Gitifying…", "Calculating…", "Crunched for…"). Key on the
> duration/token signature: `\([0-9]+m? ?[0-9]*s · [^)]*\)`. Grepping a
> fixed word read all six busy leaders as dead and nearly caused six
> duplicate re-rouses.

## ▶ THE DOC PROGRAM IS RUNNING — a SECOND, CONCURRENT track (2026-07-21 ~22:5xZ)

`origin/main @ 7610d2a1`. **`DOC-W0` is `active`** and released to a new
**doc team** (`evt_1m7j5qvvm2p2m`). This is the fleet's **one standing
exception to single-threading** (operator): the doc track runs **concurrently**
with build work, because doc WPs touch `library/` and `agent/`, not `crates/`.
**The exception is contention-free-ness, not priority** — a doc WP that would
touch a path a build WP holds defers.

| seat | tier | agent id |
|---|---|---|
| `doc-leader` | T2 Sonnet 5 | `agt_37w6sznc4nw00` |
| `doc-author` | T2 Sonnet 5 | `agt_37w6t02849400` |
| `librarian` | **T1 `gpt-5.6-sol`** | (existing) — the team's **QA** |

**★ Judgment is concentrated on the REVIEWING end, not the authoring end** —
inverted from every other team, deliberately. Documentation fails by being
*confidently wrong*, not badly written; a page whose citation does not carry
its claim reads perfectly. That is a grounding problem, which is where T1 pays.

Frame: `docs/program/12-documentation-program.md` (§0 team, §1 four **settled**
decisions — do not reopen). Overlays: `agent/teams/doc/{leader,implementer}.md`.
There is **no `doc-qa` seat** and no `agent/teams/doc/qa.md`.

**Seat provisioning, if it ever needs repeating:** `moot init` is NOT usable —
its only incremental option is `--force`, which rotates keys for **all**
already-adopted agents and would kill every live seat. Use the API directly:
`POST /api/agents` → `POST /api/registration-tickets/{id}/exchange` for the
plaintext key → write into `.moot/actors.json` (gitignored). PAT is at
`.mootup/credentials`. OAS: `local/refs/convo/docs/api/openapi.yaml`
(operator-sanctioned read; clean-room bars `local/refs/` for *writing Ken's
code*, which this is not).

## My queue, in order

0. **BUDGET-EFF** — Handoff Gate the Spec enclave (spec-leader, spec-author,
   conformance-validator). **Spec erratum FIRST**: `38` self-contradicts
   (`:404-405`/`:443-444` say *effective*, `:419-420`/`:438-440` say
   *requested*), so a code-first fix re-derives the defect from a broken
   citation. It is a **plumbing gap, not a formula fix** —
   `TransferCountV1::new(read, effective)` validates then **discards**
   `effective`, so neither reifier can compute the bound. Two closures with
   different blast radii ⇒ **Architect call**. Oracle
   ⛔ **AC-3 REWRITTEN — my earlier pin was VOID.** The R1 oracle's conclusion
   compares two values computed from its OWN constants (`RAW-count` vs
   `effective-count` = `4 == 0`) and **never reads a reifier field**, so it
   fails on ANY implementation and its failure at `e892777c` confirmed
   nothing. "Must pass unchanged" is **withdrawn**; the oracle must be
   **rewritten to observe the mechanism**. ⛔ **"Confirmed by execution" was
   also FALSE** — the defect rests on **source inspection** of the two
   reifiers. ★ **An adversary repro demonstrates a defect; a completion oracle
   defines correctness. Pinning does not convert one into the other — verify
   an oracle observes the mechanism BEFORE making it an AC.** Detail:
   `docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`
2. **SEAL-2** — re-anchor on current `main`; evidence
   `adversary/SEAL2-repros@70a603da`.
3. Then STR-BIJ → enclave (`ready`, S) · F1 → Architect (`ready`) · F3 ·
   A3 · F4 · RT-SPLIT (**L**, 22k-line `cranelift_backend.rs`).

> ### ⚠ WHERE THIS QUEUE CAME FROM, AND THE `#N` TRAP
>
> **These items are the gap between what was actually DELIVERED and what
> `research/linux-abi-ii-work-program-proposal.md` ASSUMES was already done**
> (operator, 2026-07-21). That proposal is the second Linux-ABI campaign; this
> series fills the hole in front of it. Read the proposal before sizing any of
> them — an item only makes sense against what it assumes exists.
>
> **`#37` / `#39` are indices in a PRE-RESTART STEWARD TASK LIST. They are NOT
> GitHub issue numbers.** Six issue files asserted them as `github:`
> references and the tracker propagated that into a dedicated GitHub column,
> where a task-list index read as a verified external reference — and
> `github: 38` pointed at whatever real issue #38 happens to be. **Corrected:
> `github: null` on both survivors**, with the provenance stated in-file.
>
> **`#38`/`#32`/`#24`/`#25` are DROPPED** — they carried nothing but a number
> (operator: *"no use to anyone"*). Do not resurrect them; there is nothing to
> resurrect. The old `GH-` filename prefix baked the wrong origin into the
> identifier itself — `identifiers-are-claim-artifacts`, in a schema field.

**Readiness is thin behind BUDGET-EFF — only two items are releasable.**
STR-BIJ (`ready`, enclave) and F1 (`ready`, runtime → Architect first).
The rest are `draft`; **A3 has no owner, no size, and no brief** and blocks
F4. ⚠ Verify "no brief" claims by *reading `docs/program/wp/`*, not by
globbing on the ID — the F3 brief is `F2F3-reducer-degrade.md` (it covers F2
and F3 together) and I mis-reported it as missing once already.

## In flight

**`DOC-W0` — ✅ MERGED `origin/main @ 6be9754b` (PR #830), 2026-07-22 ~01:43Z.**
Verified by content: all 8 blobs byte-identical to reviewed `d56abbb1`;
`revision_resolved` in `scripts/gen-doc-status.sh`; both shallow-clone
regressions present by name. The fleet's first `library/` tree. Retros were
being collected at time of writing — **check they are in before treating the
issue as closed.**

**Nine review rounds, six findings, and NOT ONE was a different kind of
mistake.** Every one was a **proxy standing in for the property**:

| # | proxy checked | property that mattered | found by |
|---|---|---|---|
| 1 | rejects a *fake* revision | **accepts a real one, in CI's env** | CI (red) |
| 2 | test clones `file://{repo_root}` | an **independent** history source | librarian |
| 3 | `cat-file` says object present | present **AND** ancestry provable | architect |
| 4 | symlink not *discovered* | symlink **rejected and reported** | architect |
| 5 | SHA reviewed + approved | SHA **on `origin`** (see below) | steward |
| 6 | process fix *agreed to* | seat **can perform it** (see below) | doc-author |

**5 and 6 were mine.** #3 held because the Architect built an isolated depth-1
probe rather than reasoning from source. **What finally stopped the recursion
was naming the predicate once** (`revision_resolved()` = object present AND
ancestry provable) and deriving self-heal, every deepen checkpoint, the
unshallow fallback, and all diagnostics from it — not any individual fix.

**⇒ Carry for DOC-W1 and every gate after it: when a gate depends on an
environment property (history depth, credentials, checkout topology), state
that precondition as a NAMED PREDICATE before writing the check.** A gate whose
precondition is unwritten gets discovered one CI-red at a time, each round
closing an instance and leaving the next layer live.

**New behavior worth watching:** `gen-doc-status.sh` now performs **network
fetches inside a test gate**. Fail-closed-on-unreachable-origin was verified,
but hermeticity under a flaky remote was not reasoned through. Flagged to the
adversary at merge.

**`SPEC-38-ERRATUM` — CLOSED.** Merged `origin/main @ e5a400c7` (PR #827),
retros in. Enclave carry: *keep semantic target / conformance oracle /
implementation mechanism as **separate scopes**; re-anchor with both
current-base and reviewed-subtree byte-identity checks.*
**This unblocks `BUDGET-EFF`, which stays PARKED pending operator go.** The
closure-mechanism call (reply-carries-effective vs. host-caps-the-request-
record) is an **Architect** decision routing *with* that release, not before.

**⛔ `BUDGET-EFF` AC-3 was UNSATISFIABLE and is corrected on `main`.** `count`
cancels on both sides, reducing it to `8 == 4` — no implementation could ever
have discharged it. **And it cannot be fixed in place:** `remaining` does not
occur in `ken-host/src/effect_v1.rs` where the oracle lives; it is built at
`ken-interp/src/eval.rs:4934-4935` and
`ken-runtime/src/cranelift_backend.rs:13081-13082`. The rewrite is **two new
tests** — budget it as plumbing. **The R1 defect itself still stands** on
source inspection; only its demonstration was broken. Branch aligned on
`origin/main`, clean, no orphaned polls. Build fleet idle and home-clean,
which is **correct**, not a stall.

**Also queued, not started: `Q-CLAIM-CLOSURE`** (`issues/Q-CLAIM-CLOSURE.md`,
`ready`, owner runtime) — the adversary's post-merge findings on Q-RESIDUE.
Advisory, **no live defects**. Its generator is worth reading before framing
any future rework WP: *the ACs took the TEST as the unit when the load-bearing
unit was the CLAIM*, so a rework could strengthen one claim, mutation-prove it,
and silently drop its siblings while fully satisfying the criteria. R1 (ABI
fact inventory has no independent anchor — both sides of the check come from
one generator) is the one to sequence first.

**CLOSED — not a scoping question.** `ken-host` has never compiled on any
non-linux target (`abi_v1.rs:747`, `?` on an `Option` in a `Result`-returning
fn; pre-existing since PX5 `049628f8`, adversary confirmed by extracting and
compiling it). 28 `cfg(not(target_os = "linux"))` fail-closed sites, never
built. **Cross-platform is indefinitely deferred (operator) — so this is dead
code for a deferred lane and cannot bite.** Recorded as an observation; no
action, no decision pending. See the L2-1 ruling above.

Recently landed and verified by content: **#827** (SPEC-38-ERRATUM →
`e5a400c7`), **#828** (AC-3 correction + ABI gap status → `9fb90aab`),
**#818** (Q-RESIDUE), **#819**
(Track Q closeout), **#820** (doc program frame), **#821** (doc team),
**#822** (librarian T1 + DOC-W0 release).

## ⚠ FLEET IS MID-RESEAT — leader / implementer / QA seats → Sonnet 5

The operator is reseating the build-team seats (**not** spec-leader). Seats
were cycling as of ~19:00Z.

> ### ★ NEW TRAP — a reseated agent re-posts an ALREADY-CLOSED retro
>
> `kernel-leader` came up on a fresh seat and posted a §10 retro for
> **KTR-1, which closed 2026-07-14 with retros already in** (`65d68cfc`,
> PR #675). Not an error on its part — it reported what its context showed.
> But **counting such a re-post inflates the promotion ladder.**
>
> **Verify every post-reseat retro against the RECORDED state, never the
> report:** `docs/program/issues/<ID>.md` frontmatter (`status: closed`), or
> the diary for WPs predating the issue system. Expect more of these as the
> remaining seats come up.
>
> Contrast RT-PARITY, where the leader's near-identical announcement *was*
> actionable: its retros were genuinely in and only the frontmatter lagged at
> `merged`. **The two look the same from the outside — only the recorded
> state tells them apart.**

**Do not kick any WP until the reseat is complete and the operator releases
one.** Delivering into a seat that is about to restart is the transport
failure the Handoff Gate exists to prevent.

## Programs written, NOT started

- `docs/program/10-linux-abi-completion.md` — the work Linux ABI II presumes.
  Tracks R/A/M/S/T; **PX9 gates most of Track T**.
- `docs/program/11-test-suite-and-ci-remediation.md` — **Track C is DONE.
  CI went 47 min -> ~8 min, landed `8b09fb95`.** Skip the three slow native
  binaries + nextest + shard x4 + `opt-level = 2` on deps + rust-cache
  removed. **Do not shard further** — per-shard parallelism already fell
  3.96x -> 2.5x, so 8 shards would buy ~90s for double the compute. The next
  real reduction must come from `CI-SKIPPED-NATIVE-TESTS` getting faster.
  Details and full scorecard in §1a/§1b. **Track Q (the QA-advisory sweep)
  is untouched and still the actual point of the program.**

  **Skipped-test restoration — measured, not guessed** (§1c/§1d,
  `CI-SKIPPED-NATIVE-TESTS`). Any job finishing under the ~471s critical
  shard costs **zero** wall clock, and that headroom is the budget:
  - `px8f_write_partition` ✅ restored, own `native-slow` job. **C6 gave it
    −22.7%** (309s→239s) vs 8.4% suite-wide — C6's benefit is concentrated
    in cranelift-heavy code, exactly as predicted.
  - `px8f_buffer_native` ✅ restored, own `native-buffer` job. **Measured on
    run 29850680007: Test step 149s, job 224s** — well under the 482s
    critical shard, so it costs zero wall clock, as predicted.
  - `rt_parity_native` — **a ONE-TEST problem.** Parallelizes fine (7 tests,
    266.7s wall / 470.6s CPU), but
    `fs_write_at_malformed_offset_narrows_to_invalid_offset` takes **221.4s**
    vs **42.2s** for its near-identical sibling. Fix that one and the binary
    lands ~90s. **Do not just re-enable it** — today it fits by ~1s, which
    is noise.

  ✅ **Dedicated jobs are now scoped** (`-p <crate> --test <name>`), not
  `--workspace` — that was compiling all 200 test binaries to run one. Now
  **confirmed by isolation**: `px8f_write_partition`'s Test step went
  **241s → 129s (−112s)** across runs 29850405231 → 29850680007 with
  scoping as the only variable, against a ~124s estimate. The `Build` step
  stays `--workspace`: it is only ~65s and it is what proves the workspace
  compiles under `--locked`.

  **Only `rt_parity_native` is still skipped, and the target is ONE test.**

  > ⚠ **The `native-buffer` number was right for the wrong reason — do not
  > cite it as evidence C6 generalizes.** I projected ~240s for that job and
  > measured 224s. But that projection was of an *unscoped* job, and the
  > scoping change (−112s) landed in the **same PR**. Unscoped, the job
  > would have been ~336s — the projection was ~40% high and was rescued by
  > an unrelated bundled change. **Two changes in one PR made a wrong
  > prediction look confirmed.** C6's −22.7% on `px8f_write_partition`
  > remains the only clean measurement of C6 on cranelift-heavy code; it is
  > a single data point, not an established scaling law. Sibling of
  > `green-vs-green does not confirm a fix` — a number matching its forecast
  > is not evidence the reasoning behind the forecast was sound.

  > ★ **I had the CI diagnosis backwards, and the operator caught it.** I
  > claimed a cold dependency build dominated the wall clock. Measured:
  > **build 47s, test execution 44m14s — 95% of the run.** The error was
  > reasoning from *"there is no cache"* (true) to *"the build is the cost"*
  > (never checked) without opening a single run log. The logs were
  > available the entire time. **An explanation for why something COULD be
  > slow is not evidence that it IS.**

  Measured distribution: `cargo test` walks its **200 test binaries strictly
  in series**, and **three of them — nine tests — are 56.5% of the whole
  run** (`rt_parity_native` 14m41s, `px8f_buffer_native` 5m10s in a *single*
  test, `px8f_write_partition` 5m09s in a *single* test). The bottom 150
  binaries total **48 seconds**. All three fat binaries do a real native
  codegen-and-link per test case.

  > **Operator ruling 2026-07-21: remove `Swatinem/rust-cache` as part of
  > C6** (tracked as C8). No measurable benefit, and it is a third-party
  > dependency with access to the build — a supply-chain surface taken on
  > for nothing. **A dependency must earn its place.** My counter-argument
  > (it absorbs C6's rebuild) was weak: it defended a dependency on an
  > untested hypothesis and priced only time, never trust.
  >
  > ⚠ **C6 and C8 are in latent tension** — C6 can only *increase*
  > dependency compile time, and C8 makes every run pay it. **The C6 run
  > must report the Build step**, not just test numbers. Thresholds are
  > pre-committed in §3b; if the build blows up, **return it to the operator
  > with the number** rather than quietly reinstating the cache.
  >
  > ⚠ **C2 added `taiki-e/install-action@nextest`, unpinned — same class of
  > exposure.** It is defensible because nextest earns it (it fixes the
  > actual problem) where the cache did not, but **pin it to a commit SHA**.

  **Next steps are C2 → C6 → C7, re-measuring between each:** nextest (one
  global pool replaces the serial walk), `[profile.dev.package."*"]
  opt-level = 2` (cranelift runs its codegen unoptimized — **hypothesis,
  test it in CI, do not merge on plausibility**), and splitting the two
  1-test binaries (unsubdividable, so they become the critical path the
  moment C2 lands). C1 landed and bought ~5s — **do not report it as a
  throughput win**.

  ⚠ C7 and Q7 are one edit from two sides — splitting the native binaries
  for parallelism, and giving temp dirs per-test ownership so that
  parallelism is safe. Do them together or expect a flake that reads as
  "nextest broke the suite."

  **`scripts/ci-test-timings.sh <run-id>`** regenerates the per-binary table
  from any run's log. Granularity is the binary; per-`#[test]` needs C2.

Next step for either program when the operator says go: decompose its tracks
into `docs/program/issues/` entries.

## Tooling traps — distrust a clean negative

> ### ⛔⛔ `git maintenance` CAN STARVE THE WHOLE BOX — config now guards it
>
> **2026-07-21: `git pack-objects` consumed all 8 cores; load hit 14.** Root
> cause is structural and will recur if the config is ever reset:
>
> ```
> maintenance.lock  -> .git/worktrees/<name>/maintenance.lock   PER-WORKTREE
> gc.pid            -> .git/gc.pid                              shared
> objects/          -> .git/objects                             shared
> ```
>
> **`git maintenance` locks PER-WORKTREE but repacks the SHARED object
> store.** A run in one worktree is invisible to a run in another, so the
> concurrency ceiling is **the worktree count — 30**, each defaulting to all 8
> cores. The legacy `git gc --auto` path did *not* have this hole (`gc.pid` is
> common, so the second bails). `git maintenance` lost that protection.
>
> **Guard now set repo-locally** (covers all worktrees via the shared store):
> `maintenance.auto=false`, `gc.auto=0`, `pack.threads=2`,
> `pack.windowMemory=256m`. **`maintenance.auto=false` is the load-bearing
> one** — capping threads alone still allows 30 × 2 contending.
>
> **Consequence you are now carrying:** loose objects accumulate forever.
> A deliberate `git gc` is needed during a genuinely quiet window (fleet
> idle, no WP in flight). **Never run it while a team is working.**
>
> ⚠ **Trigger to avoid:** a `git add -A` run from `/workspaces/ken` (the MAIN
> worktree) sweeps untracked `.cache/`, `.targets/`, `.tmp-*` into the object
> store, blowing past the loose-object threshold and firing maintenance from
> every worktree at once. **A `cd` chained before a broad `git add` silently
> changes which repo it applies to.** Those blobs are still present as
> unreachable objects pending a prune.

> ### ⛔ A PANE SNAPSHOT IS NOT AGENT STATE — three variants seen in ONE day
>
> | symptom | actual state | repair |
> |---|---|---|
> | stacked `[Pasted Content …]`, no `Working` | alive, **never submitted** | bare `Enter` |
> | pane **entirely blank** (even `-S -200`) | alive, blocked on a **consent modal** rendered at the buffer's START | capture from `-S -` and `grep -v '^\s*$'`, then `Enter` |
> | empty prompt, looks idle | **actively working** — narrow `tail` caught a gap between renders | capture WIDE before repairing |
>
> **★ The third one recurred TWICE on 2026-07-22** — once reading `doc-author`
> as never-engaged (it had already finished), once rousing it *while it was
> mid-fix* with `is_symlink_escape` already in its diff. **Interrupting a
> working agent is forbidden by the playbook and I did it anyway**, because a
> `tail -4` showed a bare `❯`. The spinner renders **above** the prompt. A
> `WORKING` check must grep the whole capture for `esc to interrupt` — never
> judge from the last few lines.
>
> The third one bit me *while running the check designed to catch the first*.
> Had I trusted it I would have stacked a duplicate kickoff on a working seat.
> **Always `capture-pane -S -` piped through `grep -v '^[[:space:]]*$'` before
> concluding anything about a seat.** A new seat's first launch is exactly when
> nobody is watching — the consent modal for
> `--dangerously-load-development-channels` blocks silently and
> indefinitely.

- ⛔⛔ **AFTER EVERY MERGE, RE-BASE `steward/work` ONTO `main` BEFORE THE
  NEXT COMMIT.** Cost three publish cycles on 2026-07-21 — the same trap
  each time. `main` merges are **squash** merges, so `steward/work` never
  contains the resulting commit: its merge base stays at the *previous*
  main, and GitHub's three-way merge then conflicts on any file both sides
  touched, even when the content is compatible.

  > ★ **`git diff origin/main..HEAD` will NOT warn you.** A two-dot diff
  > shows the **net difference**; a merge asks a **different question** —
  > what happens when both sides' changes are replayed from a shared base.
  > A clean two-dot diff next to a conflicting merge is not a contradiction.
  > **The check that actually predicts a conflict is**
  > `git merge-base --is-ancestor origin/main HEAD` — if that fails, rebase
  > *before* committing anything further.

  Recipe (content-preserving, verified three times):
  `git tag -f preserved/<sha> HEAD` → `git reset --hard origin/main` →
  `git checkout <old-sha> -- <changed files>` → regenerate the dashboard →
  commit. Then confirm with `git diff <old-sha> HEAD`: the **only** expected
  delta is `IMPLEMENTATION-PROGRESS.md`'s timestamp line.
  ⚠ Do **not** verify with a bare `git diff` after `git checkout -- <path>`
  — that stages the files, so unstaged `git diff` reads empty and looks like
  a mismatch. Compare **commit to commit**.

- ⛔ **`scripts/scripted-pr-automerge.sh` exits 0 on failure** (4 times on
  2026-07-21). Its **first attempt after any merge always fails** with
  `stale info`, because the merge deletes the origin head branch and stales
  the local ref. **Always `git fetch origin --prune` before publishing.**
  ⚠ Its `--description-file` must exist **before** the call — a heredoc
  inside the same `&&` chain does not reliably land, and the script reports
  `description file not found` and exits.
  Redirect its output to a file — a `| tail` pipe block-buffers it to 0
  bytes. Afterwards it sleeps ~40 min polling a PR that may already have
  merged; verify `origin/main` by content and kill the orphan. Tracked as
  issue `PUB-VERIFY`.
- **A piped exit code belongs to the last command in the pipeline**, not to
  `git`. Verify `HEAD` moved.
- **Branch-ahead ⇏ unmerged** (squash-merge trap). Verify by content.
- **Concurrent subagents in one worktree share `.git`** — path-disjoint is
  **not** commit-disjoint. Two raced the index on 2026-07-21. Use
  `isolation: "worktree"`.
- **`convo` posting can fail while the channel stays up.** An absent post is
  not a stalled agent — check the pane.
- **A literal-string grep is a proxy, not the property.** "Four tools" had
  been upgraded to "Five tools"; the grep read as content loss and nearly
  discarded good work. Grep the theme, then read the hit.
- Liveness: `tmux capture-pane -p -S -300 -t moot-<seat>` — **`-S` must be
  negative**; a positive value returns ~1 line and reads every seat as dead.
- `Press up to edit queued messages` = **busy + queued. Do not resend.**

## ⛔ "COMMITTED" IS NOT "REACHABLE" — publish, then verify ON MAIN

**2026-07-22, caught by the adversary, not by me.** I corrected `BUDGET-EFF`'s
AC-3, announced *"all four folds are in"*, and it was true **about
`steward/work` only**. `COORDINATION §15` sends rings to **`main`**, so for
the whole window a ring picking up the WP would have read the **unsatisfiable**
AC-3. Five commits, zero publishes.

**The rule, mechanically:** a Steward doc edit is not done at `git commit`. It
is done when `git grep '<plain phrase>' origin/main -- <file>` returns the new
text. Route via `§6a` (corpus branch off *current* `origin/main` → publisher
path → **verify by content**).

Two amplifiers, both real:
- **The publisher prints `merge command succeeded` on a failed push** — that is
  the open `PUB-VERIFY` issue. Its exit code is worthless; only content on
  `main` counts.
- **`git grep` is case-sensitive and false-negatives on a phrase spanning a
  line break.** Three greps false-negatived on 2026-07-22 alone. Grep a short,
  lowercase, single-line fragment — never a sentence, never across `**bold**`.

Same family as the whole week: **verify the mechanism, not a proxy.**
"Committed" is the proxy; "on `main`" is the mechanism.

## ⛔ A `git_request` SHA IS *REVIEWED*, NOT *PUBLISHED* — `ls-remote` FIRST

**2026-07-22, DOC-W0, caught with ~30 seconds to spare.** `doc-leader` sent a
`git_request` for `d56abbb1`. **`origin` was still at `8f14ff83` — the exact
SHA that had already failed CI on PR #830.** Four folds lived only in
doc-author's worktree. Running the publisher as requested would have rebuilt
the known-red commit and looked like a *regression of an already-fixed bug*.

**Why nothing upstream catches it:** the agent worktrees **share one object
store** (`/workspaces/ken/.git`), so an unpushed SHA resolves **perfectly** for
`git log`, `git diff`, `merge-base --is-ancestor`, `git grep`, and a full
detached test run. **Librarian exact-SHA QA and Architect exact-identity review
both passed on a commit that was not on `origin`.** `git ls-remote` is the
*only* check that separates reviewed from published.

**What actually exposed it:** a **number disagreeing with a report** — my scope
diff printed **900** lines for the gate test where doc-author reported ~1200.
Cross-checking a reported magnitude against my own measurement caught it; no
identity check I ran did.

**⇒ HARD PRECONDITION of every publish, before `scripts/scripted-pr-automerge.sh`:**

```sh
git ls-remote origin refs/heads/<branch>   # MUST equal the requested SHA
```

If it does not match, **push it yourself** — mint via
`.devcontainer/mint-gh-token.sh`, then
`git push https://x-access-token:$TOKEN@github.com/ken-topos/ken.git <sha>:refs/heads/<branch>`,
and **re-verify with `ls-remote` after**. Often a clean fast-forward (the stale
head is an ancestor) — check before assuming a force is needed.

**⛔ DO NOT delegate this push to the authoring seat. NO BUILD SEAT HAS GITHUB
CREDENTIALS** — only the scripted publisher and the Steward. I issued exactly
that carry, doc-author accepted it, then *tested* it and hit `could not read
Username for 'https://github.com'`. **A process rule assigned to a seat that
cannot execute it is worse than the gap it closes** — everyone believes it is
handled. **Verify a seat CAN do a thing before making it their duty.** QA seats
may carry "candidate SHA present on `origin`" as a **detection** item; the
remedy always routes to the Steward.

## Standing discipline

A success signal says a thing **ran**, never that it did what you meant —
**verify by content**. `git rev-parse --abbrev-ref HEAD` must read
`steward/work` before any write. Local builds are **targeted only** via
`scripts/ken-cargo -p <crate>` — **never `--workspace`** (it OOMs the box).
Workspace-green means green in **CI**.
