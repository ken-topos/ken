# The federation watchdog and the comms-drop backstop

Steward task procedure. Read at the point of use. Governing playbook:
`../steward.md`. `COORDINATION §13` defines the layer.

You run the top liveness layer, the watcher-of-watchers. The only thing above
you is the operator, who reads the absence of your updates as the signal that
the backstop fell over.

## What to catch

Federation-level stall patterns: a whole team gone idle; a stalled team leader
(its own watchdog died); a dropped cross-team query; a blocked dependency chain
(team B waiting on a merge from A that never came); no movement toward the
active roadmap gate. Diagnose before restarting, use graduated recovery (nudge,
re-nudge, act), and escalate what you cannot restart.

**You are the backstop, not the primary rouser — rouse the leader, not its
workers** (operator, 2026-07-11). Per-member rousing is the team leader's
watchdog job. You run on the fleet's most expensive model, so every time you
rouse an implementer or QA directly you burn premium credit doing a leader's
work — and you mask the real defect, which is that the leader's watchdog is not
running. Direct-rouse a worker only as a last resort when its leader is also
down and the work is time-critical, and then treat the stalled leader as the
actual bug. **If you notice yourself hand-rousing the same ring repeatedly,
that ring's leader-watchdog is the thing to escalate, not a cadence for you to
absorb.**

**Watch for communication-topology divergence, the opposite failure from a
stall.** A stall is too little traffic; a divergence is too much — the channel
spins on interaction without advancing state. Tells: a bilateral negotiation or
ping-pong (ownership offered back and forth, an assignment re-settled three or
more times); an ack or re-confirm fan-in; a ceremony cascade of low-value
errata; a judgment thread with no owner (N nodes opining, none holding the
pen). **The signature is commit cadence slowing while the channel stays hot.**
Intervene by naming the fixed rule, not by adding to the thread: point to the
assigner, invoke silence-as-assent, or apply the honesty-erratum filter — one
message that collapses edges, then stand down. Do not add another asserted
opinion, and never introduce a new edge to fix one.

## Arming the tick

Use the convo-channel `schedule_create` self-wake. **Never** the convo
`schedule_call`, and never a hand-rolled bash loop or `Monitor`-tool poll
(operator, 2026-07-20; this supersedes the earlier `CronCreate` guidance).

```
schedule_create(interval_seconds=900, label="steward-watchdog",
                prompt="[Steward watchdog tick] ...")
```

It delivers a tick privately into your own session and posts nothing to the
space. On each fire, run a private `get_recent_context` read plus the pane
sweep below, and message the space **only** when there is a real stall to
nudge. **Post nothing on a clear tick.** It returns a `schedule_id`;
`schedule_delete(schedule_id)` disarms it.

The convo `schedule_call` broadcasts its read into the space as a System event
everyone sees — never use it for the watchdog. A bash `while true` loop, the
`Monitor` tool, and the old `local/steward-watchdog-wake.sh` are superseded:
they only watch git refs, blind to pane-level stalls, or leak a CPU-spinning
orphan. Do not resurrect a script.

### The reconnect regression: the one way this backstop silently dies

`schedule_create` schedules live only for the convo-channel MCP process's
lifetime and **do not survive an MCP reconnect** — a package upgrade, a network
blip, or a self-compaction that re-instantiates the client. Posting can stay up
while they are gone, so you get no signal.

**Re-arm on session start, after every compaction, and after any convo-MCP
reconnect. Run `schedule_list` at the top of every tick** — an empty list while
work is open means your backstop fell over.

> **Re-arm from the stored file, never from memory.**
> `agent/playbooks/federation/steward-watchdog-tick-prompt.txt` is the
> canonical tick prompt. The live interval is the only other copy and it dies
> on exactly the events above, so a re-arm typed from memory silently drops
> whatever the prompt has accumulated. **The first casualty would be the WIP
> audit clock** (`escalation.md`), the one item with a deadline: lose it and
> the tick still looks complete, still reports all clear, and no longer fires
> the audit. Re-arm by pasting that file verbatim, and when you change the
> prompt, edit the file and publish it in the same act as the interval call.

On this Claude-Code seat the host-level `CronCreate` does survive an MCP
reconnect and is a valid durable fallback, but default to `schedule_create` for
fleet uniformity.

## The tick, in order

### 1. The stranded-delivery sweep — the first action of every tick

**It is a check; do not re-derive it as vigilance.** The single most common
transport failure in this federation is a convo mention that lands in a seat's
composer and is never submitted. The event exists, the seat looks healthy, and
its ring blocks silently. On 2026-07-14 it fired six times in one day, each
costing a premium-model tick to notice.

```sh
scripts/sweep-wedged-panes.sh            # detect + repair + VERIFY the repair
scripts/sweep-wedged-panes.sh --dry-run  # report only
```

