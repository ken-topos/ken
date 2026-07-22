# Ken coordination law (read by every agent)

Cross-cutting rules for every Ken agent, regardless of role, team, or model.
Role-specific discipline is in `playbooks/`; model tiers are in `MODELS.md`; the
git model is in `../docs/program/04-git-and-integration.md`. These rules are
adapted from hard-won mootup team lessons; each exists because skipping it
caused a real stall or a real bug. They must hold identically across Opus, GLM,
and DeepSeek agents.

## 0. The shape: a ring of rings

- **Within a team — a sequential token-ring.** Generally one agent is active at
  a time; the others are in a supporting role, called on when the active agent
  needs them (e.g. an implementer asks for a clarification). Keeping the whole
  team on one task maximizes coherence and effectiveness. Do not fan a team onto
  several tasks to chase parallelism — coherence beats it. This includes waiting
  on CI: once a WP is published, the team **waits idle** for its CI run rather
  than pipelining or stacking work (ADR 0002). Idle is cheap and load-friendly;
  throughput comes from other teams' rings, not from this one multitasking.
- **Across teams — parallel.** The teams are independent rings spinning at once;
  that parallelism is the entire reason the work is articulated into teams. The
  rings couple at only three points: merges to `main` (via the publisher path),
  the roadmap gate dependencies, and the **sanctioned cross-team query edges**
  (§9, §11). Keep that coupling thin — it is what serializes the federation if
  abused.

## 1. Event-driven, never poll

After you finish a unit of work or hand off, **post, set status, and stop.** Do
not `/loop`, self-wake, or poll for replies. The notification system delivers
what you need; polling burns tokens for zero value. A missing notification is a
*stall* — catching stalls is the team leader's watchdog job, not yours. Only
team leaders and the Steward run schedulers.

## 2. Mention discipline

**Mention an agent iff you are (a) asking them a question or (b) expecting a
specific action/move from them — nothing else.** Every mention costs the
recipient tokens and fires a notification. Mentioning to *inform*,
*acknowledge*, *converge*, *confirm*, *affirm*, *CC an observer*, or *keep
someone in the loop* is **not** a permitted reason: if the recipient is not
being asked to answer or to do a next move, **do not mention them** — post it
un-mentioned so it lands in the context for whoever reads it, or don't post it
at all. (Operator, re-tightened 2026-07-03, second pass: a regression to
ack-with-mention crosstalk — "@X acknowledged / noted / tracked for later /
standing by" fires a notification for zero required action. This is the exact
pattern to eliminate; it binds every role, the Steward included.)

**A mootup mention is the *only* way you reach a teammate.** Every agent in the
federation is an **already-running, persistent peer** with its own always-on
session — never a sub-agent you launch. To delegate, query, hand off, or reach
*anyone*, **post a convo message mentioning them** (`post_response`, `mentions:
["<actor_id>"]`). **NEVER** spawn a teammate with the `Agent`/Task tool, a
subprocess, or `claude(prompt)` — that starts a fresh, unconfigured Claude (it
fails "503 provider not configured") and is not how this federation works. This
has bitten DeepSeek leaders mis-reading "hand the WP to your implementer" as a
launch instead of a mention. Assignment, query, handoff — all are mentions; local
git only.

- Handoff that passes work to B → mention **B only**.
- "Your request is done," nothing pending → mention **nobody**.
- **A mention is a demand for attention; an ack is not — so never mention on an
  ack (operator, 2026-07-03).** You *may* post a brief **mention-free** ack for
  visibility (§4); the `mentions` field is reserved for the message whose *next
  move is the mentioned agent's*. Spend mentions only on real next-moves.
- **The escalation triangle (learned twice):** when you answer X's question but
  the *next move* belongs to Y, mention **Y**, not X. Naming Y in prose without
  a real mention is the classic silent stall.
- **Answering is not passive — a ruling / verdict / answer that moves the ball
  MUST mention whoever owns the next move, *including when that owner is the
  asker*.** The escalation triangle's blind spot: when your reply hands the move
  *back to the person who asked* (your design ruling → they author the WP; your
  review verdict → they apply the fix or merge), it feels like you are "just
  answering," so you mention no one. But that answer **expects the asker's next
  action** — which is exactly the §2(b) mention case. Populate the `mentions`
  array (not a prose `@name`, which fires no notification — §2a). Observed
  2026-07-11: three Architect design rulings that answered the Steward's
  questions each posted with an **empty `mentions` array** and no in-text
  address → invisible to the asker's `get_mentions`, surfaced only by manual
  thread-polling, so every one of them was a latent silent stall. **If your post
  moves the ball to anyone — asker or third party — mention them.**
