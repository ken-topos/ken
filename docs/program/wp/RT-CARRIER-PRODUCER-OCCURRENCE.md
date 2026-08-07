# `RT-CARRIER-PRODUCER-OCCURRENCE` — frame

Owner: **runtime**. Size: **M**. Gate: none.
Depends on: `RT-SRCBODY-BIND-ORDER` (merged, `acfcc915`).
Origin record:
[`RT-CARRIER-PRODUCER-OCCURRENCE`](../issues/RT-CARRIER-PRODUCER-OCCURRENCE.md)

Ground: `origin/main` **`d18da5c6`**. Every line number below was read at that
ref.

## 0. Posture

`c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload`
(`constructors.rs:2549`) is `#[ignore]`d. It dies at
`.expect("the C2 carrier edge emits")` before its property is evaluated, so the
row currently measures nothing.

**Treat every anchor in this frame as perishable. If a fixed input turns out
false against the landed code, say so and escalate — do not quietly build
around it.** In particular §1f is a derivation I did not execute; §2 `D0` exists
to kill it cheaply if it is wrong.

**The refusal is a guard doing its job, and this node's default posture is that
the fixture is wrong, not the guard.** Read §4 before proposing any change under
`lowering/mod.rs`.

## 1. Fixed inputs

### 1a. The refusal site

`crates/ken-runtime/src/cranelift_backend/lowering/mod.rs:4994-5001`, in
`reconcile_source_aggregate`:

```rust
let Some(occurrence) = value.source_aggregate_producer() else {
    return Err(unsupported(
        lowered_value_kind(value),
        "a source aggregate reached the carrier with no planner-issued producer \
         occurrence, so it would name no ownership record and could only be given \
         the authority of wherever it happened to be transferred",
    ));
};
```

### 1b. Only two variants can carry an occurrence

`source_aggregate_producer` (`mod.rs:9571-9580`) returns the `occurrence` field
for `Lowered::Constructor` and `Lowered::Record`, and `None` for everything
else. `reconcile_source_aggregate` is called only from the `Constructor` and
`Record` arms of the preflight walker, so that `_ => None` arm is not reachable
from the refusal path. **The observed `construct: "Constructor"` therefore names
a `Lowered::Constructor` whose `occurrence` field is literally `None`** — not a
lookup that failed, and not a different variant misreported.

### 1c. The walker is whole-graph and runs before any allocation

`source_aggregate_preflight` (`mod.rs:4895-4980`) is called from
`transfer_into_carrier` (`mod.rs:4866`) *before* `emit_carrier_transfer`. It
recurses through `HostResult` (`:4933`) and `DynamicConstructor` (`:4937`),
which carry no ownership record of their own but are not leaves. It has **no
`_` arm** by construction (`:4959-4978`), so a new `Lowered` variant with a
child position is a compile error rather than a silently unreconciled subtree.

Consequence for sizing: a refusal anywhere in the tree is reported before
anything is allocated, so the first refusal is the only one you observe. **The
panic tells you nothing about how many more sit behind it.**

### 1d. The row builds THREE edges, and the panic reports only the first

`c2_ac4` compiles three separate functions, in this order:

| # | edge | built at | how its aggregate gets an occurrence |
|---|---|---|---|
| 1 | `c2_host_result_producer` | `:2654` | `error` comes from `synthesized_constructor` (`:2682`) |
| 2 | `c2_ordinary_result_producer` | `:2721` | hand-written `Lowered::Constructor`, `occurrence: None` at `:2730` and `:2734` |
| 3 | `c2_host_result_consumer` | `:2752` | consumes; does not transfer a source aggregate in |

Edge 1 is compiled first, so its `.expect` is the one that fires.

### 1e. Edge 2's `occurrence: None` is written into the fixture by hand

`:2727-2740` constructs `Lowered::Constructor { .. occurrence: None .. }`
nested inside another with `occurrence: None`, then transfers it at
`ordinary_producer_origin` (`:2742-2747`). **The plan already contains a real
occurrence for that node** — the fixture computes `ordinary_producer_origin` at
`:2626` and asserts against its identity at `:2707-2719`. The occurrence exists
in the plan and simply is not threaded into the value handed to the carrier.

### 1f. Edge 1's refusal is a DELIBERATE production branch

Reached because the rig has no emission owner. **This is the derivation to
confirm or kill in `D0`. I did not run it.**

`synthesized_constructor` (`mod.rs:11046`) early-returns
`Lowered::Constructor { occurrence: None, .. }` at `:11064-11071` when
`self.defining_emission_owner` is `None`. Its own comment states the intent:
absent means no context is being defined, which is not an emission this
population covers, so no occurrence is issued and *the loud refusal at the
allocation stands rather than a borrowed owner being invented*. On the other
branch (`:11089`) the occurrence is resolved from the planner.

