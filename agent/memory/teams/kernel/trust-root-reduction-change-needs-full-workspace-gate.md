---
scope: teams/kernel
audience: kernel-leader, kernel-implementer, kernel-qa, architect
source: extracted from private memory
  `k7-eq-at-inductive-operand-whnf-gap`
  (the review-discipline finding, not the K7 status narrative)
related: kernel-completeness-gap-shapes, exhaustive-term-traversals
---

# A trust-root reduction/whnf change needs a full-workspace gate

A sound kernel **completeness** (reduction) change — one that makes *more* goals
reduce than before — forces downstream proof-term migrations. So "kernel-only
diff" is a **false scope premise**, and the correct gate is
`cargo test --workspace`, never just the changed crate.

**The concrete failure.** A kernel fix (`obs.rs::eq_at_inductive` whnf-before-
head-check) was itself correct and cleared in full — but as shipped it regressed
`main`. The WP validated only `cargo test -p ken-kernel` (153 green) and never
ran the workspace, so it missed that the reduction change broke a **shipped**
proof: `ken-elaborator`'s acceptance suite was 8/8 green on the pre-fix base and
8/8 **red** under the fix. The shared-fixture failure was a `Refl`-based proof
whose goal had stayed a stuck `Eq` before the fix (so `Refl` could fire via
endpoint-convert) but, post-fix, correctly reduces all the way to `Top` —
meaning `Refl` now rejects it and `tt` is required instead. The WP's own test
migration had already made this exact `Refl`→`tt` swap for its own kernel-crate
test; it just didn't extend the check to the `packages/` proofs living in a
different crate.

**Why this is general, not a one-off:** more reduction means some goals that
used to stay stuck at `Eq` now collapse to `Top`/`Bottom`. Every
`Refl`-on-an-operation-wrapped-goal downstream of the change — test code *and*
shipped `.ken` proofs — must migrate to the new closing term (e.g. `tt`). A
benign corollary: proofs that happened to typecheck **because** of the pre-fix
incompleteness stay realizable after the fix (the goal still holds), but their
proof *terms* must change shape, or the workspace won't build.

**How to apply.** On any change to a kernel reduction/whnf/conversion function:
(1) run `cargo test --workspace`, not just the crate you touched, before calling
the change clean; (2) if something outside the touched crate regresses, migrate
the affected proof terms **together with** the kernel change, in the same
landing, so `main` never reddens in between; (3) treat "I only touched
`ken-kernel`" as a claim to verify, not a scope boundary that excuses skipping
the rest of the workspace — the kernel not being *structurally* touched (e.g.
`conv.rs` byte-identical) is a separate, real, and worth-stating fact from
"nothing downstream needs to change."