- **Route an action-expecting mention only to a *live* agent — never a `moot
  init` template placeholder (promoted B1, spec-leader's catch).** The space
  carries dead template participants (`Spec`, `Leader`, `QA`, `Implementation`,
  …) left by scaffolding; a review/handoff/kickoff mention routed to one is a
  **silent no-op** — it wakes no one. The tell: a placeholder is
  `participant_type: agent` with **`agent_adapter: null`** (no recent
  `last_seen_at`); a running agent has `agent_adapter: "mcp"`. Before routing a
  vote/handoff to a participant you didn't just hear from, confirm it's live
  (`list_participants`). This was the mechanism behind the Sec1ct §14 breach —
  "Spec review" routed to the dead `Spec` placeholder, so the gate never ran.
- **Thread your reply — don't scatter to the space root** (operator 2026-06-29;
  hardened 2026-07-03: **only the Steward posts top-level, every other role always
  threads — §4**). Every event you receive carries a `thread_id`. When you respond
  to a message that belongs to a thread — a kickoff, assignment, query, handoff,
  review request, or retro call — **reply *into that thread*** (`post_response` with
  `thread_id` set to that message's thread; or `parent_event_id` on the first
  reply, which opens the thread). A bare `post_response` with no thread drops
  your message at the space root, **scattering one WP's conversation across the
  space** so the next reader — and the Steward harvesting retros — can't follow
  the exchange. **One WP = one thread:** kickoff → ack → queries → handoff →
  merge Decision → retros all live under it. **`reply_to` needs an *existing*
  thread** — it 404s ("Thread not found") on a **top-level** event that has no
  thread yet (e.g. a kickoff, which is itself a root post). To **open** a thread
  on such an event, use `post_response` with `parent_event_id` set to it; use
  `reply_to` only for an event **already in** a thread. (L5: an implementer lost a
  round-trip when `reply_to` on the kickoff 404'd.)

### 2a. Convo call cheat-sheet (form valid parameters — recurring failure)

Malformed convo calls fail **silently to the workflow** (the post never lands,
so the next move never fires). The two parameters that get rejected:

- **`message_type` MUST be one of the backend enum values** — guessing a
  natural-language type is a 400 reject. **Valid:** `question`, `code_share`,
  `git_request`, `review_request`, `retro`, `status_update`, `bug`, `feature`,
  `decision_propagated`, `pause_issued`, `connection_status`. **REJECTED (do not
  use):** `message`, `kickoff`, `assignment`, `merge_ready`, `handoff`, `nudge`,
  `ack`. Map your *intent* to a valid type:
  | Intent | `message_type` |
  |---|---|
  | kickoff / assign a WP / hand work to a teammate / deliver code | `code_share` |
  | question / query / nudge / ack / general note | `question` |
  | ask for publisher-path merge handling | `git_request` |
  | QA→leader merge-Decision request; "request review" | `review_request` |
  | a retro bullet-set | `retro` |
  | a status line | `status_update` |
  | a defect note to another team | `bug` |
  When unsure, **`question`** is always accepted. If you get
  `unknown message_type '<x>'`, re-send with the mapped value — don't drop the post.
- **`mentions` MUST be a list of participant_ids** — `mentions: ["agt_…"]`, the
  raw IDs resolved from `list_participants` / `orientation()`. **Not** display
  names (`["spec-leader"]` ✗), **not** `@name` in the `text` (that fires no
  notification — §2), **not** a bare string. One id per actor whose move is next.
- **`propose_decision`**: give the decision `text`/title + the WP/branch; mention
  the required reviewers (Architect always; Spec only on `spec/`+`conformance/`
  paths — §13/diff-scope). **`thread_id`/`parent_event_id`** take an event/thread
  id string (§2), never a name. **`reply_to` 404s on an un-threaded (top-level)
  event** — to thread under a kickoff, use `post_response` + `parent_event_id`,
  not `reply_to` (§2). When a call 400s/404s, **read the error, fix the one named
  field, and re-send** — a dropped post is a silent stall.

## 3. Status = what you're doing, in your own words

Three liveness signals exist; only the third is yours to post:
1. connection (automatic — never post "I'm online");
2. activity (file/transcript mtime — automatic);
3. **semantic status** — "drafting K2 conversion", "blocked-on-spec: OQ-17".
   Agent-composed, never auto-classified.

**Update your status on _every_ change in your activity (operator, 2026-07-03).**
Not just at handoff boundaries — whenever what you are doing changes: picking up a
WP, starting a sub-step, finishing, changing focus, going idle, blocking,
unblocking. Your status line is the federation's **at-a-glance truth**
(`list_participants`) and the Steward's primary watchdog signal; a status that
lags your real activity is a coordination hazard — it reads as *stalled* when you
are working, or *working* when you are already done, and sends the watchdog
chasing phantom stalls (or missing real ones). Treat the `update_status` call as
**part of every activity transition**, the same reflex as saving a file — cheap,
frequent, always current. This is the low-cost, always-on signal that lets the
rest of the discipline below (mention-free acks, Steward-only roots) stay quiet.

## 4. Threads are the spine

One mootup thread per work item; the kickoff message *is* the spine. All
handoffs, questions, status, and retros for that item are **replies in that
thread** — a top-level post fragments the work. After any context
reset/compaction, resolve the live thread from fresh context; do **not** reuse a
thread/event ID from a summarized memory (it may be stale).

**Top-level (root) posts are the Steward's alone — every other agent always
threads (operator, 2026-07-03).** A non-Steward post *always* carries a
`thread_id` or `parent_event_id` and lands **inside an existing thread**; no other
role ever creates a root-level message. The rationale is structural, not
stylistic: **the Steward is the coordinator of all activity, so any work another
agent does traces back — by origin — to a message the Steward sent.** That message
is therefore the natural root your reply threads under. Enforcing it makes the
whole space a clean tree rooted at the Steward's coordination posts and ends the
fragmentation where parallel root posts scatter one WP's exchange (the exact
threading-discipline failure this fixes). Consequences:
- **Reply, never root.** Use `post_response`/`share` with the WP thread's
  `thread_id` (or `parent_event_id` on the kickoff to open its thread); never a
  bare un-parented post. The mechanics are unchanged (§2's `reply_to`-vs-
  `parent_event_id` note still holds) — what changes is that rooting is now
  **prohibited** for non-Steward roles, not merely discouraged.
- **No home thread? Route to the Steward, don't root it.** If you have something
  that fits no existing thread (a genuinely new cross-cutting concern), thread it
  under the nearest relevant Steward post — or the standing `steward:` coordination
  thread — mentioning the Steward if a decision is needed. The Steward owns thread
  creation and will open a new root if the item warrants its own spine. A spun-off
  WP (its own branch + gates + merge, e.g. an erratum) is still opened as its own
  spine — **by the Steward**, per the WP-release process — with a one-line pointer
  from the parent.
- **Decisions, questions, reviews still thread.** `propose_decision`,
  `question`, `review_request`, `git_request`, and `retro` all attach to the
  relevant WP thread (their `thread_id`/`parent_event_id`, §2a); the Steward-only
  rule governs *root chat posts*, and these governance objects are never a reason
  to open a new root.

**Threading drifts in exactly two ways (operator 2026-07-02) — guard both.**
(1) **A fork spawns a side-thread.** A mid-WP escalation — a soundness fork to
the Architect, a scope fork to the Steward, a conformance fork to CV — is a
**reply *in* the WP thread** (the `mention` routes it to the owner); opening a
fresh root for the ruling scatters the WP across threads even though every
message was correctly *addressed*. The ruling and its answer belong under the
kickoff. (2) **A new WP reuses the *previous* WP's thread.** By reflex (or
post-compaction) an agent replies in the most-recently-active thread — usually
the *last*, now-closed WP's — instead of the current WP's kickoff. "Resolve the
live thread from fresh context" means **the current WP's kickoff root, not the
most-recent thread**; the kickoff event you were mentioned on *is* the anchor,
and a kickoff **states its own anchor** ("thread all `<WP>` activity here") so
there is nothing to guess. The only sanctioned *new* thread off a WP is a
genuinely separate spun-off WP (its own branch + gates + merge, e.g. an
erratum): open it as its own spine with a one-line pointer from the parent.

**Post at both boundaries — on receiving work and on handing off (operator,
2026-06-29).** When you **pick up** a task, post a brief *taking-this* note in its
WP thread and set your semantic status to it (§3); when you **finish or hand
off**, post what you did + the mention of whoever moves next (§2) and update your
status again. **Both signals — the in-thread post *and* the status — at both
ends, not just at handoff.** The boundary posts keep the flow legible in the web
view and preserve the interaction history; without them there is a silent gap
between assignment and completion that no one can audit or replay.

