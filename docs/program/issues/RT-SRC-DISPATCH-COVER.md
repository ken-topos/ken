---
id: RT-SRC-DISPATCH-COVER
title: "close the source-machine scrutinee-dispatch coverage tier surfaced by RT-SPLIT slice 4"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: [RT-SPLIT]
blocks: []
github: null
origin: RT-SPLIT slice-4 pre-cut mutation sweep (runtime-implementer, evt_6q3575w03781s); scope call ruled separate-follow-up by runtime-leader; adversary scope correction evt_46wsryq6n92a2
---

**Pre-existing gap made visible by the split, not created by it.** RT-SPLIT
slice 4 moves `lowering::core`; the pre-cut mutation sweep measured what the
suite actually reaches over exactly that surface and found a hole. The split
does not worsen it, and closing it is not a slice-4 rider — bundling would
break §10.5's "one production module plus its tests" and make the move-purity
diff unreadable, which is the property the whole series rests on.

> ### ⛔ SIZE IS DELIBERATELY `TBD` — DO NOT SIZE FROM THE NUMBER BELOW
>
> The measurement is `-p ken-runtime --lib`. **`--lib` is not the workspace
> suite.** Until the two probes in §3 report, the defensible claim is exactly
> *"nine methods unreached by the complete `ken-runtime` lib suite"* —
> **neither "workspace-uncovered" nor a nine-program follow-up size is
> established** (Architect, `evt_78bsx6d7yyf9b`). Size this **after** §3.

## 1. What was measured

Each of the 29 ruled SCC methods got `panic!("MUTANT_<name>")` as its first
statement → rebuild → full `-p ken-runtime --lib` → restore. `panic!` diverges,
so it type-checks at any return type. **20 reached, 9 unreached.**

The oracle was validated before being trusted, on both axes:

- **Discriminating:** neutering `lower_primitive_call` turned **58 of 301**
  tests red — it does not pass under its own mutation.
- **No panic absorption:** `should_panic` / `catch_unwind` occurrences in
  `ken-runtime/src` = **0**, so an unreached verdict is not a swallowed unwind
  (adversary, independently).
- **`--lib` is that crate's whole suite:** `crates/ken-runtime/tests/` does not
  exist, so the *"301-test lib suite"* phrasing is exact for `ken-runtime`.

## 2. The nine, and why the shape is the finding

```
lower_recursor_residual_call
lower_unary_recursive_nat_fold
lower_source_bounded_nat_match                    ┐
lower_source_dynamic_bool_match                   │
lower_source_dynamic_host_result_match            │ the source-machine
lower_source_dynamic_constructor_match            │ MatchScrutinee
lower_source_nested_dynamic_constructor_match     │ dispatch tier
lower_source_planned_dynamic_constructor_match    │
lower_source_declaration_call                     ┘
```

`lower_source_machine`, `_with_continuation`, `_with_continuation_inner` and
`source_call_state` each go red on **exactly the same 8 tests**. So the source
machine is entered, driven and drained by real tests, and **not one of them
ever dispatches a match through it.** The entry tier is covered; the tier one
step below it is untouched.

**Closeable, not unreachable — checked rather than assumed.** The slice-3
`Table` "refuse to manufacture" precedent was available and does **not** apply.
The entry is single-sited: `lower_computational_match_value_composed` calls
`lower_source_machine` only when `!case.recursive_positions.is_empty()`, and
the machine then evaluates `case.body`; `MatchScrutinee` is built when that
body contains a `Match`. **The gap is precisely: no test's recursive case body
contains a match.** The tell is that
`recursive_computational_host_result_keeps_established_dynamic_lane` drives a
`HostResult` *through* the machine while `lower_source_dynamic_host_result_match`
stays unreached — the values are already in flight; nothing constructs the
match over them. This is an extension of a working fixture, not a new harness,
and any test written here would fail under its own mutation.

## 3. ⛔ Gating measurement — run before sizing

A private method is never *named* in an integration test, so it is invisible
both to a grep and to a `-p ken-runtime --lib` run while still being fully
exercised by a real `.ken` program in CI. `crates/ken-cli/tests/` holds 26
files, at least 10 driving `build_native_program` /
`emit_process_entrypoint_object` — the same private lowering path.

Two filenames match unreached methods closely enough to rule out first. **This
is a hypothesis from names, not a measurement** (adversary's own framing) —
do not treat these as reached until probed:

```
scripts/ken-cargo test -p ken-cli --test px7m_hostresult_computational_match
   ~ lower_source_dynamic_host_result_match
scripts/ken-cargo test -p ken-cli --test px8l_recursive_decl_native
   ~ lower_source_declaration_call
```

Same `panic!` probe, those two methods only. **Two mutations, two named
suites, no `--workspace`** (COORDINATION §12).

Either outcome sharpens the scope: reached-in-CI narrows the caveat to
*"unreached by the lib suite, reached in CI"* and shrinks this WP; not-reached
strengthens the finding from one crate's lib target to the workspace.

## 4. Why this does not affect RT-SPLIT's acceptance

Slice 4 is a **pure move**, and **AC-3** — ordered item-level identity **plus**
removed-line closure — establishes text preservation **without reference to
coverage at all**. A corrupted dispatch arm among these nine is caught by AC-3
precisely where the suite provably cannot catch it. The residual risk a move
actually carries is wiring (imports, module paths, visibility), which surfaces
as compile failure rather than silent drift.

⚠ **Do not cite this as "AC-3b."** The closure clause is the second half of
**AC-3 itself** (`docs/program/wp/rt-split-cranelift-backend.md:232`, clause at
`:237`). The halves were fused deliberately, after ordered identity alone was
found to be a **presence** check — blind to deletion by construction. Splitting
them in the retelling is the first step back to citing the weak half alone.

## 5. Acceptance

1. **⛔ Corpus-wide reachability is measured before anything is authored.**
   §3's two probes **have now run** (runtime-implementer, `evt_6mqtdc9p5bazy`):
   `px7m_hostresult_computational_match` **2/2 green** with
   `lower_source_dynamic_host_result_match` neutered;
   `px8l_recursive_decl_native` **3/3 green** with
   `lower_source_declaration_call` neutered. **Both stay unreached.**

   > **That is 2 of 9 against 2 named suites. It clears nothing for the other
   > 7 against the rest of the `ken-cli` corpus** — the author's own scoping,
   > and the reason this WP is still `size: TBD`. The original finding was
   > *"scoped to the lib suite in wording and unscoped in implication"*; do
   > not let the two confirmations be read as confirming the set. **Probe all
   > nine against the full `ken-cli` test corpus, one targeted suite at a
   > time, and report per-method — never `--workspace`.**
2. Each remaining unreached method gets a test whose recursive case body
   contains a match over the relevant `Lowered` shape.
3. **Every added test fails under its own `panic!` mutation** — the same oracle
   discipline that produced the finding.
4. Tests land in the ruled subject modules per §10.2's assignment-by-subject.
5. Closed against **post-split** code if RT-SPLIT has landed by then; against
   pre-split code otherwise. Either way the tests must pass on the tree they
   ship into.
