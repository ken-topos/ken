# fleet — lessons every agent applies

Loaded by **every** role (referenced from root `AGENTS.md`). Keep this scope
small and universal: coordination law, attention discipline,
closure/ground-truth verification, clean-room, compaction. A lesson that only
some roles need belongs in a narrower scope.

| Lesson | One-line |
|---|---|
| [a-deferral-is-honest-a-deferral-that-reads-as-delivery-is-not](a-deferral-is-honest-a-deferral-that-reads-as-delivery-is-not.md) | A WP can merge with its headline deferred and no test fails — ground GATE status in the gate's own text, never roll it up from WP status |
| [a-dependency-is-met-when-you-can-write-the-obligation](a-dependency-is-met-when-you-can-write-the-obligation.md) | "Merged" ≠ "unblocked": a bridge gives you the SPINE, not the ELEMENTS — try to WRITE the obligation before you kick the dependent |
| [an-oracle-that-greps-a-name-fires-on-prose-that-denies-it](an-oracle-that-greps-a-name-fires-on-prose-that-denies-it.md) | A "zero-X" oracle that greps the NAME fires on the prose that DENIES it — check declarations, not substrings |
| [check-main-via-git-object-store-not-find](check-main-via-git-object-store-not-find.md) | Check main via the git object store, not `find` |
| [cleanroom-is-role-discipline-not-host](cleanroom-is-role-discipline-not-host.md) | Clean-room protection is a role discipline, not a model-host property |
| [compact-verify-survey-can-eat-the-compact-command](compact-verify-survey-can-eat-the-compact-command.md) | A Claude Code survey prompt can eat a `/compact` command |
| [compact-wiped-memory-reflog-first](compact-wiped-memory-reflog-first.md) | After a `/compact`, check git reflog before concluding you're stalled |
| [compaction-is-manual-no-clean-seam](compaction-is-manual-no-clean-seam.md) | Team compaction follows the playbook as-is; the manual-seam problem is singleton-specific |
| [compaction-render-delay-escape-aborts](compaction-render-delay-escape-aborts.md) | `/compact` has a render delay; Escape aborts it, don't send it |
| [correcting-scope-must-sweep-whole-doc](correcting-scope-must-sweep-whole-doc.md) | Correcting a false claim in a doc must sweep the whole document |
| [formatter-soundness-gates-are-blind-to-layout-conformance](formatter-soundness-gates-are-blind-to-layout-conformance.md) | A formatter's meaning-preservation gates can't see bad layout; gate what it emits against the layout spec |
| [git-author-not-per-agent](git-author-not-per-agent.md) | Git author is shared, not per-agent |
| [grep-rust-prelude-emission-for-landedness](grep-rust-prelude-emission-for-landedness.md) | A landedness grep must also check the Rust-emitted prelude, not just `.ken` sources |
| [grep-the-producer-not-the-cited-proxy](grep-the-producer-not-the-cited-proxy.md) | Verify against the real producer, not a cited proxy |
| [handoff-scope-count-must-match-full-thread](handoff-scope-count-must-match-full-thread.md) | Verify a handoff's scope count against the full thread |
| [held-branch-scaffolding-is-load-bearing-evidence](held-branch-scaffolding-is-load-bearing-evidence.md) | A held branch's scaffolding is load-bearing evidence |
| [live-review-candidate-goes-stale-reanchor-sha](live-review-candidate-goes-stale-reanchor-sha.md) | A live review candidate can go stale; re-anchor the SHA |
| [markdown-80col-reflow](markdown-80col-reflow.md) | 80-column markdown reflow: the recurring gotchas |
| [mention-discipline](mention-discipline.md) | Mention IFF question or next-action; grep the participant-id; the `mentions` array is reliable |
| [model-swap-does-not-reset-the-seat](model-swap-does-not-reset-the-seat.md) | A model swap keeps the seat's context; rouse to continue, don't tell it to re-orient |
| [mootup-posting-from-agent](mootup-posting-from-agent.md) | How a build-tier agent posts to mootup |
| [multi-worktree-cwd-drift-phantom-diff](multi-worktree-cwd-drift-phantom-diff.md) | Multi-worktree cwd drift produces a phantom diff |
| [never-pin-a-shape-that-cannot-state-its-own-contract](never-pin-a-shape-that-cannot-state-its-own-contract.md) | Never pin a shape that cannot state its own contract — the expressibility audit (PRINCIPLES #14, widened) |
| [orphaned-background-task-loops-leak-cpu](orphaned-background-task-loops-leak-cpu.md) | Hand-rolled background bash loops can orphan and leak CPU |
| [pane-suggestion-text-is-not-agent-state](pane-suggestion-text-is-not-agent-state.md) | The tmux pane's gray suggestion text is not agent state |
| [publisher-app-cannot-push-workflow-file-changes](publisher-app-cannot-push-workflow-file-changes.md) | The publisher App can't push `.github/workflows/` edits; enforce CI gates as workspace tests |
| [re-read-latest-events-immediately-before-a-stall-nudge](re-read-latest-events-immediately-before-a-stall-nudge.md) | Re-read latest events immediately before a stall-nudge |
| [reason-in-agent-team-hours-not-human-days](reason-in-agent-team-hours-not-human-days.md) | Reason in agent-team wall-clock hours, never human work-days |
| [reviewers-review-branches-not-prs](reviewers-review-branches-not-prs.md) | Federation reviewers review branches, not PRs |
| [self-contained-handoff-paste-verbatim-no-event-id](self-contained-handoff-paste-verbatim-no-event-id.md) | Hand a ruling/artifact verbatim in-thread; never make a seat fetch it by event-ID |
| [surface-the-seam-need-not-your-preferred-mechanism](surface-the-seam-need-not-your-preferred-mechanism.md) | Surface a cross-author need; leave the mechanism to the owner |
| [terminal-gate-resolve-race-resolving-on-cast](terminal-gate-resolve-race-resolving-on-cast.md) | A terminal-gate resolve can race the last voter's own resolve |
| [the-consumer-edge-ledger-spine-element-obligation-only](the-consumer-edge-ledger-spine-element-obligation-only.md) | Before migrating off a representation, classify EVERY use-site spine / element / obligation-only — a closure decision procedure you can run before writing code |
| [the-workaround-fossil-tells-you-what-the-language-could-not-say](the-workaround-fossil-tells-you-what-the-language-could-not-say.md) | Contorted code is EVIDENCE of a missing capability — and when the capability lands, go find the fossils it obsoletes |
| [thin-flow-directive-dont-cc-the-room](thin-flow-directive-dont-cc-the-room.md) | Thin-flow directive: one reviewer per lane, don't cc the room |
| [use-tier-labels-never-model-names](use-tier-labels-never-model-names.md) | Use tier labels, never model names |
| [verify-field-order-arity-against-declaration-not-prose](verify-field-order-arity-against-declaration-not-prose.md) | Verify field order/arity against the declaration, not prose |
