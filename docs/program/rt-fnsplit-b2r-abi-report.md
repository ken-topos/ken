# `RT-FNSPLIT-B2R` — the representation and call-ABI contract: evidence

**Base:** `wp/RT-FNSPLIT-B2R-representation-abi` off `origin/main` = `c5edea8b`.
**Predictions:** `docs/program/rt-fnsplit-b2r-predictions.md`, committed at
`b7aacd03` **before** `abi.rs` existed.
**Suite:** `scripts/ken-cargo test -p ken-runtime --lib` — **366 passed, 0
failed.** Workspace, `--locked`, and conformance run in CI, not here.

This is `D6`. It records what was predicted, what was measured, every mutation
outcome **including the invalid ones**, and the boundaries this node does not
cross.

## What landed

One new production module,
`crates/ken-runtime/src/cranelift_backend/planning/static_transition/abi.rs`
(1026 lines), attaching a frame layout to each `PredeclaredFunction` in `B2O`'s
validated `SemanticOwner` partition. Nine `b2r_*` controls. Three existing
files touched: the planner (module declaration, plane field, wiring, tests), the
semantic plane (one visibility widening), and `control.rs` (census
registration).

⛔ **Inert.** No `FunctionBuilder`, no `define_function`, no call edge, no
dispatch edge, no encoder, no decoder. `RT-FNSPLIT-B2F` performs the atomic
switch-over.

## `AC-10` — predicted, then measured

| # | predicted at `b7aacd03` | measured | |
|---|---|---|---|
| `P1` | 8 descriptors over the seven topology fixtures (1/1/1/2/1/1/1) | **8**, same split | ✅ |
| `P1` | 4 descriptors on the two-closure fixture | **4** | ✅ |
| `P2` | unit count unchanged under an irrelevant caller binding | unchanged | ✅ |
| `P2` | descriptor **shape** invariant, **identity** fields shift | exactly that | ✅ |
| `P3` | seed-value independence is **structural**, not a green test | the builder's inputs hold no value; enforced by the signature | ✅ |
| `P4` | no per-slot static **type** is derivable from this plane | confirmed — the atoms carry `ParamName`/`CaptureSymbol`, names not types | ✅ |
| `P5` | imported exclusion reachable and non-vacuous | reachable; witness constructed | ✅ |
| `P6` | 1 `FunctionBuilder::new`, 1 `define_function`, 0 Cargo delta | **1 / 1 / 0 bytes** | ✅ |
| `P8` | **0 source-text-reading pins in the touched files** | **13** | ⛔ **MISS** |
| `P9` | `< 20` production lines changed in existing files | ~738 added to the planner (mostly tests) | ⛔ **MISS** |

### ⛔ The `P8` miss, and why it is the interesting one

I predicted this WP would not touch `control.rs`. It does. **Registering a new
production module is not optional**: `the_backend_production_surface_inventory_
is_closed` pins the exact module inventory, so adding `abi.rs` reddens it until
the file is declared in `BACKEND_PRODUCTION_SOURCES`. That pin worked exactly as
designed and my prediction did not account for it.

The consequence is not cosmetic. Adding a file to `BACKEND_PRODUCTION_SOURCES`
**changes the input to every pin that iterates that list**, so ten pre-existing
`control.rs` pins now scan `abi.rs` too. The population `AC-9` closes over grew
from a predicted 0 to a measured 13 for that single reason.

`P9` missed because I sized "production lines changed" and then put ~700 lines
of `b2r_*` controls in the planner's existing test module rather than a new
file. The production delta in existing files is genuinely small (a `mod`
declaration, a plane field, two wiring calls, one visibility widening); the
prediction was miscategorised, not the code.

## `AC-9` — the source-text-reading pin population, closed over the touched files

**Enumeration method.** The file set is `git diff --name-only c5edea8b HEAD`,
not a list I recalled. Within it, every `fn` extent is taken by brace balance
over a **literal-masked** copy of the source, and a function is in the
population iff its body reaches `include_str!`, `include_bytes!`,
`BACKEND_PRODUCTION_SOURCES`, or `LOWERING_IMPL_SOURCES`.

