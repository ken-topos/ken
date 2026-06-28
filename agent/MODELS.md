# Model tiers

Ken runs a heterogeneous fleet to concentrate spend on the highest-judgment and
clean-room-critical work while using cheaper models for high-volume code
generation and coordination. Three tiers:

| Tier | Model | Roles | Why |
|---|---|---|---|
| **T1** | **Opus 4.8 (1M, high effort, extended thinking)** | Spec-author, Conformance-validator, **Steward**, **Architect** | Highest judgment; the clean-room enclave; design + workflow authority. These calls are worth the most. |
| **T2** | **GLM 5.2** (Fireworks) | Implementers | High-volume code generation. |
| **T3** | **DeepSeek V4 Pro** | Build-team Leaders, Build-team QA, Spec-team Leader, Integrator, Librarian | Coordination, mechanical gates, verification runs, doc observation. |

The operator is the human product owner; Steward is the primary proxy into the
federation.

## Knobs (tune by observed quality, not up front)

- **Kernel and Verify QA** are the likeliest T3→T2 upgrades — soundness-adjacent
  testing may warrant a stronger model. Start T3; upgrade if quality lags.
- **Integrator stays T3.** It enforces gates and merges; the deep correctness
  and architectural review is the **Architect's** (T1) job on the merge
  Decision, so the Integrator does not need a strong model.

## Clean-room × models (load-bearing)

The tiering *reinforces* the clean-room boundary:

- Only the **T1 Opus Spec enclave** (Anthropic-hosted) ever reads the AGPLv3
  prototype. The build teams (GLM/DeepSeek) see only `/spec` and `/conformance`,
  which are clean by construction.
- **Never send AGPLv3 prototype source to Fireworks or DeepSeek.** Prototype
  content goes only to the Anthropic-hosted Spec agents. Ken's own MIT source is
  fine to send anywhere.
- **Copyleft references (⚠ GPL/AGPL/CeCILL — `smtcoq`, `spot`, `jif`) are
  enclave-only too.** Only the T1 Opus Spec enclave (Architect / Spec) reads
  them, for *approach and behavior* only, under the leakage recheck
  (`CLEAN-ROOM.md`); they are never sent to build-team providers and never
  vendored. Permissive refs (Lean, Z3, Quint, …) may be read by the enclave but,
  like all refs, are not a source for implementer agents — those build from
  `/spec`.

## Portability

Playbooks and `COORDINATION.md` are written **model-agnostic** — no reliance on
Claude-specific behaviors — because the same coordination law must hold across
Anthropic, Fireworks, and DeepSeek.

**Routing mechanism (hybrid).** Each agent is a normal Claude Code process
launched by `moot up`, which reads its `model` + `effort` per role from
`moot.toml` (`[agents.<role>]`) and passes `--model`/`--effort`. Provider
routing is **hybrid**:

- **Opus enclave** (4 roles) runs **direct on the Anthropic subscription**
  (OAuth) — never through the proxy (the convo guardrail rejects subscription
  tokens at the proxy).
- **Build tiers** (GLM/DeepSeek) route through a **local LLM proxy**
  (`mootup_harness_sdk.llm_proxy`, `127.0.0.1:8090`) that dispatches by model
  prefix (`accounts/fireworks/*`→Fireworks, `deepseek-*`→DeepSeek); the proxy
  holds the upstream keys from `/home/node/.secrets/`.

Per-role selection is declared in `moot.toml`: each build role's
`[agents.<role>].env` sets `ANTHROPIC_BASE_URL` + `ANTHROPIC_AUTH_TOKEN`
(`${secret:llm-proxy-secret}`, resolved from `/home/node/.secrets/` at launch;
requires mootup ≥ 0.5.4), and moot injects it into the agent's launch env;
enclave roles set no `env`, so they use the default Anthropic endpoint + OAuth.
It must be **`ANTHROPIC_AUTH_TOKEN`** (a Bearer), not `ANTHROPIC_API_KEY`: the
proxy reads `Authorization: Bearer`, and AUTH_TOKEN also overrides the shared
claude.ai OAuth login (from the enclave's `/login`) that build agents would
otherwise send — which the proxy rejects as a subscription token. The proxy is
started by `run-llm-proxy.sh`. Operator runbook:
`local/mootup-agent-backends-setup.md`.