The c2 rig reaches the first branch:
`c2_compile_edge_with_arg:2497` builds its compiler with
`bare_carrier_test_lowering` (`:1884`), whose struct literal sets
`defining_emission_owner: None` (`:1926`).

Chain, end to end:

```
c2_compile_edge_with_arg:2497 -> bare_carrier_test_lowering:1884
  -> defining_emission_owner: None                     (:1926)
edge 1 error = synthesized_constructor(...)            (:2682)
  -> mod.rs:11064 early return, occurrence: None
transfer_into_carrier                                  (mod.rs:4866)
  -> source_aggregate_preflight  HostResult arm        (mod.rs:4933)
  -> recurses into `error`, Constructor arm            (mod.rs:4898)
  -> reconcile_source_aggregate                        (mod.rs:4987)
  -> source_aggregate_producer() == None
  -> refusal, construct: "Constructor"                 (mod.rs:4994)
```

That reproduces the observed signature exactly, including the `construct`
field.

**The fixture already knows.** `:2677-2681` carries a `D7` note saying this
fixture has no `Effect` occurrence, so `match_origin` is not a producer seat,
the template gets no occurrence, and it refuses at the allocation — *"which is
where it already fails."* A prior deliverable observed this refusal and left it
in place deliberately.

### 1g. Both repair routes have working precedent in the same file

- **Thread a planned occurrence into a hand-built value:** ten sibling tests do
  it, e.g. `:2059`, `:2085`, `:3041`, `:4304`, `:8079`, each
  `occurrence: Some(...)`.
- **Give the rig a defining emission owner:** `:6151` does
  `compiler.defining_emission_owner = Some(owner)`, with `:6119` setting it back
  to `None`.

Neither route requires a new API, a new primitive, or a change under
`lowering/mod.rs`. **This is the constructibility audit's answer and it is
positive** — the repair is expressible in the vocabulary the fixture already
has.

### 1h. Provenance

Base-fail and candidate-fail with an identical signature, measured two-ended by
the `RT-SRCBODY-BIND-ORDER` all-eight-package census (`evt_ksrhrv82t5ae`), with
`px8-ds-test-support` both on and off. Pre-existing base debt, not a `D1`
regression. The reported line moved `:2509` to `:2511` only because the
candidate added two lines above it.

## 2. Deliverables

**`D0` — confirm or kill §1f, before building anything.**
Run the row un-ignored and capture which of the three edges refuses and at which
`Lowered` node. Report the first refusal's `construct` and the edge name.
If the first refusal is not edge 1's `error`, §1f is wrong: **stop, say so, and
re-derive** — do not adapt the repair to the observed panic without saying which
fixed input failed.

**`D1` — enumerate the population, do not stop at the first refusal.**
§1c means the panic hides its successors. Enumerate every source aggregate this
row transfers into the carrier and state, per node, whether it carries a
planner-issued occurrence. At minimum the three sites at `:2670`, `:2730`,
`:2734` and the `synthesized_constructor` result at `:2682`.
Then widen once: of the 14 `occurrence: None` literals in `constructors.rs`,
name which reach a `transfer_into_carrier` call and which do not. A count alone
does not discharge this; the deliverable is the reaching set.

**`D2` — rule rig versus real, in writing, with the evidence.**
Two outcomes, and they are not one deliverable:
- **Rig.** The fixture manufactures a state production never produces. The
  repair is in the fixture and `D3` proceeds.
- **Real.** A production path can reach the carrier with no producer
  occurrence. Then the refusal is protecting a live hole, the row is evidence
  rather than debt, and this becomes an Architect question — **route it, do not
  repair it.**

§1f is evidence for *rig*, and edge 2 (§1e) is plainly rig. Say which outcome
you are ruling and why. **A ruling of *rig* for edge 1 must engage the `D7` note
at `:2677-2681`**, which asserts the current refusal is correct.

**`D3` — repair the fixture so the carrier edge emits.**
Use one of the §1g routes. Every source aggregate the row transfers must carry
an occurrence the planner issued for that node — not one borrowed from a
sibling, and not a value minted to satisfy the check.

**`D4` — un-ignore the row and prove it measures its property.**
Remove the `#[ignore]` at `:2548` and the annotation block at `:2530-2546`.
Note `:2530-2546` is a `//` block immediately above the attribute; check whether
a separate leading doc comment exists above it and update that too rather than
stranding it.

**`D5` — currency.**
If `D2` rules *rig*, the `D7` note at `:2677-2681` says the refusal is correct
and expected, and that sentence becomes false the moment `D3` lands. Correct it
in place. If `D2` rules *real*, leave it and say so.

## 3. Acceptance criteria

