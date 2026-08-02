# WP frame — `DOC-W6-AGENT-EVAL` (Wave 6 residual)

Node: `docs/program/issues/DOC-W6-AGENT-EVAL.md`. Program:
`docs/program/12-documentation-program.md` §Wave 6. Owner: doc ring.
Authority: the Wave 6 disposition note in that section, measured at
`origin/main = 5a0fd8e6`.

`library/agents/` exists so a cold agent can work in Ken from a selected pack.
`library/agents/evaluations/` is the only thing that establishes it does.
**That evidence is now stale, and this WP measures how stale.**

## The four judgments this frame makes, so you do not have to

### 1. This WP does NOT build a currency mechanism

`f52b0f61` removed the library currency gate by operator ruling — "both call
sites: the pre-merge gate and *the post-merge alarm*." `LIB-GATE-DECOUPLE`
(`f84e4804`) removed live documentation/content CI coupling, and the resulting
policy **explicitly accepts that source attestations drift between release
points.**

Do not build a gate, an alarm, a ledger change, or any CI coupling. The
evaluation suite is a **behavioural measurement run deliberately by a person**,
and that is the only shape authorized here.

### 2. A `false` verdict is a RESULT, not a regression to repair

`agent_core_ready(run)` may come out false against the current corpus. If it
does, **record it and stop.** Do not re-run a task until it goes green, do not
edit a core module to rescue a verdict, and do not soften a task's expected
property.

The suite's own rule is that any invention fails the whole suite regardless of
other results. **Preserving a false verdict is the most valuable output this WP
can produce** — it is the only signal that the corpus grew past what the packs
certify.

### 3. Cold fixtures are CONSUMABLE — reconcile before you spend them

`cold(seat, run)` requires a seat that has not seen the fixture, the expected
result, or any earlier result from the suite. A fixture shown to a seat is
**burnt**, and the protocol's remedy is a held-back semantically equivalent
variant with a new identifier — a finite supply.

⇒ **Do not re-run all seven tasks.** `D1` establishes which tasks the corpus
drift can actually affect, and `D3` re-runs only those. A task re-run without a
stated reason spends a fixture for no measurement.

### 4. The citation-authority question is the highest-value thing here

At the last run, `library/reference/catalog/` did not exist. It does now — 39
derived cards and five indexes over the same packages the tasks ask about.

§4c: **no `library/` page is normative.** So an agent that cites a card where a
normative source exists has produced a well-formed, plausible, **wrong**
citation. `cited_authority = "complete"` must not be recorded for a citation
that names a derived page in place of its normative source.

⇒ **`D2` settles what the correct authority is per task before any re-run**, so
the scoring is decided in advance rather than argued after a result is in hand.

## Fixed inputs

Measured at `origin/main = 5a0fd8e6`.

| input | measured value |
|---|---|
| the run of record | `library/agents/evaluations/results-2026-07-24.toml`, committed `d3b9f36c` (2026-07-25 01:14), `final_suite_ready = true` |
| the protocol | `library/agents/evaluations/README.md` — exit predicate, cold precondition, four axes |
| the tasks | 7, in `tasks.toml`, each naming its pack |
| the packs | 6, under `library/agents/packs/`; `read-review` selects `core/read-ken`, `core/proof-and-trust`, `core/toolchain`, `tasks/read-review` |
| `library/` growth | 26 markdown files then, 89 now; `library/reference/` contributes 54 |
| core modules changed since | 3 of 4 — `proof-and-trust.md`, `toolchain.md`, `write-ken.md` |
| the one prior failure | `write-effectful-boundary-1`: `partially-correct` / `partial`, repaired on a held-back variant as `write-effectful-boundary-2` |

```sh
git rev-parse HEAD
git diff --stat d3b9f36c HEAD -- library/agents/
git diff --name-only d3b9f36c HEAD -- library/reference/ | wc -l
```

## Deliverables

- **D1 — the drift reconciliation.** For each of the 7 tasks: at-risk or
  not-at-risk for the current corpus, with the specific change that puts it at
  risk (a changed pack-selected module, or newly reachable material its prompt
  would lead an agent to). **Every task gets an explicit disposition and a
  reason.** A not-at-risk task is a real finding and needs its reason stated as
  carefully as an at-risk one.
- **D2 — the citation-authority statement.** Per task, what counts as
  `cited_authority = "complete"` now that a derived `library/reference/catalog/`
  page exists alongside the normative source. State the rule before running
  anything. Where a task's correct authority is genuinely unclear, that is hard
  stop 3, not a judgment call to make mid-run.
