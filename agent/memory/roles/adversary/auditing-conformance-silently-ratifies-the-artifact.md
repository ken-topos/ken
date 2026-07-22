# Auditing an artifact against the rules silently ratifies its existence

**Lesson (RT-SPLIT slice 4, 2026-07-22 — the Architect caught what I did not).**
A new module `cranelift_backend/test_support.rs` landed. I hunted it hard and
well: I checked the module declaration against the ruled form, took a visibility
histogram of all 19 items, enumerated the 11 named items, traced every reference,
and proved there was no production consumer. I filed two real findings, both of
which the Architect then cited as contract violations.

**Every one of those checks presupposed the module should exist.** The Architect
asked the prior question — *should `test_support.rs` be created at slice 4 at
all?* — and answered no. LCA must be computed from a helper's **final ruled**
subject-test homes, not from where its users transiently sit mid-split. Reading
LCA off current locations manufactures a facade LCA for nearly every shared
helper, so the banned residual omnibus arrives one defensible hoist at a time.
`Px8dsEdgeMutation`'s uses are *all* in `control.rs`; its LCA was never the
facade. Eleven of the twelve declarations should not have moved.

## The shape

**A conformance audit is scoped strictly below the artifact's existence.** Every
question of the form *does X obey the rules governing X's contents?* takes X's
legitimacy as a fixed premise. No amount of rigor inside that scope can reach
the premise — and the more thorough the audit, the more it *reads* as clearance,
because a long list of green checks looks like coverage rather than like
coverage-of-one-layer.

This is the [[rank-subclaims-by-load-bearing-not-by-checkability]] generator in a
new venue: the checkable layer (contents vs. rules) crowded out the load-bearing
one (existence vs. derivation). Contents-conformance is tractable, greppable and
satisfying; the existence question is a judgment call against a placement rule
and has no mechanical oracle. I did the one with a tool.

## The tell was in the author's own handoff

The implementer wrote, unprompted:

> *"flag if you want the placement ruled explicitly, since I created it from the
> LCA rule rather than from a ruling naming these specific items."*

**That is an author flagging the underivedness of their own artifact** — the
exact question I failed to ask, handed to me in the message I was hunting. I
read it as background and audited the contents anyway. When a producer
volunteers that something was derived by their own application of a rule rather
than named by a ruling, that sentence *is* the finding's address.

## The rule

Before auditing a new artifact against the rules that govern it, spend one step
on the layer above:

1. **Does this artifact need to exist?** Which ruling names it, or which rule
   derives it — and was that derivation run on the right inputs?
2. **Check the derivation's inputs, not just its output.** Here the rule (LCA
   placement) was applied correctly to the *wrong* inputs (transient locations
   instead of final ruled homes). A correct rule on wrong inputs produces a
   conforming artifact that should not exist — invisible to every contents check.
3. **Mid-migration, treat "current location" as untrusted.** During a multi-slice
   decomposition, *where a thing sits now* is an artifact of the migration, never
   evidence of ownership. Any derivation keyed on present position is measuring
   the migration, not the design.
4. **Prefer "conforms, and here is why it should exist" to "conforms."** If you
   cannot supply the second clause, say so — that gap is a finding, and it is
   upstream of everything you did check.

## What stands

The two findings were real and load-bearing, and the enumeration that produced
them was sound — the Architect used both. **The evidence was good; the frame
around it was one layer too low.** Report the audit, and name the layer it did
not reach. Same separation as
[[no-option-works-name-the-axis-you-enumerated]]: the measurement was correct,
the scope claim on top of it was unearned.

Related: [[a-repro-is-evidence-not-a-completion-oracle]] — proving a property
true by a manual oracle is not the same as the required check being able to see
it, and the Architect made exactly that distinction about my no-production-
consumer enumeration.
