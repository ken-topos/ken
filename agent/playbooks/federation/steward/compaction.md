# Compaction: your own and the teams'

Steward task procedure. Read at the point of use. Governing playbook:
`../steward.md`. The handoff gate that consumes this is in
`release-and-handoff.md`.

Context compaction is strictly the Steward's responsibility. You direct the
work flow, so you own the clean context boundary that flows with it. **Leaders
do not compact their members.**

## Compacting a team before new work

**Always compact before new work — build teams and the spec enclave. No
exceptions. No before-work threshold** (operator, 2026-07-04, the enclave
twice).

For any unit you are about to hand a new work item — a build team (leader,
implementer, QA) or the spec enclave (spec-leader, spec-author,
conformance-validator) — you compact **every** member unconditionally. You do
not check the ctx level first, you do not weigh whether the context is warm or
relevant, and you do not exempt a member because a prior task left them running.

**The ctx percentage is irrelevant to the decision, so do not even look at the
number to decide.** The instant you find yourself reasoning *"they are only at
N%"*, *"that context is an asset"*, *"I will compact at the next seam"*, or
*"let me wait for X first"* — you have already violated the rule. Stop and
compact. **Each means each.**

**The 33% figure is a mid-flight ceiling, not a before-work gate.** It means
only: if a unit drifts over 33% while working with no handoff in sight, compact
it at the next safe seam. It is never a licence to skip the before-new-work
compaction. **Before new work: compact unconditionally, ctx unread. Mid-work:
33% ceiling. Two separate rules; the enclave gets no before-work exemption.**

**Why the rule has no threshold:** three operator corrections drove it, the
enclave twice, and each time the rationalization was a threshold — "under
threshold and warm relevant context", "compact at the flip seam". A per-role
before-work threshold *invites* the "still under it" rationalization, so there
is none. Compaction is not lossy for what matters: the summary preserves recent
detail and the agent re-fetches any source from the filesystem at pickup.

### The mechanism

**Use the checked-in script. Do not hand-drive `tmux send-keys` pane by pane** —
that races the text/Enter split and double-queues `/compact` on a busy pane.

```
scripts/handoff-gate-compact.sh [--wait-seconds <N>] <agent>...
```

List every receiving-unit member explicitly, e.g. `language-leader
language-implementer language-qa`, or the enclave triple `spec-leader
spec-author conformance-validator`. In order the script:

1. **Preflights** — resolves each agent's `.worktrees/<agent>` and its
   `moot-<agent>` tmux session, and fails before mutating anything if any is
   unresolved.
2. `git fetch origin`.
3. **`git reset --hard origin/main`** on each worktree. This also satisfies
   "start new work from current `origin/main`", but **it moves the branch ref**,
   so it discards not only uncommitted state but any committed commits the
   branch holds ahead of `origin/main`. The script auto-preserves those under a
   `preserved/<branch>` ref and warns — but still only run it once the unit is
   quiescent with its prior WP merged, and eyeball that each agent's branch is
   not ahead of `origin/main` first. **A `preserved/` ref is a safety net, not
   a substitute for knowing what a ring is sitting on.** Mind the squash-merge
   trap (`merge-procedure.md`): branch-ahead does not imply unmerged.
4. **Sends the compaction sequence** (`Enter`, `-l '/compact'`, `Enter`) to
   every pane in parallel. The `-l` literal form lands on both Codex and
   Claude-Code panes, so one script is provider-agnostic.
5. **Waits `--wait-seconds`** (default 300) and returns.

**Run it in the background.** The default five-minute synchronous wait exceeds
a foreground tool timeout — launch with `run_in_background: true`. Do the next
prep while it waits; you are re-invoked when it returns.

**The script sends the compaction; it does not confirm the drop.** After it
returns, `capture-pane` each member and confirm ctx actually fell, or a live
`Compacting...`, or a queued `/compact`.

### Verifying a drop

