# Current briefing (live — read this first on every Steward resume)

> ## HOW TO READ THIS FILE, AND WHEN TO DISTRUST IT
>
> **`origin/main` outranks this file, always.** If anything below tells you to
> do something `git fetch origin` shows as landed, **this file is stale and the
> repository is right.** Re-read fresh, in this order:
>
> 1. `git fetch origin && git rev-parse origin/main`
> 2. the LIVE block below — **only** the LIVE block
> 3. the open tasks (do not re-derive priority from memory)
> 4. for what is HELD, DEFERRED, or WHOSE it is: **the node**
>    (`docs/program/issues/*.md`), its operative block — never this file
>
> **This file is a resume POINTER, not an archive. Git is the archive.** When a
> window closes its block is **deleted**, not demoted to a "superseded" section —
> a superseded block left in the file gets read by someone, eventually.

> ### REWRITTEN 2026-07-26 ~19:4xZ — 2866 lines → this. Read the bound.
>
> The prior content (~2700 lines of windows stacked back to 2026-07-21) is at blob
> **`c26ee67f29d42690f461d43fe15e21c2202a31df`** — `git show c26ee67f`. Nothing was
> lost; it was archived to git with this pointer.
>
> **HONEST BOUND ON THE AUDIT: I did not read all 2866 lines.** I read every
> heading, the blocks claiming to be authoritative, and then **scanned** the
> remainder for sole-source markers, decision ids, held items, and preserved refs.
> ⇒ **That is a scan, not an exhaustive audit**, and its surface was my own idea of
> what "load-bearing" looks like. A reader who needs something from before
> 2026-07-26 should assume it is in `c26ee67f`, not that it was considered.
>
> **What the scan found is why the rewrite was worth doing: two blocks that
> advertised themselves as authoritative were WRONG** (see *Corrections*), and a
> hand-maintained list of 6 preserved refs when origin held **26**.

## LIVE — 2026-08-08 ~09:0xZ · ContinuationSpecialization COMPLETE; `RT-RECURSOR-TRANSPORT` recut and HELD

**`main = 80d41892`.** Zero open PRs. Runtime ring idle by instruction, not by
stall — see the hold below.

**The campaign is done.** All four ContinuationSpecialization seams merged:
`ASSEMBLY`, `ACTIVATE`, `LEDGER`, `WITNESS`. `RT-CONTSPEC-WITNESS` merged as
**PR #1597 at exact `2149e8c1`**, CI green, all five paths verified
blob-identical, M7 flip landed, Adversary notified — it hunted and found **no
defect** (`evt_ac42x3xhqa31`).

**Merged this session, in order:** #1582, #1584, #1585, #1586, #1587, #1589,
#1590, #1591, #1592, #1593, #1594, then #1595 (the `RT-DESCENT-RETIRE`
dependency-edge repair), #1596 (glyph cleanup), **#1597 (`WITNESS` code)**,
#1598 (M7 flip + `RT-RECURSOR-TRANSPORT` recut), #1599 (campaign-doc sweep).

**Campaign state.** Three of five `RecursiveDescentResidual` classes retired
(`TransparentDeclarationClosure`, `SeedClosureCall`, `ProducerMatchCall`).
Surviving: `MatchScrutineeRecursor`, `LexicalCallArgumentRecursor`, both live in
`lowering/core.rs` and both owned by `RT-RECURSOR-TRANSPORT`.

