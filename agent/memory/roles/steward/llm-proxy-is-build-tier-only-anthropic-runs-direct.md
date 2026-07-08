---
scope: roles/steward
audience: (see scope README)
source: private memory `llm-proxy-is-build-tier-only-anthropic-runs-direct`
---

# All agents run Anthropic models direct; the llm-proxy is retired

**Every agent runs an Anthropic model DIRECT (operator, 2026-07-03):** the whole
fleet is Anthropic — **Opus 4.8** (the Steward/spec-author/architect enclave) or
**Sonnet 5** (build leaders/implementers/QA, Librarian). They run
**directly on the Anthropic subscription (OAuth)** and carry **NO `env` block**
at all in `moot.toml` — they inherit the container's direct-Anthropic auth.
There is no open-weight tier anymore.

**The `.devcontainer/run-llm-proxy.sh` (:8090) proxy is retired/vestigial.** It
existed only to route the old non-Anthropic build tier by model prefix; with the
whole fleet Anthropic-direct nothing routes through it. A watchdog
`curl -m3 127.0.0.1:8090/v1/messages` (expects 405) is a harmless liveness
leftover, not a dependency of any live agent.

**The swap gotcha (still live):** a role runs direct **only** with no `env`
block. If a stray `env = { ANTHROPIC_BASE_URL = "http://127.0.0.1:8090", … }` is
left on a role, its relaunched session asks the dead proxy for a model it can't
route → an API-error retry loop ("attempt N/10") + the warning
"ANTHROPIC_API_KEY … takes precedence over your claude.ai login". **Fix: the
`env` line must be absent.**

**Relaunch mechanics:** `tmux kill-session -t moot-<role>; moot exec <role>`
(run from `/workspaces/ken`, not a worktree — bash cd main repo vs steward
worktree). A freshly `moot exec`'d session shows the Claude Code **first-launch
consent modal** ("1. I am using this for local development / Enter to confirm")
— answer it (Enter) or it sits wedged. `moot attach` reuses the running process
(won't re-read a changed config); a real relaunch needs kill + `moot exec`. See
steward coldstart infra checks, fleet model rollout stagger restarts.