⚠ **The literal-masking step is the whole instrument.** `B2O`'s equivalent sweep
reported **16** where the truth was **14**, because its brace counter did not
skip string literals and `{}` inside `assert!` format strings ran extents past
their ends. Extents here are 22–90 lines, with no 900-line bodies — the
signature of that failure is absent, and the count was cross-checked against a
raw per-file needle count.

**Measured population: 13.**

| # | pin | file | outcome |
|---|---|---|---|
| 1 | `the_backend_production_surface_inventory_is_closed` | `control.rs` | ⚠ **stays green** — `E1` |
| 2 | `b2r_ac6_the_abi_plane_declares_no_emission_construct` | `static_transition.rs` | ⭐ **cannot compile** (`E4`) **+ reddens** (`E4b`) |
| 3 | `b2r_ac7_the_abi_plane_adds_no_parser_and_no_dependency_edge` | `static_transition.rs` | ✅ **reddens** — `E5` |
| 4 | `the_entry_carrying_types_are_module_private` | `control.rs` | inherited residual — see below |
| 5 | `the_owner_classification_has_a_closed_production_naming_inventory` | `control.rs` | inherited residual — see below |
| 6–13 | `correspondence_adds_no_emitted_unit_to_the_production_census`, `exactly_one_plan_origin_to_expression_lookup_exists`, `the_retained_body_helper_carries_no_visibility_qualifier`, `every_source_term_carrier_holds_an_occurrence_and_never_a_bare_expression`, `retained_closures_carry_a_static_origin_and_no_body_term`, `no_collection_is_keyed_by_a_scheduling_entry`, `the_lower_expr_call_population_is_dispositioned_by_owner_not_by_site`, `the_semantic_seed_api_accepts_only_occurrence_origins` | `control.rs`, `static_transition.rs` | mechanism **byte-unchanged** by this WP; classified in `B2O`'s `AC-12` table |

⚠ **Honest basis for rows 6–13, stated rather than implied.** Their *mechanism*
is untouched here, so their `B2O` classification carries. Their *input* did
change — they now scan one more file. All are green, and none of them is a
claim about `abi.rs` specifically, so a fresh evasion per pin would re-measure
`B2O`'s answer rather than this node's. **What this WP does not do is re-derive
those eight outcomes from scratch**, and a reader should treat them as inherited
rather than as measured here.

### Rows 4 and 5 — inherited residuals, not fresh discharges

Both are **naming inventories**, and both carry `B2O`'s recorded residual
verbatim: *visibility and naming bound **naming**, not **reaching***. A type is
reachable through a method that returns it, an `impl Trait`, a re-export, or a
derived ordinal without ever being named. That residual is unchanged, is
review-enforced, and is **not** claimed as detected.

⭐ **Row 5 moved twice in this WP, and the round trip is the point.** I first
renamed it — `..._is_named_in_production_only_by_the_module_that_defines_it` →
`..._has_a_closed_production_naming_inventory` — because `abi.rs` named
`SemanticOwner` and the old name asserted a sole-consumership this node
falsified. Then `AC-11` deleted the only code that named it, and the inventory
returned to one member. The name stayed at the new spelling because the new
spelling is the more honest claim either way: what is guarded is that the
inventory is **closed**, not that it is **singular**.

⇒ **`B2R` consumes the owner partition without naming its type.** It reads the
seed table (`plane.functions`); the classification enum never appears. Consuming
a partition and naming its type are different things, and only the second is
visible to this pin.

## Mutation ledger — every attempt, including the ones that are not evidence

⚠ **A verdict from a mutation that did not apply is not evidence, and it looks
exactly like evidence.** Each row records whether the mutation *applied*
(measured as a **content delta**, never an anchor re-count), whether the crate
actually *recompiled*, and whether the tree restored **byte-identically**.

### `AC-8` — the control inverts. All four must stay GREEN.

| # | mutation | applied | recompiled | verdict | predicted | |
|---|---|---|---|---|---|---|
| `M1` | rename a production `fn` (`declared_arity`) | ✔ | ✔ | **GREEN** | GREEN | ✅ |
| `M2` | add a Rust wrapper around `frame_header` | ✔ | ✔ | **GREEN** | GREEN | ✅ |
| `M3` | widen a type's visibility (`AbiSlotKind`) | ✔ | ✔ | **GREEN** | GREEN | ✅ |
| `M4` | move a production `fn` **between files** | ✔ | ✔ | **GREEN** | GREEN | ✅ |

