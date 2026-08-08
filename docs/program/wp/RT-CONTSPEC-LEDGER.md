# WP frame — `RT-CONTSPEC-LEDGER` (ContinuationSpecialization seam 3 of 4)

Node: `docs/program/issues/RT-CONTSPEC-LEDGER.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ruling `evt_1v9m7t4m9dmj7` (2026-08-08), sustaining hard
stop 7 and recutting this seam. Position in the four-seam sequence is unchanged:
after `RT-CONTSPEC-ACTIVATE`, before `RT-CONTSPEC-WITNESS`.

**This seam deletes the four `BoundaryUse*` axes from the
continuation-specialization contract.** They are an unowned schema fragment: no
lowering, ABI, selection, lifetime, or emission consumer reads any of them.

> ## RECUT 2026-08-08 — THE WHOLE PRIOR CONTRACT IS RETIRED, NOT AMENDED
>
> Every deliverable and acceptance criterion in the previous version of this
> frame is withdrawn. The prior frame told you to make the four second variants
> production-reachable and to prove the distinct-tuple count moves off 1. **That
> is now forbidden, not merely unnecessary.**
>
> Architect, `evt_1v9m7t4m9dmj7`: *"Making the second variants reachable would
> manufacture semantically duplicate units and then call the duplication
> evidence that the ledger works."*
>
> **Specifically withdrawn**, so that no reader hunts for them: the old
> `AC-1` tuple-count discriminator (1 on base, greater than 1 on candidate); the
> old `AC-2` ungating requirement; the old `D2` production-reachable vocabulary;
> the old `D5` boundary-use census; the old `D7` four-axis collapse control and
> its `AC-5`. **The old `D1`-`D7` numbering is retired wholesale.** The
> deliverables below are a fresh `D1`-`D5` and do not correspond to it.
>
> The old `D1` re-measurement accepted at `5d430082` measured a base three
> merges old and a subject that no longer exists. **A fresh `D1` is owed.**

> ## SUPERSEDED — `RT-DECL-CLOSURE-PORT` `D7` IS NOT THIS SEAM'S AUTHORITY
>
> The previous frame said the boundary-use mapping would come from
> `RT-DECL-CLOSURE-PORT` `D7`, and named that node a `depends_on`. **The
> Architect superseded that claim in `evt_1v9m7t4m9dmj7`, correcting their own
> prior ruling `evt_40ra70t92mjd2`:**
>
> *"Its positive ownership claim was wrong: `RT-DECL-CLOSURE-PORT D7` did not
> owe one global boundary-use record for every continuation input. The landed
> D7 correctly derived the narrower `PlannedEffectSeat` population from actual
> host-effect consumers and explicitly kept its vocabulary separate."*
>
> `evt_40ra70t92mjd2` **remains correct in its negative parts**: no mapping
> existed, the four literals were not a classifier, and coercion into the binary
> enums was forbidden. Only its ownership assignment is withdrawn.
>
> ⇒ **There is no mapping authority to wait for, and none to build.** The
> already-merged `D7` is historical context. It has been removed from this
> node's `depends_on`. `PlannedEffectSeat` is a host-operation semantic seat,
> not a continuation ABI-slot contract; it cannot be projected or reused here.

> ## The `46d29783` lineage remains an ORACLE
>
> `46d29783`, `1aef3192`, `9d58df12`, and
> `refs/preserved/rt-contspec-lower-held-core-rs = 88972207` are preserved and
> **may not be merged, rebased onto, or cherry-picked wholesale.** This seam
> branches from `main` and carries only its own delta. The 138-row census is not
> an input to any deliverable or AC.

## The subject, measured at `main = 0fd9f6e8`

All anchors are in
`crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`.
**The deletion surface is that one file.** Measured by the Steward at
`0fd9f6e80685126f3c3c47466bb40429b4925ab5`:

```
grep -rn "BoundaryUse" crates/ --include=*.rs        # 27 hits, all in static_transition.rs
grep -rn "boundary_phase\|boundary_operation\|\
boundary_need\|boundary_avail" crates/ --include=*.rs # 28 hits, all in static_transition.rs
```

| what | anchor |
|---|---|
| the four enum declarations | `:878`, `:885`, `:892`, `:899` |
| the four fields on `ContinuationInputProjection` | `:923-926` |
| the four fields on `ContinuationInputView` | `:2296-2299` |
| the view-copy path | `:2272-2275` |
| the one production construction site, all four hardcoded | `:7491-7494` |
| the omission copy-back cases | `:7649-7658` |
| a test-side construction of the same tuple | `:22914-22917` |
| the `mutate_projection_field` flips to the second variants | `:23453-23462` |

In each of the four enums the **second variant is `#[cfg(test)]`** and does not
exist in a production build. Every continuation input the planner produces
therefore carries the identical tuple `OperationalCarrier / Forward /
PreserveValue / Value`.