> ### HOLD — `RT-RECURSOR-TRANSPORT` IS RECUT BUT NOT RELEASED
>
> It is `ready` and unblocked in the tracker as of #1598. **That is not a
> release.** Both `runtime-implementer` and `runtime-leader` have acknowledged
> the hold and are correctly refusing to treat the edge clearing as one.
>
> **Blocked on:** the Architect's bounded re-read of the campaign-doc sweep
> (#1599, `80d41892`). My post `evt_1fzv70k8cgnzt`; their finding
> `evt_4a8eb00h5349t`. **When they clear it, release explicitly** to
> `runtime-leader` (`agt_37reqrd72cg00`) with the `COORDINATION §4a` anchor line.
>
> **Open question I put to them and must not drop:** the `PX8` release-gate block
> above the fence asserts `D7` *"lands atomically with `483ef7ab`"*, which did not
> happen. I gave it a dated status line rather than a rewrite, on the grounds
> that its gates have all merged and it routes nobody to a wrong base. If they
> read that boundary differently, rewrite it.

> ### THE RECUT — Architect `evt_237tbdsacqbk4`, three withdrawals
>
> Answering my re-derivation request `evt_4hr31qp6ab5xg`. Node and frame
> **rewritten whole**, not patched; the frame was 730 lines of five layered
> recuts citing closed nodes.
>
> 1. **The global `BoundaryUse` population authority is withdrawn as superseded,
>    not unfinished.** Zero hits in `crates/`. `D7`'s landed authority is
>    `PlannedEffectSeat`, discharged for its own host-effect domain. **There is no
>    missing universal authority** — separate exact authorities per semantic
>    population is the design.
> 2. **The "population authority FIRST" ordering is withdrawn** as a pre-`D7`
>    artifact. Replaced by `D0`/`D1` re-census and activation probe → conditional
>    `D2` narrow consumer port → `D3` joint retirement.
> 3. **`07ce6ef1` is NOT the repair base** — and this is the dangerous one. It is
>    **not an ancestor of `main`**; `StaticRecursorWorker` has zero hits on `main`
>    and the four core files diverged by **`+58,582/-17,365`** (at `837f9296`).
>    Resuming it would have overwritten the landed architecture. The node had
>    protected it as the base since 2026-07-29.
>
>    **The `+44,986/-16,942` I first published was wrong, and I inherited it
>    rather than deriving it** — it came from the Architect's own ruling text and
>    I transcribed it into four artifacts. Caught on their bounded re-read
>    (`evt_6x0d8h54gpvvm`). A figure handed to you by an authority is still an
>    unverified figure; the fix carries the command so the next reader can
>    re-derive it in one line.
>
> Size `L` → provisional `M`. `D1` may close a class for free and is the re-size
> point.

> ### WHAT I GOT WRONG THIS WINDOW, positionally
>
> **I published #1598 while the Architect's review of that same object was in
> flight.** Ruled and written is not reviewed. It turned a pending correction into
> a live contradiction on `main`: the corrected node and frame were merged while
> the campaign document — which both instruct the implementer to read *first* —
> still carried the withdrawn contract in five present-tense blocks.
>
> **Why my own sweep missed those five.** I swept the two sites structurally like
> the thing I was changing (the DAG edge, the schedule row — both tabular, both
> obviously sequencing). The five were **prose chronology under dated headings**,
> and I read them as history. ⇒ **A dated heading dates the OBSERVATION, not the
> instruction underneath it.**
>
> **A reviewer's named site count is a floor, not the perimeter.** Twice today:
> two flagged → six found on `WITNESS`; five flagged → six found here.

**Seam 3 was recut mid-flight from populate to DELETE** (Architect
`evt_1v9m7t4m9dmj7`, sustaining hard stop 7). The four `BoundaryUse*` axes were
an unowned schema fragment; zero occurrences now remain in `crates/`. The same
ruling **superseded the Architect's own `evt_40ra70t92mjd2`**:
`RT-DECL-CLOSURE-PORT` `D7` never owed a global boundary-use record, and is no
longer a `depends_on` of the ledger seam.

> ### DONE ~07:5xZ — `RT-CONTSPEC-WITNESS` IS RELEASED AND ACTIVE
>
> Kick `evt_6x15cftnmmycg`, base `main = 47ef28b1`, frame blob `f7ec00d2`, node
> blob `9b2f2fcf`. All three Runtime seats compacted to **ctx 0%** before the
> kick (implementer verified 27% to 0%). Leader confirmed `Working`.
>
> **The next Steward action is the merge**, when the ring posts a checkpoint SHA
> and its Decision resolves. Read `merge-procedure.md` at point of use.

> ### TWO OPERATIONAL FACTS MEASURED TODAY — both cost a round before they were caught
>
> - **`propose_decision` does not wake the Architect, and prose naming it wakes
>   nobody.** The `RT-CONTSPEC-LEDGER` ring sat blocked ~50 minutes on an
>   Architect vote while that seat's last state still read "Runtime remains
>   stood down." **If a merge Decision sits `proposed`, mention `@architect`
>   explicitly.**
> - **The publisher can fail while its wrapper reports exit 0.** It DELETES the
>   head branch on merge, so a follow-up push is rejected `stale info` and the
>   task still exits clean. **Verify every publish by CONTENT on `origin/main`,
>   never by exit code or SHA.** Recover with `fetch --prune`, `reset --hard
>   origin/main`, cherry-pick, republish.

> ### THE ADVERSARY EDGE IS ONE-DIRECTIONAL — I BREACHED IT TODAY, RECORDED SO IT IS NOT REPEATED
>
> `COORDINATION §10⁻a`: the Steward **may** notify on a code merge and **may**
> receive reports. **Nothing else** — no acknowledgement, no thanks, no routing
> note, **no correction of the adversary's framing**, no reply of any kind.
>
> My `RT-CONTSPEC-LEDGER` merge notification embedded triage commentary
> ("your last report is triaged", a retrieval remark). **That is an ack and it
> is banned.** The rule's own rationale is that the ack is where the servicing
> loop restarts, and a Steward who may reply "just this once" has no rule.
> ⇒ **Merge notifications carry merge FACTS only.** Act on findings inside
> product work, or not at all.
>
> Two of its corrections, both verified and both mine: I cited the `D4` prose
> sites as `core.rs:4729`/`:6304` when the file is `static_transition.rs` — the
> line numbers were right and I transcribed the wrong filename from my own
> grep. **A citation naming a REAL thing that is not the thing is worse than one
> naming nothing**: an auditor opens live unrelated code and concludes the
> deliverable did not land. And its reports are **not** threadless root posts —
> they carry `parent_event_id` and return a `thread_id`; my `get_thread` 404 was
> misuse, not an absent thread.

> ### UNRESOLVED, and it needs the Architect — route it when `RT-RECURSOR-TRANSPORT` nears the frontier
>
> That node's frame carries a "what this node now owes" paragraph written while
> `RT-DECL-CLOSURE-PORT` `D7` was unlanded, demanding a `BoundaryUse` record per
> static lowering event. **I flagged it rather than resolving it** (2026-08-08).
> The `BoundaryUse` it names is the **host-effect** population, not the four
> deleted continuation axes — same word, different domain, and conflating them
> is the exact confusion `D7` was built to prevent. **The flag is a caution the
> text is stale, NOT a ruling the obligation is discharged. Nobody has measured
> that.** Ask the Architect as "what remains owed against the merged `D7`?"

> ### OPEN RISK CARRIED OUT OF `RT-SEED-CALL-PORT` — a cost that MOVED, with no code change to prompt a re-read
>
> **Nobody has ever executed an `AC-6` control under a committed swap
> mutation** — not the ring, not QA, not the Architect, not me, not the
> Adversary. That was **harmless at `D2`**, where the port was production-inert
> and the controls guarded a path nothing took.
>
> **At `D3` the port went live. Those same unmutated controls are now the only
> thing standing behind the `Parameter ++ Capture` ABI order in production.**
>
> ⇒ **Nothing about the controls changed, so nothing prompts anyone to re-read
> their strength.** The identical control silently went from guarding nothing
> to guarding the real path. **This is a general shape worth catching
> elsewhere: activating a mechanism re-prices every control in front of it,
> and the re-pricing is invisible in the diff.**
>
> The structural argument is strong — `AC-6.3`'s oracle is a fresh literal and
> the callee ABI is planner-owned, so a swap silently computes `-3` — but
> **structural closure is not execution.** Route this if `RT-PRODUCER-MATCH-PORT`
> or a later node touches that transport.
>
> ### OPERATOR SEQUENCING RULING — 2026-08-07, verbatim intent. THIS GOVERNS.
>
> **"continue with the D-series, then return to `RT-SEED-CALL-PORT` and
> continue from there."**
>
> ⇒ **Finish `RT-CARRIER-BYTESPAN-OBSERVE` `D4`, `D5`, `D6` first.** Then the
> **RecursiveDescent retirement campaign resumes, entering at
> [[RT-SEED-CALL-PORT]]** — which is `ready`, framed, and unblocked
> (`RT-DECL-CLOSURE-PORT` is merged).
>
> **This ANSWERS the open sequencing question and it is no longer mine to
> weigh.** I had raised that the runtime ring had been off the campaign's
> critical path for two days while unblocking work compounded, and flagged it
> as a priority call between ready WPs — the operator's, not mine. It is now
> decided: **D-series first, campaign second, `RT-SEED-CALL-PORT` is the named
> entry point.** Do not re-litigate it and do not insert a third thing ahead of
> the campaign without asking.
>
> **Note on `RT-SEED-CALL-PORT` when you get there:** its `D1` may legitimately
> return "already retired" if `RT-DECL-CLOSURE-PORT`'s machinery subsumed
> `SeedClosureCall`, at which point the node closes for free. That is recorded
> in `docs/program/16-recursive-descent-retirement.md §2` as a **prediction, not
> a measurement** — measure it, do not assume it either way.
>
> **PR #1528 is CLOSED** (operator, 2026-08-07). The stale-branch revert hazard
> is retired and there are now **zero open PRs**.

> ### NEXT ACTION ON RESUME — read this line first
>
> **`D1` MERGED. `D2` IS UNDER REVIEW. `D1` inverted the prediction —
> `SeedClosureCall` FIRES, no free close.** `main` **`0b232f42`**, worktree
> clean, zero open PRs, **nothing owed by me**.
>
> - **`D1`**: PR #1563, exact `05e6e801`, both paths blob-verified. M8 sent.
>   Adversary hunted it: **no defect**.
> - **`D2`**: candidate `d4856f194fd56d619b063f2c7c02822864fa03a6`, under
>   QA/Architect review. **It merges on its own when accepted — do not hold it
>   for `D3` or for `AC-5`.**
> - **Node stays `active`** until `D3`. Flip to `merged` only when it closes.
>
> **The `D2` fork was RULED (Architect `evt_7p8dmg1rez02c`): PORT the
> capability, do not delete or defer.** No elaborator proof owed on this
> branch. The six mechanism points and `AC-6` are in the frame — build and
> review against the frame, not against the thread's restatements.
>
> **`AC-5` IS `D3`'s, ruled at `evt_7y8qz0raz6x0k`**, and most of its
> instrumentation already exists (`scale_b_record_function`,
> `lowering/mod.rs:213`, per-emitter incl. `RecursiveDescentRoot` and
> `FunctionizedUnitBody`). Only the per-function **distribution** is missing —
> the existing metrics are sums. **Report the gap; do not add a parallel
> recorder.**
>
> **The node is NOT deferred.** "Buys no per-function-ceiling benefit" is not
> grounds: the operator's directive is that a half-migrated state is debt for
> no benefit, and `RT-DESCENT-RETIRE` is blocked on this class either way.
>
> ### The population is FIXTURE-ONLY, and I verified that myself
>
> `SeedClosureCall` fires only on hand-authored `RuntimeExample` IR. The
> implementer labelled their elaborator census **MEASURED-BY-SOURCE-CENSUS, NOT
> EXECUTED** and handed it over rather than absorbing it. **I re-derived it and
> it holds** — do not re-derive it a third time:
>
> - `ken-elaborator` has **5 `RuntimeExpr::Closure` constructions, 4 match
>   arms**. `erasure.rs:268`/`:2072` construct into
>   `RuntimeDeclarationKind::Transparent`; `:3956` is `lower_top_level_body`
>   whose **sole** caller `:3915` wraps it `Transparent`; `:4550`/`:4556` is
>   `shift_runtime_vars`, position-preserving; `:6468` is under
>   `#[cfg(test)] mod px7l_tests`.
> - `CheckedCoreBodyTerm::Lambda` lowers to `LexicalClosure` at `:4325`.
> - **I closed the domain gap the implementer flagged.** `px7l:115` and
>   `px8l:63` are **match arms, not constructions**, and `ken-interp`
>   references neither variant ⇒ no crate outside `ken-elaborator` and
>   `ken-runtime`'s fixtures constructs the shape. The wider domain **cannot**
>   add members.
>
> **THE FACT THAT CUTS THE OTHER WAY, and it is the Architect's to weigh:**
> `RuntimeExpr` is **public API** — `ken-runtime/src/lib.rs:38` `pub mod ir;`
> and `:68` `pub use ir::*;`. ⇒ *"No elaborated Ken program produces it"* is
> **not** *"unreachable"*; deleting the variant narrows a **published**
> surface.
>
> ⇒ **An executed elaborator proof is owed ONLY under the
> delete-the-capability branch.** A source census is sound for "no in-tree
> producer constructs this" — a closed enumeration, verified. It is **not**
> sufficient to justify deleting a published variant. Under "port it", no
> proof is owed.
>
> `RT-CARRIER-BYTESPAN-OBSERVE` is **CLOSED** (PR #1555, exact `f49a2255`, 15
> paths blob-verified; M7/M8/Librarian all discharged). Nothing owed on it.
>
> **OWED BY ME, DEFERRED ON PURPOSE:** route the [[RT-SITEOP-CARRIED-WITNESS]]
> mechanism fork to the Architect **when that node approaches the frontier, not
> before** — routing it now inserts work ahead of the campaign. Frame it as
> *"which mechanism?"*, never *"may I do X?"*.
>
> ## `RT-SEED-CALL-PORT` IS KICKED — `evt_51606kvqqjhba` is its thread anchor
>
> Released from `origin/main` `1b877875`. Compaction gate discharged first:
> implementer **ctx 0%**, QA *"Context compacted"*, all three worktrees clean
> and zero ahead. **The RecursiveDescent campaign has resumed.**
>
> **Expect a free close, and do not let that expectation do the measuring.** The
> node's `D1a` exact-set gate is the whole point — a reachability control passes
> while a short-circuiting enumerator is the defect it exists to catch.
>
> ## I CORRECTED MY OWN KICK — the enumerator EXISTS. PR #1560, `main` `211a208a`
>
> **My kick told the ring `D1` included BUILDING the enumerator and that a
> re-size would follow. Both were wrong**, and I caught it ~7 minutes in, while
> they were building it. Correction posted at **`evt_60kx15saf97ve`**.
>
> Measured at `origin/main`, landed by `RT-SRCBODY-BIND-ORDER` (`7ca5cfc0`):
>
> | what | where |
> |---|---|
> | entry point | `lowering/core.rs:598` `enumerate_recursive_descent_residuals` |
> | non-short-circuiting walk, `BTreeSet`, no wildcard | `core.rs:616` `collect_recursive_descent_residuals` |
> | `SeedClosureCall` classified | `core.rs:707` |
> | `D1a`'s exact-set control, live `#[test]` | `core/tests/control.rs:10849` |
> | its `SeedClosureCall` firing witness | `control.rs:10723` |
>
> The control asserts `assert_eq!` over a `BTreeSet` of four variants — exact
> set, not membership. **The re-size is WITHDRAWN; `M` over-covers.** `AC-2`'s
> lineage half is discharged by inheritance.
>
> **HOW I GOT IT WRONG, because this is the reusable part.**
> `RT-DECL-CLOSURE-PORT`'s enumerator genuinely never entered the candidate
> lineage — **true when that node closed.** I carried it forward and never
> re-derived it after a *later, different* node landed a durable one. **A fact
> about the tree decays. Re-derive it against current `main` at each use.**
>
> **WHAT STILL BINDS — do not let the correction over-swing.** `D1`'s real
> question is untouched: does `SeedClosureCall` fire on the **committed program
> corpus**? And **a test witness is not a population** —
> `d1_seed_closure_call_witness()` is a hand-built `RuntimeExpr`, not a Ken
> program; it proves the walk can *see* the class, nothing more. The control
> must also be **run by name**, since later deliverables rewrite `core.rs`
> underneath it: presence is not greenness.
>
> ## TWO ADVERSARY FINDINGS ON THE `D5` MERGE — one live, both dispositioned
>
> 1. **`library/learn/reading-ken/06-execution.md:186-199` is FALSE on `main`.**
>    It says five `assert_narrowed_alike` rows await
>    `RT-CARRIER-BYTESPAN-OBSERVE`; that node **closed without re-arming them**,
>    so a reader who checks it concludes the differential is restored when it is
>    not. **FIXED AND MERGED — PR #1558**, `dec_3wpsmg08xmvaa` resolved by me
>    under §14a (`library/`-confined ⇒ Librarian verdict + diff-scope, no
>    Architect). **The chapter now states the CAUSE and names no node id at
>    all** — verified: zero occurrences of `RT-CARRIER-BYTESPAN-OBSERVE` in it.
>    **The lesson is bigger than the fix: the `LIB-ASBUILT` guard argued the
>    passage could not silently invert BECAUSE it named both owner nodes — that
>    assumed the node would land AND re-arm. It landed and closed. A guard
>    written for a whole deferral points the wrong way when only part of it
>    lands**, and the naming meant to protect the passage is what misled.
> 2. **The 5 rows in `rt_parity_native.rs` say "awaiting Steward recut" — the
>    recut already happened.** Folded into
>    [[RT-SITEOP-CARRIED-WITNESS]] `D1a`: repair the pointer, keep the
>    diagnosis.
>
> **CORRECTION TO MY OWN M8, in my favour and recorded so I do not repeat the
> weaker claim.** I told the Adversary `AC-11`'s discharge *"rests entirely on
> the `BOUNDARY_LOCAL_HELPERS` inventory staying closed"*, flagged as an
> assumption. **It is better than that: the inventory is pinned by
> reconciliation at `boundary_value_clif.rs:3712-3788` and `:5410`**, which
> compares it against what is actually emitted and fails telling you to extend
> it. **That closure has a MECHANISM behind it, not a convention** — the
> opposite outcome from `rebuild_from_collected`.
>
> **UN-RE-DERIVED BY ANYONE: the 3/4 seven-call census partition.** I took it
> from QA and the Architect; the Adversary tried to re-derive it and its three
> instruments each measured the wrong population. **Nobody has independently
> confirmed it.** Also unread by anyone: 23 of the 29 restated quarantine
> reasons.
>
> ## `D5` WAS HARD-STOPPED AND RE-CUT — 2026-08-07 (PR #1554)
>
> **29 of the 30 quarantined rows do not discharge from the byte-span
> mechanism.** Each `Fs*` path seat is read twice: once as a wire span, which
> the observer satisfies, and once as `SiteOperand(0)` of the synthesized
> `FileError`, which demands a **compile-time `Lowered` template**. Supplying
> that from a boundary word is the `Carried -> Lowered` inverse §5 bans.
>
> **I confirmed it structurally, independent of the implementer's runtime
> measurement:** `site_operand_argument` (`lowering/mod.rs:11354-11362`) calls
> `seats.specialized(...)?`, and `mod.rs:11650-11654` states the refusal in the
> code's own voice. **Activating a seat and reading it as `SiteOperand` are in
> direct structural conflict.**
>
> **The recut, all three parts landed:**
>
> - **`D5` re-scoped** — the all-30 un-skip bar is retired, replaced by an
>   **attributed-residue** bar (remove what the byte-span gap caused, restate
>   every remaining row with its measured cause, pin the count). **Not a
>   reduction in rigour** — an unattributed residue is how a quarantine outlives
>   its defect.
> - **[[RT-SITEOP-CARRIED-WITNESS]] filed `draft`** — owns the 29 rows, the four
>   `SPECIALIZED_ONLY` seats, and the mechanism. **`draft` because a design fork
>   is OPEN, not because it is unscoped.**
> - **`D6` MOVED there** — its premise was "`D5` is the activation" and the
>   activation is now split. **That is what lets this node close on `D5`.**
>
> **OWED BY ME, DEFERRED DELIBERATELY: route the `RT-SITEOP-CARRIED-WITNESS`
> mechanism fork to the Architect when that node approaches the frontier — NOT
> now.** Routing it now inserts work ahead of the campaign, which the operator's
> sequencing forbids without asking. The node cannot go `ready` until it is
> ruled.
>
> **`(FsWriteFile, Argument(2))` is activated with NO end-to-end row** — its
> sibling path seat blocks every program that reaches it. That satisfies `AC-4`
> as written (per-seat evidence), and the implementer flagged it rather than
> letting it ride. **The hazard is ATTRIBUTION**: the receiving node is the first
> that can exercise it, so a failure there traceable to the activation belongs to
> `RT-CARRIER-BYTESPAN-OBSERVE`, not to that node. Recorded in its frame §7.
>
> ## Carried from the prior window — still true
>
> **`AC-10` IS DISCHARGED ONLY IN THE NARROWED FORM — construction authority,
> NOT provenance. I published the stronger claim and it was wrong.**
> `rebuild_from_collected(self, pointer, len)` returns `Self { pointer, len }` —
> **the receiver is never read**. A bearer/warrant condition, not a dataflow
> proof. Architect `dec_5ghh87fvg7skn`.
>
> ⇒ **DO NOT "derive the new values from `self`"**, which I had offered as the
> expensive option. At `rebuild_recursive_argument`, `self` holds **preheader**
> SSA handles while the arguments are **new loop-header block parameters**;
> reusing `self` would discard the phi-like recursive values and can break
> dominance.
>
> **My execution check did not cover this, and that is the lesson.** Running the
> evasion probe to `E0451` proved the **braced literal** is refused. It said
> nothing about whether a *permitted* mint constrains its own values.
> **Verifying a mechanism refuses the bypass you thought of is not verifying it
> enforces what you wrote down.**
>
> **`integrator` "PR #365 awaiting Steward routing" is NOT work.** That seat was
> **retired by the operator 2026-07-26**; no `moot.toml` entry, no pane, and
> Ken's PRs are in the 1500s. **It has fooled a Steward twice — do not route it a
> third time.**
>
> **Fleet is still SINGLE-THREADED** (operator, 2026-08-07).
> `KERNEL-NESTED-IND` stays stood down despite a written frame and an idle
> kernel ring. **A posture, not an oversight — do not "fix" it.**
>
> **STALENESS — READ THE DIRECTION.** A moved `main` proves this block is
> stale; **an unmoved `main` proves nothing**, because a ring advancing
> `D3`→`D4` merges no ref until its candidate lands. ⇒ **Re-derive ring state
> from `thr_36ey6e0byg9r8` and `list_participants`, never from this paragraph.**

### OPEN, MINE: M8's doc-only discriminator contradicts the Adversary's scope

**Surfaced 2026-08-07 by `evt_5r0hgy45v1r8x`** — the Adversary hunted the
`89916fc1` merge off its own ground-truth read and opened with *"no merge
notification was sent to me for this one."* It was right that none was sent.

**Both of my rules fired and they disagree.** `§10⁻a`'s scope table puts
**`library/` inside** the Adversary's surface. `merge-procedure.md` M8 says
doc-only merges do not concern it and makes **`--doc-only` the discriminator**.
A `library/`-only merge is published `--doc-only`, so it is in scope by one rule
and skipped by the other.

**Cost this time: zero** — the hunt found no defect. **Do not fix this now.**
It is process work, it blocks no product WP, and servicing an Adversary report
immediately is the exact §10⁻ loop. It needs one operator-facing sentence at a
real seam: is the discriminator the *flag* or the *path*? My reading is the
path — `--doc-only` is a CI-cost flag and was never meant to carry scope — but
that is a guess about intent and the operator owns it.

### RT-WORKER-FIXTURE-DECODE: framing falsified two of its own node's claims

Both were read out of the object store at `89916fc1` while writing the frame.
**The node text still contains the originals; the frame §1c/§1d governs.**

1. **"The worker fixture cannot run" is FALSE as a general statement.**
   `run_worker_fixture` (`constructors.rs:5772`) has **exactly two callers**,
   and the other one — `nested_worker_depends_on_both_levels` (`:5895`) — is
   un-ignored and **passing** at both refs the two-ended census measured. Same
   helper, same compile step, same `.run(None)`, same decode path. ⇒ The
   discriminator is the **expression**, not the helper, and that live sibling
   is a working differential already in the tree. **The node title carried the
   false claim into the tracker's releasable-frontier list** — corrected there
   too.
2. **`token` is the native RETURN VALUE (`compiled.rs:132`), not an error
   code.** Eight sites across five decoder kinds raise the one
   `NativeResultDecode` variant, so `token: 9` names no arm. Naming it is `D1`
   and it is the whole diagnosis.

**My hypothesis is recorded IN the frame as a hypothesis, not a pin** — the
`Boundary` unrecognized-tag arm at `compiled.rs:204`. I did not run it. If a
resume tempts you to treat it as settled, it is not.

### The Adversary's F1 is the one open technical question from the merge

`evt_2yxmdfhvt4fm0`, **both findings verified in source by me before folding**,
then folded into the nodes that own them (`RT-WORKER-FIXTURE-DECODE` was `draft`
then and is `ready` now). **The channel is report-only
(COORDINATION §10⁻a) — I did not reply and must not.**

**F1, folded into [[RT-WORKER-FIXTURE-DECODE]].** `RT-SRCBODY-BIND-ORDER`
reversed the Parameter run and left the Capture run in descriptor order
(`lowering/units.rs:4060-4067`), on a predicate `source_body_binding_order`
(`:3689-3699`) that returns `true` for `CallableDeclaration` **and
`ClosureBody`** — and `ClosureBody` is the unit kind that carries captures. The
covering comment at `:4057-4059` appeals to descriptor order, **which is the
exact ground that same commit refuted for the sibling run.**

**Whether captures SHOULD be reversed is NOT established** — it turns on the
elaborator's de Bruijn assignment across a closure environment, which nobody
has read. **Both answers are live. Do not repair it as a bug**; if the
measurement says the capture run is wrong, that is lowering semantics and it
goes to the Architect.

It folded into `RT-WORKER-FIXTURE-DECODE` rather than a new node because
`two_same_shape_workers_are_distinguished` **is** the direct discriminator, and
restoring that fixture is already that node's deliverable. **The axis is
unmeasured precisely because the fixture is dark.**

**F2, folded into [[CI-IGNORED-SWEEP]].** The declared population of 46 counted
what this program **authored**; a sweep selects on the attribute, and the
anchored count is **50** (`ken-cli` 34, `ken-verify` 10, `ken-runtime` 3,
`ken-interp` 3). The four extra are **ignored by policy, not base debt** — one
for ~142s of cost, three for capability not yet in scope for L1 — and would
answer "still belongs" forever. Node now carries a second cut on
reason-for-ignoring, plus the caution that it must not be enforced by parsing
prose reason strings. Also recorded: the one-off no-over-annotation check
covered **44 of 50**.

### THE ENUMERATION WAS CLOSED OVER THE WRONG POPULATION — my error

**"The complete `--no-fail-fast` surface is 40" ranged over `ken-cli` and
`ken-verify`. That is 2 of 8 workspace members.** The workspace is
`ken-kernel`, `ken-elaborator`, `ken-interp`, `ken-cli`, `ken-foundation`,
**`ken-runtime`**, `ken-host`, `ken-verify`, and CI runs
`cargo nextest run --workspace`.

⇒ **Six packages were never enumerated — including `ken-runtime`, the crate the
`D1` repair MODIFIES.** An enumeration that excludes the crate under repair
cannot bound that repair's blast radius. **I accepted the number without asking
which packages it ranged over.**

**What failed** — both unannotated `ken-runtime` lib unit tests in
`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/constructors.rs`:

| test | line | why it matters |
|---|---|---|
| `two_same_shape_workers_are_distinguished` | `:5772` | its own doc comment says swapping a body **or its capture order** changes the linked result, and calls it `AC-5`'s target-redirect red |
| `c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload` | `:2531` | nested payload selection |

**RESOLVED — the `D1`-regression hypothesis is FALSIFIED, and two was the whole
population, not a floor.** Kept because the *shape* of the error is worth
recognising: I accepted "the complete surface is 40" as authoritative sizing
without asking which packages it ranged over.

### THE CENSUS IS CLOSED OVER ALL EIGHT MEMBERS. Both rows are BASE DEBT.

`evt_ksrhrv82t5ae`, `--no-fail-fast`, one `-p` per command, never `--workspace`
(`COORDINATION §12` unchanged). Absolute counts, passed/failed/ignored:

| package | base `21fd46dc` | candidate `fb99d0fc` |
|---|---|---|
| `ken-kernel` | 215 / 0 / 0 | 215 / 0 / 0 |
| `ken-elaborator` | 1106 / 0 / 0 | 1106 / 0 / 0 |
| `ken-interp` | 178 / 0 / 3 | 178 / 0 / 3 |
| `ken-foundation` | 19 / 0 / 0 | 19 / 0 / 0 |
| `ken-host` | 55 / 0 / 0 | 55 / 0 / 0 |
| `ken-runtime --lib` | 778 / 2 / 1 | 783 / 2 / 1 |

**Both rows fail at BOTH ends with identical signatures.** Base debt, annotated
under the fork, **not** a regression. The `+5` at the candidate is `D3`'s tests
plus the sentinel, not a behaviour change.

**NEITHER IS AN ASSERTION FAILURE — I verified this in the source, not from the
report.** `two_same_shape_workers_are_distinguished` panics on its **first
statement**, in `run_worker_fixture` at `.expect("the worker fixture runs")`, so
all three `assert_ne!` comparisons are **dead code at both refs**.
`c2_ac4_...` panics at `.expect("the C2 carrier edge emits")` — the carrier
refuses to emit. **A `D1` capture-order regression presents as two
configurations comparing EQUAL, an `assert_ne!` firing.** It cannot present as a
fixture that will not execute, and not at a base predating the change.

**The consequence, and it must not be lost in the annotation:** `AC-5`'s
target-redirect red is asserted by a test that cannot reach its assertions.
Annotating switches off nothing that was working, and **un-ignoring it later is
NOT the repair** — restoring the fixture is. [[RT-WORKER-FIXTURE-DECODE]] owns
that; [[RT-CARRIER-PRODUCER-OCCURRENCE]] owns the other.

### THE COLD-WORKTREE ARTIFACT — why no repair credit is claimed

**The ring's first base run reported 738 passed / 42 failed. Re-run WARM, same
ref, same command: 778 / 2 / 1.** The extra 40 were
`native_execution_differential` and `object_linker_packaging` rows needing built
artifacts on disk. **The 42 is a build-state artifact, not a property of
`21fd46dc`** — so any reading that this candidate "fixes 39 rows" is false, and
the **"fixes six base failures" clause is removed from the publish
description.** The implementer caught it because base-full disagreed with
base-lib, and flagged it rather than quietly using the warm number. **A
single-ended run could not have produced that disagreement.**

### THE FEATURE HYPOTHESIS IS MEASURED DEAD. Do not re-raise it.

> **`-p ken-runtime --lib --features px8-ds-test-support --no-fail-fast` is base
> 778/2/1 and candidate 783/2/1 — IDENTICAL to the feature-off runs at both
> ends, same two rows, same signatures. Nothing flipped.** It was mine
> (`evt_6e8kxam15ax4g`) and the ring killed it by measurement rather than
> argument, which is what I asked for.
>
> **The audit closed BY CONSTRUCTION, which is stronger than the grep I asked
> for:** `ken-runtime` is the **only** workspace member declaring a `[features]`
> table at all, and the only member-to-member activation anywhere is `ken-cli`'s
> dev-dependency on it. The other five declare **zero** features, so their `-p`
> runs cannot differ from `--workspace`. The three other `features = [` hits are
> third-party (`rustix`, `linux-raw-sys`, `criterion`). **No reruns owed.**
>
> **There was never a local/CI disagreement to explain.** Both CI rows occur
> exactly once and **neither passes at the candidate**. The "duplicated name"
> in the interim was a **third** test,
> `object_linker_packaging::tests::each_of_the_eight_authorized_limits_is_part_of_the_package_identity`,
> listed twice in one binary's base failure list.

**The mechanism below is still TRUE and still worth knowing — it just was not
the cause here.** Filed as fleet memory
`a-p-scoped-run-and-cis-workspace-run-compile-different-feature-sets`.

**The bare-name ambiguity DOES NOT EXIST for these two rows.** Each name occurs
**exactly once** in all of `crates/` (`constructors.rs:5772` and `:2531`; every
other hit is under `docs/`). `mod constructors;` is declared once
(`core/tests/mod.rs:59`), `mod tests;` once (`core.rs:12`), there is **no
`#[path]` anywhere in ken-runtime**, and the crate declares **no `[[test]]`,
`[[bin]]` or `[[bench]]` target**. ⇒ One symbol, one binary. **My "disambiguate
by binary" instruction could not have produced a finding, and the premise was my
paraphrase of the implementer's interim, not their report.**

**The actual axis is FEATURE UNIFICATION.** `ci.yml:120` runs
`cargo nextest run --workspace --locked --partition count:N/4`; the ring runs
`-p ken-runtime --lib`. ken-runtime has one non-default feature,
`px8-ds-test-support`, and **`ken-cli`'s `[dev-dependencies]` enables it**. The
workspace is `resolver = "2"`, so one `--workspace` test invocation compiles
ken-runtime once under the **union** its dependents demand. ⇒ **feature ON in
CI, OFF under `-p`.**

**Most gated sites are `#[cfg(any(test, feature = "px8-ds-test-support"))]`,
true in ANY lib-test build — they do NOT discriminate.** The only feature-only
sites are `cranelift_backend.rs:76` and `lowering/mod.rs:12381`. **Do not settle
this by reading them.** The probe:

```sh
scripts/ken-cargo test -p ken-runtime --lib --features px8-ds-test-support --no-fail-fast
```

at **both** ends. Rows flip ⇒ that is the reconciliation. Rows do not flip ⇒ the
hypothesis is dead and the axis is elsewhere; **a dead hypothesis measured beats
a live one assumed.** I did **not** run it — the build lock is the ring's and
contending for it would stall the critical path.

**The general form, and it binds every local measurement:** a `-p <pkg>` run
activates that package's **default** features only; CI's `--workspace` run
activates the **union every member demands, dev-dependencies included.** So the
operator's `-p`-only rule (§12) that keeps the box alive also makes a local
differential **non-equivalent to CI** for any package a sibling enables a
non-default feature on. Cheap check: grep the workspace `Cargo.toml`s for
`features = [` on path deps and carry the matching `--features` on **both** ends.

### RETRACTED — there was NO mention gap and NO stall. Both were MY instruments.

**This block previously reported a three-occurrence mention gap on
`runtime-leader` and a nine-minute implementer stall. Neither exists.**
Retracted to the ring at `evt_4yh3568e6f488`.

**Error 1 — I read a field that does not exist.** Convo events carry mentions
at **`metadata.mentions`**, not at a top-level `mentions` key. My probe read
the top-level key, so it returned `None` for **every event in the space,
including my own posts whose mentions demonstrably arrived.** Measured
correctly, all three of the leader's handoffs were right:
`evt_72f386mgvth4t` → `['agt_37reqvb6ce400']`, `evt_3mj0a2bv40266` →
`['agt_37reqftfe6g00']`, `evt_cjx7b0kgywca` → `['agt_37reqg3nync00']`.

**Error 2 — I truncated the pane probe.** `tmux capture-pane -S -4` showed an
empty composer and no `Working` line, and I called it idle. **The window was
too small to contain the spinner directly above it.** The implementer had been
running continuously for 12+ minutes.

**Both are the same failure: a confident explanation built on an unverified
instrument, which launders a measurement error into a finding.** The tell I
walked past twice: **my own messages showed the identical `None`**, and I did
not ask why. ⇒ **When an instrument reports a defect, first check what it says
about a case whose answer you already know.**

**Cost:** a false correction posted to a teammate and one redundant
re-dispatch. **Do not re-derive the mention gap from an older briefing copy.**
>
> **Chain:** `21fd46dc` (frozen base) → `aa032cc2` → `8696e8c5` (`D13`, 31
> rows) → `7d204438` (`D14`, +9 `scenario.rs`) → `d0942803` (`D15`, `ci.yml`)
> → **`fb99d0fc`** (`:103` respin, QA-approved).

### PUBLISH PRE-CLEARED at `fb99d0fc` — both axes, measured 2026-08-07 ~12:3xZ

| axis | result |
|---|---|
| **base/intersection** | merge-base `e6b4a13b`; `comm -12` of the candidate's changed files against `main`'s is **EMPTY** ⇒ **immaterial, do NOT rebase** |
| **attributability** | `main` is **GREEN** at `533f7c06` ⇒ any red on the candidate is the candidate's |

**`git diff --stat origin/main fb99d0fc` shows 89 files / -22863 and that is
NOT a staleness signal.** A squash applies **merge-base → branch**, never
**main → branch**, so files `main` gained since `e6b4a13b` that the candidate
does not touch are **not** reverted. That stat **fires identically on safe and
unsafe candidates**; the intersection is the only test that discriminates, and
it is empty. (COORDINATION §14(5) correction.)

### DEFECT to raise with runtime-leader AFTER this lands, not now

**Both review requests carried `mentions: None`** — `evt_72f386mgvth4t` (QA)
and `evt_3mj0a2bv40266` (Architect) name the reviewer **in prose only**. That
is the classic silent stall (§2). It did not bite: QA answered and the
Architect pane read `Working`, i.e. **a redundant wake path masked it**. The
next handoff may not be so lucky. One line to the leader at the seam; **do not
interrupt a live review over it.**
> 2. **When it lands**, run M6-M9 (see "Owed the moment the merge lands").
> 3. **Do NOT release a second ring.** Operator ruled single-threaded
>    2026-08-07; `KERNEL-NESTED-IND` stays stood down despite a written frame
>    and an idle kernel ring. This is a posture, not an oversight.
>
> **My unpublished work sits on `steward/work` ahead of `origin/main`.** It is
> held under COORDINATION §10⁻ rule 1 while Runtime holds finished unmerged
> work. Publish it at the same seam as the candidate, not before.

### THE 40-ROW SET CANNOT CLEAR CI ON ITS OWN. Adversary F1/F3, confirmed.

**Do not re-derive this and do not re-open it.** Adversary `evt_4mwy8tmfmm7tw`;
every claim re-measured by me against the tree, not taken on report.

All three binaries that merged node `CI-SKIPPED-NATIVE-TESTS` exists to run are
gutted by the annotation:

| binary | at base `21fd46dc` | annotated |
|---|---|---|
| `px8f_write_partition.rs` | 1 live | **0 live** |
| `px8f_buffer_native.rs` | 1 live | **0 live** |
| `rt_parity_native.rs` | 7 live | **1 live** |

**The two zero-selection jobs HARD-FAIL.** CI installs `cargo-nextest@latest`
= **0.9.140**, whose `--no-tests` default is `auto`, *"defaulting to fail"*.
**Measured directly on a dependency-free scratch crate, with a positive
control:** a binary whose only `#[test]` is `#[ignore]`d exits **4**
(`error: no tests to run`); a binary with one live test exits **0**. The
aggregator (`ci.yml:296-304`) tests only `result == success`, so it fails with
them.

**My "CI should go green in one pass" instruction to the ring was FALSE** and
is corrected at `evt_7wyhwwcnec4yq`.

**The third binary fails the OTHER way and is the more dangerous one.**
`rt_parity_native` keeps one test, so it reports **green** — but the six ignored
rows are exactly the six calling `assert_narrowed_alike`, and the survivor
(`:786`) calls `elaborates()`, a source-scope check. `ci.yml:227` states in its
own voice the condition the job was built to end: *"a green CI carried no
information about whether native and interp agreed."* **True again, and nothing
reports it.**

**Operator ruling 2026-08-07: fold a minimal `ci.yml` companion** — `D15`,
released at `evt_xy3s7s2qf2pq`. `--no-tests=pass` on the two `px8f` jobs **only**
(the flag is fail-open; it is defensible only where the emptiness is known,
owned and named), plus an **in-place correction** of the false `ci.yml:227`
claim. `AC-2` is the load-bearing one: un-ignore one `px8f` row, observe the job
go RED, restore — proving the flag suppresses only empty-selection and not a
real failure.

**Publisher CAN push `.github/workflows/`** — tested, not cited: `ken-ci[bot]`
authored **seven** workflow commits on `main`, latest `a1e29284` 2026-07-27.

### `D15` RESPIN — a SECOND copy of the false claim, at `ci.yml:103`

**The ring found it and asked before spending the review cycle.** My `AC-3`
named `ci.yml:227`, so it was **structurally unable** to catch a second copy
160 lines earlier — the one a reader meets first.

**`AC-3` restated as a property, and this is the durable form:** *no sentence
anywhere in `.github/workflows/ci.yml` asserts in the present tense that
`rt_parity_native` currently carries the interp-vs-native differential;
discharge by grep over the whole file, not by reading the block you edited.*

**Bounded deliberately — do NOT sweep further.** `:92-94`
(`(14m41s, 7 tests) RESTORED`, with a cited measurement source) and the
measurement sentences after `:103` (`7 tests / 266.7s / 470.6s`, the ~250s
outlier, the nested-brackets structural fact) are **records and still true**.
**Instructions get corrected; records stay records.** Over-correcting them is
the opposite error and was ruled out explicitly.

### No over-annotation. The suppressed population was swept once, by hand.

At `7d204438`, on request: `ken-cli` `-- --ignored` = **0 passed / 34 failed**;
`ken-verify` = **0 passed / 10 failed**. **All 44 suppressed rows still fail.**

**Nothing in the repo does this automatically** — `--ignored`,
`--run-ignored`, `include-ignored` grep **empty** across `.github/`, `scripts/`,
`docs/program/`. Every skip is write-only, so a landed repair ships with its own
regression cover off. Filed as **[[CI-IGNORED-SWEEP]]** (`draft`, verify, S) on
the operator's ruling; **not** folded into this candidate.

### Both operator decisions — RULED 2026-08-07, do not re-ask

| question | ruling |
|---|---|
| Nine `ken-verify` `scenario.rs` lib unit tests — in the skip set? | **IN. The set closes at 40, not 31.** Same discipline as the other 31: exact signature, owning node, `fails at base 21fd46dc`. |
| Runtime held, kernel idle with a shovel-ready frame — open a second ring? | **HOLD SINGLE-THREADED.** Do not release `KERNEL-NESTED-IND` or any other node in parallel. |

**PR #1265 was CLOSED at 11:25:19Z** — that item is resolved and needs no
further recommendation.

**The GitHub outage is over and CI works. `main` is `533f7c06`.**

### The candidate is exonerated and `ken-cli` is GREEN

**Candidate: `wp/RT-SRCBODY-BIND-ORDER` at `8696e8c5`. HELD — not routed for
review.** `wp/RT-DECL-CLOSURE-PORT-typed-units` frozen at `21fd46dc`.

**`D12` complete `--no-fail-fast` enumeration** (a closed enumeration, because
fail-fast is per **binary**, not per test): **40 candidate failures, every one
also fails at base `21fd46dc`. ZERO REGRESSIONS.** The candidate additionally
**FIXES SIX** base failures.

**31 authorized rows annotated at `8696e8c5`** — annotation-only, 290+/21-,
nine files, each row carrying its **exact signature**, **owning node**, and
**"fails at base `21fd46dc`"**. Result:
`ken-cargo test -p ken-cli --no-fail-fast` = **120 passed / 0 failed / 34
ignored**. **`px7o` is UN-ignored and passing 3/0.**

### PR BACKLOG CLEARED — 9 open to 3. Do not redo this.

**`main` is `616a0b49`** — PR **#1530** landed 45 commits of Steward corpus
(doc-only; blob-identity verified 20/20). `steward/work` reset to it.

**Closed as stale or superseded, each verified individually, none merged:**

| PR | evidence |
|---|---|
| #1455, #1449 | all commits patch-present on `main` (`git cherry` 0 missing) |
| #1519, #1502, #1421 | differ only on `CURRENT-BRIEFING.md` / `RT-CONTSRC-PRODUCER-LOCAL.md`, both updated on `main` 2026-08-07 |
| #1322 | its 173-line `wp/DOC-W4-LANGUAGE.md` is on `main` **byte-identically** (`87a87052` both sides); every other file has a **newer** `main` version |

**NO consolidated docs branch is needed — that plan is retired.** The four docs
PRs had **no surviving content**; merging any of them would have **reverted**
newer work.

**The instrument that mattered:** blob-difference alone is NOT evidence of novel
content — `main` moving past a PR produces the same signal. Use `git cherry`
(patch-equivalence) plus **which side was touched later**.

**Still open: #1529 (candidate), #1528 (closes as superseded when #1529
lands). #1265 was CLOSED at 2026-08-07T11:25:19Z** — it carried `fb8ec38`,
`430798bf`, `548682c3`, `42ccd8ec`, all explicitly banned from import. **Two
open PRs, both accounted for.**