⇒ **No pin in this node reddens on a rename, a wrapper, a visibility change, or
a cross-file move.** Boundary classification is not measuring source topology.

### ⛔ Invalid attempts — reported, not discarded

| attempt | what happened | why it is not evidence |
|---|---|---|
| `M1`, first run | harness expected 4 anchor occurrences, found 3 | **anchor miss** — the harness refused to emit a verdict, which is the correct behaviour |
| `M2`, first run | renamed the definition and the call site to *different* names, never defining the wrapper | **broke the build** — a malformed mutation, not an evasion |
| `M1`/`M4`, first run | reported `applied=False` while the edit had in fact landed | ⛔ **my provenance check was wrong**: it counted the anchor *after* replacement, and the replacement string **contains** the anchor. Fixed to compare file content; both then measured `applied=True` with a non-zero byte delta |

⭐ The third row is the one worth carrying. The instrument that exists to
certify a mutation *applied* produced a **false negative**, and a false negative
there is as corrupting as a false positive: it would have led me to discard two
sound GREEN results as unproven.

### `AC-9` evasions

| # | evasion | applied | recompiled | verdict | predicted | |
|---|---|---|---|---|---|---|
| `E1` | inline `mod evasion_probe { }` in `abi.rs` | ✔ | ✔ | ⚠ **GREEN** | GREEN | ✅ |
| `E4` | name `FunctionBuilder` in `abi.rs` production | ✔ | ✔ | ⭐ **CANNOT COMPILE** | REDDENS | ⚠ miss |
| `E4b` | **import** `FunctionBuilder`, then name it | ✔ | ✔ | ✅ **REDDENS** | REDDENS | ✅ |
| `E5` | `include_str!` in `abi.rs` production | ✔ | ✔ | ✅ **REDDENS** | REDDENS | ✅ |

**`E1` — ⚠ a FINDING, reported and deliberately NOT fixed.** The module
inventory enumerator filters on `trimmed.ends_with(';')` (`control.rs:3726`), so
its population is *lines ending in a semicolon*. An inline `mod x { }` has no
semicolon and escapes it. This is the **same defect in a third substrate** —
`impl` escaped it first, `mod x { … }` second — and it is already tracked as
`RT-FNSPLIT-B2O-CHECK`, whose fix keys on **item heads** rather than adding a
third accepted spelling. Repairing it here would be scope growth into another
node's tracked work.

**`E4` — the miss is in the favourable direction, and the follow-up is what
makes it honest.** I predicted REDDENS; the result was CANNOT COMPILE, because
`FunctionBuilder` is not in scope in `abi.rs`. ⭐ The compiler enforces it, which
is the strongest and cheapest outcome. But *"stronger than predicted"* is not a
licence to stop: had I stopped there, I would have recorded a pin as
compiler-enforced without knowing whether the pin does any work at all. `E4b`
takes the route that **does** compile — import the construct, then name it — and
the pin reddens. Both layers are real, and neither alone would have said so.

## The two premise errors this node made, and what caught them

Recorded because the mechanism that caught each is the reusable part.

**1. `semantic_sources` is in WALK order, not positional by origin.** My lookup
assumed `sources[origin.0]` was that origin's seed. It is not — which is exactly
why `build_semantic_plane` calls `positioned_sources` before reading one. The
error surfaced as 51 failing tests, and it surfaced *as an identity assertion*
(`"source seed origin is not its preallocated positional identity"`) rather than
as a plausible wrong seed silently threaded into every descriptor. **A lookup
that asserts the identity of what it retrieved fails loudly; one that trusts the
index does not fail at all.** Fixed by reusing `positioned_sources` — widened to
`pub(super)` — so the two planes cannot disagree about what "the seed for this
origin" means.

