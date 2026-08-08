---
id: RT-FNSPLIT-B2O-CHECK
title: "the B2O checking layer advertises more than it enforces — structural closure for the item enumerator and reachability for the validator arms"
status: draft
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B2O, RT-FNSPLIT-B2F]
blocks: []
github: null
origin: Findings on landed B2O (origin/main e470ab65). (a) Steward's own AC-12 tally recorded one GREEN row as a reported follow-up rather than fixing it inside a subtraction WP. (b) Adversary report evt_kzc8ntfsyhn9 (thr_2seh2bm1kr5mh), P1/P2/P3. SCOPE GREW 2026-07-25 on the B2R hunt (adversary evt_3wjme1fk20dw5) plus Architect ruling evt_7ggqdk61pxzzf, which routed the C4 imported-edge repair HERE and explicitly forbade B2F from patching it; two further findings (P1 the C4 exclusion, P2 the entailed second direction in AbiPlane::validate) joined, and the size moved S -> M because P1 is a design task rather than a mechanical closure. Grounding status, stated precisely: the Steward independently re-measured Finding A's filter, Finding B1's shadowing (both call sites and both error strings), Finding B3's vacuous conjunct, and derived P2 independently against 6c6de5cc. Findings B2 (the six witness-less arms) and C (the zero-instance capture class) are the Adversary's measurements, relayed unverified; for P1 the Steward verified the code shape but NOT the fixture measurements. The ring re-measures everything and must not treat this file as anyone's corroboration. Steward-filed; Steward owns the frame and the AC/control placement.
---

> ## STATUS CORRECTED `ready` TO `draft` — 2026-08-08, Steward
>
> **`ready` means shovel-ready: a written frame, dependencies merged.**
> This node was not, because
> no frame exists.
>
> **The correction is not a downgrade of the work.** A node advertising
> startable work it does not have makes the backlog read deeper than it
> is, and that depth is exactly what a Steward reads to decide whether a
> team is idle for want of work or for want of a lane.

> ## ▶ THE FRAME IS WRITTEN — read it, not this file
>
> `docs/program/wp/RT-FNSPLIT-B2O-CHECK-advertised-vs-enforced.md`
>
> **Read the frame for scope and acceptance.** This file records
> the findings as measured; the frame carries the deliverables, the ACs, and the
> do-not-reopen guardrails.
>
> ### ⛔ `RT-FNSPLIT-B2F` IS LISTED AS A DEPENDENCY ON CONTENTION, NOT ON LOGIC
>
> `B2F` is **not** a logical prerequisite. The Architect ruled
> (`evt_7ggqdk61pxzzf`) that `B2F` must establish representability
> **independently and fail-closed**, so it does not wait on `P1`'s repair.
>
> The constraint is **file contention**: `B2F` is an atomic `L` switch-over that
> rewrites the emission path across **all four** files this node touches
> (`abi.rs`, `semantic_ir.rs`, `static_transition.rs`, `tests/control.rs`). It
> cannot be split, so it goes first. The dependency is recorded in `depends_on`
> so the tracker stops offering this node as frontier — **the mechanism is a
> dependency; the reason is contention.**
>
> ⚠ **Every anchor in this file will move when `B2F` lands.** That already
> happened once: these citations were written against `e470ab65` and `B2R` made
> them stale *before the frame existed*. Re-derive at pickup.

> ## One node, because both findings are the SAME defect in two substrates
>
> A checker **states** a population of laws and **enforces** a smaller one, and
> in both cases the suite is green and the gap is invisible to every consumer.
> The two fixes are both structural closures, both `S`, both in `ken-runtime`,
> and neither is worth its own three-gate cycle.
>
> ⛔ **Neither finding is a correctness defect in `B2O`'s derivation.** The owner
> partition is sound — the Adversary attacked it directly and could not break it,
> and the `shared_exits` / seed-class / overlap / owner-comparison detectors are
> fail-closed and correctly *ordered*. Everything here is in the **checking**
> layer. Do not re-open the partition.

## Finding A — the item enumerator's population is "lines ending in `;`"

`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs:3722`
filters candidate declarations on `trimmed.ends_with(';')`. A Rust item is
**brace-shaped**, so the filter misses every braced form:

| form | ends with `;` | seen by the enumerator |
|---|---|---|
| `use a::b;` / `pub type T = …;` | yes | yes |
| `impl T for S { … }` | no | **no** — the first measured hole |
| `mod x { … }` | no | **no** — the second, found by `B2O`'s `AC-12` |
| `fn f() {}` · `struct S {}` · `trait T {}` | no | **no** — expected, not yet measured |

⭐ **This is why the fix must not be a third accepted spelling.** Two holes were
found one at a time, a third class is already predictable from the filter's
shape, and `the_backend_production_surface_inventory_is_closed` is the pin that
is supposed to make the production surface *closed*. **Key on the item head** —
the leading keyword after attributes and visibility — not on the line's
punctuation.

⚠ **A category fix needs a structural closure, not hand-enumeration.** The
acceptance criterion must therefore include a **positive control per item form**:
each braced form, inserted into production source, is *seen* by the enumerator.
An enumerator that grew a `mod` arm and nothing else has reproduced the bug.

## Finding B — `validate_function_units` advertises 12 laws, 5 are live

`crates/ken-runtime/src/cranelift_backend/planning/static_transition/semantic_ir.rs`

