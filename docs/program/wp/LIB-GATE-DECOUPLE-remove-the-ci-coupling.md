# WP `LIB-GATE-DECOUPLE` — remove the library documentation gates' CI coupling

**Node:** `docs/program/issues/LIB-GATE-DECOUPLE.md` · **Owner:** verify ·
**Size:** S · **Gate:** none · **Blocks:** `KW-ORACLE-REMOVE`

> ## ⛔⛔ `main` IS RED AND YOU ARE THE FIX. Read the node first.
>
> An Architect-approved candidate (`68c3d870`) is queued behind you, and every
> non-doc-only merge in the fleet is blocked. ⭐ **The node carries measurements
> and one named trap — read it before you classify anything.**

## Objective

`crates/ken-cli/tests/library_documentation_gates.rs` asserts facts about the
**live repository's documentation**: cited-source attestation currency, manifest
coverage, and measured token counts. Per the operator, 2026-07-26 —
**"remove the CI coupling."**

Two standing rulings converge here. On the gate: *"no remove it. it's just
friction. we can generate such a document at version release points. Including
it as a CI-type system induces coupling that causes just the sort of slowdown and
waffling that we're dealing with now."* On the class: *"Test oracles that assert
facts about source code, catalog, or documentation lines are an invitation for
failure and delay. Tests should focus on behavior."*

## Scope

**In:** remove, from that file, every test whose verdict depends on the **content
of the live tree's documentation**.

**Keep:** every test that builds a **scratch fixture** and checks the
generator's *behaviour* on it. ⭐ Those are behavioural tests that merely read the
tree to seed a fixture — they are what the operator's policy asks for, not what
it prohibits.

**Out — ⛔ hard stops, route to Steward rather than deciding:**
- ⛔ **Do not run `scripts/gen-source-attestations.sh` or `gen-doc-status.sh` to
  make anything green.** That was the rejected option and it would assert a
  corpus revalidation that never happened.
- ⛔ **Do not delete the generator scripts.** They are kept, to be run at version
  release points.
- ⛔ **No `src/` change**, and no edit to `library/`, `catalog/`, `docs/`, or
  `agent/`. If you believe a manifest or ledger must move, stop and route it.
- ⛔ **Do not delete the whole file** unless your own measurement shows every
  test in it is coupled. If it does, say so with the evidence and route it — do
  not assume it.

## ⛔ THE TRAP — it cost the Steward two wrong answers, do not repeat it

I classified this file twice with a static call-graph script. **Both runs
mis-classified `registered_record_validation_gates_run` as fixture-only** — the
test whose CI failure had *already named it*. Its whole body is:

```rust
#[test]
fn registered_record_validation_gates_run() {
    for gate in VALIDATION_GATES { (gate.run)(); }
}
```

It reaches the live tree through an **11-row static function-pointer table**, so
there is **no textual call site** for a classifier keyed on `name(` to find.

⛔ **This is not a regex bug. A static call graph is structurally blind to
indirect dispatch**, and this file dispatches that way on purpose. ⭐ I caught it
only because CI had handed me a case whose answer I already knew.

## `AC-1` — the coupled set is measured BEHAVIOURALLY, not inferred

⭐ **Use the operator's own rule as the instrument, because it is executable:**

> *"Does an edit that changes nothing about how any program behaves make this
> test fail?"*

In a scratch copy of the tree, run the suite three times:

| run | perturbation | expected |
|---|---|---|
| **baseline** | none | the 2 known failures below, nothing else |
| **P1** | change a line in a **cited catalog source** | coupled tests flip red |
| **P2** | change a line in a **`measured_tokens`-bearing agent module** | coupled tests flip red |

**Report the three verdict sets and the diff between them.** The tests that flip
are the coupling. ⛔ A test you *believe* is coupled but that never flipped is
**not** measured — say which basis you are on for each one.

⚠ **⛔ REPORT THE COUNT AS A FLOOR.** A test coupled to some *third* doc-shaped
input that neither perturbation touches will not flip, and your set will look
complete when it is not. ⭐ **Name the two perturbations you ran** so a reader can
see exactly which surface your number covers.

## `AC-2` — the two confirmed-coupled tests are gone

Measured by the Steward at `origin/main = 11b21039`, a tree containing **no**
candidate:

```
scripts/ken-cargo test -p ken-cli --test library_documentation_gates
test result: FAILED. 29 passed; 2 failed
```

| test | why it fails today |
|---|---|
| `registered_record_validation_gates_run` (`:636`, panics at `:1048`) | shells `gen-doc-status.sh --check`; **12 cited sources** drifted from their attestations |
| `agent_library_manifest_schema_contract_and_measurements_hold` (`:3356`) | `library/agents/tasks/author-package.md` declares `measured_tokens` **480**, recomputes **459** |

⛔ **These two are the floor, not the scope.** Removing only them satisfies
nothing — the next documentation edit reopens the node.

## `AC-3` — the kept set is stated, with its reason

List every test you **kept**, and for each, one clause saying what *behaviour* it
verifies. ⭐ This is the AC that makes the removal auditable: a reader must be
able to see you decided, rather than stopped when CI went green.

⚠ **Known-good starting point, ⛔ not a permission to skip `AC-1`:** these 7
showed no live-tree read on any static path —
`invalid_kind_detector_rejects_case_variant`,
`status_record_population_detector_rejects_second_status_record`,
`field_lines_inside_open_arrays_detects_the_reported_shape`,
`slugify_matches_the_proposals_own_worked_anchor`,
`agent_pack_integrity_rejects_missing_modules_and_cycles`,
`agent_key_space_detectors_reject_duplicate_pack_and_task_ids`,
`checked_examples_detector_rejects_invalid_example_and_stale_reject`.
⛔ **That list came from the instrument the trap section says is unreliable.**
Confirm each behaviourally or drop the claim.

## `AC-4` — the crate is green, targeted only

```sh
scripts/ken-cargo test -p ken-cli --test library_documentation_gates
scripts/ken-cargo test -p ken-cli
```

⛔ Never `--workspace`; workspace-green means **green in CI**
(`COORDINATION §12`). ⭐ Report before/after for both, and state the test **count**
each time — a count that drops by more than your removal list is a build
problem, not a cleaner suite.

## `AC-5` — the residual is stated in the handoff

After this lands, between version releases a `library/` page may cite a source
that has since changed and **nothing will report it**. ⚠ That is the accepted
cost of the decoupling. ⛔ Do not report it as a defect, and ⛔ do not propose a
replacement checker — not in this WP, not as a follow-on you add yourself.

## Evidence bar

- ⛔ **`git diff --stat` always exits 0** — it is not an emptiness test. Use
  `--quiet` or read `--name-only`.
- ⭐ **A green suite after a removal is not evidence the removal was right.**
  Deleting a failing test always turns the suite green; `AC-1`'s perturbation
  runs are what distinguish *"removed the coupling"* from *"removed the
  messenger."*

## Handoff

Return **one exact candidate SHA** with the branch freed, plus the `--name-only`
diff, the three `AC-1` verdict sets, the kept-set table, and the before/after
counts. ⛔ **No Decision is opened by the ring** — that is the Steward's.