### The nine `scenario.rs` rows are IN — ruled, dispatched, in flight

The ruling went to runtime-leader as `evt_gjaaqcn0sftc` in `thr_69qrsxjk8wrcd`:
annotate all 40, then **one** review cycle — QA then Architect on the exact
final SHA. A new SHA voids `dec_wyn3kvzhs9at`; read Decisions from the
**object**.

**An unmatched row gets its own node, never a nearest fit.** The ring was told
to stop and ask rather than assign a row to the nearest shape.

### THE NUMBER IS 31 — my off-by-one, twice

I said 39, then 30. **`D12`'s 40 are CANDIDATE failures and `px7o` is NOT among
them — it PASSES.** Removing its wrong annotation removed no row from the
failing set. **40 = 30 `ken-cli` + 10 `ken-verify`**; only **9** of `ken-verify`
are the held `scenario.rs` tests, and `px8f_write_partition` is an authorized
integration row already annotated. **40 − 9 = 31**, plus the four `px4b` = **35
`#[ignore]` total.** The ring caught this and escalated the arithmetic while
following the unambiguous substantive instruction — the right precedence.

### `RT-ENTRY-TRAP-PX7O` IS CLOSED — false premise, mine

CI reports the bare name `nested_err_payload_reaches_both_real_executors`,
defined in **two** binaries. I attributed it to `px7o`; the red one is **`px7n`**
(owned by [[RT-FRAME-MARKER-ONCE]]). **`D10` measured `px7o` at the BASE, where
it does fail, and I carried that forward as if it described the tip.** I also
told the operator the repair "cleared `px4b` but not `px7o`, so it may be
incomplete" — **it cleared both.** Do not re-file the node; do not re-skip
`px7o`. **A bare test name shared by two binaries names neither.**

