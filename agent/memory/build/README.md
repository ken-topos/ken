# build — cross-team build-practice lessons

Loaded by **every** build-team function (leader, implementer, QA) across Kernel,
Verify, Language, Runtime, Ergo, and Foundation, in addition to `fleet` and the
team/role scopes. Practice that applies regardless of which build function you
are: test-authoring discipline, tooling gotchas, and verification habits that
cut across leader/implementer/QA. A lesson specific to one function belongs in
`build/leaders`, `build/qa`, or `build/implementers` instead; a lesson specific
to one team belongs in `teams/<team>`.

| Lesson | One-line |
|---|---|
| [a-check-that-measures-a-proxy-passes-for-the-wrong-reason](a-check-that-measures-a-proxy-passes-for-the-wrong-reason.md) | A check comparing a PROXY (self-identity, label-membership, cardinality) instead of the PROPERTY is green either way; write the claim as a sentence and confirm the code's operands are its nouns |
| [an-ac-saying-positive-control-reads-as-run-one-not-leave-one](an-ac-saying-positive-control-reads-as-run-one-not-leave-one.md) | "Positive control" reliably buys a control that was RUN, not one COMMITTED — a WP merged with zero `#[test]` lines after QA ran a genuine discriminating mutation by hand. Name the committed artifact, or the evidence evaporates when the reviewer closes the terminal |
| [assert-specific-error-variant-not-is-err](assert-specific-error-variant-not-is-err.md) | Assert the specific error variant, not a bare `is_err()` |
| [declaration-order-claim-needs-three-probe-net](declaration-order-claim-needs-three-probe-net.md) | A load-bearing declaration-order claim needs a 3-probe net (acyclic-forward / backward / mutual-cycle) before it becomes authoring guidance |
| [failed-post-condition-probe-suspect-the-probe-first](failed-post-condition-probe-suspect-the-probe-first.md) | When a post-condition probe reports content MISSING, suspect the probe first — a phrase grep false-negatives on markdown line-wrap and blockquote markers; a count-on-both-sides discriminates where an exact-phrase grep does not. Same disease hits build artifacts: a stale `target/debug/deps` rlib, `find`/`ls` order mistaken for recency, and `\| tail -N` eating both the evidence and the exit code |
| [general-fix-can-conflate-similar-shaped-different-cases](general-fix-can-conflate-similar-shaped-different-cases.md) | A general fix can conflate similar-shaped but different cases |
| [green-vs-green-does-not-confirm-a-fix](green-vs-green-does-not-confirm-a-fix.md) | Green-vs-green does not confirm a fix |
| [lawful-instance-needs-three-axis-acceptance-net](lawful-instance-needs-three-axis-acceptance-net.md) | A new lawful instance needs a 3-axis acceptance net (provenance + concrete compute + each law field), from brick 1 |
| [mid-branch-correction-regrep-whole-branch-for-stale-claims](mid-branch-correction-regrep-whole-branch-for-stale-claims.md) | A mid-branch design correction must re-grep every file the branch touches for the old claim's substance |
| [mid-review-fix-inline-escalate-or-track](mid-review-fix-inline-escalate-or-track.md) | A defect caught mid-review: resolve it yourself only if structurally determined, else escalate to the lane owner — then fold before the vote, or track once merge is imminent, never move a proposed Decision's SHA anchor |
| [named-floor-must-be-grepped-not-assumed](named-floor-must-be-grepped-not-assumed.md) | A named floor must be grepped, not assumed |
| [probe-recursion-depth-before-writing-the-real-test](probe-recursion-depth-before-writing-the-real-test.md) | Probe recursion depth before writing the real test |
| [rename-wp-needs-whole-workspace-basename-sweep](rename-wp-needs-whole-workspace-basename-sweep.md) | A rename/move/delete WP needs a whole-workspace old-spelling sweep — and every survivor classified live-to-update vs intentionally historical |
| [timeout-does-not-kill-grandchild-cargo-test](timeout-does-not-kill-grandchild-cargo-test.md) | `timeout` doesn't kill a grandchild cargo test binary |
| [wp-branch-handoff-deadlock-leader-holds](wp-branch-handoff-deadlock-leader-holds.md) | A leader who checks out a WP branch in their own worktree deadlocks the handoff — every next-ring role (implementer, QA, conformance-validator) can be the blocked author |