### `AC-1` — the first refusal is identified by execution, not by reading

> **MEASURED:** `D0` reports the edge name and `construct` of the first refusal,
> from a run.
> **CLAIMED:** §1f's chain is the actual path.
> **THE GAP:** §1f was derived by reading. It reproduces the signature including
> the `construct` field, which is strong, but a second node that also refuses
> with `construct: "Constructor"` would be indistinguishable from it on the
> signature alone.

### `AC-2` — the reaching set is enumerated, not sampled

> **MEASURED:** `D1` names every source aggregate this row transfers and its
> occurrence status, plus the reaching subset of the file's 14 `occurrence:
> None` literals.
> **CLAIMED:** the repair is sized against the whole population.
> **THE GAP:** §1c guarantees the panic reports one refusal and hides the rest.
> **A repair sized on the observed panic is sized on a sample of one.**
> **Positive control:** after repairing only edge 1, the row must still fail —
> at edge 2. If it goes green, §1e is wrong and `D1` mis-enumerated. Record that
> intermediate result; do not skip straight to the full repair.

### `AC-3` — the restored row discriminates its property

> **MEASURED:** with the row un-ignored and passing, a **population-side**
> mutation reddens it — swap the two arm identities, or point the consumer at
> the other constructor, so the selection genuinely changes.
> **CLAIMED:** the nested payload is separately generated and correctly
> selected.
> **THE GAP:** a green row proves the carrier now emits. It does not prove
> selection happened, and emitting is exactly what `D3` changed. The row's
> existing `assert_ne!` at `:2646-2649` guards identity distinctness, not
> selection.
> Restore the mutation afterward and say so.

### `AC-4` — no production behaviour changed under a *rig* ruling

> **MEASURED:** if `D2` ruled *rig*, `git diff origin/main` touches no file
> under `crates/ken-runtime/src/cranelift_backend/lowering/` other than the
> `core/tests/` subtree.
> **CLAIMED:** the guard is intact.
> **THE GAP:** the cheapest false fix here is a one-line relaxation at
> `mod.rs:4994` that no test would catch, because every test that would catch it
> is the one being repaired. **This AC is the only thing standing on that line.**

### `AC-5` — no regression

Green in CI. Per `COORDINATION §12` this means CI, **not** a local
`--workspace` run. Locally, run only `-p ken-runtime`.

## 4. Banned scope

- **Do not relax or delete the refusal at `mod.rs:4994-5001`, and do not add a
  fallback owner.** The refusal text names precisely what a fallback would grant:
  the authority of wherever the value happened to be transferred. `mod.rs:4882-
  4887` states that a missing producer is a refusal and never a fallback.
  If the proposed repair is to make the emit succeed by accepting an aggregate
  with no ownership record, **that is a mechanism question and it returns to the
  Architect** — it does not land here.
- **Do not change `synthesized_constructor`'s early-return branch
  (`mod.rs:11064-11071`) without an Architect ruling.** §1f says that branch is
  deliberate and `:2677-2681` says the resulting refusal is correct. Changing it
  is a design change wearing a fixture repair's clothes.
- **Do not weaken the `expect` at `:2511`** to make the row pass.
- **Do not re-baseline or re-scope the row's assertions** to fit whatever the
  repaired edge produces. If the row cannot assert its stated property after the
  repair, that is a finding to report, not an assertion to adjust.
- **Do not repair the other census row.** `two_same_shape_workers_are_distinguished`
  is `RT-WORKER-FIXTURE-DECODE`'s, and it is `ready`.

## 5. Hard stop

Stop and report, rather than proceeding, if any of these holds:

- `D0` contradicts §1f.
- `D2` rules *real* — a production path can reach the carrier with no producer
  occurrence. That is an Architect question and it is more important than this
  row.
- The repair cannot be written without touching `lowering/mod.rs` outside
  `core/tests/`. §1g says it can; if that turns out false, the constructibility
  audit was wrong and the size is wrong with it.
- `D1`'s reaching set is materially larger than the four sites named — that is a
  re-sizing conversation with the Steward, not a longer turn.

Per the one-hour turn target, a genuine hard stop is a good outcome. Neither
finishing nor stopping is the bad one.

## 6. Contention

Touches `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/
constructors.rs`, which is also
[[RT-WORKER-FIXTURE-DECODE]]'s file. **Both nodes are runtime-owned and the
fleet is single-threaded, so they cannot run concurrently** — sequence them, do
not parallelize. Their target rows are far apart — `c2_ac4...` at `:2549` here,
`two_same_shape_workers_are_distinguished` at `:5816` there — and their
deliverables are disjoint, so either order works.

`RT-CARRIER-BYTESPAN-OBSERVE` is `active` on the same crate. This node is
sequenced behind it.