### Owners, all filed

`BytesPointerLength` → [[RT-CARRIER-BYTESPAN-OBSERVE]]; **`ResourceScalar` →
[[RT-CARRIED-RESOURCE-SCALAR]]** (same refusal shape, **different need** — never
call these byte-span); frame marker → [[RT-FRAME-MARKER-ONCE]]; closure lane →
[[RT-CLOSURE-BOUNDARY-LANE]]; `ComputationalMatch` →
[[RT-COMPMATCH-TREE-SCRUTINEE]]; `ProcessExitStatus` →
[[RT-PROCESS-EXIT-STATUS]]. **An unmatched row gets its OWN node, never a
nearest fit.**

### CI 14 vs local 40 — EXPLAINED 2026-08-07. It was never a second population.

**`.github/workflows/ci.yml:117` runs `cargo nextest run --workspace --locked`
with no `--no-fail-fast`, and there is no `.config/nextest.toml`.** nextest
therefore **cancels the run on first failure**; only the tests already in
flight finish and report.

⇒ **CI's 14 is a TRUNCATED VIEW of the same set the ring enumerated at 40**,
not a disagreeing measurement. This is the identical per-binary peeling caught
at `D10`, one layer up — and it is why sizing an annotation off CI's list can
never converge while sizing off the local `--no-fail-fast` run can.

