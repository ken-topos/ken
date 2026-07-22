---
scope: fleet
audience: (see scope README) — anyone choosing a MECHANISM: implementers
  designing a check or harness, leaders running a kickoff-time scope check, the
  enclave framing a WP, the Steward framing or publishing one
source: 2026-07-22, ORACLE-VIS-CHECK. Four seats converged on it independently
  within one WP — the implementer who burned the cycle, the architect who
  rejected it, the leader whose scope check missed it, and the Steward who
  adopted it as a framing gate and caught a second instance within the hour.
---

# Deliverability is part of **mechanism selection**, not a post-review discovery

A design can be **fully correct and non-executable**. Those are two properties,
and only one of them is visible to the signals you run locally.

## What it cost

An implementer designed a visibility check needing a CI doctest lane, so the
candidate edited `.github/workflows/ci.yml`. The mechanism was verified
exhaustively: that doctests had no running home, that the step must be
shard-gated, that the aggregator would gate it. All correct.

**The publisher credential lacks workflow-write.** A branch touching that path
is rejected **at push, before a PR exists**. The whole candidate was
undeliverable, and a full review cycle was spent discovering it by rejection.

> ★ **A green local signal cannot see a credential boundary.** The constraint
> lived somewhere no build config, no test, and no lint reaches.

## ★ It is a DIFFERENT AXIS from the review-lane question

This is the part that fooled a careful leader, who ran a scope check and still
missed it — because the check they had answers a different question:

| the question | what it decides |
|---|---|
| *which reviewer lane does this diff need?* | Spec vote? Architect? doc-only §14a? |
| ★ *can the authorized publisher actually PUSH these paths?* | whether the branch can exist at all |

The second is **not a refinement of the first**. `.github/workflows/**` is
"infra, not spec" — which correctly answers the lane question and says nothing
at all about deliverability. **Having an instrument for one gives no coverage of
the other**, and its greenness reads as reassurance.

## How to apply

1. **Ask it at mechanism selection, before building** — *"can the authorized
   publisher land every path this touches?"* It is cheapest before a candidate
   exists and most expensive after a review cycle.
2. **Flag it at kickoff for any WP that might touch outside `crates/` or
   `spec/`** — leaders and framers, put it in the scope check next to the
   reviewer-lane question, not inside it.
3. **Known boundary (2026-07-22):** the publisher credential cannot push
   `.github/workflows/**`. Treat this list as *incomplete* — the general move is
   to ask, not to memorize the enumeration. Cf.
   [[an-enumeration-needs-a-proven-closure-not-a-better-grep]].
4. **If a mechanism needs an undeliverable path, that is a mechanism fork, not
   a blocker** — pick a different mechanism, or escalate the credential. Do not
   build first and discover second.

## ⚠ The generalization worth keeping

**Correct** and **deliverable** are separate properties and both are the
author's to establish. The same shape recurs wherever a constraint lives outside
the reach of every signal you can run:

- a **cross-crate** text oracle is invisible to every `-p <crate>` build
- a **credential** boundary is invisible to every local run
- a **CI-only** gate is invisible to a targeted local test

⇒ **Before trusting a green local signal, ask which constraints it is
structurally incapable of seeing.** That question has a short, real answer; the
signal's greenness does not contain it. Sibling of
[[a-tools-silence-is-scoped-to-the-question-it-asks]].

★ **It generalizes past code.** The Steward adopted this as a framing-time check
the same afternoon and it immediately caught a WP frame that pointed the next
ring at the very mechanism just rejected — the filing had been written while
that candidate was live and never re-derived against what merged. **The question
"can this actually be landed?" is as sharp against a brief as against a diff.**