**Acks are allowed — but never with a mention (operator, 2026-07-03; refines the
earlier "no bare acks").** The problem was never the acknowledgment itself — it
was that a **mention fires a notification and *obligates* the recipient** to spend
tokens reading and deciding whether to respond. So the rule is about the
*mention*, not the *post*: **a mention is a demand for attention; an ack demands
nothing, so an ack MUST mention no one.** Concretely:
- **You MAY post a brief, mention-free ack in a thread** — "seen", "picked this
  up", "proceeding", "noted, folding in" — so that anyone scanning the thread, and
  the Steward harvesting flow, can *see* the activity without anyone being pinged.
  A mention-free ack is a **passive visibility signal**, not noise: it does not
  interrupt or obligate a soul.
- **Reserve `mentions` strictly for the message whose *next move is the mentioned
  agent's*** (§2) — a handoff, a blocking question, a finding they must act on.
  **Never @mention on a mere ack** — that manufactures a false obligation and is
  the exact interruption the earlier rule was reaching for.
- **Silence is still fine.** If you need nothing back and diverge on nothing, you
  may simply proceed and let the next *substantive* post (candidate-ready, a
  finding, a handoff) be your acknowledgment. Mention-free ack and silence are now
  **both acceptable**; the audit trail is still carried by the substantive posts +
  your always-current status (§3), and the hard line is unchanged only in its
  sharpened form: **the mention, not the post, is the scarce resource — spend it
  only on a real next-move.**

Note this pairs with the two rules above: **every activity change updates your
status** (§3, the always-on signal) and **only the Steward roots a thread** (every
ack is therefore a mention-free *reply* inside an existing thread). Together they
let an agent broadcast "I'm alive and moving" through cheap, non-interrupting
channels — status + threaded mention-free ack — reserving mentions for genuine
handoffs.

**A mention is never an acknowledgment (operator, 2026-07-03, second pass).**
The regression that keeps recurring is an ack that *also* mentions — "@X
acknowledged," "@X noted, tracked for later," "@X standing by," "@X converged,
agree." Even when the prose feels substantive, if the recipient is not being
**asked** or **tasked**, the mention is pure notification-noise. So: if your only
reason to name someone is to tell them you received / agree with / are proceeding
on their message, mention **no one** and post **nothing** — proceed silently
(silence is acceptance, above). Reserve every mention for a question or an
expected next move (§2 opening). When you *do* have a substantive routing post
(a decision, a finding, a handoff), mention **only** the one actor whose move is
next — not the observers you're "keeping in the loop" (they read the context).
This applies to the Steward's own posts as much as anyone's.

**Title convention (the single-space articulation tag).** The whole federation
runs in one space (`ken-topos`), so a thread's **title is its team tag** — there
is no separate per-team channel. Begin every kickoff title with a tag, then a
terse subject:

- **Work-package threads:** the WP ID, whose letter encodes the owning team —
  `K*` Kernel · `V*` Verify · `L*` Language · `X*` Runtime · `Sec*` security ·
  `B*` seam · `T*` tooling · `F*` foundation. E.g. `K2: decidable conversion`.
- **Non-WP threads** (a cross-team query, a process/retro note): prefix with the
  originating role/team and an arrow if it targets one — `steward: cadence
  pass`, `kernel→spec: OQ on cast normal form`.

This makes the one space scannable and `search_spaces`-filterable by team
without the cost of per-team spaces, and pairs with the `message_type` taxonomy
(§8): the title says *who/what*, the type says *what kind*, and `list_threads`
filters on type with per-actor unread counts. Keep both accurate.

## 5. Decisions are for judgment, not deduction

Open a mootup Decision (`propose_decision`) for choices with tradeoffs where a
reasonable peer might differ — kernel/semantics design, an API shape, a
content-store policy. Do **not** open one for deductive/mechanical choices (a
bug fix is not a decision). Decisions are how future agents query *why* Ken is
the way it is. **Merge/review approvals are also Decisions** — the merge
Decision *is* the review record (see `04-git-and-integration.md`).

## 6. Resolve when structurally determined; escalate only real forks

Before escalating or querying another team, ask: *is there a strategic choice
between materially different futures?* If **no** — the published spec + kernel
invariants + existing code already determine the answer; resolve it yourself and
record the resolution with a cited rationale (`file:line` or spec §). If **yes**
— escalate. For clean-room questions, "the published spec" means `/spec`, never
prototype source. This filter is the volume control on the cross-team query
edges (§11): without it, Spec and the Architect become bottlenecks.

## 7. Ground every premise before locking

Before locking a spec, ADR, or design claim, verify each premise against
reality: "X exists" → grep for it; "matches pattern Y" → read Y end-to-end. For
a *verified* language a spec claim about the kernel must be checked against the
kernel, not assumed.

**Verify the *property*, not the representative case (promoted from F4 + K1).**
A passing obvious-case test and correct *prose* are **false signals** — the
algorithm or code can still be wrong in the cases you didn't exercise. (F4: a
Map/Set determinism test passed only because `BTreeMap` pre-sorts, never
exercising the encoder's own sort. K1: the positivity conformance tested only
direct arrow-domain negativity and missed the hidden-negative classes — `Pair
(Bad3→Empty) Unit` — that the Architect caught; the §8.2 *prose* was right while
the *algorithm* silently dropped those positions, a soundness hole in the trust
root.) So — for QA, conformance authoring, spec authoring, and review alike:
when an algorithm has N guard/discard positions, **exercise each independently**
(one case per guard) plus one where the problem hides behind indirection; write
algorithmic pseudocode **defensively** (every discarding position
explicit-guarded, every guard backed by a rejection case). The obvious case
passing ≠ the property holds.

**A discriminator boundary needs a non-degenerate *pair*, not a positive case
(promoted Sec1/Sec1ct/V3/B1 — ≥4 domains, build *and* spec enclave).** When a
rule, classification, projection, or lattice-axis *orientation* decides between
two outcomes on a boundary (accept/reject, `proved`/`assumed`, taint/safe), a
single positive case is **green-vs-green** under the exact swap it should catch:
flip the order/orientation and the lone case still passes, for the wrong reason.
The net is **one non-degenerate pair on a *shared* input** — the two states that
must bucket differently, identical in every other respect (Sec1ct: a `ct⊤` value
rejects *while* a `Secret`-not-`@ct` value accepts on the *same* sink shape; B1:
a `proved` claim maps to `Q` *while* a hole maps to `P` on the *same*
postcondition) — so a flipped boundary inverts **both** and the pair fails. And
the discriminator the case keys on must be a **structural / kernel-side signal**
(`trusted_base()` membership, a constructor, a real value through a real sink),
never a **self-reported string** the untrusted layer can forge.

