---
scope: teams/kernel
audience: kernel-leader, kernel-implementer, kernel-qa, architect
source: extracted from private memory
  `k7-eq-at-inductive-operand-whnf-gap`
  (the review-discipline finding, not the K7 status narrative)
related: kernel-completeness-gap-shapes, exhaustive-term-traversals
---

# A trust-root reduction/whnf change has a workspace-wide blast radius (gated in CI)

A sound kernel **completeness** (reduction) change — one that makes *more* goals
reduce than before — forces downstream proof-term migrations. So "kernel-only
diff" is a **false scope premise**: the blast radius reaches `catalog/packages/`
proofs and other crates' tests.

**The gate is the full workspace — but it runs in CI, NOT locally** (operator
hard rule, COORDINATION §12: a local `cargo test --workspace` OOMs the shared
box). Locally you validate **targeted** (`scripts/ken-cargo -p <crate>` /
`--test <name>`); the whole-repo `--workspace --locked` run is CI's, and the
scripted publisher gates the merge on it. The lesson here is therefore **not**
"run a local workspace build" — it is **design the landing so CI's workspace run
stays green**: migrate every downstream proof term *in the same landing* as the
kernel change, so `main` never reddens. If CI does redden, it comes back as a
publisher `blocked` mention on your implementer — that is the workspace gate
doing its job.

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
test; it just didn't extend the check to the `catalog/packages/` proofs living in a
different crate.

**Why this is general, not a one-off:** more reduction means some goals that
used to stay stuck at `Eq` now collapse to `Top`/`Bottom`. Every
`Refl`-on-an-operation-wrapped-goal downstream of the change — test code *and*
shipped `.ken` proofs — must migrate to the new closing term (e.g. `tt`). A
benign corollary: proofs that happened to typecheck **because** of the pre-fix
incompleteness stay realizable after the fix (the goal still holds), but their
proof *terms* must change shape, or the workspace won't build.

**How to apply.** On any change to a kernel reduction/whnf/conversion function:
(1) **enumerate the blast radius up front** — grep for `Refl`-on-operation-
wrapped goals in `catalog/packages/` and every crate's tests, and migrate those
proof terms **together with** the kernel change in the **same landing**, so
`main` never reddens (CI's workspace run is what would otherwise catch it, late);
(2) validate **locally targeted** on the crates you migrated (`scripts/ken-cargo
-p <crate>` / `--test <name>`) — do **NOT** run a local `cargo test --workspace`
(COORDINATION §12); the whole-repo gate is CI's and the publisher polls it at
merge; (3) treat "I only touched `ken-kernel`" as a claim to verify, not a scope
boundary — the kernel not being *structurally* touched (e.g. `conv.rs` byte-
identical) is a separate fact from "nothing downstream needs to change." The
frame should design the workspace coverage in (steward.md: distinguish the
soundness surface from the landing unit) so CI confirms it rather than discovers
a regression.
