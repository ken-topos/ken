# roles/steward — Steward-specific lessons

Loaded by the Steward, in addition to `fleet` and `enclave`. Fleet coordination,
watchdog, worktree, and infra lessons specific to the operator's primary proxy
into the federation.

| Lesson | One-line |
|---|---|
| [bash-cd-main-repo-vs-steward-worktree](bash-cd-main-repo-vs-steward-worktree.md) | Bash `cd /workspaces/ken` targets the main repo, not the steward worktree |
| [bundled-frame-doc-goes-stale-when-mechanism-flips](bundled-frame-doc-goes-stale-when-mechanism-flips.md) | A bundled frame doc goes stale when a WP's mechanism flips mid-build |
| [entrypoint-abi-change-is-never-corpus-disjoint](entrypoint-abi-change-is-never-corpus-disjoint.md) | An entrypoint-ABI change rewrites every example's `main` — grep the touch-set before calling a lane catalog-disjoint |
| [dont-gate-pipeline-fill-on-credit-cost](dont-gate-pipeline-fill-on-credit-cost.md) | Spend the weekly token window — idle capacity is the waste; keep every unit busy, parallelize aggressively |
| [fleet-model-rollout-stagger-restarts](fleet-model-rollout-stagger-restarts.md) | Stagger restarts when rolling the fleet onto a new model |
| [llm-proxy-is-build-tier-only-anthropic-runs-direct](llm-proxy-is-build-tier-only-anthropic-runs-direct.md) | All agents run Anthropic models direct; the llm-proxy is retired |
| [my-own-tracker-capability-landed-line-can-be-stale](my-own-tracker-capability-landed-line-can-be-stale.md) | Your own tracker's 'capability X landed' line can be stale |
| [orphan-watchdog-timer-record-id](orphan-watchdog-timer-record-id.md) | Watchdogs use a private CronCreate timer, not the convo schedule_call |
| [playbooks-state-mechanism-not-intent](playbooks-state-mechanism-not-intent.md) | Playbooks must state the mechanism explicitly, not just the intent |
| [representative-file-review-only-covers-constructs-that-file-has](representative-file-review-only-covers-constructs-that-file-has.md) | A one-file tool review certifies only that file's constructs; spot-check distinct constructs before a catalog sweep arms a gate |
| [splay-gate-only-as-good-as-its-detector-verify-the-check](splay-gate-only-as-good-as-its-detector-verify-the-check.md) | A style spot-check is worthless unless the detector provably fires on a known-bad and targets the over-width failure mode; awk `\s` is non-portable — use `[[:space:]]`; prefer an automated CI gate |
| [steward-coldstart-infra-checks](steward-coldstart-infra-checks.md) | On a Steward cold-start, check fleet-wide infra before concluding stalled |
| [steward-must-relay-publisher-merges](steward-must-relay-publisher-merges.md) | The Steward must relay publisher-path merges |
