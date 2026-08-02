---
id: DOC-AGENT-CITE
title: "agent core modules name normative authorities as a reading list rather than binding them to claim classes, so seven of seven cold runs made material claims without citing the sources D2 requires"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-W6-AGENT-EVAL]
blocks: []
github: null
origin: "DOC-W6-AGENT-EVAL D5, merged 723989ba. The Wave 6 evaluation measured agent_core_ready = false on cited_authority with all seven tasks failing the same way, and recorded a per-module recommendation. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# The defect the Wave 6 evaluation isolated

`DOC-W6-AGENT-EVAL` ran seven cold seats against the current corpus and got a
uniform result: every task **correct**, every task **zero inventions**, every
task **zero unnecessary loads**, and every task `cited_authority = partial`.

Seven of seven failing one axis the same way is not seven defects. It is one.

## What the failure actually is

Not derived-page substitution — that was the hypothesis the WP was framed to
test, and it recorded **zero** substitutions across seven runs. The 54 new
`library/reference/` pages did not displace normative citations.

The failure is that an answer **asserts** something and never loads the section
that governs it. The `diagnose-layers` scorer stated it exactly:

> merely naming normative paths without loading their content does not ground
> the claims

And the modules invite that. `library/agents/core/read-ken.md` §9 currently
reads:

> The normative authority is `spec/`, especially
> `spec/30-surface/33-declarations.md` and `spec/30-surface/36-effects.md`.

That is a **reading list**. It tells an agent where authority lives; it does
not tell the agent that making a runtime claim obliges it to load
`spec/40-runtime/42-evaluation.md` first. So a cold agent names the paths it
was given, asserts beyond them, and scores `partial`.

## The five modules and what each one dropped

Measured at `main = 579d369e`:

| module | blob | governs the claims that went uncited |
|---|---|---|
| `core/read-ken.md` | `a4d9f82e` | runtime and execution-limit claims (`explain-contract` omitted `spec/40-runtime/42-evaluation.md`) |
| `core/proof-and-trust.md` | `9032ad37` | reduction, conversion, no-new-trust (`write-pure-law`, `repair-proof-terminal`, `find-package-by-task`) |
| `core/write-ken.md` | `eb66dd00` | declaration and conversion authority for authored `fn` and `theorem` |
| `core/toolchain.md` | `f64437b0` | proposed command vs observed artifact; CLI, FFI, portability |
| `tasks/effects-and-capabilities.md` | `68c9b421` | FFI and host-boundary claims, including inside an honest refusal (`refuse-unsupported`) |

`core/toolchain.md` is the widest: a **proposed** `ken check` invocation is not
an **observed** artifact, and five of the seven runs asserted a command's
behaviour without either implementation evidence or a run.

## Why this is grounded and not housekeeping

`library/agents/` exists so a cold agent can work in Ken from a selected pack.
An agent that produces correct answers resting on claims it cannot source is
producing exactly the thing Ken's mission is against — a plausible result whose
trust boundary is unstated. The gap is **measured**, from seven independent
cold runs, not inferred from reading the modules.

## What this node does not do

It does not re-run the suite. All seven fixtures were spent at Wave 6
checkpoint 2 and are burnt; fresh ones are the Librarian's content judgment
under that WP's hard stop 2, and they are not this node's cost. The
verification here is a paper check against the seven **recorded** answers,
which the Wave 6 record preserves in full.

It does not touch `D2`'s rule, the packs' `includes`, the tasks, the fixtures,
or any historical result.

Frame: `docs/program/wp/DOC-AGENT-CITE.md`.