**Two consequences, both given to the ring.** The closed local enumeration is
the authoritative sizing input. And after all 40 are annotated CI should go
green in **one** pass — **a 41st row would be new information, not another
peel; report it, do not absorb it, do not re-baseline.**

**The ring was right to refuse to size on the discrepancy.** The refusal is
what kept the number honest until the mechanism was found.

### Why the set kept growing, recorded because I missed it once

**`cargo test` and CI are both fail-fast PER BINARY.** One failure and the
binary's remaining tests never run and never report, so **annotating a row
un-hides the next row in the same binary.** The implementer caught this during
`D10` and reported it; **I then scoped `D11` off CI's truncated list and called
it complete.** `D12`'s whole-package `--no-fail-fast` run is the closed
enumeration that ended the peeling.

### ACTIONS IS RECOVERED. The outage block is retired — do not re-diagnose it.

Incident `qcvjkzcs7j74` is over. **Measured 2026-08-07: runs complete normally
and `main` is GREEN at `533f7c06`** (run at 04:47:35Z, conclusion `success`).
Greenness on `main` is what makes a red on a candidate *attributable* rather
than inherited — re-establish it before publishing any cut.

### State of the publish

| item | value |
|---|---|
| local branch `wp/RT-SRCBODY-BIND-ORDER` | **`8696e8c5`** — the `D13` annotation tip, local only, **never pushed** |
| PR **#1529** head | **`aa032cc2`** — the PRE-annotation SHA |
| #1529 CI | **8 failed / 4 passed** — expected at `aa032cc2`, and **not a finding about the ring's work** |
| PR **#1528** | open; closes as superseded when #1529 lands |
| `main` | `533f7c06`, green |