**2. `C4` excludes imported EDGES, not imported mentions.** My first
implementation rejected every plan whose *any* occurrence had an unrepresentable
result carrier, which condemned any program merely containing an
`ImportedDeclarationRef`. `C4`'s scope is the position where such a value would
have to **cross a frame boundary and be given a carrier** — a capture slot.
Caught by `every_expression_typed_field_is_a_reachable_positional_child_origin`,
a pre-existing property test that legitimately enumerates every expression
shape. **An over-broad guard is a defect even when it never admits anything
wrong**, and the test that caught it was not testing `C4` at all.

## `AC-11` — which arm actually fires

The amendment landed mid-build and was applied while `D5` was being written,
which is the cheap order. It reproduced its own stated failure mode **in this
node's validator**.

`D5` advertises six rejection classes. Each has a constructed witness; each
asserts the **exact** planner error, never `is_err`/`expect_err`.

| `D5` class | arm that actually returned | |
|---|---|---|
| missing capture slot | `"abi descriptor is missing a declared capture slot"` | ✅ own arm |
| extra capture slot | `"abi descriptor declares a capture slot its origin does not have"` | ✅ own arm |
| implicit caller-env tail | `"abi frame carries an implicit caller-environment tail"` | ✅ own arm |
| edge layout agreement | `"abi descriptor is not positional for its function unit"` | ⚠ **subsumed** |
| recursive-bundle forward declaration | `"abi descriptor population is not exact for the function unit partition"` | ⚠ **subsumed** |
| imported capture edge | `Unsupported(ImportedDeclarationRef, "… requires dependency linking …")` | ✅ own arm |

### The two subsumed classes, and the deletion

Both lived in a `validate_edge_agreement` function that advertised **six** laws.
**Every one of them was unreachable.**

⚠ **Two independent witnesses were tried for the edge-agreement arm, not one.**
Mutating the descriptor alone fires the positional check. Mutating the
descriptor **and** its unit together — so the positional check can no longer
shadow it — fires `"function unit seed is neither a scheduling entry nor a
static body target"` from the definition re-derivation at the top of `validate`.
Dead on both routes.

⇒ **The function is deleted**, per `AC-11`'s remedy, with the composition that
makes it redundant cited in its place:

- `B2O` proves `functions[callee].planned_node == edge.to`
  (`semantic_ir.rs:1093`);
- this plane proves `descriptors[i].planned_node == functions[i].planned_node`;
- together: `descriptors[callee].planned_node == edge.to` — **exactly what the
  deleted arm asserted.** It could not fail without one of its two premises
  failing first, and each premise has its own witness.

**The property is enforced; what was deleted is a restatement.** That
distinction is the whole content of the row, and stating it the other way round
— "the check was removed" — would be false.

⛔ **`B2O`'s own validator is NOT repaired here.** It is tracked as
`RT-FNSPLIT-B2O-CHECK`. Inherit the discipline, not the diff.

## Boundaries this node does not cross

State these as limits, not as coverage:

1. **The value-independence claim is about the DESCRIPTOR.**
   - **MEASURED:** `build_abi_plane`'s inputs contain no `RuntimeGroundValue`
     and no `Lowered`.
   - **CLAIMED:** a seed capture's layout is not chosen by inspecting its value.
   - **THE GAP:** this says nothing about whether `B2F`'s *emission* path stays
     value-independent. That obligation is `B2F`'s.
2. **No per-slot static type is derivable from this plane**, so the layout
   language is a closed **carrier** language keyed on slot role and provenance —
   the sanctioned answer where a layout cannot be derived statically, not a
   derived type lattice dressed up as one.
3. **`AC-6` is a declaration inventory plus a behavioural pin.** A source census
   cannot see an executable edge; inertness itself is pinned behaviourally by
   `correspondence_adds_no_emitted_unit_to_the_production_census`.
4. **Rows 6–13 of the `AC-9` table are inherited**, not re-measured here.
5. **The `E1` inline-`mod` hole is open**, reported, and owned by
   `RT-FNSPLIT-B2O-CHECK`.
6. **Ownership modes are declared, not enforced.** `AbiOwnership` states each
   carrier's lifetime/transfer/reclamation rule and the validator checks each
   slot carries **its carrier's own** declaration — it does **not** verify any
   emitted code obeys those rules, because nothing is emitted. Enforcement is
   `B2F`'s.
