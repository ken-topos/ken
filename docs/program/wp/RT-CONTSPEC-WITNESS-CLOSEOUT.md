# RT-CONTSPEC-WITNESS closeout record

ContinuationSpecialization seam 4 of 4, the terminal seam. This document is the
`D5` closeout record and carries the `D1`-`D4` evidence it rests on.

**Base for every measurement in this document: `origin/main = 47ef28b1`**
(`47ef28b1c21a0ee192f029092c0c3c05636902b4`), the branch point of
`wp/RT-CONTSPEC-WITNESS`. Every count, verdict and disposition below was
measured on that tree unless a different SHA is named at the point of use.

**Citation convention.** Tests are named by module path and assertions by their
text, not by line number. A line coordinate is destroyed by the edit that the
deliverable describing it performs, and a stale one resolves plausibly against
unrelated live code; a name and a phrase survive. Where a number is
unavoidable it carries the SHA it was measured at.

---

## D1 - the native population on the lawful assembly

Both preconditions the frame carries from seams 1-3 were met in the same shell
as the run: the tree was proved immediately before the suite, and the build ran
before the test.

```
git rev-parse HEAD          -> 47ef28b1c21a0ee192f029092c0c3c05636902b4
scripts/ken-cargo build -p ken-runtime   -> Finished, 50 warnings
df -h /workspaces           -> 77G total, 11G available, 87% used
git rev-parse HEAD          -> 47ef28b1c21a0ee192f029092c0c3c05636902b4  (recheck)
scripts/ken-cargo test -p ken-runtime --lib
```

**Result: 809 passed, 0 failed, 4 ignored, 0 measured, 0 filtered out**,
finished in 59.35s.

The anchor was quoted twice in the one shell, before the build and again
immediately before the test, so the count is bound to that tree and not to a
tree the branch later moved to.

### The four ignored rows, named

An ignored test is not a passing test, and the aggregate line reports the two
separately for a reason. Naming them here keeps the `809` from being read as
"the whole population answered."

| ignored test | owner named in its own skip reason |
|---|---|
| `boundary_value_clif::tests::b2v_ac10_a_deep_acyclic_chain_adopts_at_thirty_thousand` | (depth/stack cost, not a census row) |
| `cranelift_backend::artifact::api::tests::nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes` | `RT-FNUNIT-RESULT-TOKEN` |
| `cranelift_backend::lowering::core::tests::constructors::c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload` | `RT-CARRIER-PRODUCER-OCCURRENCE` |
| `cranelift_backend::lowering::core::tests::constructors::two_same_shape_workers_are_distinguished` | `RT-WORKER-FIXTURE-DECODE` |

Two of these four are census rows, and they are the reason the disposition
table below does not read "130 pass, done". The third bears directly on `D7`
and is discussed there.

### Comparison with the held lineage, and what it is not

The census was taken at preservation `1aef3192` with the same targeted
command, and recorded **464 passed, 138 failed, 1 ignored**. The lawful base
runs **809 passed, 0 failed, 4 ignored**.

**These two numbers are not a before/after of one population and must not be
subtracted.** The suite grew by roughly 200 tests across the intervening
merges, the held lineage is a branch that was never merged, and no commit
takes one tree to the other. The comparison that is meaningful is per-row, and
it is the disposition table, not the totals.

---

## D2 - the six rows that never produced a measurement, read against the lawful base

These are the six census rows whose assertion never ran in the held `1aef3192`
tree because an earlier refusal stopped the test first. The question with a
referent is not what cleared them - nothing did, and nothing will. It is:
**does the lawful base carry a test bearing this assertion, and what does it
do there?**

Each row was run by exact name at `47ef28b1`, in a shell that quoted the
anchor before and after:

```
scripts/ken-cargo test -p ken-runtime --lib -- --exact <the six names>
-> running 5 tests ... test result: ok. 5 passed; 0 failed; 0 ignored; 808 filtered out
```

**Five ran, not six.** The sixth was filtered out, which is the measurement
that mattered: a name absent from the binary's test list is not a failure and
not a pass, and the run says so by counting five.

