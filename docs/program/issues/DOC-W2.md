---
id: DOC-W2
title: "documentation Wave 2 — agent core modules, task packs, and cold-context evals"
status: draft
owner: doc
size: L
gate: none
depends_on: [DOC-W1]
blocks: []
github: null
origin: research/librarian-documentation-program-proposal.md Wave 2; framed 2026-07-22 per operator directive (frame Waves 1-2 only)
---

**⛔ BLOCKED on `DOC-W1`.** Waves run sequentially (operator, 2026-07-22:
three seats, no fan-out). The agent modules describe the same product
knowledge the Wave 1 curriculum teaches; authoring them first would fork the
explanation and guarantee drift between the two.

> **⚠ `status: draft` here means GATED, not unscoped.** The issue schema has no
> `blocked` state, so an unmet `depends_on` and an unfinished scope share one
> word. **This brief is complete and shovel-ready** — the state reflects the
> dependency only. Same convention as `SEAL-2`. Contrast `A3`, whose `draft`
> genuinely means no owner, no size, and no brief.

Program frame: `docs/program/12-documentation-program.md` §4b Wave 2. **Read
§4a first.** §1's four decisions are **settled**.

## 1. Objective

`library/agents/` as a **context library, not an agent manual**: selectable
product-knowledge modules a Ken-untrained coding agent can load to perform the
core read / write / prove / diagnose tasks — and, where it cannot, to **refuse
honestly rather than improvise**.

## 2. Deliverables

**Manifest and protocol**

| artifact | contents |
|---|---|
| `library/agents/README.md` | selection protocol, authority rules, and the product-context vs. workflow-instruction boundary |
| `library/agents/manifest.toml` | every module and pack: purpose, triggers, prerequisites, included files, source anchors, revision, validation, **measured token size** |

**Four core modules**

| module | contents |
|---|---|
| `core/read-ken.md` | minimal syntax and semantic orientation needed to inspect any Ken source |
| `core/write-ken.md` | canonical authoring forms and the probe/check/format loop |
| `core/proof-and-trust.md` | proof terminals, claims, `Axiom`, trusted-base accounting, and the hard boundaries between proved / tested / delegated / unknown |
| `core/toolchain.md` | exact current commands, file roles, expected artifacts, fail-closed handling of unavailable features |

**Six task modules**

| module | contents |
|---|---|
| `tasks/read-review.md` | procedure and output contract for reviewing a Ken program for a human |
| `tasks/write-program.md` | requirement → checked source, contract-first decomposition |
| `tasks/author-package.md` | package identity, literate entry shape, public API/laws, trust/derivation, examples, validation |
| `tasks/prove-or-repair.md` | goal inspection, reduction, induction/case decomposition, **trusted-boundary refusal**, diagnostic routes |
| `tasks/diagnose.md` | evidence-gathering order for parse, elaborate, kernel, proof, interpreter, and native failures |
| `tasks/effects-and-capabilities.md` | effect rows, capability supply, handlers, authority, resources, supported execution paths |

**Plus:** pack manifests under `agents/packs/`, the schema under
`agents/schemas/`, pack integrity checks (**reject missing modules and
circular pack dependencies**), and the §5 evaluation suite.

> ### ⛔ `tasks/ffi-and-platform.md` is the DEFERRED seventh module
>
> The proposal lists it alongside the other six. **It is not in this wave and
> must not be written here.** The FFI/platform surface is exactly what
> `docs/program/10-linux-abi-completion.md` is still landing; a module
> documenting it today would be obsolete before the wave closed — or would
> document *aspirational* syntax, which §2 of the program frame forbids
> outright. It is framed with Wave 1b or after `PX8` closes, whichever is
> later.

**`agents/domains/` and `agents/catalog/` are also out of scope here.** Domain
modules follow the Wave 3/4 reference work they summarize; generated package
cards follow Wave 5's D4 re-check.

## 3. The ten-point module contract — every module, in this order

1. **Use when** — positive triggers **and explicit non-triggers**.
2. **Prerequisites** — modules or facts that must already be present.
3. **Current capability** — what the landed toolchain supports, **with no
   aspirational syntax mixed in**.
4. **Canonical forms** — smallest checked examples, exact declaration/command
   shapes.
