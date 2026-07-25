# `RT-FNSPLIT-B2O-CHECK` — close the advertised-vs-enforced gap in the checking layer

**Owner:** Team Runtime · **Size:** `M` (was `S`; the `B2R` hunt added two
findings, one of which is a design task) · **Gate:** none

| dependency | landed | PR |
|---|---|---|
| `RT-FNSPLIT-B2O` — the validated `SemanticOwner` partition | `origin/main` = `e470ab65` | #963 |
| `RT-FNSPLIT-B2R` — the representation and call-ABI contract | `origin/main` = `c986d0a3` | #967 |
| `RT-FNSPLIT-B2F` — the atomic switch-over | **IN FLIGHT** — see sequencing | — |

> ## ⛔⛔ READ FIRST — EVERY LINE NUMBER BELOW IS PERISHABLE BY CONSTRUCTION
>
> Anchors here were re-derived at `origin/main` = **`6c6de5cc`** and were
> correct then. **`RT-FNSPLIT-B2F` is building right now and rewrites the
> emission path across all three files this node touches.** It is an `L` on an
> atomic boundary; assume it moves every anchor below.
>
> ⇒ **Re-derive every citation against the landed code at pickup. Do not copy a
> line number out of this frame into an assertion.** Where this frame and the
> code disagree, **the code is right and the frame is stale** — say so and
> escalate; do not quietly build around it.
>
> ★ This is not boilerplate caution. This node's own predecessor file was
> written against `e470ab65` and was stale **before the frame existed**, because
> `B2R` edited both files it cited. That happened once already on this exact
> node. It will happen again with `B2F`, and this time it is predicted.

## The thesis — one defect, five instances, three files

A checker **states** a population of laws and **enforces** a smaller one. In
every instance the suite is green, the checker is genuinely fail-closed on the
paths that *do* fire, and the gap is invisible to every consumer.

| # | file | what is advertised | what is enforced |
|---|---|---|---|
| **A** | `lowering/core/tests/control.rs` | the production-surface inventory is *closed* | only items whose line ends in `;` |
| **B1** | `planning/static_transition/semantic_ir.rs` | 12 laws in `validate_function_units` | 5 live; one arm is provably unreachable |
| **B3** | `semantic_ir.rs` | a two-conjunct population check | one conjunct cannot differ |
| **C** | `planning/static_transition.rs` | an AC arm covering *"a capture, or any child of a non-closure"* | the capture class has zero instances |
| **P1** | `planning/static_transition/abi.rs` | `C4` excludes imported values *crossing a frame boundary* | only the capture child's own top-level shape |
| **P2** | `abi.rs` | *"both directions are asserted"* | one direction and a restatement of it |

⛔ **None of these is a correctness defect in `B2O`'s derivation.** The owner
partition is sound; the Adversary attacked it directly and could not break it.
`shared_exits`, the seed-class, overlap, and owner-comparison detectors are
fail-closed and correctly *ordered*. **Everything here is in the checking layer.
Do not re-open the partition.**

## ⛔ Sequencing — this node lands AFTER `RT-FNSPLIT-B2F`, on CONTENTION

`B2F` is not a logical prerequisite: the Architect ruled
(`evt_7ggqdk61pxzzf`) that `B2F` must establish representability
**independently and fail-closed**, so it does not wait on `P1`'s repair. The
sequencing constraint is **file contention**, which is a separate axis:

```
B2F touches      static_transition.rs · abi.rs · semantic_ir.rs · tests/control.rs
B2O-CHECK touches            abi.rs · semantic_ir.rs · tests/control.rs · static_transition.rs
```

Every file intersects. `B2F` is an **atomic** switch-over that cannot be split,
so it goes first and this node re-derives afterward. ⇒ **Do not start this node
while `B2F` is in flight.** The Steward releases it; the population of live arms
in `validate_function_units` may itself change when `B2F` lands, which is
precisely why the reachability sweep below must be run against the **post-`B2F`**
tree and not this frame's tables.

## Grounding — what I verified, and what I am relaying

**Per `pin-a-property` §4, stated per finding, because the ring must know which
claims carry a Steward measurement and which do not.**