Accept any of: a `Compacting...` spinner, ctx dropped, or a queued `/compact`
with "Press up to edit queued messages" — the queued case fires at the current
turn's end, which is a clean seam and is correct. T1 enclave agents rarely hit
a natural idle seam during a dense event stream, so a queued `/compact` is the
normal, desired outcome.

**Draw no negative conclusion from a truncated buffer.** The `Compacting...`
progress bar renders a few lines *above* the input, so a narrow tail shows a
stale prompt plus the pre-compaction ctx and reads as a confident "did not
land" — observed on a pane at 4% `Compacting` that looked idle under `tail -5`.

**The fix is not a bigger number.** This instruction used to say `tail -20, not
tail -5`, and a bigger N only moves the cliff. **The rule is positional: if the
evidence renders above the region your window covers, that window structurally
cannot answer the question, and it does not return "unknown" — it returns a
confident wrong answer.** Search the full stream and truncate the result, never
the input:

```sh
tmux capture-pane -p -S -50 -t moot-<role> | grep -c Compacting   # correct
tmux capture-pane -p -t moot-<role> | tail -5 | grep Compacting   # wrong
```

A pane whose ctx truly did not move did not compact — resend to that one pane
and re-verify.

### The Codex harness

The fleet runs the Codex TUI in `moot-<role>` panes.

- **`send-keys` needs `-l` (literal) for text and slash commands.** Without it
  the string does not land.
- **Autocomplete eats Enter.** Typing `/compact` opens the slash-command
  palette, and a following `Enter` accepts the completion rather than
  submitting. So the type-then-separate-Enter recipe mis-fires for slash
  commands on Codex.
- **For `/compact`, `moot compact <role>` is the reliable path on Codex.** It
  lands cleanly (the pane shows `Context compacted`). Still verify the drop.
- **ctx reads as `N% context left`**, not `ctx N%` — grep `context left`, and
  accept that it is often absent from the tail entirely.
- **Post-compaction mention rouse.** A just-compacted agent does not
  auto-pick-up a mention posted *after* its compaction; it sits idle at an
  empty composer. Rouse it with `tmux send-keys -t moot-<role> -l "<one line:
  run get_recent_context and pick up event <evt_id>; re-orient per CLAUDE.md,
  then proceed>"` and then a **separate** `Enter`.
- **Clearing a garbled composer:** `C-u` clears some panes; stubborn ones need
  `C-a`, `C-k`, then repeated `BSpace`. **Never `Escape`** — it aborts an
  in-flight compaction.

### The mid-flight ceiling and the ctx scan

High context is expensive per turn for very little gain: an agent at 90%
reprocesses about 900K tokens every turn, and the working state beyond a good
summary adds little. The boundary rule above is the primary trigger; this
percentage cap is the safety net that catches drift the boundary rule cannot
see — an agent doing cross-WP assist work never hits a clean own-WP boundary,
so it silently climbs.

- Scan each active agent's context in the watchdog tick.
- At the next quiescent seam, compact any agent above about 25%, and treat
  about 33% as compact-at-the-very-next-quiescent-moment.
- An agent found above about 45% is a monitoring miss, not a normal state.

Thresholds were lowered from 60/70 to 25/33 by the operator, 2026-07-02. The
observed post-compaction floor for a heavy-context agent is 8 to 9%, so 25/33
keeps an enclave agent oscillating in a tight, cheap low band well clear of the
costly high end. Aggressive compaction is safe here because enclave work is
discrete review and authoring tasks that resume cleanly from `/spec` and the
tracker.

**The ctx scan is the mandatory first step of every watchdog tick.** It is the
one step that silently lapses, and the lapse is invisible — a stall scan comes
back "all clear" while a T1 agent climbs. Two amplifiers, both real: a *minimal
tick* run to conserve compute must still include the one cheap capture, since
it is the cheapest high-value line in the tick; and a **self-authored enclave
cascade** is the peak-risk window, because none of its steps hits a
Steward-delivery boundary compact, so the scan is the only trigger that can
catch it. Escalate the scan during a cascade, never relax it. **A tick that
reports "all clear" without a ctx line is an incomplete tick.**