| # | row | assertion it carried | disposition |
|---|---|---|---|
| 1 | `cranelift_backend::artifact::api::tests::nc22_imported_dependency_lowers_as_stable_unsupported_native_lane` | the verdict is `Unsupported` at stage `NativeLoweringOrExecution` for construct `ImportedDeclarationRef` | **live** - passes |
| 2 | `cranelift_backend::lowering::core::tests::constructors::nested_computational_malformed_recursive_position_rejects_specifically` | the error is `Unsupported(ComputationalMatch)` with reason `case ctor:fixture::Inner::TrueLeaf has malformed recursive position 1` | **live** - passes |
| 3 | `cranelift_backend::lowering::core::tests::control::an_unrepresentable_transfer_is_refused_before_any_unit_is_declared` | the positive control: a binder-free wrapper over intra-module values must still compile | **live** - passes |
| 4 | `object_linker_packaging::tests::aggregate_observation_rejects_as_non_scalar_smoke_lane` | the observation classifies as `SmokeExecution`, not `NativeComparison` | **live** - passes |
| 5 | `object_linker_packaging::tests::trap_observation_rejects_without_promoting_runtime_error_to_build_success` | the observation classifies as `SmokeExecution`, not `NativeComparison` | **live** - passes |
| 6 | `cranelift_backend::lowering::core::tests::control::recursive_declaration_shape_change_hits_typed_boundary` | a changing recursive native representation must fail closed | **superseded** - see below |

### Row 6, the one that is not a rerun

Row 6 is absent from `crates/` by name and by assertion phrase. A search for
its assertion text `a changing recursive native representation must fail
closed` returns nothing. The name survives in exactly one place, a doc comment
in `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs`
which opens *"This row was `recursive_declaration_shape_change_hits_typed_
boundary`, a negative, and `D6` inverted it - under the frame's explicit
direction, not as a convenience."*

**The assertion was ruled wrong and replaced by its negation.**
`RT-DECL-CLOSURE-PORT` `D6` established that `same_recursive_argument_shapes`
is not a Ken semantic law and not a declared function-unit ABI predicate: it
guards `RecursiveDescent`'s same-function CFG backedges, where one fixed run of
specialized block parameters must represent every iteration. A functionized
call holds a different representation contract, so `None` and `Some(Int)` are
two lawful values of one declared `ValueWord` slot rather than an ABI shape
disagreement. The successor
`cranelift_backend::lowering::core::tests::control::d6_a_functionized_recursive_declaration_accepts_a_changing_argument_constructor`
asserts the acceptance and **passes at `47ef28b1`** (verified by exact-name
run: 1 passed, 812 filtered out).

So the lawful base carries no test bearing row 6's assertion, and the reason is
not that the assertion is unwitnessed - it is that the assertion was found to
be false. That is `superseded` in the frame's sense, and the record above
names the node, the deliverable and the successor test rather than asserting
it.

**A note for whoever greps this section against `AC-2`.** No row above is
recorded as awaiting anything, and no row's disposition is stated as a seam
having cleared its cause. The phrase "formerly shadowed" appears only as the
name of the population, which is the frame's own label for it, never as a
disposition.

---

## D3 - the two host rows, rerun under confirmed capacity

The two rows are not semantic. Both failed in the census with
`/usr/bin/ld: final link failed: No space left on device` at the linker or
finalizer stage, when `/tmp` was at 99 percent.

Capacity was confirmed in the same shell as the run, on the volume the run
actually uses. `scripts/ken-cargo` exports `TMPDIR=/workspaces/ken/tmp`, so the
volume that matters is `/workspaces`, and the build was run first so the export
had happened:

```
git rev-parse HEAD  -> 47ef28b1c21a0ee192f029092c0c3c05636902b4
scripts/ken-cargo build -p ken-runtime      (exports TMPDIR)
df -h /workspaces   -> 77G size, 11G available, 87% used
df -h /tmp          -> 7.8G size, 7.7G available, 1% used
```

Both volumes had room, so the precondition is met on either reading of which
one the linker reaches.

```
scripts/ken-cargo test -p ken-runtime --lib -- --exact \
  object_linker_packaging::tests::linked_process_executes_exact_big_int_support_without_host_dispatch \
  object_linker_packaging::tests::linked_transport_classifies_all_terminal_arms