**Exhaustive-by-construction: make load-bearing completeness a *compile error*,
not a test (promoted across the V-spine + X1-effects; verify-team-requested).**
When the **completeness** of an enumeration is load-bearing — every obligation
classified, every effect dispatched, every verdict projected, every variant
handled — encode it as a **single `match` over a sealed type with NO `_ =>`
catch-all arm**, so adding a variant without handling it is a **compile error**,
not a silent drop. This is *structural*, stronger than any test: it's the
constructive form of the two-soundnesses *omission* backstop (a silently-skipped
case supplies nothing for a downstream checker to catch). Validated at four tiers
— V2 `lift_obligation` (no `_=>` over `ObligationKind`), V3 `classify` (HO is the
**explicit** default arm, not a skip), V4 `project_diagnostic` (one `match` over
`Verdict`, no relabel path), and X1-effects `drive_h` (sealed effect-class enum →
new variant = compile error, out-of-row tag → honest `panic!`). Reach for a
catch-all **only** where the residual is genuinely uniform; where completeness is
the safeguard, an explicit arm per variant is the safeguard. Implementer writes
it; QA + the Architect verify **no `_ =>`** sits on a completeness-critical match.

## 8. Message-type taxonomy (routing metadata)

Tag each message with a type; the **first line is the thread title** — no
`[TYPE]` prefix in the body. The type MUST be a **backend enum value** — see the
**§2a mapping table** for the valid set and how to map your intent. In
particular, a local `wp/<ID>` branch ready for publisher-path merge handling is
`git_request` (the old `merge_ready` is rejected), and a QA→leader
merge-Decision request is `review_request`. When unsure, `question` is always
accepted.

## 9. Topology is invariant — including the query edges

Who hands off to whom, who reviews, who merges, and **which cross-team query
edges exist** is operator-owned and fixed. The sanctioned edges are exactly:

