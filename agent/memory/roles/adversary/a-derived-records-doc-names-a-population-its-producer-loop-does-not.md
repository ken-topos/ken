---
name: a-derived-records-doc-names-a-population-its-producer-loop-does-not
description: "Derived metadata is documented by the POPULATION it summarizes ('any negative use in the declaration') while its producer loops over one sub-population. The first consumer reads the doc. Attack: put the identical term in every position the grammar allows and diff the readings."
scope: roles/adversary
---

# A derived record's doc names a population its producer loop does not

`KERNEL-NESTED-IND` `D1a` recorded per-parameter polarity for an inductive
family. `env.rs` documented it as *"known to occur only in strictly positive
positions **in the declaration**… **any** unsupported or negative use is
`NonPositive`."* The producer looped over `constructor.args` and nothing else.

A declaration has **four** populations where a parameter can occur —
`constructor.args`, `constructor.target_indices`, `ind.indices`, and the
parameter telescope itself. The same term `A -> Bool` recorded `NonPositive`
in the first and `StrictlyPositive` in the other three.

⭐ **The producer was not wrong; the doc was a universal quantifier over a set
the loop never enumerated.** Producer and doc are written in the same commit by
the same author, and reviewing them together makes the loop look like the
definition of the doc rather than a subset of it.

## The attack, in one move

**Put the identical term in every position the grammar allows, and diff the
readings.** One term, N positions, one table. It needs no theory about which
position is dangerous — the disagreement *is* the finding, and a reviewer
cannot answer that your two witnesses differed in some other way.

Get the population from the **type's own field list**, not from prose: a
`struct` with four `Vec<Term>` fields declares the four positions. Then check
which of them the producer's loop mentions. Here the whole audit was: the
function body names `constructor.args`; `indices`, `target_indices` and
`params` appear nowhere in it.

⚠ **Always include the position whose answer you already know** as a control —
the one the WP's own test asserts. Without it a uniform reading is
indistinguishable from a broken probe
([[audit-a-detector-against-the-one-case-whose-answer-you-already-know]]).

## Why it is worth filing while the record is still inert

The record's first consumer landed one merge later and used
`== StrictlyPositive` as the **permissive** branch of a safety gate. So the gap
and the consumer's polarity aligned in the unsafe direction, and the consumer
was written against the doc.

⇒ **A metadata node is at its most attackable in the window between "derived"
and "consumed"** — the doc is written, the loop is written, and nothing has
forced them to agree yet. Once a consumer exists the two get reconciled by
whatever the consumer happens to need, and the over-claim survives as the part
nobody re-derived
([[ask-whether-a-load-bearing-premise-was-derived-or-merely-inherited]]).

⛔ Do not upgrade this to an unsoundness you have not built. I showed the
gate's premise is established without inspecting where the counterexample
lives; I did **not** show a negative occurrence in an index can be turned into
an inconsistency. State the direction (over-permissive) and stop
([[an-error-in-the-safe-direction-is-a-claim-about-what-you-did-not-measure]]).

## ⚠ The mutation that flipped two rows for one reason

Adding a `target_indices` pass flipped **two** rows. Only one was attributable
to it: the `ind.indices` witness used `λ(x:A). false` as its target index, and
the `Lam` arm marks its domain `Unknown` independently. Reporting "the mutation
proved both" would have been false, and the collateral row is the one a
reviewer would have checked.

**A mutation proves a gap only for the population it actually added.** Diff the
rows that moved against the rows you expected to move, and report the
difference rather than the aggregate — same discipline as
[[a-mutation-that-reddens-does-not-confirm-which-detector-caught-it]].

Related: [[close-a-class-partition-the-declared-population]] closes a class in
your **own** enumeration; this one attacks a class in **someone else's**.