**Do not read #1529's reds as a candidate defect.** The PR head is two
commits behind the annotated tip. The reds are the unannotated base-debt rows,
which is exactly what the 40-row annotation exists to close.

**The publisher force-pushes the head branch, so publishing moves #1529's head
to whatever `wp/RT-SRCBODY-BIND-ORDER` points at.** That is the intended path
here — but assert the branch is at the **reviewed** SHA immediately before the
run, never at whatever the ring last committed.

### THE PUBLISH REF IS FROZEN AT `21fd46dc`. A live trap was disarmed here.

**The ring committed `c4112237` (RT-ENTRY-TRAP-254 `D6`) onto
`wp/RT-DECL-CLOSURE-PORT-typed-units` — which is the HEAD OF APPROVED PR
#1528.** Local moved to `c4112237` while origin stayed at `21fd46dc`.

**The publisher force-pushes `refs/heads/$head_branch` by design, and
`resolve_branch` derives `head_sha` from the BRANCH, not from `--target`.** So
the queued re-run would have pushed `c4112237`, moved the PR head off the
approved SHA, and voided `dec_4w8wn4ymn32cm`.

**Disarmed 2026-08-06:** `D6` preserved at **`wp/RT-ENTRY-TRAP-254-d6`**
(`c4112237`, nothing lost); `wp/RT-DECL-CLOSURE-PORT-typed-units` reset to
`21fd46dc`, matching origin. Ring told to work on the `-d6` branch.

⇒ **BEFORE ANY PUBLISHER RE-RUN, assert the branch is still at the approved
SHA:**

```sh
git rev-parse wp/RT-DECL-CLOSURE-PORT-typed-units   # MUST be 21fd46dc...
```

**`--target <sha>` does NOT protect you.** The publisher resolves the SHA to a
branch and then re-reads that branch's tip.

### RT-ENTRY-TRAP-254 is CLOSED (superseded). The repair is RT-SRCBODY-BIND-ORDER.

**Architect mechanism ruling `evt_7yfs6qxp9hm5b`.** The `D0`-`D9` chain found a
**general multi-parameter source-body binding permutation**; the skipped
`ProcessInput` row is **one discriminator** for it.

**THE DEFECT.** `lowering/units.rs:3701-3790` does **one slot-order walk doing
two jobs**: it records `defining_abi_operands` in ABI descriptor order
(**correct**) and pushes the same operands into `env` in that order
(**wrong**). A declaration body reads **de Bruijn-NEAREST-FIRST**, and
`core.rs:14705-14714` **already states** reverse-then-append. So
`main(input, caps)` gives `env = [input, caps]` while the body names `input` as
`Var(1)` ⇒ `Var(1)` reads `ProgramCaps`. **A bug fix restoring a stated
contract, not a mechanism change.**

**REPAIR:** keep the ABI run and `defining_abi_operands` unchanged; build the
semantic env as `reverse(Parameter run) ++ Capture run in D3 order`.

**`D9`'s ATTRIBUTION WAS REFUTED and the refutation is load-bearing.** It blamed
the common transfer coordinate. `call_declared_unit_target` **already pairs
positionally**, `carry_call_input` cannot select a sibling or change position,
and **a carried word bypasses `transfer_into_carrier` entirely** — so a caller
occurrence there cannot change which word occupies slot 0. ⇒ **Per-argument
transfer coordinates are BANNED: a design change that would leave the defect
intact.** Also banned: reversing the process root's ABI roles, rewriting
continuation specializations, touching `carry_source_call_inputs`,
`carry_call_input`, `call_declared_unit_target`, or `mod.rs:5958-5978`.

**BLAST RADIUS — AGGREGATE-NESS IS NOT CAUSAL.** Not one row, and **not "every
aggregate through `call_declared_unit_target`" — that framing was the Steward's
and is wrong.** The class is **every activated non-root functionized source-body
unit with at least two parameters whose body distinguishes parameter
positions**; it surfaces for ints, bools, capabilities, borrowed handles or
constructors. The 97-`Constructor` census does **not** bound it. **Unary units,
unused parameters and equal values MASK it.** Operator told; **it does not alter
the publish ruling by itself** but is materially larger in logical scope.

**`D2` is not optional:** generated contexts claim **byte-for-byte equivalence**
with the raw unit while installing parameter-then-capture order
(`units.rs:2523-2547`). Fixing the unit alone makes that committed claim
**false** — worse than the original defect, because the claim is what a reader
relies on.

**Four controls, and control 1 is the important one:** a two-parameter
declaration with distinct **NONAGGREGATE** values reading both positions
(proves the fix is not aggregate-shaped); the `ProcessInput`/`ProgramCaps`
discriminator; a root-adapter control proving its ABI-role order was **not**
reversed; raw-worker vs generated-context equivalence on a body that
**distinguishes** its parameters (a unary body proves nothing — unary units are
invariant under reversal). **Expect CI reds and attribute each individually;
never re-baseline.**

**`D6` landed** (stale carried-scrutinee reachability comment) at `c4112237` on
`wp/RT-ENTRY-TRAP-254-d6` — follow-up PR when Actions returns.