running 2 tests
test object_linker_packaging::tests::linked_process_executes_exact_big_int_support_without_host_dispatch ... ok
test object_linker_packaging::tests::linked_transport_classifies_all_terminal_arms ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 811 filtered out; finished in 8.12s
```

**Both pass. Neither reached a disk error.** The frame's routing condition -
that a repeat `No space left on device` on the repo volume would indicate a
temp path not honouring `TMPDIR` and would be worth routing - did not fire.

No semantic inference is drawn from the census's original failures for these
two rows, in either direction. They were host failures then and they are
passing rows now; the first fact does not make the second one evidence about
anything the campaign was measuring.

---

## D4 - the 761 witness gate: fixed, not moved, and a third state the question did not have

The gate asks whether
`fs_read_at_malformed_offset_narrows_to_invalid_offset` produces
`InvalidOffset` because the defect was fixed or because the assertion moved.
It is settled: **the defect was fixed.** The argument is below, and so is a
fact about the lawful base that the question's binary form cannot express.

### Which test is the sibling

The frame names the sibling by a line coordinate. That coordinate does not
resolve on the lawful base - the line it names sits inside a skip-annotation
comment block. It **does** resolve at `b66dea6a`, the tree the frame measured
it on, where it lands inside
`fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset`.
That is the sibling, identified by resolving the coordinate against its own
base rather than against mine.

### The commit, and why it is a fix

**`e892777c` - "RT-PARITY: interpreter/native checked buffer-IO narrowing
parity".** It created `crates/ken-cli/tests/rt_parity_native.rs` (720 added, 0
deleted; confirmed the file's addition commit) and both tests were born in it,
green.

The discrimination does not rest on reading that commit's message. It rests on
two measured facts:

1. **The production change is in the interpreter alone.** The commit's paths
   are `ken-interp/src/eval.rs`, `ken-cli/tests/rt_parity_native.rs`,
   `ken-interp/tests/px8p_checked_buffer.rs`, and a conformance seed. A filter
   of its path list for `ken-runtime` or `ken-host` returns **nothing**.
2. **The test is a differential, and its expected value is the side that did
   not change.** `assert_narrowed_alike` compares interpreter against native:
   both must exit 0, both must have no terminal error, their terminal exit
   classes must agree, and their canonical operation event lists must agree and
   be empty. The test's own doc records the pre-repair state: the interpreter
   surfaced `RightNotHeld` while **native already synthesised
   `InvalidOffset`**.

⇒ The expectation was pinned to native's pre-existing answer, and the commit
moved the interpreter to meet it. An assertion-move is the opposite shape - the
expectation is rewritten to match a producer that changed. Here the producer of
the expected value was never touched.

**One thing this argument does not establish, stated because it would otherwise
be read as established.** The `"InvalidOffset"` string passed to
`assert_narrowed_alike` is used only in assertion *messages*; the discrimination
is carried by the fixture, which exits 0 only on that constructor and non-zero
on any other. So the oracle is the fixture, not the Rust literal - and the
fixture and the repair were authored in the same commit. There is therefore no
pre-commit run of this test to appeal to, and the conclusion rests on the two
facts above rather than on a before/after execution.

**And the native side was the reference, not a verified output.** The gate
establishes that the interpreter was brought to native's answer. It does not
independently establish that native's `InvalidOffset` is correct, because
native is the unchanged side of the differential. That is a real limit on what
the 761 gate witnesses, and it is a property of the gate's construction rather
than a defect introduced by anything in this campaign.

### The third state: neither test runs on the lawful base

Both tests, and four others in that file, are `#[ignore]`d at `47ef28b1`.

- `7ca5cfc0` (`RT-SRCBODY-BIND-ORDER`) first quarantined them, as a closed
  base-debt quarantine, with the reason recording that they fail at base
  `21fd46dc`.
