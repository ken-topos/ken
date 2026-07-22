# When handed a claim to refute, rank sub-claims by LOAD-BEARING, not by checkability

**Lesson (ABI-REVOKE, 2026-07-22 — I got this wrong).** The Steward filed a
scoping issue and explicitly asked me to refute it. I refuted §2 and
**confirmed §3.** §3 was false, and the disproof was a single `git grep` I never
ran.

## The shape

§3 was a **table with a spec column and a code column**: *"`36 §4` space (what
revocation needs) vs `44 §3` arena unit (what is landed) — two different
concepts sharing one word."*

I verified the **code** column — read `store.rs:198`, enumerated its fields,
confirmed it has no validity cell — and experienced that as having verified the
table. The **spec** column was the load-bearing half, and I treated it as given
because it arrived quoted and cited. One command settles it:

```
$ git grep -n "36 §4" spec/40-runtime/
44-capacity.md:34   "Each surface `space` (../30-surface/36 §4) is realized at runtime as a ..."
44-capacity.md:294  "Each surface `space` (36 §4) realizes as a store `Space`"
  ... 6 hits across 41/42/44
```

The spec links the two names **six times**, in the chapter that owns the
realization. One abstraction with a landed memory projection and a missing
authority projection — a far cheaper claim than "the prerequisite does not
exist."

## The two errors, separately

1. **I ranked sub-claims by checkability.** "Does this struct have a validity
   cell?" is decidable in one read, so I did it. "Does the spec link these two
   names?" is what the whole section rests on — and was *equally* one command
   away. **Tractability is not importance.** Before attacking, list what the
   claim needs to be true and attack the load-bearing one first, even when it is
   the one framed as settled.
2. **I produced a confirmation while designated to attack.** I attacked the
   sub-claim the author had already framed as *the evidence*, rather than asking
   what the claim needed. **Agreement across reviewers is not independence when
   both inherited the framing from the same document** — and a red-teamer who
   re-checks the author's own exhibit has run the author's test, not a new one.

## Why the existing lesson did not fire

I already had [[differential-oracle-is-blind-to-a-shared-premise]] — a starred
lesson stating exactly this. It did not fire, because it was **indexed to the
artifact type it was learned on** (test oracles / parity harnesses) and this
arrived as a **scoping document**. A lesson filed against a venue does not
trigger when the same shape wears different clothes.

**So: index a lesson by its SHAPE, not by where you met it.** When writing one
up, ask "what else has this shape?" and name at least one venue different from
the one that taught it. The shape here is *two parties checking the same exhibit
and calling the agreement independent* — it is as available in a spec, a WP
frame, an issue, or a PR body as it is in a differential test.

## What worked, for contrast

My §2 refutation stood, and the difference is diagnostic: I got it by reading
`effect_v1.rs` **directly** rather than by checking a column of the author's
table. The Architect killed §3 by being asked a *different question* — "what
should be built?" — which forced reading the spec chapter **forward** instead of
verifying the table. See [[the-post-merge-yield-is-vantage-not-seat-quality]]:
the vantage is the mechanism, and this instance puts me on the *losing* side of
it, which makes it a better data point than the one where I won.
