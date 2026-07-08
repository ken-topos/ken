# roles/steward — Steward-specific lessons

Loaded by the Steward, in addition to `fleet` and `enclave`. Fleet coordination,
watchdog, worktree, and infra lessons specific to the operator's primary proxy
into the federation.

| Lesson | One-line |
|---|---|
| [bash-cd-main-repo-vs-steward-worktree](bash-cd-main-repo-vs-steward-worktree.md) | Bash `cd /workspaces/ken` targets the main repo, not the steward worktree |
| [bundled-frame-doc-goes-stale-when-mechanism-flips](bundled-frame-doc-goes-stale-when-mechanism-flips.md) | A bundled frame doc goes stale when a WP's mechanism flips mid-build |
| [fleet-model-rollout-stagger-restarts](fleet-model-rollout-stagger-restarts.md) | Stagger restarts when rolling the fleet onto a new model |
| [llm-proxy-is-build-tier-only-anthropic-runs-direct](llm-proxy-is-build-tier-only-anthropic-runs-direct.md) | All agents run Anthropic models direct; the llm-proxy is retired |
| [my-own-tracker-capability-landed-line-can-be-stale](my-own-tracker-capability-landed-line-can-be-stale.md) | Your own tracker's 'capability X landed' line can be stale |
| [orphan-watchdog-timer-record-id](orphan-watchdog-timer-record-id.md) | Watchdogs use a private CronCreate timer, not the convo schedule_call |
| [playbooks-state-mechanism-not-intent](playbooks-state-mechanism-not-intent.md) | Playbooks must state the mechanism explicitly, not just the intent |
| [steward-coldstart-infra-checks](steward-coldstart-infra-checks.md) | On a Steward cold-start, check fleet-wide infra before concluding stalled |
| [steward-must-relay-publisher-merges](steward-must-relay-publisher-merges.md) | The Steward must relay publisher-path merges |