**Read every line number as an anchor to re-find, never as a value to check.**

### Why this is deletion and not population

The occurrence census is closed, and it is closed in the direction that matters:
**nothing consumes these fields.** Architect, `evt_1v9m7t4m9dmj7`: *"no lowering,
ABI, selection, lifetime, or emission consumer reads any of them. The only
production consequence of changing a tuple would be to intern another
specialization key whose emitted semantics are otherwise identical."*

⇒ This is not a dormant authority waiting to be populated. Widening the
vocabulary would split interned units without changing emitted semantics, and
the resulting extra units would then be offered as evidence the ledger
discriminates. **That is the vacuity trap inverted** — not a proof over an empty
population, but a population manufactured to satisfy a proof.

The existing control `continuation_keys_equal_under_mutation` proves, per field,
that the interning key separates two units differing only in that field. **That
proof is real and it is about a distinction production cannot make.** Removing
its four boundary rows is removal of tests for nonexistent distinctions, not a
weakening of a production guarantee. The rest of that matrix stays and stays
green.

## Fixed inputs

| input | measured value |
|---|---|
| base | `main = 0fd9f6e8`, seams 1 and 2 merged |
| the deletion surface | `planning/static_transition.rs` only, per the census above |
| prior-slice surfaces | `planning/static_transition/abi.rs`, `planning/static_transition/semantic_ir.rs`, `boundary_value.rs`, `boundary_value_clif.rs` — frozen at their `main` blobs |
| baseline suite | `scripts/ken-cargo build -p ken-runtime` then `scripts/ken-cargo test -p ken-runtime --lib` |

## Two preconditions on every suite run, carried from seams 1 and 2

Both produced a false hard stop on seam 1 (`evt_3q972fhrnsr0b`, ruled
`evt_1pt7rmmw2k5d0`). Neither is optional.

1. **Prove the tree in the same shell as the run.** `git rev-parse HEAD`
   immediately before the suite; quote its output as the base. A `git switch`
   onto a branch held by another worktree fails silently inside an `&&` chain.
2. **Build before you test.** `crates/ken-runtime` is
   `crate-type = ["rlib", "staticlib"]`; `cargo test --lib` never emits
   `libken_runtime.a`, and without it `ken_runtime_staticlib()` fails ~40 rows
   with a `Toolchain` error whose text names ken-host.

## Deliverables

- **`D1` — the re-measurement, written before any edit.** `git rev-parse HEAD`,
  plus the two greps from the census block above with their full output. State
  the domain you ran them over beside the result. **If any `BoundaryUse` or
  boundary-field hit appears outside `static_transition.rs`, or if any
  non-`cfg(test)` site reads one of the four fields, stop** — see hard stop 1.

- **`D2` — delete the schema.** The four enums; the four fields from
  `ContinuationInputProjection` and `ContinuationInputView`; the constructor
  literals at the single construction site; and the view-copy path.

- **`D3` — delete the test-only apparatus for the deleted distinctions.** The
  corresponding `ContinuationProjectionOmission` variants, their mutation cases,
  their copy-back cases, and the four boundary rows from the key-discrimination
  control. **Only those four rows.** The surviving exact-key omission matrix
  stays and stays green.