**B1 — one arm is unreachable by construction, and it is the only quadratic
one.** `validate_function_units` (`:987`) opens (`:993`) by calling
`partition_function_units`, which at `:657` rejects a `StaticBody` edge aimed at
a scheduling entry as *"scheduling entry is also a static body target"*. The arm
at `:1121` — *"scheduling entry has an incoming static body edge"* — tests the
**identical condition**, so it cannot fire. Witnessed against **both** entry
classes (the root, and a transparent declaration — the `⚠` comment at `:1115`
exists precisely to say the root is not the only entry); both return the `:657`
message.

⚠ It is also **quadratic**: `:1117` copies `entries` into a fresh `Vec` and
`:1119` calls `Vec::contains` (linear) inside a loop over **all** edges —
O(|edges| × |entries|)
with both operands scaling with program size, paid on every `validate()`, for a
law that never runs.

**B2 — six further arms have no witness.** Each attempt to construct their input
landed on an earlier detector: `:1035` (population not exact for the partition),
`:1050` (owner names an unknown unit), `:1062` (shared exits not `1 + 1`),
`:1086` (static body edge targets a shared exit), `:1094` (target is not its
unit's seed), `:1106` (transfer edge crosses without a static body edge).

⚠ **Claim discipline — only B1 is *provably* unreachable.** The six are "no
witness found on the one route that reaches them." `validate_function_units` is
private and called only from `SemanticPlane::validate` (`:1189`), so the route is
fixed, but absence was not proved. **`:1072`** (*"ownership edge endpoint has no
semantic descriptor"*) was **not probed** — it is reachable in principle via a
`StaticBody` edge whose `from` is out of range, the one endpoint
`partition_function_units` never bounds-checks. Treat that as an open probe, not
a cleared one.

**B3 — a vacuous conjunct.** At `:1006`, `partition.seeds.len() != expected_units`
cannot differ: `seeds` is pushed exactly once per entry and once per `StaticBody`
edge, in the same function that computes `expected_units` from those two counts.
The live conjunct is `self.functions.len()`.

## Finding C — an AC arm naming a class with zero instances

`static_transition.rs:4224-4232` counts *"every other child — **a capture**, or
any child of a non-closure"* into `interior_children` under a no-silent-caps
`interior_children > 0`. Every `B2O` ownership fixture builds `LexicalClosure {
captures: Vec::new(), .. }`, so the **capture class has zero instances** and the
assertion is discharged entirely by non-closure children. `RuntimeExpr::Closure`
— the other of exactly two `EdgeKind::StaticBody` producers — is never planned in
any `B2O` test.

⭐ **Measured, not suspected, and the mechanism is correct on both untested
forms:** a `LexicalClosure` with two captures yields 2 units, 1 `StaticBody`
edge, and 2 capture children both owned by the parent's unit; a bare
`RuntimeExpr::Closure` yields 2 units and 1 `StaticBody` edge. ⇒ **Zero live
risk** — the cost to close it is one fixture plus a `capture_children > 0`
counter. The test's own stated discipline (*"fail if a class never appeared"*) was
applied on the boundary/interior axis and not on the class it names first.

## ⚠ Every line number above is anchored on `e470ab65` — `B2R` moves two of them

`RT-FNSPLIT-B2R` (candidate `293f26ed`, published as PR #967) edits **both** files
this node cites — `semantic_ir.rs` (+9) and `tests/control.rs` (+51/−6). So the
citations here were **stale before the frame was even written**, which is the
perishable-current-state hazard in its ordinary form.

**Measured against `293f26ed`, so the frame can anchor correctly:**

| finding | anchor on `e470ab65` | anchor after `B2R` | still live? |
|---|---|---|---|
| A — `ends_with(';')` filter | `control.rs:3722` | **`:3726`** | ✅ unchanged text |
| B1 — unreachable scheduling-entry arm | `semantic_ir.rs:1121` | `:1121` | ✅ unmoved |
| B3 — vacuous `seeds.len()` conjunct | `semantic_ir.rs:1006` | `:1006` | ✅ unmoved |

⭐ **`B2R` does not touch `validate_function_units` at all** — its `AC-11` work
landed in `validate_edge_agreement` (deleted) and the `D5` surface. So none of
Findings A/B/C is fixed, subsumed, or disturbed by it; they are live exactly as
written.

⛔ **The frame must still re-derive every anchor against `origin/main` at
authoring time, not copy this table.** The table is a measurement of one
candidate, and a further merge invalidates it the same way `B2R` invalidated the
original. Treat *"verify against the landed code, not this line"* as binding on
every citation in this file.

## What the frame must require

1. **Structural closure, not a spelling.** Finding A keys on item heads, with a
   positive control per braced form.
2. **A reachability row per advertised law.** For every arm, either a witness that
   reaches *that* arm (asserting the **exact** error string, never `is_err`) or an
   explicit **"no witness — shadowed by `<arm>`"** row. A law with neither is not
   a law and must not be counted as one.
3. **Deliberate ordering.** Where an earlier detector legitimately subsumes a
   later arm, the resolution is to **delete the dead arm** (and its cost) or to
   re-order deliberately — not to leave twelve stated laws over five live ones.
4. **Do not weaken the live detectors.** The `⛔` paragraph at `:963-991` already
   records that an earlier revision credited one law with work the overlap check
   was doing. The same hazard applies to every arm below it: a reader who trusts
   the count may weaken a check that is actually load-bearing.

## Why this is worth a node at all

`RT-FNSPLIT-B2R` attaches signatures and frame layouts to these units and reads
this validator as its guarantee. **A downstream node inherits an
advertised-but-unenforced law silently**, because the law is *stated*, the
validator *is* fail-closed on the paths that do fire, and the suite is green.
`B2R`'s `AC-11` carries the **discipline** forward so the same gap is not rebuilt
one node up; this node closes the **instance**.
