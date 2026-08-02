# Releasing a WP: the sequence and the handoff gate

Steward task procedure. Read at the point of use. Governing playbook:
`../steward.md`. Frame content is in `frame-authoring.md`.

A WP is not releasable as a terse catalog pointer. Build teams run T2; the T1
enclave must front-load the design judgment and hand the team a detailed,
shovel-ready brief — the implementer should execute mostly mechanically, not
design (operator, 2026-06-29). The sequence is fixed.

## The handoff gate: run before every kickoff or handoff mention

Forward progress is credit-bound and the T1 enclave is the most expensive unit
in the fleet, so a stale-context elaboration is the biggest waste lever there
is.

> **A kickoff is `compact-verified` THEN `mention`, one indivisible act.** The
> compaction is the first half of the mention, never a separate step you might
> reach.

**The tell that you are about to skip it:** you have drafted the handoff
mention and feel ready to post. That feeling is the gate trigger.

**Proof of execution:** you must be able to log *"<unit> compacted @
ctx-verified <n>% to ~0"* beside the kickoff in the tracker. If you cannot
write that line truthfully, you did not run the gate.

### Steps 1-2: clear the unit

1. **No in-flight obligation** on any receiving-unit member — no pending review
   vote, open `question`, unfinished handoff, or cleanup pass. Compaction drops
   it; resolve or reassign first.
2. **Quiescent.** `capture-pane` each member; none mid-reasoning, because
   compaction summarizes in-flight work away.

### Steps 3-4: compact and verify the drop

3. **Start all compactions, before the kickoff, never after.** Mechanism and
   the unconditional-compaction rule: `compaction.md`. A post-kickoff
   compaction eats the just-delivered kickoff and forces a re-kick.
4. **Verify every drop after the batch start.** A "sent" report is not proof.
   Unchanged ctx means resend that pane and re-verify. Do not post until every
   required pane is compacted, compacting, or queued.

### Step 5: verify every object you are about to name

5. **Verify every object you are about to name exists at the base you are about
   to name.** One command per object, before the mention leaves your hands.

   ```sh
   git cat-file -e <base>:<path>   # frame, node, every file you cite
   git rev-parse <base>:<path>     # quote the blob so the ring can bind it
   ```

   **A named base that does not contain the named artifact is not a release.**
   2026-07-26: a kickoff opened with *"Both files are on `main` at the base
   above"* — the frame was committed on `steward/work` and never published. It
   was false when sent. Both receiving seats caught it independently and
   neither synthesized a base; do not read that as the backstop working, since
   a plausible constructed base would have produced a tree no reviewer could
   reproduce. The frame is written long before the kickoff is posted, and
   publishing it is a separate act you can complete in your head. Check it
   mechanically rather than remembering whether you published.

### Steps 6-7: contention and the ledger

6. **Contention check, against every WP in flight** — not just the frontier
   candidates you are choosing between. The WP that is merging right now is an
   operand too. List the files the new WP will touch and intersect them with
   each WP already active or publishing.

   Detail and the two further axes: **Contention checks**, below.

7. **Cited-source check — a solo WP with no contender can still be blocked by
   the ledger.** Step 6 asks whether two WPs collide through the ledger; this
   asks whether the thing you are about to publish stales an attestation. One
   command, in `merge-procedure.md` as M3. Run it instead of reasoning about
   it.

   2026-07-26: a candidate reached the publisher with a resolved Decision after
   a three-candidate review, and the publisher refused the merge result because
   six revised spec files are cited sources. **Two exhaustive T1 reviews could
   not see it and neither was negligent** — CV verified that the
   candidate/current-main changed-path intersection was empty, which was true
   and not the question, because the ledger lives in `library/`, outside the
   scope both reviews were correctly bounded to. **Path-intersection-empty is
   not publishable.**

   **First ask whether `main` is already red on those rows.** If `main` is
   green and your candidate moved them, the staleness is your blast radius; if
   `main` is already red, you have found a pre-existing defect and the fix is
   not yours to fold.

   **Keep it out of the ring's frame.** Operator ruling, 2026-07-26, verbatim:
   *"The librarian's responsibilities are a distraction to the spec enclave and
   the implementation teams. For them, the librarian is not a concern,
   downstream, and unobserved."* You may find the blast radius and you route it
   downstream to the Librarian after the fact. Do not write it into an AC, do
   not ask a ring to coordinate with the Librarian, and do not ask a ring to
   touch `library/`. A build or enclave candidate is `crates/`, `spec/`,
   `conformance/`; the ledger is somebody else's plane. **You may not install
   an attestation yourself in any case** —
   `gen-source-attestations.sh` writes only a `.proposed` sibling by
   construction, because the attestation is the claim that someone
   re-validated.