- **`D4` — the prose and intra-doc sweep. This is a named deliverable because
  two production comment sites reference the deleted vocabulary and neither is
  in the Architect's deletion list.** Both were found by the Steward's census;
  assume there are others and sweep rather than fix these two.

  1. **`:4729` is a broken intra-doc link waiting to happen.** The doc comment on
     `EffectSeatPhase` reads *"Deliberately its own type rather than a reuse of
     [`BoundaryUsePhase`]."* Deleting the target breaks the rustdoc link.
     **Preserve the rationale and drop the dead reference** — that comment is
     the record of exactly the domain confusion this ruling turns on, and it is
     the one piece of the old vocabulary's story worth keeping.
  2. **`:6304-6305` is a load-bearing impossibility premise.** An arm leaves the
     IH prefix `Open` and justifies it partly by *"an IH is a callable, whose
     continuation-input vocabulary (`BoundaryUseAvail::Callable`,
     `BoundaryUseNeed::PreserveCallableIdentity`) exists only under
     `#[cfg(test)]`."* After `D2` those types do not exist, so the justification
     names nothing. **The arm's behaviour must not change** — leaving it `Open`
     is still correct, and `evt_9krmbv834z9p` still forbids a default carrier.
     Restate the reason without the deleted names.

  ⇒ **This is the campaign's standing shape and no test catches it:** a claim
  that counts or names a member of a population your delta changes is falsified
  by that delta, and prose cannot go red. Sweep for every place the four enums
  or four fields are **named or described**, including comments and string
  literals, and state the selector you ran.

- **`D5` — the preservation evidence.** That the real continuation-input
  authorities are untouched: `ContinuationSourceCoordinate`,
  `ContinuationSourceSlotAuthority` (carrier / ownership / storage / affinity),
  ordinal and ABI position, and the finalized `ContinuationAvailabilityViews`
  publication gate. Show that no other key field and no behavioural path moved.

## Acceptance criteria

- **`AC-1` — the schema is gone.** No `BoundaryUsePhase`,
  `BoundaryUseOperation`, `BoundaryUseNeed`, `BoundaryUseAvail`, or the four
  field names survive anywhere in `crates/`, in production or test code, in
  identifiers or in prose.
  *Control:* the `D1` greps re-run on the candidate, returning zero, with the
  domain stated. This is a **review** obligation on the QA seat and a compile
  consequence — **not** a grep oracle committed as a test (operator: source-text
  oracles are an invitation for failure and delay).

- **`AC-2` — nothing else in the key moved.** `ContinuationInputProjection` and
  `ContinuationInputView` differ from the base **only** by the removal of the
  four fields. No field added, renamed, reordered relative to the survivors, or
  retyped.
  *Control:* a field-by-field diff of both declarations against the merge base,
  in the handback.

- **`AC-3` — the surviving discrimination matrix is intact and green.**
  `continuation_keys_equal_under_mutation` passes, and its non-boundary
  assertions are **byte-unchanged**. Exactly four rows removed.
  *Control:* the diff of that control, plus the green run.

- **`AC-4` — emitted behaviour is unchanged.** The activation and emission
  controls landed by seams 1 and 2 stay green.
  *Control:* `scripts/ken-cargo build -p ken-runtime` then
  `test -p ken-runtime --lib`, with `git rev-parse HEAD` quoted in the same
  shell.

  **This is the AC that carries the seam's real risk.** The four fields
  participate in an interning key. Removing them can only **merge** units that
  were previously distinct — which is the intended correction, since their
  emitted semantics were identical by construction. **If any unit count or
  emitted artifact changes in a way that is not that merge, stop** (hard stop 3).

- **`AC-5` — the prior-slice surfaces are blob-identical to the merge base.**
  *Control:* `git rev-parse <candidate>:<path>` against
  `git rev-parse <merge-base>:<path>` for each of the four surfaces.