- any team → **Spec** leader — behavioral-contract questions ("what must this do
  to be correct?").
- any team → **Architect** — component-design questions ("how should I structure
  this / which design?").
- any team → **Steward** — scope/priority (forwarded to the operator),
  workflow/process, and research requests.
- any team → **Steward** — merge status or publisher-path workflow questions.
- any team → **Steward** — `git_request`: a ready WP branch for publisher-path
  merge. The Steward is the **sole merge router**; the Integrator role is retired
  (2026-07-09, operator directive). Route all `git_request`s to the Steward.

**There is NO enclave → build-team edge — the enclave elaborates autonomously
(operator, 2026-07-03).** The clean-room enclave (spec-leader / spec-author / CV)
is the authority for `/spec` + `/conformance` elaboration; it does **not** pull a
build-team member (a team's leader / implementer / QA) into an elaboration
thread. When elaboration needs a **fact about landed code** — "is this heap tag /
symbol / primitive referenced anywhere?" — the enclave **greps the code itself**
(grounding against landed code is standard enclave practice, §7), never asks a
build lead. When it needs a cross-cutting **judgment** call, it routes to
**Architect** (the `any → Architect` edge — soundness/means span lanes), never to
a build-team leader. *Live catch (Map elaboration): spec-author @mentioned
runtime-leader to confirm a heap-tag (`0x07`/`0x08`) retirement was inert — a
grep-able fact (`canonical.rs:23-24`, zero refs), and the coupled soundness call
was Architect's. Self-ground or route-to-Architect; don't cross into the build
lane.*

Agents may improve *what they do inside a node*, never *add a communication edge
or a review cycle* between nodes. When integrating a retro lesson, reject any
carry-forward that would add/move an edge — and do not soften the rejection to
"candidate, watch one more run." That softening is how coordination entropy
creeps in.

**The invariant is traffic, not just edges — trust the simple flow.** The base
flow is deliberately thin, and it *works*: **spec** — Steward frames →
spec-leader assembles → spec-author writes → CV (Spec vote) + Architect
(soundness vote), one pass each → publisher path merges. **build** — Steward
frames → leader frames the team WP → implementer builds → QA verifies →
Architect (soundness) + CV (conformance) → publisher path merges. Two reviewers
per WP, one
pass each; a mid-WP fork goes to the **one** owner of its lane
(soundness → Architect, conformance → CV, scope/process → Steward) who rules —
others do **not** pile onto the thread. Coordination entropy creeps in *below*
the edge level: **more parties cc'd on a thread, verbatim relays, "flagging in
parallel," "cross-checking with," a committee where one decider suffices,
pre-confirming what a gate will already check.** None of these adds a new
*edge*, so the edge-filter above misses them — but each multiplies tokens on
**every future WP**, and the enclave is serial, so the cost compounds. Treat
added *traffic* exactly like an added edge: it is **very expensive** (paid
per-WP, forever) and it is the **operator's to sanction** — never a retro's to
add. Default: **route to one, trust the gate, don't convene the room.** When in
doubt, the thinner flow is the right one.

### 9a. Assign spillover work; never negotiate it (operator-directed)

A sound change often forces **spillover** work in another team's file — a
companion proof-term migration, a downstream fixup that must land in the **same
PR**. **That work attaches to the WP-owner by default: whoever owns the branch's
PR/Decision assigns it, unilaterally, in one message.** Cross-team
file-familiarity is an *input* the owner may weigh, never a competing claim to
route around them.

Do **not** assign spillover work by **offer-form** ("you take it" / "no, you
take it"). Offer-form between two leaders is bilaterally unstable — each reply
answers the other's *prior* message, so they cross and ping-pong (a companion
migration once flipped ownership four times in one minute; it has recurred). A
fixed assigner has no cross-wire: one message settles it.

**Silence = assent on a settled handoff.** Only the assigner posts the
assignment; the assignee acks **once**; downstream nodes do **not** each
re-confirm, and no one relays a "you look stale" fan-in — a single correction to
the assigner suffices. When your own assertion turns out stale, **retract and
defer** in one message: a clean withdrawal ends a ping-pong; a second asserted
correction extends it.

This is a fixed *assignment* rule, not a new edge — it **removes** edges (the
whole negotiation cluster) rather than adding one.

## 10. Knowledge promotion: retro → synthesis → promotion ladder

- **The retro is a mandatory step, not an afterthought.** A work package is not
  *done* until its retro is in. The moment a WP's work is verified/merged, every
  working agent in the ring (implementer, QA; spec-author,
  conformance-validator) posts a short **`retro`** in the WP's thread — three
  bullets: **trap** (what cost time, or a defect the process caught or missed),
  **held** (a discipline that worked, with its prior-run validation count if it
  has one), and **carry** (a candidate rule to promote). Tag each bullet
  node-internal or topology-touching, so the Steward's invariance filter (§9) is
  pre-sorted.
- **The leader collects and hands off.** When a WP merges, the team leader
  confirms each working agent's retro landed, adds a one-bullet coordination
  retro, and posts a `retro`-typed "retros in" to the Steward with the WP ID and
  pointers. 15-min timeout: hand off what is in and name who is missing. This
  rides the existing team→Steward workflow edge (§9) — it adds no new edge.
- The **Steward** harvests retros across teams and promotes lessons up a ladder
  (see the steward playbook): team-local → archetype source → this file.
- A lesson promotes only when it passes all three: **(a) validated across ≥3
  runs *or* independently in ≥2 teams, (b) effort-/model-/operator-agnostic, (c)
  a normative rule, not a one-off fact.** Exception: an explicit operator
  correction promotes on a single data point. On promotion, retire the source
  note atomically. Cross-team replication is a *stronger* generalization signal
  than single-team repetition — use it.
- **(d) The ratchet guard — retros only ever *add*.** No retro proposes
  *removing* a hop; each says "also loop in X," "relay verbatim so nothing's
  lost," "cross-check Y in parallel." Every one is locally sensible and
  collectively ruinous — absent a hard default the communication topology
  monotonically complexifies. So any carry that adds a communication **party,
  relay, gate, or confirm-hop** is **topology-touching (§9)**: it does **not**
  promote on validation alone — it needs **explicit operator consent**, exactly
  like a new edge. Prefer the node-internal form every time: sharpen *what* an
  existing reviewer checks, never *who else* gets looped in. The bias is toward
  the **thinner** flow; simplifying it back is the Steward's standing mandate,
  complicating it is the operator's call alone.

## 11. Cross-team query protocol

The edges in §9 are thin synchronous couplings between otherwise-parallel rings.
Use them sparingly and always event-driven:

1. **Filter first (§6).** Most "what should I do here" answers are already in
   `/spec` + conformance + the component design. Only a genuine gap or fork
   earns a query.
2. **Ask and stop.** Post a `question` mentioning **only** the target's leader
   (Spec leader / Architect / Steward), set status `blocked-on-<target>`, and
   stop. Resume on notification — never poll.
3. **Bias to staying on-task.** Your team's default is to *wait out* a short
   block, preserving ring coherence; your leader reorders to an independent
   ready task only when the block is genuinely long.
4. **Front-desk on the answering side.** The target's leader triages to protect
   its own ring's focus — answers trivial/known questions itself, batches
   non-urgent ones, interrupts its active agent only for true blockers.
5. **Outcomes:** a quick interpretive answer; a **durable artifact edit** (a
   `/spec` clarification + conformance test, or a component-design note) so the
   next team never asks again; or, for a real fork, a **Decision**. Every query
   should leave the shared artifacts better — the query rate is a health gauge,
   and it should decay over time.

## 12. Resource discipline (shared 8-core / 16 GB laptop)

Build parallelism multiplies with agent parallelism; the dev box is small.
Violating this OOMs the machine and stalls everyone. Full rationale +
configuration: `../docs/ops/compute-budget.md`.

- **Build and test only through `scripts/ken-cargo`** — never raw `cargo build`/
  `cargo test`. It holds a machine-wide lock (`KEN_BUILD_SLOTS`, default 1) so
  only one build runs at a time across all agents. Bypassing it is the fastest
  way to swap-death the box.
- **⛔ Scope to the touched crate** (`-p <crate>`, or `--test <name>` for one
  suite), **NEVER `--workspace` (operator hard rule).** Full-workspace builds,
  the `--locked` gate, the conformance suite, and any `--release`/LTO build run
  **in CI on GitHub**, not on the laptop — a local `--workspace` run is what OOMs
  the box and stalls everyone. Lean on CI green (the publisher path polls those
  exact checks before merging), don't reproduce it locally. **Every local agent
  is responsible only for the affected areas** — implementer, QA, leader, enclave,
  Steward alike. A WP frame's "no-regression" / "workspace-green" acceptance
  criterion therefore means **green in CI**, *never* a local `cargo test
  --workspace`; authors write frame ACs that way and readers execute them that
  way. (Operator, 2026-07-13 — a kenfmt implementer burned ~1h on a local locked
  workspace run the frame wrongly mandated; the venue is CI, full stop.)
- **`source scripts/ken-env.sh`** at session start for the shared `sccache` +
  `CARGO_HOME`, so you don't recompile dependencies other agents already built.
- **Idle = paused.** A resident agent costs RAM even when not building. If your
  ring is blocked or waiting (including waiting on a CI run — ADR 0002),
  quiesce; don't hold the box hot.
- This is a *current-hardware* constraint, not a design value — it relaxes as
  hardware grows (the Steward/operator raises the caps; do not raise them
  unilaterally).

### 12a. ⛔ NEVER `git stash` — the stash stack is SHARED across every worktree

**Binding on every seat, no exceptions** (Steward, 2026-07-22, after a live
near-miss).

`git stash` stores entries in `refs/stash` on the **shared repository**, not
per-worktree. The fleet runs **~70 agent worktrees over one clone**, so every
seat sees and mutates **one stack**. A bare `git stash pop` from *any* worktree
takes `stash@{0}` — **whichever agent parked something last**, which is almost
never you. Observed live: **12 entries from at least four different agents.**

**The near-miss.** A build implementer ran `git stash pop` mid-slice and nearly
consumed another team's parked diagnostic work. It was saved **only because the
apply hit a merge conflict** — git retains the entry on conflict rather than
dropping it. **On a clean apply it would have silently destroyed another
agent's work**, and the owner would have found an empty stack with no error, no
log, and no way to learn who took it. **This is the failure class no individual
seat can detect**, which is why it is fleet law and not a team retro item.

**Use one of these instead — all per-branch, so they cannot collide:**

- **Commit it.** A WP commit on your own `<role>/work` branch is the normal
  move, and is what the handoff gate expects anyway.
- **`git worktree add`** a scratch worktree for the experiment.
- If you genuinely must stash: **`git stash push -m "<role>: <what>"`**, and
  thereafter **only `git stash apply stash@{n}`** on an entry whose message you
  wrote yourself. **Never `pop`. Never a bare index.** Prefer committing.

**⚠ Do not reap the existing stack.** Those entries belong to other seats.

> Same shared-substrate family as the **single object store** (a commit that
> verifies locally may never have been pushed — §14) and the **shared `/tmp`**
> (at 99% full it silently dropped a git write). **Worktrees look isolated and
> are not** — before assuming any git state is yours alone, ask whether the
> underlying ref lives in the clone or in the worktree.

## 13. Liveness: keep the rings turning

Token rings stall — an agent finishes, forgets to hand off, and the ring goes
quiet. Treat stalls as the **default** failure mode, defended in depth by three
recurring watchdogs, each catching the layer below it failing:

- **Team leader → its own ring.** Enumerated patterns: handed-off-but-silent,
  merge-Decision-open-no-reviewer, blocked-without-a-blocker-mention,
  idle-with-ready-work.
- **Steward → the merge pipeline.** Branch-published-CI-pending-too-long,
  CI-green-but-Decision-unresolved, Decision-approved-but-CI-red,
  approved-and-green-but-unmerged.
- **Steward → the federation (the backstop).** A whole team idle, a *stalled
  leader*, a dropped cross-team query, a blocked dependency chain, no movement
  toward the active gate. The watcher-of-watchers — it catches a watchdog that
  itself stalled.

Rules for every layer:
- **Enumerate the stall patterns explicitly** in the watchdog prompt — a generic
  "check for activity" misses the nuance.
- **Diagnose before you restart.** Capture the stalled agent's state first; a
  blind restart no-ops a permission-prompt or rate-limit stall.
- **Distinguish waiting from stalling.** A team idle while its CI run is *in
  progress* is normal (ADR 0002), not a stall — leave it alone. Recover only
  when CI has *finished* and no one took the next step (open the merge Decision,
  vote it, fix red, merge).
- **Graduated recovery:** detect → mention the one blocked agent → re-mention
  next interval → escalate up the chain.
- **Escalation chain:** member → team leader → Steward → the operator. The buck
  stops at the operator (human): if the Steward goes quiet, the absence of its
  updates is the operator's signal. Watchdogs are the only schedulers (§1);
  everyone else is event-driven.
- **Arm your watchdog with the convo-channel `schedule_create` self-wake — NOT
  the convo `schedule_call`, and NOT a hand-rolled wake script** (operator
  2026-07-20; supersedes the earlier `CronCreate`-only guidance). `schedule_create`
  is the **one sanctioned, provider-agnostic** watchdog mechanism — it works
  identically on Claude-Code **and** terra/Codex seats, so the whole fleet
  converges on a single command. A scheduler (team leader, Steward) sets up its
  recurring pass at **session start, while its ring/pipeline has open work** — e.g.
  `schedule_create(interval_seconds=900, label="steward-watchdog",
  prompt="[watchdog tick] read get_recent_context, sweep the active panes, scan
  the enumerated stall patterns, mention only a blocked agent; if clear, do
  nothing")` (`interval_seconds` is the recurring "set_interval"; `cron="…"` for a
  5-field expression, `delay_seconds` for a one-shot). It **posts nothing to the
  space** — it delivers the prompt privately into your *own* session (a Claude-Code
  channel push when possible, else a guarded tmux `send-keys` that **skips a tick
  rather than overtyping a human's not-yet-submitted input**). On each fire you run
  your *own* direct `get_recent_context` / `get_space_status` read (private, not
  posted) and do the stall-pattern assessment + recovery above, **messaging the
  space only when there is an actual stall to nudge.** It returns a `schedule_id`;
  **`schedule_delete(schedule_id)` disarms it** ("clear_interval") and
  `schedule_list` shows the schedules you own. A leader may arm a **teammate's**
  pane by passing an explicit `target`.
  **Do NOT use the convo `schedule_call`** for a watchdog: it executes the read
  *on the backend* and posts the result back into the space as a **System event
  visible to every participant** — pure broadcast noise (and the
  `get_recent_context` variant reads its own prior fires and recursively nests
  them — an exponential self-feeding loop the Architect + runtime-leader caught).
  A watchdog is a *private* wake, not a public post; `schedule_create` is private,
  `schedule_call` is not. Likewise **do NOT** hand-roll a bash `while`-loop, the
  `Monitor` tool (git-refs only — blind to the pane-level stalls), or the interim
  `local/steward-watchdog-wake.sh` external tick script — that script was the
  terra-seat stopgap **before** `schedule_create` existed and is now superseded.
  **A scheduler that never arms its watchdog catches nothing** — the operator
  caught exactly this (a QA-approved WP left unmerged because the leader wasn't
  watching).
- **`schedule_create` schedules are process-local, which is mostly a feature —
  they cannot orphan across a restart — but they carry ONE regression you must
  defend against: they live only for the convo-channel MCP server process's
  lifetime and do NOT survive an MCP reconnect** (a package upgrade, a network
  blip, or a compaction that re-instantiates the client all drop them silently —
  and posting/notification can stay up while they're gone). So **re-arm on session
  start, after every compaction, AND after any convo-MCP reconnect**, and
  **reconcile with `schedule_list` at the top of each tick** — if the list is
  empty while your ring/WP is open, your backstop silently fell over; re-arm it.
  **`schedule_delete` it when your ring/WP closes.** (On a Claude-Code seat the
  host-level `CronCreate`/`CronDelete` remains available and *does* survive an MCP
  reconnect; it is a valid durable fallback for that seat, but default to
  `schedule_create` for fleet uniformity.) **If you still hold a convo
  `schedule_call` timer from the old guidance, `cancel_call` it and re-arm with
  `schedule_create`.**

## 14. Agents never touch GitHub; the publisher path is the gateway

**Only the publisher path has GitHub credentials.** Build/spec agents do
**local git only** in their worktrees (commit, rebase onto the already-fetched
`origin/main`) — no `gh`, no push, no fetch, no token, no PR. Under explicit
operator direction, the Steward runs the checked-in scripted publisher path
(`scripts/scripted-pr-automerge.sh`) with only these inputs: an exact approved
branch/SHA, public PR title, public PR body, and the docs-only flag. This keeps
one GitHub identity; it does not give teams GitHub access, does not make the
Steward a code author, and does not replace the mootup review/Decision record
(`../docs/program/04-git-and-integration.md`).

### 14a. ⛔ THE ARCHITECT DOES NOT VOTE ON DOC-ONLY WPs

**Operator ruling, 2026-07-22:** *"architect does not need to rule on docs."*

**A WP whose diff is confined to `library/` merges on its QA approval plus the
diff-scope check.** No Architect vote. The Steward resolves the merge Decision
and publishes.

**The Architect's vote is still required** when a documentation change touches
**`agent/` law**, makes a **normative claim about the language** (`spec/` is the
sole authority — `library/` is explanatory/derived), or reaches outside
`library/` into `crates/`, `spec/`, `conformance/`, or `catalog/`.

> **Why this is a structural fix, not a convenience.** The doc track runs
> **concurrently** with the build track on the basis that it is
> **path-disjoint** — `library/` and `agent/` versus `crates/`. But routing
> every doc WP's merge Decision through the Architect **re-couples the two
> tracks at the review seat**, which reintroduces exactly the serialization the
> concurrency exception exists to avoid. **The paths do not contend; the
> reviewer does.**
>
> This surfaced live: DOC-W1-1 was QA-approved 34 minutes after kickoff and
> then sat unpublished behind an Architect who was working through seven
> consecutive build-track frame reviews. The operator noticed the absence of
> `library/` commits on `main` before the Steward escalated it — **a queue is
> invisible from the outside, and "the ring is running" and "work is landing"
> are different claims.**

**Steward duty:** when a track is nominally concurrent, check that its *review
and merge path* is disjoint too — not just its file paths. **Concurrency that
funnels into one reviewer is sequencing with extra steps.**

The publisher path pushes `wp/<ID>` branches to trigger CI, reads checks,
merges, fetches `main`, and mirrors GitHub state into mootup. If GitHub reports
that a separate review or branch-protection change is required, the script must
stop and route that fact; it must not pretend a same-identity PR review can
satisfy the gate.

- **Agents get no GitHub notifications and never poll GitHub.** Because GitHub
  is the publisher path's concern, every actionable signal reaches the fleet
  only as a **mootup message that mentions the actor whose move it is.** Act on
  the mention, not on a timer.
- **CI is the publisher path's to watch — never a worker's.** It reads check
  status for the branches it published (`gh pr checks` / the checks API) and
  posts the outcome: red → mention the implementer (team space) with the failing
  job; green → advance toward merge. The optional `ken-ci` bridge mirrors
  `check_suite` results automatically; until then the publisher caller posts
  them. After you hand a WP off, you **stop** and learn a red result from a
  mention.
- **Landing integrity: a merge isn't trusted until *verified on `main`* — and a
  multi-piece WP isn't landed until *every* piece is (promoted V1, ★★★
  near-miss).** A **squash-merge can silently drop pieces** of an assembly built
  from multiple cherry-picks: the K2c-s2 seam-3 erratum shipped its **kernel**
  fix in `ecbb279` but **dropped the spec (`da344a6`) + conformance (`f3ece75`)**
  pieces, leaving `16 §5.1` normatively **contradicting its own kernel** and the
  corpus guarding nothing — and it **survived the ship**, invisible in every
  "shipped" status/notification. Caught only by **grounding against the landed
  files**. So: (1) **don't trust a "shipped `<sha>`" notification or a status
  line** that a thing landed — for anything load-bearing, confirm it on `origin/
  main` (the files: `git grep`/`git show`). (2) **An approved N-piece erratum/WP
  is not "landed" until you verify each piece on `main`** (the Architect's
  3-piece-on-main gate) — the leader who assembles a multi-cherry-pick branch
  checks the post-merge `main` carries all of them. (3) **Authors ground
  the WP *base* against the landed corpus, not the notifications,** before
  building on it.
  **(4) PREVENTIVE — a multi-piece erratum is ONE branch, ONE Decision (promoted
  Σ-sort, the *2nd* recurrence).** The §5.1 drop recurred **immediately** on the
  Σ-sort 3-piece (`badc78d` shipped only the kernel `sort_sigma` split; spec `13`
  + the pi-sigma conformance were absent from `main`) — so points (1)–(3) are the
  *detective* control; this is the *preventive* one. **Root cause:** when one
  piece is a **crates-only kernel fix**, its diff-scope reads crates-only
  (Architect + CI, **no Spec**) and it merges **alone** — the spec + conformance
  pieces, sitting on *other* branches, are never in the merged tree. It isn't the
  squash dropping commits; the pieces were **never assembled onto the branch that
  merged**. **Fix:** assemble **all** N pieces (kernel + spec + conformance) onto
  a **single** `wp/<erratum>` branch *before* the merge Decision — the combined
  diff-scope then **touches `spec/`+`conformance/`**, correctly pulls a **Spec
  vote**, and all N commits ride **one squash** to `main` together. Never publish
  the kernel piece on its own crates-only branch while siblings wait. **The
  publisher caller confirms the Decision's branch carries every cited piece
  before merge** (validated on `s51-sigma-reland`: one branch, both
  `spec/`+`conformance/` pieces → the Decision correctly pulled a Spec vote →
  "§14 compliant").
  **(5) Single-branch is necessary but NOT sufficient — verify an assembled tip on
  TWO axes, against CURRENT `main`, right before the Decision (promoted X1-effects-
  elab; both enclave authors + the Architect converged on it).** One branch
  guarantees the pieces are *together in one diff*; it does **not** guarantee they
  are **complete** or that the **base is current**. Two faults nearly merged a wrong
  tree on X1-effects-elab: (a) the assembler took the author's *§6-body* commit but
  **not their branch tip**, dropping a follow-up (`a3b887e`) that fixed an
  **actively-wrong** stale pointer; (b) the re-fold sat on a **stale base** that, as
  `main` advanced, would have **reverted V2-build + the V3 frame**. So before the
  merge Decision, re-run (right then — *"rebased onto current main" is a perishable
  claim*): **content** — `git diff <author-full-tip>:<file> <assembled>:<file>`
  **EMPTY**, taking **all** the author's commits since merge-base (a dropped
  follow-up is invisible if you only diff the section you read); **base/scope** —
  `git diff --stat origin/main <assembled>` is **only** the WP's intended files and
  **the WP's deps are ancestors of the tip** (a stale base silently reverts
  unrelated landed siblings). A correct-content tip on a stale base is as
  unmergeable as the reverse — the base axis is a distinct failure mode.
  **Lane note:** the independent checker (conformance-validator) is *stronger
  verifying the assembler's tip against these gates than performing the rebase
  itself* — when grounding surfaces an assembly hazard, **flag it to the
  assembler**, don't reach past your lane into the git (where the X1-effects-elab
  drops happened).
  **(6) TEMPORAL — a fold/erratum that races an in-flight merge must HOLD the
  merge or be an erratum-on-current-`main` from the start (promoted Sec1; a
  *race*, not an assembly fault).** Points (1)–(5) assume the pieces sit on
  branches you control at Decision time; this one is about **timing**. On Sec1
  the Architect approved `61` with an N1/N2 honesty fold flagged "fold before
  build, not a merge-blocker." The author committed the fold declaring it
  "supersedes `b3e7989`," the coordinator had already rebased the **pre-fold**
  tip + posted merge_ready, and the publisher path squashed the pre-fold spec to
  `main` **9 seconds before** the fold landed — so the honesty chapter hit `main`
  carrying the exact §9 over-claim the fold fixed; netted only by verify-on-main,
  re-landed in minutes. **A "supersedes X" line interlocks nothing, and
  decision-time "hasn't merged yet" is perishable within seconds.** So: **author
  side** — never "fold + declare supersede" against a WP whose votes/merge are in
  flight; either mention the leader to **HOLD the merge** *before* committing
  the fold, or author it as a fresh erratum-on-current-`main`.
  **Coordinator side** — "not a merge blocker" ≠ "merge *before* it lands"; when
  a fold is in-flight and the merge is seconds away, **briefly HOLD** the merge
  so the chapter is correct at first landing (the coordinator can prevent the
  race the author cannot gate). The only reliable net remains **verify-on-main
  after**, never a declaration before.
- **Review is a mootup Decision — and a *soundness vote is a frontier-class
  responsibility* (promoted Sec1ct/B1 + operator).** The Architect/Spec read the
  diff from the shared local branch (`git diff origin/main...wp/<ID>`) and vote
  the merge Decision in mootup; there is no GitHub PR approval to mirror. **Two
  load-bearing reviewers, both Opus-tier:** the **Architect** (external
  soundness/design, always) and the **Spec** vote on `spec/`+`conformance/`
  paths — cast by the **frontier-class (Opus) member of team Spec**, currently
  the **conformance-validator** (the independent checker by role; spec-author
  authors, so cannot self-review). The **DeepSeek coordinator (spec-leader, a
  build leader) assembles the Decision but does NOT cast the soundness
  vote** — soundness judgment needs the strongest model (MODELS.md tiering, made
  explicit for review routing).
- **The publisher path merges only on a *resolved* Decision, verified fresh —
  never on a `merge_ready` post's prose (promoted Sec1ct breach; fixed +
  validated 2/2).** A `merge_ready` is a *request*; the **resolved Decision with
  recorded approvals** is the authorization. Before `gh pr merge`, the publisher
  caller re-reads the Decision and confirms `status: resolved` (not `proposed`)
  with the Architect's (and Spec's, on spec paths) votes in — it never infers
  approval from a reviewer *named in prose* (the Sec1ct merge skipped the gate
  exactly this way: "`(Architect + Spec)`" read as approval while the Architect
  never voted). Symmetrically, whoever posts `merge_ready` states
  `Decision: dec_XXX — status: resolved` (or `proposed — awaiting <reviewer>`)
  and fires **real @mentions** to reviewers' actor_ids (§2 live-participant),
  not prose names.