| finding | grounding |
|---|---|
| **A** | ✅ Steward re-measured the filter and both known holes (`impl`, `mod`) |
| **B1** | ✅ Steward re-measured the shadowing — both call sites, both error strings |
| **B3** | ✅ Steward re-measured the vacuous conjunct |
| **P2** | ✅ **Steward derived this one independently** — see below |
| **B2** | ⚠ **Adversary's measurement, relayed.** Six arms, "no witness found on the one route that reaches them" — *not* proof of unreachability |
| **C** | ⚠ Adversary's measurement, relayed — the zero-instance capture class |
| **P1** | ⚠ **Split.** Steward verified the **code shape**; the fixture measurements (*"planned green, 2 descriptors, 10 slots"*) are the Adversary's, **relayed, not corroborated** |

⛔ **The ring re-measures everything. This frame is not corroboration of anyone's
finding.**

## The findings

### A — the item enumerator's population is "lines ending in `;`"

`lowering/core/tests/control.rs:3726` filters candidate declarations on
`trimmed.ends_with(';')`. **A Rust item is brace-shaped**, so every braced form
is invisible to the enumerator that is supposed to close the production surface:

| form | ends `;` | seen |
|---|---|---|
| `use a::b;` · `pub type T = …;` | yes | yes |
| `impl T for S { … }` | no | **no** — first measured hole |
| `mod x { … }` | no | **no** — second, found by `B2O`'s own `AC-12` |
| `fn f() {}` · `struct S {}` · `trait T {}` | no | **no** — predictable, not yet measured |

⭐ **The fix must not be a third accepted spelling.** Two holes were found one at
a time and a third class is already predictable from the filter's *shape*. **Key
on the item head** — the leading keyword after attributes and visibility — so the
closure is structural. ⚠ **An enumerator that grew a `mod` arm and nothing else
has reproduced the bug.**

### B — `validate_function_units` advertises 12 laws; 5 are live

**B1 — one arm is unreachable by construction, and it is the only quadratic
one.** `validate_function_units` (`semantic_ir.rs:987`) opens by calling
`partition_function_units`, which at `:657` rejects a `StaticBody` edge aimed at
a scheduling entry — *"scheduling entry is also a static body target"*. The arm
at `:1121` — *"scheduling entry has an incoming static body edge"* — tests the
**identical condition** and cannot fire. Witnessed against **both** entry classes
(the root, and a transparent declaration); both return the `:657` message.

⚠ It is also **quadratic**: a fresh `Vec` of entries, then `Vec::contains`
(linear) inside a loop over **all** edges — `O(|edges| × |entries|)`, both
operands scaling with program size, **paid on every `validate()` for a law that
never runs.**

**B2 — six further arms have no witness.** Every attempt to construct their input
landed on an earlier detector: `:1035`, `:1050`, `:1062`, `:1086`, `:1094`,
`:1106`.

