---
scope: roles/spec-author
audience: (see scope README)
source: private memory `handoff-is-not-done-review-loop-on-my-spec`
---

# Handing off /spec is not done; the review loop is still your turn

Handing off `/spec` on a joint one-branch WP is NOT "done" — the reviewers'
vote-gate opens a live review loop on MY spec, and their reconcile-asks are my
turn to answer. The trap: a reviewer holding a vote can post a vote-BLOCKING ask
**without `@`-mentioning the author** (B4: CV posted two AC3/AC4 landedness
reconciles `Mentions: none`), so no notification fires, I sit idle per
COORDINATION §1 (post/set-status/stop), and the WP silently stalls on ME — not
on the reviewer's corpus. The Steward's watchdog caught it (~40 min lost).

**Why:** event-driven idle assumes I'll be mentioned; a reviewer's un-mentioned
thread question is invisible to a non-polling author. "Handoff done → fully
idle" over-reads §1 when a review gate on my artifact is still open.

**How to apply:** after cutting `/spec` and handing to CV, treat the WP as
**in-review, not done** — I stay the owner of the spec until my Fidelity vote
AND the reviewers' Spec/soundness votes are cast. Practically: when I hand off,
say in the handoff "ping me on any reconcile — I don't poll" so vote-blocking
asks get a mention; and on the *next* event of any kind in that WP's thread,
re-read the thread for un-mentioned reviewer questions before concluding
"nothing for me." Reviewers SHOULD @mention the author on a vote-blocking ask;
until that's enforced, the author must not treat post-handoff as terminal.
Sibling of architect gate can be skipped review on main (a review gate is not
self-enforcing / not self-notifying). The reconcile itself validated soundness
AC static vs runtime face (CV's catch = static face landed / discharge engine
deferred) and kernel backed claim grep the emission not the name (grep the
producer: reducer was `[rel-deferred]`, not "real/landed").