- **`AC-6` — no test asserts a fact about source, catalog, or documentation
  lines.** `D1` and `D4` are review artifacts, not gates (operator test policy).
  *Control:* a read of the added tests.

- **`AC-7` — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run (`COORDINATION §12`).

## Banned scope

- **No new classifier, no widened record, no split-phase ledger.** The Architect
  ruled out all three by name. If you believe a continuation consumer needs a
  phase, retention, or Need/Avail contract, that is a future node owned by that
  consumer, which must derive a domain-specific checked record and **prove the
  record changes validation or emission**. It is not anticipated in this key.
- **No promotion of the `Callable` / `PreserveCallableIdentity` pair.** That
  pair belongs to `RT-CONTSRC-CALLABLE-CONTRACT`, whose ruled shape is a closed
  sum **beside** `ContinuationSourceSlotAuthority` — value source versus
  static-callable source with no value carrier. This seam deletes the test-only
  pair along with the rest of the schema; it must neither pre-implement that
  node's contract nor obstruct it. See the downstream note below.
- **No planner or ABI repair, and no planner reordering.** A planner- or
  ABI-worded refusal on the deletion is a new interface fact and routes back as
  an exact hard stop. `PlannedEffectSeat`'s construction after `join_results` is
  required by the effect seat's own result-representation dependency and is not
  an invitation to move or split continuation interning.
- **No re-derivation of `RT-DECL-CLOSURE-PORT` `D7`.** It is merged, it is
  correct, and it is not this seam's authority.
- **No lowering activation work.** That was seam 2 and it is merged.
- **No edit to any prior-slice surface** (`AC-5`).
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**

## Downstream note — `RT-CONTSRC-CALLABLE-CONTRACT`

That node is `status: ready` and does **not** depend on this one, so the two can
be picked up in either order. Its `D0` and its first hard stop are currently
stated in terms of `BoundaryUseAvail::Callable` and
`BoundaryUseNeed::PreserveCallableIdentity` — the exact variants this seam
deletes. **The Steward has amended that frame in the same change that lands this
recut**, so that its `D0` is answerable and its hard stop is not silently
vacuous after the deletion. The mechanism it owns is unaffected: its ruled shape
is a new closed sum, never a promotion of these variants.

**If you are the implementer and that frame still reads as though the variants
exist, the amendment did not land — stop and route it.**

## Contention

Runtime is single-threaded. Take the shared build lock for the suite runs; probe
without blocking first. **Targeted only:** `scripts/ken-cargo test -p ken-runtime
--lib`. **Never `--workspace`** — the full-workspace build, the `--locked` gate
and conformance run in CI.

## Sizing

**Size `S`.** This is a single-file deletion with a closed census and no design
left in it. The two things that can inflate it are `D4`, if the prose sweep
finds more than the two known sites, and `AC-4`, if the interning-key change
moves an emitted artifact.

**One checkpoint, not three.** Commit `D1`-`D5` together and post the exact SHA.
The deletion is not meaningfully separable: `D2` alone does not compile, since
`D3`'s mutation cases reference the deleted variants. If the work runs past an
hour, stop and route; the recut is the Steward's.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **`D1` finds a consumer.** Any non-`cfg(test)` site reads one of the four
   fields, or any occurrence appears outside `static_transition.rs`. The
   ruling's central premise is that no consumer exists; a consumer falsifies it
   and the seam is a different node.
2. **The deletion cannot complete without touching a prior-slice surface or the
   planner.** Interface fact, exactly as at seams 1 and 2.
3. **`AC-4` moves.** An emitted artifact or a unit count changes in any way
   other than the merging of units whose emitted semantics were already
   identical. That would mean the fields were load-bearing after all, which
   contradicts hard stop 1's premise from the other direction.
4. **A planner- or ABI-worded refusal appears.** New interface fact.
5. **The `D4` sweep finds a comment whose claim is load-bearing and whose
   correct restatement you cannot derive.** Do not guess at why an arm refuses;
   the reason is the arm's contract.