### RT-SRCBODY-BIND-ORDER in flight — candidate `5d388e37`, QA HELD

- **`5d388e37`** meets `D1`, `D2`, `D4`, all four controls red-before-green.
- **`D3` control 4 and `AC-3` were AMENDED by the Steward** (`evt_gpekyt7jzb67`).
  The required population does not exist — no body is present at both hosts,
  retargeted raw workers are template-only, **every generated-context worker is
  unary**. The ring reported the weakness instead of widening the fixture.
  **`AC-3` as originally written was UNDISCHARGEABLE and that was a frame defect
  of mine.**
- **The ruling, one fact:** `reverse([p]) ++ captures` is identical to the
  parameter-then-capture order already installed, so **`D2` is INERT at unary
  arity.** ⇒ the obligation becomes an **ACTIVATION GATE** on the first
  multi-parameter generated-context worker, shipped as a **TRANSITION
  SENTINEL** that asserts the measured population and reddens by itself.
  **Non-vacuity required:** observed RED against a hand-added two-parameter
  worker, then restored. **`AC-3` must NOT be recorded as "equivalence
  verified"** — equivalence is unfalsifiable at unary arity.
- **NOT authorized:** changing the checked IH call-site arity to manufacture the
  fixture. Population expansion into a checked mechanism, and the constraint
  demanding it was **this frame's own prose**.
- **ARCHITECT is reviewing `D3c` unrequested** (`evt_28gv50xst6sqf`): tracing
  `RootIsImmediate` from its stored coordinate into the **post-`D1` semantic
  environment**. **This is the right question and I did not think to ask it** —
  `D1` reorders the semantic env, and `D3c`'s per-consumer availability claims
  are keyed to that env, so a claim could now name the wrong value. **QA is
  correctly held pending it. Do not release QA or alter `D3c` until it lands.**

### Compaction verification: a LOW ctx is proof, a HIGH ctx is NOT disproof

**Measured 2026-08-06 on runtime-implementer.** `handoff-gate-compact.sh`
returned, the pane still read `ctx 27%`, and a **full-stream** grep for
`Compacting|Context compacted` returned **0**. I resent `/compact` to that one
pane and ctx went **27% -> 7%** — while the marker grep *still* returned **0**.

⇒ **The marker text is transient and its absence proves nothing** (the progress
bar clears). **The ctx number showing a DROP is conclusive; the ctx number
showing HIGH is inconclusive**, because it can be a stale render.

**So verify in this order:** ctx dropped ⇒ done. Otherwise resend to that one
pane and re-check ctx. Do not conclude "did not compact" from an absent marker,
and do not conclude "did compact" from the script returning.

### GATE MISS, MINE: I released a new node without the before-work compaction

**`COORDINATION §15` / `steward/compaction.md`: always compact before new work,
no exceptions, no threshold, ctx unread.** I released `RT-SRCBODY-BIND-ORDER`
after roughly fifteen consecutive diagnostic releases to the same ring **and
never ran the handoff gate once.** The **implementer asked for the seam** —
the backstop caught what the gate should have.