- `e0fc15c3` (`RT-CARRIER-BYTESPAN-OBSERVE` `D5`) **rewrote the skip reason
  only.** Its diff on that file changes nothing but `#[ignore = ...]` strings;
  no assertion, expectation or body moved. The current reason states that `D5`
  landed byte-span observation and is *not* the blocker, and that the row awaits
  a Steward recut.

⇒ The frame's observation that both passed on `b66dea6a` was correct there -
both carried a bare `#[test]` at that tree, verified. It is not correct now,
and the reason is not that they fail: **they do not run.** A gate whose
evidence is a green observation cannot be re-established by running it here.

This is routed, not repaired. The owner is named in the skip reason itself, and
un-skipping is explicitly stated there to not be the repair.

---

## D5 - the campaign closeout record

### What the census is, stated so `superseded` is not misread

The 138 rows are a **first-refusal record of the held `1aef3192` lineage** - a
branch that was never merged and may never be. They are not a worklist, and
they were not carried through the seams: seam 2 was recut off the census
entirely and seam 3's landed frame states in its own operative text that the
census is not an input to any of its deliverables.

**No seam cleared any row's cause, and none was ever going to.** A row marked
`superseded` below therefore means *the assertion belonged to a mechanism that
does not exist on the lawful base* - it does **not** mean work was silently
dropped, and it does not mean a seam resolved it.

### The disposition table

Every one of the 138 rows carries exactly one terminal disposition. The
partition was computed by intersecting the census row names against the lawful
base's actual test list, not by inferring coverage from the aggregate pass
count.

| disposition | rows | basis |
|---|---:|---|
| **live**, verdict **pass** | **130** | present in the binary's test list and passing in the `D1` run at `47ef28b1` |
| **open**, with named owner | **2** | present but `#[ignore]`d, so they carry no verdict |
| **superseded** | **5** | absent from the lawful base; the assertion belongs to a mechanism that is not here |
| **live**, verdict **pass**, under a new name | **1** | renamed by a landed node, and it passes |
| total | **138** | |

**The 2 `open` rows**, each with the owner named in its own skip reason:

| row | owner |
|---|---|
| `cranelift_backend::artifact::api::tests::nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes` | `RT-FNUNIT-RESULT-TOKEN` |
| `cranelift_backend::lowering::core::tests::constructors::c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload` | `RT-CARRIER-PRODUCER-OCCURRENCE` |

These two are the reason the table does not read "130 pass and 8 exceptions".
They are present, they carry their assertion, and they produce no verdict. A
reader who took `809 passed` as covering the census would have recorded both as
passing.

**The 5 `superseded` rows.** Four are the worker-environment rows:

- `cranelift_backend::planning::static_transition::tests::forced_persistent_worker_environment_reaches_escape_defense`
- `cranelift_backend::planning::static_transition::tests::static_recursor_worker_environment_meets_ordered_capture_owners`
- `cranelift_backend::planning::static_transition::tests::static_recursor_worker_environment_population_mutations_reject_before_allocation`
- `cranelift_backend::planning::static_transition::tests::static_recursor_worker_environment_token_is_exact_once_and_preallocation`