### Steps 8-11: post, confirm, record

8. **Only now post the kickoff or handoff mention.**

9. **Confirm the mention actually reached the recipient's turn.** A kickoff is
   not complete when you post it; it is complete when you have seen the
   recipient go `Working`. `post_response` returning an `event_id` proves the
   event exists, not that any agent read it — this transport failed four times
   on 2026-07-14. Treat it as an expected failure and build the check into the
   gate, not into your vigilance. Pane-reading rules and the repair table are
   in `watchdog.md`.

   **When re-delivering someone else's message, point at it, do not rewrite
   it.** The owner's kickoff is authoritative; your job is the transport.
   Restating it substitutes your words for the owner's and quietly makes you
   the de-facto leader.

10. **Flip the WP's tracker status to `active`, as part of the kickoff, not
    "later."** Edit `status:` in `docs/program/issues/<ID>.md`, run
    `scripts/gen-progress.sh`, and bundle it into your next publish.

    This is a repeated defect, not hygiene. 2026-07-22: three WPs kicked, zero
    flipped, and separately one sat at `active` for hours after "tracker
    updated" was announced. **The root cause is structural** — the gate ended
    at "confirm the seat is Working", so the flip had no step to live in. A
    stale `ready` is the frontier the next sequencing pass reads, and it
    invites releasing a WP that is already out. **Where a backstop depends on
    you remembering to look, convert it into a step.**

11. **On publish, ack into the WP thread immediately:** *"PR #N is open at
    `<SHA>` — the branch is FROZEN."* A `git_request` becoming a live PR is
    invisible to the ring that produced it: the branch is now the head of an
    open PR with reviewers attached, and any force-push re-points it at a SHA
    no reviewer approved. Near-miss 2026-07-22: a "STOP, do not rebase" and a
    leader's "rebase now" crossed two seconds apart on a branch already in CI.

**`origin` carries `main` only.** `scripted-pr-automerge.sh` pushes its own
candidate branch to open the PR; that is the only push. Do not push a WP or
seat branch to origin, and do not treat a branch living on one local ref as a
finding — that is the normal state of every seat's work.

## Contention checks

**Contention has a ledger axis.** Two WPs contend if one mutates a source
the other's domain attests (`library/SOURCE-ATTESTATIONS`), even with
disjoint scopes. 2026-07-22, both halves failed in one hour: the ledger-axis
rule was written onto one WP as a sequencing constraint, then a second was
released while a third was publishing, having checked only the two frontier
items. The result:

```
Auto-merging library/SOURCE-ATTESTATIONS       <- SILENT UNION, no conflict
CONFLICT (content): Merge conflict in library/STATUS.md
```

The ledger is one row per source; the two WPs changed different rows, so git
merged a union of two independently-correct halves with nothing to complain
about. Exactly one of the two colliding files shouted, and that was a
generated digest's layout, not a guard. **Do not conclude that a ledger
collision gets caught.** Build the result (`git merge-tree --write-tree`)
and assert a post-condition you predicted before measuring. If they contend,
sequence them and re-derive the consumer population after the first lands.

**Third axis: reachability and visibility.** A placement can be
contention-free and still impossible. Contention asks who else touches this
file; it never asks whether the code you are framing can see what it must
measure. When a frame names where a test or probe lives, write both maps and
treat either one's failure as a pre-code hard stop. Measured 2026-07-26: a
"low-contention placement" (a new integration-test file, no `lib.rs` edit)
contradicted the planner's real visibility boundary, since the planner types
are `pub(in crate::cranelift_backend)` and an integration test is an
external crate. The three constraints were jointly unsatisfiable, and the
only way to satisfy them would have been a permanent public measurement API
whose sole consumer is a test. **"Committed and auto-run" does not imply an
integration-test crate** — that conflates permanence with placement.

## The five-step release sequence

### 1. Author the brief on the frame branch

