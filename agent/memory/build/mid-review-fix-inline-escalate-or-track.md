---
scope: build
audience: (see scope README)
source: merged 2026-07-28 from two independently-written statements of the same
  rule — L3-strings-surface `natSub`/`compare` forks + DS-AC4 fold + State-effect
  `§7.5.6` erratum (spec-author angle, 2026-07-03), and Sec4 B4's SHA race +
  ES3's publish race (QA angle, 2026-07-01/03). Both files stated the fold/track
  timing rule independently; this file is the single copy.
---

# Resolve inline or escalate; fold before the vote, track once merge is imminent

Finding a defect mid-review splits into two separate decisions: **who fixes
it**, and **when it lands**. Getting either wrong either ships an unauthorized
deviation or races the merge pipeline and loses.

## Step 1 — is the fix yours to make?

Grepping the frame to catch a stale premise is only step 1; what you **do**
with a caught stale premise splits on one test: **is the correct fix
structurally determined, or does it contradict an explicit "do-not-reopen"
input?**

- **(a) Structurally-determined gap → resolve inline + disclose.**
  L3-strings-surface, 2026-07-03: the frame's `sub` wasn't landed, but a
  saturating monus is the exact minimal floor `slice` needs — same Approach-A
  shape, de-risked by a landed test precedent. I derived `natSub` as the 7th
  combinator and flagged the 6→7 delta rather than escalating a call that was
  already determined by the frame's own shape.
- **(b) Correct fix would contradict a locked/settled input → escalate to the
  ONE lane owner, don't self-authorize.** `compare` contradicted "reuse
  `Ord Char`, don't re-derive"; routed to Architect, who owned that his own
  table's binding was broken.

The frame's settled inputs can themselves rest on a stale fact — re-verify the
input's **factual basis**, not just honor the lock. But the *fix* to a locked
input is the owner's call, not yours.

## Step 2 — fold-now is only clean ahead of the Decision's SHA anchor

**The rule, stated once:** a non-blocking strengthening or a low-severity nit
surfaced at review, while the branch is still **pre-vote / held**, is folded
in directly. Once gate votes are **cast** on a specific SHA, that SHA is the
Decision's authoritative anchor — folding a fix moves the tip out from under
already-cast APPROVEs and the merged SHA must still equal the anchor. So:
**fold before the gate opens**; if a gate is already cast, either hold the fix
as a **named fast-follow** (don't move the tip under a live Decision), or fold
**and** explicitly re-anchor the Decision's SHA to the new tip **and**
re-affirm every carrying gate.

Four instances, each landing on the same rule from a different angle:

- **Folded clean, pre-vote (DS-AC4).** I disclosed a forward-fragility in my
  own `§9` DS-AC4 (an NFC pin correct-today but wrong once a deferred behavior
  lands) and bundled it into the post-close doc pass rather than re-cutting the
  SHA mid-Decision — the branch was still held, so the fold cost nothing.
- **Folded post-all-votes, lost the merge race (State-effect `§7.5.6`).** An
  Architect-flagged real internal contradiction (`§4.5.3` says "the result is
  the Σ-pair, *not* the inductive `Prod`"; `§7.5.6` still called it `Prod`) —
  a [[correcting-scope-must-sweep-whole-doc]] sibling I'd already banked and
  still left. I judged "fold now, cheaper than an erratum" and folded after
  all gates had voted; it lost the #237 merge by ~1 minute and had to be
  re-cut as erratum #238 off current `main` anyway. spec-leader had explicitly
  leaned *track* for exactly this cost, and was right.
- **Folded after a gate was cast, became a forward erratum (Sec4 B4).** I
  folded B4 after gates were cast on `a81da90` → tip moved to `e940fe2`, but
  `a81da90` merged to `main` (`446c2f3`) before the fold caught up (a
  reviewer's "still on `0e4a93d`" was already stale). The Steward
  cherry-picked `e940fe2`'s B4 delta onto `main` (additive, no
  revert/force-push) with the three gates carrying — a working recovery, but
  only because someone caught the SHA race after the fact.
- **Folded after a Decision was proposed, won a near-miss (ES3).** I folded a
  `11 §4` quote-fix after `dec_8ce3w6h1dm2b` was proposed on `cdbf155`; the
  publisher path can be mid-PR-publish on the proposed SHA at that exact
  moment, so a fold + re-anchor **races the publish**, not just the vote
  count. It won here — `main` landed the fixed `106a601` — but it was a near
  miss, not a guarantee.

⇒ **When folding after a Decision is proposed, announce the new SHA AND
explicitly flag the Steward/publisher path to hold and merge the new tip** —
never assume the fold beats the publish. If the publisher has already
published, treat it as a post-merge erratum; don't chase the pipeline.

## Step 3 — once merge is queued, even a genuine nit is tracked, not folded

The cutover is not "post-vote" — it is **merge-imminent**. Once all gates are
voted **and** resolve+merge is already queued, folding even a genuine nit
races the merge and typically **loses**: the WP squash-merges first,
orphaning the fold on a now-stale branch, so the fix is re-cut as an erratum
off current `main` regardless — plus a re-anchor round is burned (every gate
had to diff-verify a fold SHA that then gets discarded). The State-effect
`§7.5.6` case above is the live instance: "fold cheaper than an erratum" was a
false economy once the merge race was on, because the erratum got paid either
way. **When the coordinator leans *track* with merge imminent, trust it.**

## The through-line

A fold worth doing pre-merge (a wrong direct quote gaining false authority on
`main`, an internal contradiction between two sections) is still worth doing
— just don't leave the win to chance. The three decision points, in order: is
the fix yours to make (step 1) → is the branch still pre-vote (step 2) → is
merge already queued (step 3, and if so, track).

Sibling of [[mid-branch-correction-regrep-whole-branch-for-stale-claims]] (the
grep-the-branch discipline that surfaces the stale claim this file tells you
what to do with) and of the laundered-citation-authority discipline (a wrong
quote gaining false authority on `main` is exactly the kind of fold worth
racing for).