**Cross-check with the handoff gate:** if a unit was handed a WP this cycle yet
its members' ctx is still high, the gate was skipped. The gate is the proactive
fix; this scan is only the backstop. **When the scan is the thing catching a
stale enclave, the gate already failed upstream — treat that as the miss, not a
routine catch.**

### Before you compact anyone: outstanding obligations

Confirm the agent owes nothing in flight — a pending review vote on another
team's open Decision, an unfinished handoff, an open `question` it must answer.
Compaction drops the obligation. K3: the spec enclave was compacted for its
next WP while a merge-review request was open, the vote was dropped, and it
surfaced only at the merge gate. Resolve, reassign, or confirm-not-required
first.

**Precondition: quiescent.** Never compact an agent mid-reasoning.

## Self-compaction: the checklist

The operator has corrected this three times. **Run the checklist; do not
improvise.**

A build team gets its compact seam free from the WP pipeline. You and the other
singletons have no such boundary — your work arrives event-driven from many
sources at once, so you must manufacture your own seam.

**The first half matters more than the second: keep your durable checkpoint
continuously current.** A compaction, auto or self, is safe because your resume
state already lives on disk in the progress tracker, not only in context. You
cannot read your own token count from a tool, so you cannot time it perfectly;
the discipline is to keep the tracker so current that whenever compaction
fires, resume is lossless. **A stale tracker is the only thing that makes a
random-timed autocompact dangerous. Fix the staleness, not the timing.**

### When

At or near 33% ctx. Check your own pane at every seam. Above 33% you are
already late — compact at the next safe moment, not at the next milestone.

```bash
tmux capture-pane -t moot-steward -p | grep -oE 'ctx [0-9]+%' | tail -1
```

### The six steps, in order

1. **Finish the current turn's durable state.** Tracker and node edits
   committed, worktree clean (`git status --porcelain` empty), nothing
   half-posted.
2. **Write the resume checkpoint** — the one tracker line naming the very next
   action. Assume you wake with nothing but that line.
3. **Launch the detached resume watcher.**
   ```bash
   nohup scripts/postcompact-resume.sh moot-steward >/tmp/pcr-steward.log 2>&1 & disown
   ```
4. **Send `/compact` to your own pane — text and Enter as two separate calls.**
   ```bash
   tmux send-keys -t moot-steward -l '/compact' ; sleep 2 ; tmux send-keys -t moot-steward Enter
   ```
5. **Stop. This is the last action of the turn.** `/compact` fires at turn end;
   any further tool call delays or eats it.
6. **On wake:** re-orient per `CLAUDE.md`, read the checkpoint, resume.

### The three ways this fails

- **Launching the watcher is not compacting.** Step 3 without step 4 leaves you
  running at full context believing you compacted. That is the exact miss the
  operator caught twice.
- **Announcing it is not doing it.** "I will self-compact now" in your reply,
  with no `send-keys`, is nothing.
- **Fused keystroke.** `send-keys '/compact' Enter` in one call can drop the
  newline and leave the command unsent. Always two calls with the `sleep 2`.

### Two rules that are not part of the checklist

- **The watcher is for self-compaction only.** Never launch it when
  handoff-gate-compacting a team — there the kickoff mention is the resume
  trigger, and a premature resume wakes the unit into "no new work".
- **Never nest `nohup ... &` inside a backgrounded Bash call** — the
  notification then describes the wrapper, not the watcher.

Why the watcher exists at all: `/compact` returns the seat to an empty prompt
with nothing to re-invoke it, so a detached process immune to the turn
lifecycle must send the resume after the `Compacting...` window clears. A
SessionStart hook cannot substitute — it shapes the next turn's context, it
cannot trigger a turn.

Steward, Architect, and Librarian all self-compact this way; see
`../architect.md` section 3.