5. **Invariants and prohibitions** — rules that must not be inferred from
   examples alone.
6. **Decision procedure** — a short task sequence with **observable stop
   conditions**.
7. **Failure signatures** — common diagnostics, likely layer, next source to
   inspect.
8. **Validation** — exact targeted checks for the artifact type.
9. **Authority and sources** — normative and generated sources, plus the
   revision the module was verified against.
10. **Known unavailable or partial behavior** — **fail closed rather than
    invite the agent to improvise.**

> **★ Point 10 is load-bearing and it is the one that will get shortchanged.**
> An agent module's characteristic harm is not being incomplete — it is being
> **confidently silent** about a boundary, which reads to the consuming agent
> as permission. The **negative knowledge** is the part that pays: unsupported
> forms, misleading near-syntax, `tt` versus `Refl`, and the exact point at
> which an agent must **stop** instead of inventing a proof, primitive,
> capability, or package.

Prefer tables, signatures, checked examples, and explicit contrasts over
extended narrative.

## 4. The `write-ken` refactor — a migration with two live consumers

`agent/playbooks/tools/write-ken.md` keeps its **workflow trigger** and moves
its **product facts** into `library/agents/core/write-ken.md`; the skill then
selects a pack (D3, settled).

**⚠ Inventory BOTH consumers before moving a fact:** the fleet's own seats
(which load the skill today) and any external agent (which will load the
module). A fact that moves without its in-fleet consumer being re-pointed
leaves the seat silently short of it, and skills register at **session
start** — a mid-session edit does not reach a running seat.

## 5. Exit property and the evaluation suite

> **A Ken-untrained coding agent can perform the core read / write / prove /
> diagnose tasks without loading the entire spec, catalog guide, or fleet
> memory — and, on the tasks it cannot do, refuses honestly rather than
> improvising.**

**Seven cold-context tasks:**

1. explain a small Ken program's contract and trust posture;
2. write and check a pure function with one real law;
3. distinguish and repair `tt` versus `Refl` proof endpoints;
4. find and use a catalog package **by task rather than guessed name**;
5. author an effectful boundary without omitting its capability or row;
6. **refuse an unsupported or unproved request honestly**;
7. diagnose one parse, one elaboration, one kernel, and one runtime failure.

**Record for each run:** correctness, **unnecessary file loads**, **invented
syntax or capabilities**, and **whether the agent cited the authority it
used.**

> ### ⛔ DO NOT COLLAPSE THE SUITE INTO A PASS RATE
>
> A run that passes six of seven **while inventing a capability on the
> seventh** is a worse outcome than one that passes five and refuses two — and
> a pass-rate acceptance criterion cannot express that. Score the four axes
> separately and report them separately. **An invented capability is a
> failure of the whole suite regardless of the other six.**
>
> And the goal is **not** the smallest token count. It is the smallest context
> that reliably produces a correct, **reviewable** result.

## 6. Acceptance criteria

1. **Every artifact named in §2 lands** — enumerate them against §2 in the PR
   body rather than asserting completeness — each satisfying the §3 ten-point
   contract **in order**, with point 10 non-empty and specific.
2. **`manifest.toml` records a measured token size per module and pack** — the
   measurement mechanism goes in the PR body.
3. **Pack integrity checks reject a missing module and a circular pack
   dependency**, each **proved by a planted violation**, not by passing on a
   valid corpus.
4. **The seven-task suite runs against a genuinely cold seat**, with the four
   axes reported separately per §5.
5. **Every checked example in every module actually checks** at the candidate
   SHA, by the fence gate — not by the author having run it once.
6. **No module contains federation workflow.** D3 is settled; a reviewer must
   be able to check this cheaply.
7. **The `write-ken` refactor re-points both consumers** (§4), and the in-fleet
   seat path is verified, not assumed.

## 7. Framing traps

- **The evaluation suite is itself a corpus-shaped oracle** and is only as
  strong as its seven tasks. It cannot see a defect in a module nothing in the
  suite exercises. Say so in the retro rather than reporting the suite as
  coverage.
- **A cold seat is cold only once.** Design the runs so a module fix can be
  re-evaluated without the seat having already seen the answer, and state how.
- **Measured token size is an anchor and it is perishable** — it goes stale on
  every module edit. Regenerate it; do not carry a number forward.