It sweeps every `moot-*` session, submits any stranded delivery on the composer
line, re-reads each pane to confirm it cleared, and reports any that did not
take the `Enter`. It **skips** anything marked *"Messages to be submitted after
next tool call"* — that is queued and healthy, and re-sending would
double-deliver — and it never touches `moot-steward`, whose composer is your
own. **A tick that has not run the sweep is an incomplete tick.**

Classification lives in `scripts/classify-pane-composer.py` — verdicts `paste`,
`slash:<cmd>`, `ghost`, `other`, `queued`, `clear`, `busy`, `unreadable` — with
controls in `scripts/test-classify-pane-composer.sh`. **Add a shape there,
never by widening a regex in the sweep.**

Two properties worth knowing when you read its output:

- **`other` is never submitted.** An idle pane renders its own suggestion text
  on the composer line, so with colour stripped a suggestion is
  indistinguishable from a real delivery, and submitting one sends an agent an
  instruction nobody wrote. The discriminator is the escape stream: suggestions
  are wrapped in `ESC[2m` (dim), real content is not, so the sweep captures with
  `-e`. The allow-list is `{/compact}`; widening it widens what the fleet can be
  made to run unattended.
- **Honest residual:** a short delivery that lands as raw composer text with no
  delivery envelope classifies as `other`, so the sweep reports it but does not
  repair it. That is deliberate — it is indistinguishable from half-typed
  operator input — and it means **the sweep does not replace the per-seat
  `Working` check.** Likewise a `busy` verdict is never repaired: a run
  reporting four `busy` seats has answered nothing about those four.

### 2. The ctx scan

See `compaction.md`. It is the mandatory first *reading* step and the one that
silently lapses.

### 3. The idle sweep

**Every tick proactively sweeps active seats' panes for idle, not only
reactively after a convo signal.** Operator-grounded, 2026-07-11: an
implementer kicked-but-never-engaged sat idle about 75 minutes, invisibly.

The stall patterns mostly fire off convo reads, and the git-ref check keys off
pushed branches — but the worst stall emits **no convo signal and no branch at
all**: a seat whose threaded-mention kickoff never woke it, or one that
compacted and re-oriented to "awaiting kickoff", silently dropping its
assignment. It posts nothing and pushes nothing, so both the git check and the
context read come back all clear while it burns wall-clock parked. **Only a
direct pane sweep catches it.**

On the same capture you take for the ctx scan, read idle-versus-Working. A seat
at an empty input prompt while it holds an active assignment, or one showing
`Context compacted` or `awaiting kickoff` *after* it was kicked, is stalled.
Re-rouse it directly — `tmux send-keys -t moot-<role> -l '<pickup text pointing
at its durable in-thread assignment>'`, then a **separate** `Enter` — and
confirm the pane flips to `Working`. **A fresh convo mention alone will not
wake a no-poll idle seat.**

### 4. Read the Adversary's threads

Its reports do not surface in the space-level event read, so a clear tick is
not evidence it has filed nothing. The command is in `merge-procedure.md`, M8.

## Reading a pane

**Capture `-S -40` or more. A short window manufactures a false IDLE.** The
spinner and elapsed line sits *above* the composer, so a narrow capture renders
the composer, the ctx line, and the permissions line — a display identical to a
healthy idle seat — while cutting off the one token that proves the seat is
live. **It does not return "unknown"; it returns a confident wrong answer.**
Never conclude idle from a window whose topmost line is the composer: that
window structurally cannot hold the evidence.

| pane shows | meaning | repair |
|---|---|---|
| `Working (Ns...)` | delivered | none |
| any spinner plus elapsed, e.g. `<glyph> <verb>... (Nm Ns` | busy; high-effort turns show no `Working` and no `esc to interrupt` | nothing |
| a paste marker on the composer, no spinner | delivered to the buffer, never submitted | send a bare `Enter` |
| a channel-mention envelope on the composer, no spinner | the same strand, and the commoner shape — the marker is often absent or split across the wrap | send a bare `Enter` |
| empty composer, no paste, no spinner | **ambiguous, not "never delivered"** | re-capture wider first, then re-deliver |
| `Working` plus `Queued follow-up inputs` | busy, message queued | nothing — do not resend |

**The empty-composer row used to say "never delivered at all" and that was
wrong.** Measured 2026-07-26: that row was read off a six-line capture and a
leader was told its implementer's turn had ended mid-sequence. It was 32
minutes into one continuous turn and posting its handoff. The leader re-tasked
a seat that was already working. A kickoff re-delivery into a live turn
double-delivers; a `/compact` there destroys in-flight work.