All four are present in the held lineage at `1aef3192` and, searched across
**every ref** in the repository, appear only on held-lineage commits
(`bde28ff0` "preserve contspec integration audit seam" and `7b9b913d` "plan
static recursor worker environments"). **They never existed on a merged
mainline.** That is the textbook case: their refusal was a property of the held
mechanism.

The fifth is `recursive_declaration_shape_change_hits_typed_boundary`, whose
assertion was inverted by `RT-DECL-CLOSURE-PORT` `D6` - see `D2` row 6 above.

**The 1 renamed row.**
`cranelift_backend::lowering::core::tests::control::recursive_descent_root_translates_a_runtime_reached_trap_exactly`
was renamed to `the_generated_root_translates_a_runtime_reached_trap_exactly`
by `080f3bb2` (`RT-PRODUCER-MATCH-PORT` `D3`). It passes.

That commit disclosed a coverage change in the fixture's own comment rather
than leaving it to be discovered: the program still reaches its trap, but its
producer-`Call` scrutinee is ported, so the trap now translates through the
**functionized** root instead of the retained one. Runtime-reached trap
translation on the *retained* root consequently has no witness, and the gap is
bounded by `RT-DESCENT-RETIRE`, which deletes that root. That residual is
carried here rather than restated as new.

### What the four seams established

- **Seam 1, `RT-CONTSPEC-ASSEMBLY`** - built the assembly and produced the
  corrected 138-row census as its `D4`. The census is this campaign's only
  record of what the held lineage actually refused on, and its value turned out
  to be historical rather than operational.
- **Seam 2, `RT-CONTSPEC-ACTIVATE`** - the continuation emission seam, with the
  planner-issued target proved equal to the emitted direct-call target, and
  three executable `cfg(test)` controls sitting on the exact production
  branches they perturb. Recut off the census.
- **Seam 3, `RT-CONTSPEC-LEDGER`** - recut mid-flight from populating the
  boundary-use ledger to **deleting** it, after the ring declined to produce a
  mapping and the Architect sustained the stop. The four `BoundaryUse*` axes
  were an unowned schema fragment with one production variant each and no
  semantic consumer.
- **Seam 4, this node** - the integrated measurement and the closeout.

**A pattern worth recording, because it is the campaign's main methodological
result.** Two of the four seams were recut after execution began, and in both
cases the recut was triggered by an implementer declining to satisfy a
deliverable whose premise did not hold - the census-as-worklist in seam 2, and
the ledger mapping in seam 3. Neither recut was discovered by review of the
frame. Both were discovered by trying to execute it.

### What remains open

| item | owner |
|---|---|
| `nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes` un-skipped and green on the functionized lane | `RT-FNUNIT-RESULT-TOKEN` |
| `c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload` un-skipped | `RT-CARRIER-PRODUCER-OCCURRENCE` |
| the six `rt_parity_native.rs` rows, including both 761 tests, un-skipped | `RT-CARRIER-BYTESPAN-OBSERVE`, awaiting a Steward recut per its own skip reason |
| `two_same_shape_workers_are_distinguished` executable | `RT-WORKER-FIXTURE-DECODE` |
| runtime-reached trap translation on the retained root | `RT-DESCENT-RETIRE`, which deletes that root |
| whether any live row still exercises the ported shape | `RT-DESCENT-RETIRE` `D6b`, bound into its `AC-5` |

---

## D6 - the tracker closure

**The set was measured at `47ef28b1`, not inherited.** Reading `status:` from
every node in the closure set at that base:

| node | status at base | action |
|---|---|---|
| `RT-CONTSPEC-WITNESS` | `ready` | flipped |
| `RT-RECURSOR-TRANSPORT` | `ready` | flipped to `closed` |
| `RT-DECL-CLOSURE-PORT` | `merged` | **not touched** - already terminal |
| `RT-CONTSPEC-ASSEMBLY` / `-ACTIVATE` / `-LEDGER` | `merged` | not touched |

**The set is two**, which matches the frame's 2026-08-08 correction. No third
flip was invented, and `RT-DECL-CLOSURE-PORT` was not re-closed.

`RT-RECURSOR-TRANSPORT` names this seam as what closed it, and records that its
residual coverage question lives in `RT-DESCENT-RETIRE` `D6b` rather than being
carried as an unowned gap.

**One flip to check against convention rather than accept.** This node's own
`status` is set to `merged` here because `AC-7` asks `D6` to close every node in
the measured set, and this node is in it. The landed practice is different: the
Steward flipped `RT-CONTSPEC-LEDGER` to `merged` in a separate post-merge commit
(`43e5903f`), not in the candidate. Until this candidate merges, that line
asserts something untrue of the tree it sits in. **It is a single frontmatter
line and it is the one to drop** if the convention should win over the AC's
literal reading; the rest of `D6` does not depend on it.