- **The publisher caller mirrors each GitHub state change into mootup mentioning
  whoever moves next** — CI red → the implementer; merged → Steward. A GitHub
  state change nobody mirrors is a silent stall.
- The full event→message map (what, where, mentioning whom, posted by whom) is
  in `../docs/program/04-git-and-integration.md §5`.

## 15. Context compaction is the Steward's (teams) or self (singletons)

Token efficiency: an agent should start each work package with a clean, minimal
context. Who triggers a compaction is fixed (operator, 2026-06-29):

- **Teams are compacted by the Steward**, not by their own leader. The Steward
  `moot compact`s the **whole team** (leader + implementer + QA, or spec-leader +
  spec-author + conformance-validator) **before delivering a WP** to the leader.
  Leaders never `moot compact` anyone — `request_context_reset` is self-only, so
  only the Steward can compact another agent (`moot compact`), and it does so for
  teams alone.
- **Gated by retros.** The Steward compacts a team **only after** its prior WP's
  retros are posted (compaction would otherwise summarize the retro away), and
  delivers the next WP **only after** compacting. So a team's WP boundary is:
  done → leader calls for retros in-thread → members post → leader signals the
  Steward "retros in" → Steward reviews → Steward compacts → next WP.
- **Singletons self-compact.** Agents with no team/leader — **Steward,
  Architect, Librarian, Research, Adversary** — self-compact at their own task
  boundaries (Architect after a review, Librarian after a pass, Adversary after a
  hunt or a triage handoff, Steward after a directing cycle) via
  the `tmux send-keys -t moot-<role> -l '/compact'` path (two-step: `/compact`,
  ~2s, then a **separate** `Enter`). To auto-continue afterward — a self-compact
  otherwise leaves the seat idle at `❯` with nothing to re-invoke it — launch
  the **detached resume watcher** `scripts/postcompact-resume.sh moot-<role>`
  (`nohup … & disown`) **before** sending `/compact`: it outlives the turn and
  the compaction, waits for the `Compacting…` window to appear and clear, then
  sends `resume`. Do **not** instead type `resume` right after `/compact` and
  rely on host buffering — that races the still-active turn and can fire the
  resume *before* compaction (the reason the watcher exists). Do **not** use
  `request_context_reset` — it is broken in this harness (it looks for a
  nonexistent `convo-<role>` session and its error message *names* `convo-<role>`,
  which is the bug, not a target to retry). Full mechanics:
  `playbooks/federation/steward.md` (self-compact).
- **Never mid-reasoning.** Compact only at a clean boundary; it summarizes away
  in-flight work.
- **Start new work from current `origin/main` (operator, 2026-06-29).** A WP
  branch is born from the **fetched** ref — leaders cut `wp/<ID>` with
  `git branch wp/<ID>-<slug> origin/main`, and every member `git rebase
  origin/main` before working. Never build on stale local `main` or a stale
  worktree (the §1 / 04-git worktree-mismatch trap). New work = latest `main`.
- **On resume after a compaction, ground-truth your state before trusting the
  summary.** A compaction can summarize away the fact that you *finished* — so
  re-orient from reality, not the lossy summary: `orientation()` **plus**
  `git reflog -10`, `git status`, `git branch -vv`, **plus check unread
  mentions** (`orientation`'s unread count / `get_mentions`). Your pre-compact
  self's commits and checkouts are in the reflog; check there before concluding
  you (or a teammate) are "stalled" or re-doing delivered work. **Notification
  delivery is best-effort** — a mention that landed while your session was
  bouncing (e.g. a fleet restart) is recorded but may never have woken you, so a
  resume is when you catch it. (F4 + K1, operator-observed — promoted here
  because the Steward-compacts-every-WP rule makes post-compact resume a constant
  for every team member.)