At `docs/program/wp/<ID>-<slug>.md`, on branch `wp/<ID>-frame`
(`git branch wp/<ID>-frame origin/main` — the fetched ref, never a stale local
`main`). Content requirements are in `frame-authoring.md`.

> **The frame branch and the build branch must not share a name.** Name the
> frame branch `wp/<ID>-frame`, never `wp/<ID>-<slug>`. The build branch is cut
> fresh from `origin/main` by the team after the frame merges.
>
> **Why continuing it is structurally impossible:** the frame branch is
> squash-merged at step 3, which deletes the remote branch and leaves the local
> ref dangling ahead of `origin/main` while its content is already in. A team
> told to continue that branch is pointed at a stale leftover — and if your own
> worktree is still on it, the team is hard-blocked, because one branch cannot
> be checked out in two worktrees.
>
> Measured on LET-3 Phase 2, 2026-07-14: the frame published from the build
> branch name, squash-merged, the Steward worktree stayed on it, and
> `foundation-leader` could not take the WP.
>
> **Two names, always.** `wp/<ID>-frame` (yours, merges and dies) and
> `wp/<ID>-<slug>` (theirs). **Switch your own worktree off the frame branch
> the moment you publish it** — a Steward parked on a WP branch is a silent
> ring-blocker.

### 2. Hand the WP branch to the spec-leader for full elaboration

Run the handoff gate first. Mention **only the spec-leader**; the spec-leader
assigns spec-author and conformance-validator internally. The enclave brings
the brief plus the relevant `/spec` and `/conformance` to full, team-ready
rigor on that branch.

**This is the place the compaction miss recurs.** The enclave feels like a
continuous reviewer you can keep feeding, so its context silently accumulates
across unrelated units. It is not continuous: every new work unit handed to the
enclave is a fresh compact-first kickoff, no matter how warm or how far under
any threshold it looks.

This elaboration step sits between you and the build team. The team never
receives a brief the enclave has not elaborated.

### 3. Merge the elaborated brief to `main`

Via the publisher path. The spec-leader opens the merge Decision (it touches
`/spec`, so the Spec paths apply) and posts the `git_request` handoff to you
for scripted publisher handling (`COORDINATION §14`). It must be on `main` so
every team reads the canonical artifact from its own worktree, not a drifting
inline message.

### 4. Release and kick off the responsible team

Run the handoff gate first, for the whole team (leader, implementer, QA). Then
mention the **leader only**, in the WP thread, pointing at the now-on-`main`
elaborated brief and spec. The team cuts `wp/<ID>-<slug>` fresh from current
`origin/main`. **Confirm your own worktree is off the frame branch before you
kick.** Leaders do not compact their members; compaction is yours.

### 5. Stay one release ahead

Run the successor check in `../steward.md`, section 4, as the last step.

## A kickoff is a live signal until you explicitly retract it

Learned twice. If you kick a WP off and then hold, re-scope, or re-route it,
the original kickoff's mention stays unread in that team's queue and fires
whenever it is next surfaced — a resume, a `get_mentions` check, an operator
nudge. K1: a 04:30 kickoff sat unread until the operator surfaced it in tmux,
and Kernel then ran the old scope because the re-route had mentioned only
spec-leader.

**When you supersede a kickoff, mention the originally-kicked team and stand
them down.** Mention discipline says mention whoever's next move it is — when
re-routing, the stand-down is the old team's next move.

**Notification delivery is best-effort.** A mention can be correctly recorded
yet never notify the agent's session. Two consequences: on any resume, check
unread mentions; and a mentioned agent that does not respond may simply not
have been notified, so re-mention before assuming a stall.

## Gate spec-honesty errata on the context-alignment test

The self-authored enclave cascade (errata, un-stages, prose reconciles) is a
real token and coordination sink, and at this maturity most of it is
honesty-for-its-own-sake. A spec or conformance honesty correction that
re-attributes prose is justified **only** if it passes: *would a fresh agent
read the inaccuracy as ground truth and act on a false premise?* If no agent
acts on it, fold it as a one-line inline touch into the next substantive WP
that edits the file, or skip it.

- **Keep:** a conformance net correction, or a functional gate-state fix.
- **Cut:** a prose flip that a capability-agnostic net already discriminates.

Correct-under-every-outcome is necessary but not sufficient — the erratum must
also change what an agent would do.
