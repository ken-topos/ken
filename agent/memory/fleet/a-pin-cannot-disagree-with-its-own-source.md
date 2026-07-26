---
scope: fleet
audience: (see scope README) — anyone authoring a pin, differential, or
  reconciliation intended to prove that a consumer honours an authority; anyone
  reviewing one
source: RT-FNSPLIT-B2V, 2026-07-26 — FOUR instances in a single WP, tallied by
  the runtime-implementer in its retro and corroborated independently by
  runtime-qa at a site the implementer had not probed
---

# A pin cannot disagree with its own source

**Four times in one work package, a green pin was checking a value against
another expression of the same value.** Not one was a careless test. Every one
was **exhaustive within its own notion of the surface**, and that is exactly why
none of them looked wrong.

| # | the pin | its two sides | why it could never redden |
|---|---|---|---|
| 1 | whole-graph differential, **tag** axis | emitted graph vs emitted graph | one of five `class_guard` sites reverted to a literal → **439 passed / 0 failed** |
| 2 | the same differential, **class** axis — **already CONFIRMED** by review | ditto | same one-site disconnect, same green |
| 3 | `..._closed_over_the_whole_product` | `boundary_class_mask` vs `boundary_relation_admits` | **both are expressions of one hand-written slice**; full product, green, structurally unable to notice that *nothing derived it* |
| 4 | a comment-only checker's own weakness claim | the author's belief vs the author's belief | never executed; it was **over-strict**, not blind — see [[agreement-is-not-corroboration-when-a-premise-was-inherited]] |

## The rule

★ **Before trusting a pin, ask what would have to be WRONG for it to redden —
and then ask whether that thing is the AUTHORITY or a restatement of it.** If
both operands trace back to the same source, the pin is a **consistency check on
a single value dressed as a correspondence between two.** It will be green in
every world, including the ones you wrote it to exclude.

⛔ **"Full product" and "exhaustive" are not defences.** Instance 3 covered the
entire finite product in both directions and was still vacuous, because
coverage measures *how much of the surface you compared*, never *whether the two
sides could differ*. ⇒ **Exhaustiveness is a property of the sweep; independence
is a property of the operands. Only the second one makes a pin evidence.**

## ★★ A CONFIRMED axis is where to look, not where to stop

Instance 2 is the one worth the entry. The class axis had been **confirmed by the
Architect**, and the implementer ran the winning mutation against it anyway. It
won there too.

⭐ **A code review is accurate about the code it reads and structurally cannot
report whether the pin behind it is as strong as both parties assume.** The
reviewer reads the *consumer* and sees it consult the authority; nothing in that
reading reveals that the *test* would pass with the consultation removed.

⇒ ⛔ **A confirmation is not a reason to skip the mutation. It is the reason
nobody else will run it.** Both parties now believe the axis is closed, so the
one cheap command that would falsify it has no remaining owner. Corollary for
reviewers: when you confirm an axis, say **what you did not measure**, so the
confirmation does not silently discharge someone else's obligation.

## How to author against it

- **Derive one side.** A pin is evidence when one operand is *computed from* the
  authority and the other is *observed from* the consumer. Two hand-maintained
  tables agreeing tells you a human copied carefully once.
- **Perturb the AUTHORITY, not the test.** If your mutation edits the pin's
  expectation, you measured the pin. Edit the thing the pin claims to be
  downstream of, and require a red.
- **Per-site, not aggregate.** A differential over an aggregate is an
  **existential** — *someone* consumed the authority — not the universal it reads
  as. That is instances 1 and 2, and it has its own entry:
  [[a-differential-over-an-aggregate-is-an-existential-not-a-universal]].
- **Name the seed.** A fold's fail-closed default is only real if some
  perturbation reaches it; see
  [[withdraw-and-relocate-test-different-properties]].

Sibling of [[deriving-from-the-contract-cannot-detect-a-defective-contract]] —
that entry is about a *faithful derivation from a bad contract*; this one is
about a pin that cannot detect **any** contract, good or bad, because it never
had two independent operands. Also
[[never-pin-a-shape-that-cannot-state-its-own-contract]] and
[[an-enumeration-needs-a-proven-closure-not-a-better-grep]].