⇒ **The gate is the fix, the ctx scan is only the backstop.** When the scan (or
a seat's own request) is what catches it, **the gate already failed upstream.**
Run `scripts/handoff-gate-compact.sh <every member>` at each new-node release.

**Mechanism note for next time:** the script does `git reset --hard
origin/main` on each worktree. It was safe here **only because** all three
runtime worktrees sat on their own `<role>/work` branches at `3015aafd ==
origin/main`, so no protected ref was checked out. **Check that first** — the
frozen publish ref `wp/RT-DECL-CLOSURE-PORT-typed-units` (`21fd46dc`) and
`wp/RT-ENTRY-TRAP-254-d6` (`c4112237`) must never be the checked-out branch when
it runs. **And note the base mismatch:** this node's base is `21fd46dc`, not
`origin/main`, so the reset puts worktrees on the wrong base and the ring must
re-checkout.

### Origin's WP ref was DIVERGENT and the force-update was safe

`origin/wp/RT-DECL-CLOSURE-PORT-typed-units` stood at `03f0510c` — **34 commits
that were neither ancestor nor descendant** of the candidate, left by the failed
`fc758323` publish. **`git cherry 21fd46dc 03f0510c` returned `-` for all 34**,
so every one is patch-equivalent-present in the candidate. The publisher's
`--force-with-lease` lost nothing. **Checked before the push, not after.**

### The five rows that ship marked `#[ignore]`

All in `crates/ken-cli/tests/px4b_native_production.rs`, Linux-only. Each
comment carries the exact observed signature, the owning node, and the
branch-introduced provenance (absent at merge base `e6b4a13b` and `main`
`3015aafd`).

| test | owner |
|---|---|
| `fs_write_and_read_resume_through_the_native_capability` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `fs_scope_denial_reaches_ken_as_the_named_error` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `canonical_fs_identity_exactly_matches_across_real_producers_and_drift_fails` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `linked_console_broken_pipe_reaches_ken_instead_of_signal_termination` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `public_source_observes_raw_argv_environment_cwd_bytes_in_field_order` | `RT-ENTRY-TRAP-254` |

Suite at `21fd46dc`: **14 passed / 0 failed / 5 ignored, no sixth row.**

**A skipped row measures nothing.** Greenness here is achieved by not asking the
question. That is the whole reason both successor frames make un-skipping a
deliverable rather than a courtesy.

### M1 CAUGHT A REAL GATE MISS — record the shape, it will recur

**Both reviewers approved the exact SHA in PROSE and no Decision object bound
it.** `dec_wyn3kvzhs9at` was `resolved` — against the **superseded**
`aa032cc2`. Publishing on the strength of two convincing posts is exactly the
Sec1ct breach, where *"(Architect + Spec)"* read as approval while the
Architect had never voted.

⇒ **Read the Decision store from the OBJECT at merge time, every time.** A
resolved Decision on a superseded SHA is not authorization; it is the most
persuasive possible way to be wrong. Routed back at `evt_6fp23qzn7cf34`; the
leader opened `dec_6zp34ra9hjb58` and it cost one round trip.

**Also: I did NOT append the tracker-sync commit** that `merge-procedure.md`
step 3 asks for on ring candidates. It would have changed the SHA and voided an
approval the Architect scoped to `fb99d0fc` **alone**. The tracker rides my own
corpus publish immediately behind — same durability, no voided gate.

### Owed the moment the merge lands, in order

1. **M6** — blob-identity verify every changed path against `origin/main`;
   then `git reset --hard origin/main` on `steward/work`, which is stale the
   instant any publish lands.
2. **M7** — flip `RT-DECL-CLOSURE-PORT` and `RT-CONTSRC-PRODUCER-LOCAL` to
   `merged`, run `scripts/gen-progress.sh`, publish doc-only.
3. **M8** — notify the Adversary. **This merge carries code, so the step is
   required.** Look the id up at post time with `scripts/moot-actor-id.sh
   adversary`.
4. **Librarian** — `crates/ken-runtime/src/cranelift_backend.rs` is a **cited
   source** in `library/SOURCE-ATTESTATIONS`. Routes to the Librarian **after**
   the merge, never into a ring's frame.
5. **M9** — the stay-one-release-ahead check. Already satisfied: both
   successors are `ready` with written frames.

### Both successors are framed. The ring will not idle.

**`RT-CARRIER-BYTESPAN-OBSERVE`** — `ready`, size L, frame at
`docs/program/wp/RT-CARRIER-BYTESPAN-OBSERVE.md`. Base is **`main`**, not the
branch: the publisher squashes, so `21fd46dc` is not an ancestor of `main`
afterwards. Two findings from grounding it, neither known when the rows were
measured:

- **The `BytesPointerLength` seat population is SIX, not the three that fail.**
  `host_effect_seat_contract` binds one `bytes` tuple at six
  `(operation, ordinal)` pairs. `FsWriteFile Argument(2)`, `FsChangeMode
  Argument(0)` and `FsOpen Argument(0)` are unmeasured. Repairing three leaves
  an identical seat refusing identically; flipping six asserts a capability
  nobody measured — and a shared tuple is exactly what makes it a bad
  discriminator. `AC-4` requires a per-seat disposition over all six.
- **The carrier reads a carried byte value one byte at a time and cannot read
  its extent at all.** `BOUNDARY_LOCAL_HELPERS` has `ken_boundary_byte_local` by
  index and a `store_bytes_len` **writer**, and no length reader anywhere.
  `ken_boundary_int_view_local` is the precedent for the missing shape and
  `narrow_carried_int_u64` for its reader. **A per-index reader does not
  establish that a contiguous pointer can be produced** — that is `D1`, and a
  negative answer is a representation boundary that returns to the Architect.

**`RT-ENTRY-TRAP-254`** — `ready`, size **S and DIAGNOSIS ONLY**; the repair is a
separate cut on its return. Two things its frame settled:

- **The exit `1` is not the defect and must not be investigated.** The linked
  shim ends `if (value < 0) return 1;`, so every negative sentinel collapses to
  1 and the exit code cannot distinguish `-1` from `-4`. Only the stderr line
  can. The fact that matters is that the entrypoint returned **`-4`**.
- **`254` IS the correct expectation — that open obligation is DISCHARGED.** The
  test sets `K` to byte `0xfe` under `env_clear()` and asserts `254`, a second
  arm at `253`, and `assert_ne!` between them. Those are legitimate non-negative
  exit codes: the program observes a raw process byte and returns it, and
  `return (int)value` passes non-negative values through. The program is meant
  to compute a byte and traps instead. **Do not repair the row by changing the
  expectation** — the frame bans it as the cheapest available false fix.

### Standing bans that survive this window

- **Do not fold `RT-ENTRY-TRAP-254` into byte-span** because "bytes" appears in
  its test name. That is the vocabulary inference the Architect refuted
  (`evt_7v61ed5pn9q3t`). Shared root cause is **unmeasured**.
- **Do not justify `RT-CARRIER-BYTESPAN-OBSERVE` from the historical
  `c7410b79` `BoundaryCarrier` signature.** Same refutation.
- **Read a Decision from the object, never from a message.** Measured again this
  window: the Architect posted "resolving on cast" and the object still read
  `proposed` for ~30s.

## CORRECTIONS — two claims the old file made that were FALSE

Both were **time-varying state wearing a permanent-looking hat** — the exact
failure the heartbeat prompt bans. Recorded so the *shape* is recognisable, not
just the instances.

### 1. "ARMED COUNTERS — the SOLE count of record" was stale AND retired

It read `RT-NATIVE-FNSPLIT: hard-stop 10 · next research pull #11` and `Architect
production blocks: 6 · next check #9`. **Both numbers were behind**, and the chain
they counted **is retired** — the operator stopped the FNSPLIT effort on
2026-07-26 and `SPEC-STORE-SPLIT` replaces it.

**A counter calling itself "the SOLE count of record" is the worst thing to
leave stale**: it invites a reader to trust it *instead of* measuring. ⇒ **There
are no armed counters now.** When the re-cut program exists, its node owns its
counts.

### 2. "TRANSPORT — convo MCP mostly DEAD" is FALSE

The old block claimed only `set_interval`/`subscribe` survived and routed all
reads through scratchpad HTTP scripts. **Measured across this entire session:
`orientation`, `list_decisions`, `post_response`, `list_participants` all work
over MCP.** Tracked as task `#110` because **the heartbeat prompt still repeats
the claim.**

**What IS true — the part worth keeping:**

- **NEVER call `mcp__convo__get_transcript`.** Its `limit` does not bound the
  response and it takes the stdio connection down with it. Operator prohibition;
  fleet law in `AGENTS.md`.
- **Mentions arrive TRUNCATED** — a doorbell, not a message. Fetch full text via
  the HTTP read path, with **your own** credential.
- **`list_decisions` can exceed the result cap** and spill to a file — grep the
  file rather than retrying the call.
- `claude mcp list` reporting `convo: ✔ Connected` **is not evidence** — it
  health-checks a fresh process.

## Preserved refs — QUERY LOCALLY. `origin` carries `main` ONLY.

> ### THIS SECTION WAS FALSE AS WRITTEN. Both halves.
>
> It said *"Origin holds 26"* and gave `git ls-remote origin
> 'refs/heads/preserved/*'` as the query. **Operator ruling, 2026-07-26:** *"clean
> up all of the non-main branches at origin."* ⇒ **All 63 non-`main` origin
> branches are deleted.** That `ls-remote` now returns **nothing**, and a reader
> running it would conclude the work was lost.

**Measured 2026-07-27 — the query is local, and the population is larger, not
smaller:**

```sh
git for-each-ref 'refs/heads/preserved/*'    # 78 refs
git ls-remote --heads origin                 # refs/heads/main — and nothing else
```

**A branch on one local ref is the NORMAL state of preserved work, not an
exposure.** Do not raise an unpushed ref as a finding.

**AND THE "EXISTS NOWHERE ELSE" CLAIM WAS WRONG ON EVERY ITEM IT NAMED.** Each
was checked at `origin/main = a1e29284`:

| the old claim | measured |
|---|---|
| `preserved/b2e-rejected-source-oracle` = `159f4109` | **present locally at that exact SHA** |
| `wp/RT-FNSPLIT-B2E-boundary-value-elimination` = `e1b540e2` | **present locally at that exact SHA** — delete neither |
| `preserved/rt-fnsplit-b2f-hardstop-{9,10,11}-evidence` | **no local ref of that name exists** — and it does not need to. Hard-stops #9/#10/#11 are all on `main`, across **12** files (`RT-FNSPLIT-B2{E,F,O,R,V}.md`, `RT-NATIVE-FNSPLIT.md`, `RT-VALUE-TOTALITY.md`, the B2O report + predictions, two WP frames, `diary/2026/Jul/25.md`). `bce75fec` is literally *"make hard-stop #11's evidence durable"*. |
| `preserved/architect-state-*` | **wrong prefix** — the refs are `preserved/architect-work-*` (5 locally). A ref name you cannot resolve is not a backup. |

**The transferable part: a "this exists nowhere else" note is a claim about a
population you did not enumerate, and it decays in both directions at once** — the
copy you were protecting had already landed in the repo, while the ref name you
recorded it under never existed. ⇒ **Re-derive from `for-each-ref` and `git grep`
on `main`; never from a hand-kept list of what is precious.**

## Operator rulings — 2026-07-21 ~12:45Z. SETTLED, do not reopen.

Kept inline deliberately: this is law, and a settled ruling is a **fixed input,
never a question to re-ask.**

- **No "ratification."** The Linux ABI II charter is a **planning document, not a
  commitment.** Nothing outside the project depends on our timelines. Do not
  re-raise status-correction as a decision.
- **Where anticipated and done diverge, fill the gap first** — hence
  `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated 2026-07-21 after I re-raised it). Manifest v2 is family-scoped and
  generated, **not** cross-target.
  **This ruling ALREADY ANSWERS any non-linux finding** — do not route one back
  as a scoping question. Record such findings as *observations against a deferred
  lane* and stop.
- **L2-0: all desirable, nothing deferred.** All nine `RepresentedUnavailable`
  operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** Do not reason
  about schedule or cost.
- ★ **My lane is token efficiency in terms of delivered work.** That is the axis
  to optimize and the one to report on.

**Standing test policy (operator, 2026-07-26):** *"Test oracles that assert facts
about source code, catalog, or documentation lines are an invitation for failure
and delay. Tests should focus on behavior."* ⇒ Executable form: **"does an edit
that changes nothing about how any program behaves make this test fail?"**

**Standing gate policy (operator, 2026-07-26):** the library currency ledger is
generated **at version release points**, **not enforced per merge.**

**`origin` CARRIES `main` ONLY (operator, 2026-07-26; restated 2026-07-28).**
A branch living on one local ref is **normal** and is never a finding. No
durability sweeps, no pushes of WP or seat branches, no ring reporting an
unpushed ref. The publisher's own candidate-branch push stays — that is how it
opens a PR.

**THE `integrator` SEAT IS RETIRED (operator, 2026-07-26).** *"remove any
references to the integrator. that seat was retired weeks ago."* ⇒ Every operative
reference is gone as of PR #1052 (`a1e29284`, 50 files) — PR template, CODEOWNERS,
`ci.yml`, four devcontainer files (including a **functional** `ctx-nudge.sh` case
arm), `COORDINATION.md`, `04-git-and-integration.md`, 40 WP frames, the roster
(29→28), git refs, worktrees. **The chronicles keep the word deliberately** —
`docs/program/diary/`, `agent/memory/MIGRATION-LOG.md`,
`docs/program/ds-campaign-judgment-log.md` (17 files, 501 occurrences): there it is
a true account of what the process **was**. **Instructions get corrected; records
stay records.** One residual is not mine to clear — the convo **participant**
still exists; see the LIVE block's operator-owed list.

**Canonical width: 96 (operator, 2026-07-26).** *"re 88 v 96. 96 is what it should
be. It was an incomplete revision, apparently."* ⇒ `spec/30-surface/31-lexical.md`
and `CANONICAL_WIDTH` are correct; `conformance/` is the stale side.
`SPEC-31-WIDTH-ERRATUM` reconciles it. Do not re-argue the value.

## Where durable law lives — do not restate it here

**The old file's real defect was restating durable rules inside a diary.** A
rule copied into a briefing drifts from its source and then contradicts it. ⇒
**Point, never copy.**

| what | where |
|---|---|
| federation law, §2c handoff gate, §14 merge gate | `agent/COORDINATION.md` |
| my playbook, publish discipline | `agent/playbooks/federation/steward.md` |
| hard-won operational lessons | `agent/memory/` (`fleet` + `enclave` + `roles/steward/`) |
| model tiers | `agent/MODELS.md` |
| reasoning charter | `docs/PRINCIPLES.md` |
| no local `--workspace` builds — CI only | `agent/COORDINATION.md §12` |
| build status against the DAG | `docs/program/IMPLEMENTATION-PROGRESS.md` |
| spec status | `spec/SPEC-PROGRESS.md` |

## Standing traps — only the POSITIONAL ones

Each is here because it fires **at a specific command**. That is the whole test
for belonging in this file rather than in `agent/memory/`.

- **Verify landed content by BLOB IDENTITY, never ancestry.** The publisher
  squashes, so an approved SHA is correctly *never* an ancestor of `main`.
- **Verify every object you NAME exists at the base you NAME** —
  `git cat-file -e <base>:<path>`, and quote the blob (§2c step 5b).
- **`git diff --stat` always exits 0.** Use `--quiet` for an emptiness test.
- **The publisher's exit code is the LAUNCHER's** — confirm it exited *and* that
  `main` moved.
- **Never `git fetch` while the publisher is inside its merge→verify window** —
  `refs/remotes/origin/main` is shared across ~70 worktrees.
- **Never `pkill -f`** (matches your own shell) · **never `git stash`**
  (`refs/stash` is shared) · **never `git checkout <ref> -- .`** (reverts
  uncommitted edits worktree-wide).
- **A probe truncated before its filter is not a measurement.** Search the full
  stream; truncate the RESULT.
- **Never dump `.moot/actors.json`** to learn its shape — use
  `scripts/moot-actor-id.sh <role>`; the schema-discovery step is what leaks a
  key. Look up a participant id **at post time**, never from memory.
- **`steward/work` is stale immediately after every publish** — reset onto the
  squashed `main` before writing anything new.
- **A `--doc-only` merge can redden `main` and is structurally unable to notice.**
  After one, **enumerate consumers of the touched paths** — attestation ledger,
  measured-token censuses, source-text oracles. This is how `95bc855c` broke three
  things and reported none.