⛔ **Claim discipline — only B1 is *provably* unreachable.** The six are *"no
witness found on the one route that reaches them."* `validate_function_units` is
private and called only from `SemanticPlane::validate` (`:1189`), so the route is
fixed — **but absence was not proved.** ⚠ **`:1072`** (*"ownership edge endpoint
has no semantic descriptor"*) was **not probed at all**; it is reachable in
principle via a `StaticBody` edge whose `from` is out of range — the one endpoint
`partition_function_units` never bounds-checks. **Treat it as an open probe, not
a cleared one.**

**B3 — a vacuous conjunct.** At `:1006`, `partition.seeds.len() != expected_units`
cannot differ: `seeds` is pushed exactly once per entry and once per `StaticBody`
edge, in the same function that computes `expected_units` from those two counts
(`:1002`). The live conjunct is `self.functions.len()`.

### C — an AC arm naming a class with zero instances

`static_transition.rs` counts *"every other child — **a capture**, or any child
of a non-closure"* into `interior_children` under a no-silent-caps
`interior_children > 0`. Every `B2O` ownership fixture builds `LexicalClosure {
captures: Vec::new(), .. }`, so **the capture class has zero instances** and the
assertion is discharged entirely by non-closure children.

⭐ **Zero live risk, and the mechanism is correct on both untested forms** — a
`LexicalClosure` with two captures yields 2 units, 1 `StaticBody` edge, and 2
capture children owned by the parent's unit; a bare `RuntimeExpr::Closure` yields
2 units and 1 `StaticBody` edge. The cost to close it is one fixture plus a
`capture_children > 0` counter. ⚠ The test's own stated discipline (*"fail if a
class never appeared"*) was applied on the boundary/interior axis and **not on
the class it names first**.

### P1 — `C4`'s imported-edge exclusion is narrower than the module says

**This is the one the Architect explicitly routed here** (`evt_7ggqdk61pxzzf`):
*"`reject_imported_capture_edges` is a defect in `B2R`'s inert checking layer, so
repairing that function and its advertised law belongs in
`RT-FNSPLIT-B2O-CHECK`. `B2F` must not patch it."*

`reject_imported_capture_edges` (`abi.rs:514`) walks a lexical closure's **direct
capture children** and calls `result_carrier` (`:582`) on each. That answers
*"is this capture expression's own top-level shape `ImportedDeclarationRef`?"* —
**not** *"can an imported value reach this frame slot?"*

| | |
|---|---|
| **Hole A** | any wrapper defeats it — `If { Bool(true), imported, imported }` is **binder-free**, so no de Bruijn reading makes its result anything but the imported value, and it receives a full `Capture / ValueWord / OwnedByFrame` slot |
| **Hole B** | needs no wrapper at all — `LexicalClosure { captures: [], body: imported }`; the function iterates **capture children only**, so the unit's own **result** slot is never carrier-checked |

**The violated invariant is the module's own.** `abi.rs` states that `C4`
*"excludes the position where an imported value would have to cross a frame
boundary and be given a carrier."* Both holes are exactly that position.

⭐ **The shape is why this is a category fix.** The file records that the first
implementation rejected *every* occurrence with an unrepresentable result
carrier, that this was strictly stronger than `C4`, and that a property test
caught it. **The repair moved from *"any occurrence anywhere"* to *"the capture
child's own node"* — and skipped the correct middle: the set of occurrences whose
value can REACH a boundary slot.** Corrected past the target, on the same axis,
and documented with more care than the original error — **which is exactly why it
reads as settled.**

⚠ **Reachability from a real Ken program was NOT established** by the Adversary
or the Steward. That is an open question, not a cleared one, and it bears on
whether the repair rejects or links.

### P2 — the second "direction" is entailed by the first

**Steward-derived independently against `6c6de5cc`.** `AbiPlane::validate`
asserts at `:922` that `self.descriptors.len() == plane.functions.len()`. The
loop at `:930` then iterates `self.descriptors.iter().enumerate()`, so `ordinal`
ranges over `0..descriptors.len()` — which `:922` has just forced equal to
`functions.len()`. **`plane.functions.get(ordinal)` therefore can never be
`None`, and the `ok_or_else` arm at `:934` is dead.**

The comment at `:932` reads *"A one-directional check passes happily on an
orphan, so both directions are asserted."* ⛔ **What is asserted is one direction
and a restatement of it.**

⭐ **The live content is the check immediately below** — `descriptor.function !=
id || descriptor.planned_node != function.planned_node || descriptor.origin !=
function.origin` (`:939`). That is a real positional-identity law and it is
**load-bearing**. ⚠ **Do not delete it while removing the dead arm above it** —
that is exactly the "a reader who trusts the count weakens a live check" hazard
this node exists to prevent.

## Deliverables

**D1 — structural closure for the item enumerator (Finding A).** Key on the item
head, not the line's punctuation. **A positive control per braced item form**:
each form, inserted into production source, is *seen*.

**D2 — a reachability row per advertised law (Findings B1/B2/B3).** For every arm
of `validate_function_units`, produce **either** a witness that reaches *that*
arm — **asserting the exact error string, never `is_err`** — **or** an explicit
*"no witness — shadowed by `<arm>`"* row. **A law with neither is not a law and
must not be counted as one.** Probe `:1072` specifically; it was never attempted.

**D3 — deliberate ordering (Finding B1).** Where an earlier detector legitimately
subsumes a later arm, **delete the dead arm and its cost**, or re-order
deliberately. Do not leave twelve stated laws over five live ones. B1's deletion
also removes the only quadratic scan in the validator.

**D4 — close the zero-instance class (Finding C).** One `LexicalClosure`-with-
captures fixture plus a `capture_children > 0` counter.

**D5 — repair `C4` (Finding P1).** Move the exclusion from *"the capture child's
own top-level shape"* to *"occurrences whose value can reach a boundary slot"* —
the middle the original repair skipped. ⛔ **Do not restore the original
over-strong form** (*"any occurrence anywhere"*); a property test already
rejected it, and re-landing it re-breaks intra-module values that must stay
accepted. The obligation covers **the unit's own result**, not only capture
children.

**D6 — correct `AbiPlane::validate` (Finding P2).** Remove the dead arm and state
what the remaining check actually enforces. ⛔ **Preserve the positional-identity
law at `:939`.**

## Acceptance criteria

**AC-1 — every advertised law has a reachability row.** For each arm in
`validate_function_units`: exact-error witness, or an explicit shadowed-by row
naming the shadowing arm. **The count of stated laws equals the count of rows.**

**AC-2 — the enumerator is closed against a POSITIVE CONTROL PER FORM.**
Enumerate the forms explicitly — `impl`, `mod`, `fn`, `struct`, `trait`, `enum`,
`use`, `type`, `const`, `static` — and record a per-form result. ⛔ **"Each form"
as a quantifier the reader resolves is not an AC.** A per-form table with a cell
per result is.

**AC-3 — `C4` rejects both measured holes and still accepts intra-module
values.** Both directions are required. Assert the **exact** error. ⚠ A repair
that rejects `If { true, imported, imported }` by rejecting all `If` captures has
reproduced the over-strong form and fails this AC.

**AC-4 — the dead arm in `AbiPlane::validate` is gone and the positional-identity
law is intact.** A control that reddens if `:939`'s comparison is neutered.

**AC-5 — no live detector was weakened.** For each arm **deleted** under `D3`,
show the condition it tested is still refused, by the shadowing arm, with that
arm's exact error string.

> ### ⛔ PER-PIN EVASION ATTEMPT — THIS IS AN AC, NOT A HAZARDS NOTE
>
> For **each** pin above, attempt a **compile-preserving evasion** and record the
> result **per pin**, in a table with one row per pin. Name the positive control
> that would fire if the attempt were skipped.
>
> ★ **This is stated as an AC because stating it as advice demonstrably fails.**
> On `RT-FNSPLIT-B2O` the identical sentence sat under a heading whose mood was
> *advice*; the implementer ran **one** evasion attempt of several, and ran the
> rest only after the same sentence arrived in a message — immediately finding a
> real overclaim. **Two of that WP's three review folds were then in the family
> the paragraph named.**
>
> ⚠ **And an evasion attempt must vary the axis the pin NAMES.** `B2R`'s
> implementer wrote two witnesses for an edge-**layout** law and both mutated the
> same field, so both landed on **identity** detectors — green row, named for
> layout, testing identity. **Failing to find a witness is evidence about the
> witnesses you could think of, never about the property.**

## Do-not-reopen guardrails

1. ⛔ **The `B2O` owner partition is sound. Do not re-derive or re-litigate it.**
2. ⛔ **`B2F`'s job is not yours.** This node repairs the *checking* layer. It
   does not establish emission-time representability — that is `B2F`'s `AC-11`,
   ruled explicitly by the Architect.
3. ⛔ **Do not restore `C4`'s over-strong original form** (`D5`).
4. ⛔ **Do not delete a live detector while removing a dead one** (`D6`, `AC-5`).
5. **Every anchor is perishable.** If a fixed input in this frame is false
   against the landed code, **say so and escalate — do not quietly build around
   it.**

## Standing

- ⛔ **Local builds/tests are TARGETED ONLY** — `scripts/ken-cargo -p ken-runtime`,
  or `--test <name>`. **Never `--workspace`** (`COORDINATION §12`, operator hard
  rule). Workspace-green and `--locked` mean **green in CI**.
- **Report an unpushed ref and KEEP GOING.** Build seats have no GitHub
  credential by design; the Steward pushes. Raising it is not gating on it.
- **Hard-stop protocol.** `RT-NATIVE-FNSPLIT` count of record is **9**;
  **next research pull = #12**. Symptom inventory is
  `docs/program/issues/RT-NATIVE-FNSPLIT.md` — one line per hard-stop, appended
  by the Architect, never rewritten; **next predicate check at the 3rd entry.**
- Read `agent/playbooks/tools/pin-a-property.md` before writing any assertion.