**The decisive test is the elapsed counter's continuity across two captures.**
`14m 22s` then `32m 16s` is one turn; a genuinely new turn restarts near zero.
When you believe a seat went idle, take a second wide capture and compare the
two readings before acting.

**Do not key on the spinner verb — it is randomized.** Key on the shape
`<glyph> <verb>... (<elapsed>`. Past-tense forms (`Worked for 1m 47s`) are what
a *finished* turn leaves behind, not a live one.

**None of these is evidence a turn ended:** a clean worktree, commits already
present, or an empty composer. All three are equally true of a live turn
sitting inside a tool call.

## The stale-status discount

**Never diagnose a stall from a status string or ghost text** (operator rule,
2026-07-03; one WP false-stalled four times). Participant statuses from
`orientation` or `list_participants`, and the tmux ghost-text suggestion, are
point-in-time and can be more than a day stale — a status can still say
"awaiting X's re-run" a full day after X landed.

1. A status or ghost line is a hint to verify, never evidence of a stall.
2. Ghost text is a next-prompt suggestion, not state.
3. **Verify WP or arc closure by content on `origin/main`** — grep the landed
   change, or `is-ancestor` the **merged** SHA — never a specific local branch
   SHA. A rebased or squashed merge lands under a different SHA, so checking
   the pre-rebase tip falsely reads "unmerged". The canonical trap: K7 merged
   as `4ae2baf` while the pre-rebase local tip was `b7396ae`, and
   `is-ancestor b7396ae` gave a phantom stall that nudged a reviewer to re-run
   closed work.

`capture-pane` tells you busy-versus-not-busy right now; git-by-content tells
you done-versus-not-done. **A status string tells you neither.**

## Since-window blindness

`get_recent_context(since_event_id=X)` shows only events *after* X, so
anchoring X on a recent event hides all earlier activity. Before diagnosing a
"done-but-unrouted" or "no-movement" stall, check the authoritative artifact
directly — the Decision object for its status, and a wide context scan —
because the routing, votes, and resolution may predate your anchor.

**A branch commit existing is not evidence it was not posted.** 2026-07-03: a
committed candidate plus a forward-only read returning "(no events)" produced
the conclusion "CV never routed it", and a nudge. CV had routed it, all three
gates had voted approve, and the Decision had resolved and gone to
`merge_ready` — all before the anchor. The retracted false nudge burned two T1
seats' attention.

**"Unrouted", "unmerged", and "no votes" are claims about a Decision. Verify
them on the Decision object, never infer them from a commit plus a narrow
forward-only read.** When it turns out stale, retract explicitly; a clean
withdrawal ends it.

## Do not assert a fast-moving routing or ownership state from a stale read

And never adjudicate intra-team task assignment. Steward routing is cross-team
sequencing and the WP gate structure — that is yours. **Who on a team does a
mechanical companion task is the leaders' call.**

The failure: a "@X owns it, free the branch for @Y" ask built on a
recent-context read about four minutes stale, on a thread that had flipped
ownership four times in one minute — reintroducing a contradiction the leaders
had already triple-confirmed closed, and forcing an implementer to stop and ask
which of two authorities to obey.

1. A post that *asserts* a routing or ownership state must be timestamp-current.
   On a fast-moving thread, re-read at the moment of posting, or frame it as
   *"defer to the leaders' settled state"* rather than naming an owner.
2. Scope your post to what is cross-cutting (the gate structure) and leave the
   assignee to the owning leaders.

When your assertion turns out stale, **retract it explicitly and defer.** Do
not re-argue: a clean withdrawal ends the ping-pong, another asserted
correction extends it.

## The comms-drop backstop

`capture-pane`, then git-verify, then relay. The federation's recurring defect
is dropped notifications: a handoff, retro, or `git_request` correctly posted
but never waking the target. When a stall pattern fires, do not restart or
re-mention blind.

1. **`capture-pane` to diagnose** — working indicators mean stand down; an idle
   prompt with an unprocessed mention on screen means wedged.
2. **git-verify the handed-off work actually exists** — the commit or branch
   the post claimed.
3. **Relay** — a real mention if the channel is flowing, or for an idle-wedged
   session `tmux send-keys -t moot-<role> "<text>"` then a **separate**
   `tmux send-keys -t moot-<role> Enter`. Text and Enter in one call does not
   submit.

**Log every relay. Never interrupt a working agent; capture-pane first,
always.**

## Verify pickup after every kickoff or handoff

Delivery is not engagement. The same 2026-07-11 miss upstream: a build leader
posted a correct threaded kickoff and held a "producing the SHA" belief while
its implementer had never engaged. After any kickoff or handoff — yours or a
leader's you are backstopping — confirm the target actually engaged before
treating the work as in-flight. **Never carry a "producing X" belief on the
strength of the mention merely having been posted.**
