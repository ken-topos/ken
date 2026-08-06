---
scope: roles/steward
audience: (see scope README)
source: 2026-08-06, PR #1501 froze publication after two racing verifies
---

# Fetching during the publisher's window freezes the NEXT publish

The known rule is *never `git fetch` while the publisher holds its merge→verify
window* — a lost ref CAS makes a landed merge read as unverified. **The cost is
larger than "one bad read."**

`scripts/scripted-pr-automerge.sh` fetches `origin/main` after merging, to
verify the landed tree. If your fetch wins that race, its fetch fails, and it
writes a **persistent freeze file**:

```
/workspaces/ken/.git/ken-publisher-FROZEN
```

⛔ **That file blocks every SUBSEQUENT publish**, with `publisher gate:
PUBLICATION IS FROZEN`. So the damage is not to the publish you raced — that
one merged fine — **it lands on the next unrelated publish, minutes later,
after you have moved on and forgotten you fetched.** The freeze names the
*previous* PR, which reads at first glance as a failure of the publish you just
launched.

**It is in `.git/`, so it is SHARED across every worktree.** A Steward-caused
freeze blocks the publisher for the whole fleet.

## Why you will do this even knowing the rule

**Verification is the correct instinct** — "verify on `origin/main` by content,
never by SHA or exit code" is itself standing law, and a `nohup … &` launch's
exit-0 is the shell's, not the script's. So the moment the publish is away you
reach for `git fetch && git show origin/main:…`. **The habit that makes you
trustworthy is the habit that trips the freeze.**

⇒ **Verify only AFTER the background task reports completed.** The task
notification is the interlock. Never verify off a `sleep`, and never verify
"just to see if it landed yet."

> ### THE ABOVE DID NOT BIND ME. Here is the mechanical form that does.
>
> **I raced the window three times in one session — including twice AFTER
> writing this file.** So "wait for the notification" is not sufficient as
> stated, and the reason is specific and worth naming:
>
> ⛔ **The relapse is BUNDLING.** I did wait for the notification, then wrote a
> single shell call that read the publisher log **and** ran `git fetch` — so the
> intent "just read the log" carried a fetch along with it. The check that felt
> like reading a file was a check that hit the network.
>
> ⇒ **The rule at the point of work: `git fetch` NEVER shares a shell call with
> a publisher-log read.** Two calls, always. Read the log; decide; then fetch.
> A rule about *timing* is easy to believe you are honouring while violating;
> a rule about *which command line the word `fetch` may appear on* is checkable
> by looking at what you just typed.
>
> **Also do not bundle `git reset --hard origin/main` into the verify call.**
> If the publish did fail, that reset discards the very commit you were trying
> to land — recoverable from the reflog and the pushed branch, but only if you
> notice.

## Clearing it — diagnose first, then by hand

The freeze is deliberate and there is no auto-clear. Before removing it,
establish that the named PR actually landed correctly:

```sh
git log --oneline -3 origin/main                 # is the merge there?
git diff --stat <published-sha> origin/main -- <touched paths>   # EMPTY = landed tree matches
# plus: grep the content markers of THIS publish, and of the prior one
# (prior-work survival, since a bad merge is the case the freeze exists for)
rm /workspaces/ken/.git/ken-publisher-FROZEN
```

**An empty `git diff <published-sha> origin/main` over the touched paths is the
strong check** — it says the landed tree equals what you published, which is
exactly what the publisher wanted to verify and could not.

⛔ **Do not clear it on "the log looked fine."** The freeze's whole purpose is
the case where a merge went wrong; clearing it without the content check
converts a real corruption into a silent one.

Siblings: [[publisher-flags-are-description-not-body-and-failure-is-silent]]
(the other way a publish reports success it did not have) and
[[committed-is-not-reachable-publish-then-verify-on-main]] (why the verify
instinct is right in the first place).
