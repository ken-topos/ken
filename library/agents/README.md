# Ken agent context library

This directory provides selectable Ken **product knowledge** for a coding
agent. It is not an agent manual. Repository roles, coordination, branch
workflow, model routing, and fleet memory remain under `agent/`.

## Selection protocol

1. Name the task before loading context.
2. Select exactly one pack from `packs/` whose triggers match and whose
   exclusions do not.
3. Load the pack's files in listed order, recursively loading any pack
   dependencies first.
4. Follow module prerequisites only when the current task reaches them.
5. Stop if no pack matches. Do not approximate an unsupported task with the
   nearest pack.

The manifest at `manifest.toml` is the mechanical index. Its `measured_tokens`
values use the deterministic `unicode-whitespace-v1` measurement described
there. Size is a selection aid, not a correctness score.

## Available packs

| Task | Pack |
|---|---|
| explain and review existing source | `read-review` |
| write a pure checked program | `write-pure` |
| write an effectful checked boundary | `write-effectful` |
| author a literate catalog package | `author-package` |
| repair a proof without adding trust | `repair-proof` |
| locate a parse-to-runtime failure | `diagnose` |

There is no FFI/platform pack in this wave. If a task needs one, selection
fails closed.

## Authority rules

`library/agents/` is explanatory and derived. It does not define Ken.
`spec/` is the sole normative authority. A module may summarize a rule for
use, but its Authority and sources section points to the applicable spec and
current checked artifacts. Implementation sources and tests establish only
the as-built behavior they exercise.

When a module, checked artifact, and spec appear to disagree:

1. do not reconcile them by invention;
2. record the exact conflicting claims;
3. prefer no capability claim over an unsupported capability claim; and
4. request review of the discrepancy.

## Product context versus workflow

Product context includes syntax, proof boundaries, CLI behavior, packages,
effects, capabilities, and diagnostics. Workflow instructions include who
reviews a change, how a branch is published, or which agent acts next. The
former belongs here; the latter does not.

The thin in-repository workflow trigger remains
`agent/playbooks/tools/write-ken.md`. It selects these packs for Ken product
facts while preserving the repository's own workflow.

## Integrity and evaluation

`schemas/agent-manifest.schema.json` and `schemas/pack.schema.json` are the
only schemas, because those are the two controlled manifest formats. The test
target exercises detector behavior through planted fixtures. It:

- exercises the shipped schema constraint classes and fails on an unsupported
  schema keyword;
- rejects a planted pack include whose module is absent and a planted circular
  pack dependency; and
- rejects planted duplicate pack and task IDs and repository-escaping paths.

The test target does not currently apply the agent-manifest, source,
token-measurement, contract-section, or resolved-closure helpers to the live
agent-library corpus.

`evaluations/README.md` defines the seven cold-context tasks. Results report
correctness, unnecessary loads, invented syntax or capabilities, and cited
authority separately. Any invention fails the suite, regardless of the other
scores.