- **D3 — the re-runs**, cold, for the at-risk tasks only, following the protocol
  in `README.md` exactly: record the seat identifier, cold evidence, every extra
  file load, the preserved answer, and all four axes independently.
- **D4 — a NEW dated results artifact**
  (`library/agents/evaluations/results-<date>.toml`) recording this run and
  evaluating `agent_core_ready` for the current corpus. **State the verdict
  plainly, including `false`.**
- **D5 — pack reconciliation, as a RECOMMENDATION only.** Whether any pack's
  `includes` should change given Waves 3-5, grounded in what the re-runs
  actually showed. **Recorded, not applied** — see banned scope.

## Acceptance criteria

- **AC-1 — all 7 tasks carry an explicit at-risk disposition with a reason.**
  *Control:* `D1`'s table against `tasks.toml`. A task absent from `D1` fails
  this AC — the failure it catches is a task quietly dropped because re-running
  it looked expensive.
- **AC-2 — `results-2026-07-24.toml` is byte-unchanged.**
  *Control:* its blob against the merge base. The historical run is a record,
  not a working file.
- **AC-3 — `agent_core_ready` is evaluated and stated for the current corpus**,
  with the failing axis named if it is false.
  *Control:* `D4`. **A `false` verdict satisfies this AC.** What fails it is an
  absent verdict, or a verdict reached by re-running until green.
- **AC-4 — every re-run records cold evidence and no fixture is shown to a seat
  that has seen it.**
  *Control:* seat identifiers across `D4` and `results-2026-07-24.toml`; a
  repeated pairing fails.
- **AC-5 — citation authority is scored against `D2`'s stated rule**, and any
  citation naming a derived `library/` page where a normative source exists is
  recorded as a finding rather than scored `complete`.
  *Control:* each re-run's `authority_paths` against `D2`. This is the positive
  control for judgment 4.
- **AC-6 — no currency gate, attestation ledger change, or CI coupling is
  introduced.**
  *Control:* the candidate's path list — nothing under `scripts/`, no
  `library/SOURCE-ATTESTATIONS`, no workflow file.
- **AC-7 — no pack, core module, or task file is edited.** `D5` recommends.
  *Control:* the diff touches no path under `library/agents/packs/`,
  `library/agents/core/`, or `library/agents/tasks/`.

## Banned scope

- **No currency gate, alarm, ledger regeneration, or CI coupling** (operator
  rulings `f52b0f61`, `f84e4804`).
- **No edit to any pack, core module, or task file** (`AC-7`). If a re-run shows
  a module is wrong, that is `D5` plus a routed finding, and it is the next WP.
- **No edit to `results-2026-07-24.toml`** (`AC-2`).
- **No re-run of a task to change its verdict.** One cold run per fixture.
- **No `library/releases/`**, no snapshot, no migration note — Ken has no public
  releases and the section's rule holds.
- **No HTML, offline artifact, or search index.** Deferred with the rest of
  Wave 6; see the Wave 6 note in the program document.
- **No test asserting facts about source, catalog, or documentation lines**
  (operator test policy). Every deliverable here is a measurement record.
- **No normative claim**; name the spec section instead.

## Contention

`library/` and `docs/program/` only. No `cargo`, no build lock, no contention
with the runtime ring.

## Sizing

**Size `M`.** The reconciliation and the authority statement are judgment and
cheap; the cold re-runs are the cost, and judgment 3 exists to keep that cost
proportional to what `D1` actually shows is at risk.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. `D1` drift reconciliation and `D2` citation-authority statement. **Stop here
   and let the leader see which tasks you propose to spend fixtures on** —
   burning a fixture is not reversible.
2. `D3` the re-runs.
3. `D4` results artifact and `D5` pack recommendations.

**Expect to end your turn at each checkpoint.** If any checkpoint runs past an
hour, stop and route.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **`agent_core_ready` comes out false.** Record the failing axis and the run,
   and stop. Do not repair the corpus inside this WP.
2. **A task's fixture is burnt and no held-back variant exists.** The protocol
   requires a semantically equivalent variant with a new identifier; minting one
   is a content judgment the Librarian owns.
3. **A task's correct citation authority is unclear** — the derived page and the
   normative source both look defensible. That is the Librarian's ruling and it
   belongs in `D2` before any run.
4. **A pack selects a module that does not resolve**, or a module's content
   contradicts the task's expected property. Both are content defects, not
   scoring problems.
5. **A re-run needs a file outside its pack that no cited authority or module
   prerequisite justifies.** That is an exit-predicate violation and a finding
   about the pack, not something to permit so the run can finish.
