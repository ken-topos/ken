# Model tiers

Ken runs a two-tier fleet to concentrate the most expensive model on the
highest-judgment and clean-room-critical work, using a capable
high-throughput model for high-volume code generation and coordination. Every
agent runs an **Anthropic model directly** on the subscription (OAuth); there is
no open-weight tier and no provider proxy.

| Tier | Model | Roles | Why |
|---|---|---|---|
| **Enclave** | **Opus 4.8 (1M, high effort, extended thinking)** | Spec-author, Conformance-validator, **Steward**, **Architect** | Highest judgment; the clean-room enclave; design + workflow authority. These calls are worth the most. |
| **Build & coordination** | **Sonnet 5** | Build-team Leaders, Implementers, QA; Spec-team Leader; Integrator; Librarian | High-volume code generation, coordination, mechanical gates, verification runs, doc observation. |

The operator is the human product owner; Steward is the primary proxy into the
federation.

## Knobs (tune by observed quality, not up front)

The model split is fixed (enclave = Opus, everyone else = Sonnet 5); the tunable
knob is **effort**, set per role in `moot.toml`.

- **Kernel and Verify QA** are soundness-adjacent — they are the likeliest
  candidates for a higher effort setting if verification quality lags. Start at
  the team-default effort; raise it on observed misses, not up front.
- **Integrator effort stays low.** It enforces gates and merges; the deep
  correctness and architectural review is the **Architect's** (Opus) job on the
  merge Decision, so the Integrator does not need a high-effort budget.

## Clean-room × roles (load-bearing)

The clean-room boundary is a **role** discipline, not a property of the model in
the seat (`CLEAN-ROOM.md`; the enclave↔reference-access mapping is incidental —
any model in an enclave seat reads references under the same discipline, any
model in a build/coordination seat reads none):

- Only the **enclave** (Spec-author, Conformance-validator, Architect) may
  consult copyleft references. The build and coordination roles see only `/spec`
  and `/conformance`, which are clean by construction. The AGPLv3 prototype
  (`yon`) is **not mounted** and is not consulted by anyone — there is zero
  AGPLv3 contact, which is strictly cleaner than the alternative.
- **Never send copyleft material to a build or coordination role.** Only the
  behavioral description in Ken's own words (the spec/conformance artifacts) may
  pass to the build tier; never copyleft source text itself. Ken's own MIT
  source is fine to send anywhere.
- **Copyleft references (⚠ GPL/AGPL/CeCILL — `smtcoq`, `spot`, `jif`) are
  enclave-only too.** Only the enclave (Architect / Spec) reads them, for
  *approach and behavior* only, under the leakage recheck (`CLEAN-ROOM.md`); they
  are never sent to a non-enclave seat and never vendored. Permissive refs (Lean,
  Z3, Quint, …) may be read by the enclave but, like all refs, are not a source
  for implementer agents — those build from `/spec`.

## Portability

Playbooks and `COORDINATION.md` are written **model-agnostic** — no reliance on
any single model's idiosyncratic behaviors — because the same coordination law
must hold regardless of which model sits in each seat.

**Routing mechanism.** Each agent is a normal Claude Code process launched by
`moot up`, which reads its `model` + `effort` per role from `moot.toml`
(`[agents.<role>]`) and passes `--model`/`--effort`. Every role — enclave and
build/coordination alike — runs **direct on the Anthropic subscription** (OAuth),
with **no `env` block** and **no proxy**. (A retired `run-llm-proxy.sh` /
`127.0.0.1:8090` once dispatched a non-Anthropic build tier by model prefix; it
is vestigial now that the whole fleet is Anthropic-direct — a stray `env` block
pointing a role at it would break that role's launch.) Operator runbook:
`local/mootup-agent-backends-setup.md`.
