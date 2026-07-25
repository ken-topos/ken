# `RT-FNSPLIT-B2V` — per-pin evasion attempts

One row per pin, as the frame's evasion AC requires. **Branch:**
`wp/RT-FNSPLIT-B2V-executable-value-abi`, base `origin/main` = `aecdb001`.

> ⛔ **Failing to find a witness is evidence about the witnesses you could
> think of, never about the property.** Every "not found" row below says which
> surface was searched, so the limit is inherited rather than hidden.

## ⛔ TWO MORE PINS WERE FALSE GREENS

Found by QA — and by generalizing QA's finding rather than fixing its instance.

**Runtime QA blocked `c3f6da02`** with a compile-preserving mutation I had
not attempted: `LAST_TAG` from `8` to `8 + 1`. **Every boundary test stayed
green (13/13).** The emitted-code negative probed the single byte `0xFF`, so
tag `9` became accepted by `define_resolve` and nothing asked about it.

★ **The Rust-side twin already swept all 256 bytes.** The discipline was applied
to one side of the same property and not the other — which is precisely how a
per-candidate reminder gets satisfied by the control you were thinking hardest
about. *Sampling one witness is not a sweep, and it was my own table that called
the tag set "closed".*

⇒ **Generalizing the defect class rather than fixing the reported instance found
a second one.** `AC-9`'s inventory pin asserted properties of
`BOUNDARY_LOCAL_HELPERS` — its length, its uniqueness — and **never asked the
emitter anything**. Renaming a helper at its `declare` site left it green.
**The row below that claimed a name swap reddens was FALSE, and I wrote it.**

| pin | mutation | before | after |
|---|---|---|---|
| tag closure at the emitted interface | `LAST_TAG + 1` | 🔴 green (13/13) | 🟢 reddens: *"emitted `class` admitted tag byte 9"* |
| `AC-9` inventory | rename one helper at its `declare` site | 🔴 green | 🟢 reddens, naming `ken_boundary_owner_RENAMED` |

**Both repairs interrogate the artifact instead of the declaration of intent:**
the tag pin sweeps all 256 bytes through the emitted helpers and derives its
expectations from `from_bits`/`referent_owner` — a *different* expression of the
rule than the CLIF's threshold comparisons, so the two must agree rather than
one restating the other; the inventory pin reads the module's actual declared
`ken_boundary_*` symbols. ⚠ Both repairs are **test-only** — every diff hunk
sits inside `mod tests`, and the production bytes QA reviewed are unchanged.

## The one that succeeded — and what it cost

| | |
|---|---|
| **pin** | `b2v_ac3_the_lowered_boundary_disposition_has_no_wildcard_arm` |
| **first form** | `!region.contains("_ =>")` |
| **evasion** | `unhandled => ProtocolOnly { .. }` — a **binding** catch-all |
| **compiles?** | ✅ yes (unreachable-pattern warning only) |
| **pin verdict** | 🔴 **GREEN with the catch-all in place — the evasion won** |

★ A binding catch-all silences exhaustiveness exactly like `_` while matching
no `_ =>` substring. **The pin was a claim about one spelling where the
property is about the shape of every arm** — a *granularity* error, which is
the diagnosis that says to enumerate rather than to add the second spelling to
a list that is open at the top.

**Repair:** every line carrying `=>` in the region must begin with `Lowered::`
or a `|` continuation. Verified as a non-degenerate pair — with the evasion
applied the repaired pin reddens **and names the offending arm**; with it
removed the pin greens.

## Every pin

| pin | evasion attempted | compiles? | verdict |
|---|---|---|---|
| `AC-1` tag set closed (**Rust**) | mint a word with tag byte `9`..`255` | ✅ | 🟢 refused — `from_bits` is a closed `match` with `_ => None`, and the test sweeps **all 256 bytes** rather than a sample |
| `AC-1` tag set closed (**emitted**) | `LAST_TAG + 1` | ✅ | 🔴 **DEFEATED — QA's finding.** The emitted negative sampled one byte. Repaired with a 256-byte sweep through the emitted helpers; verified as a non-degenerate pair |
| `AC-1` list/enum drift | add a `BoundaryTag` variant, leave `ALL` alone | ✅ | 🟢 `ALL`'s declared length `9` reddens; the sweep's `byte < ALL.len()` boundary reddens too |
| `AC-2` no value-specialization | pass a seed value into the construction site | ❌ **does not compile** | 🟢 **strongest row — the surface is closed, not merely unwatched.** `boundary_value` imports no `NativeSeedEnvironment` and no environment vector, so there is nothing in scope to specialize from |
| `AC-2` magnitude boundary | claim the immediate range one value wider | ✅ | 🟢 reddens — the case list tests `MAX`, `MAX+1`, `MIN`, `MIN-1`, not typical magnitudes |
| `AC-3` no wildcard | `_ =>` / `unhandled =>` | ✅ | 🔴 **defeated in first form; repaired — see above** |
| `AC-3` demote a live arm | move `Lowered::Constructor` into `FailClosedForbidden` | ✅ | 🟢 reddens — the fail-closed block is checked **positionally**, not by counting arms |
| `AC-3` second dispatch | add a second `boundary_disposition` with a wildcard | ✅ | 🟢 reddens on the single-dispatch assertion |
| `AC-4`/`AC-5` template read | `field` returns a baked-in word | ✅ | 🟢 **M1** — reddens `left: 7, right: -3`. Discriminating: the head=`7` case still **passed**, so the catch came only from running one probe against three values |
| `AC-4`/`AC-5` constant table | `host_payload` selects a fixed arm | ✅ | 🟢 **M2** — reddens `left: 11, right: 22`, one test, no collateral |
| `AC-6` wrong referent owner | collapse `referent_owner()` to one constant | ✅ | 🟢 **M3** — reddens `left: 1, right: 2` |
| `AC-6` degenerate pair | — | — | 🟢 guarded: the test asserts the two owners **differ**, so an oracle that collapsed them fails instead of agreeing with itself |
| `AC-7` escape | return `OK` for invocation-owned words | ✅ | 🟢 reddens; and a malformed word must report `ERR_TAG`, **not** `ERR_ESCAPE` — a corrupt word and a lifetime violation are different answers |
| `AC-7` vacuity | — | — | 🟢 positive control on the permitted side: persistent and immediate words must return `OK`, so "everything refused" cannot pass |
| `AC-9` inventory | add a 14th helper | ✅ | 🟢 reddens — the count is derived from `BOUNDARY_LOCAL_HELPERS`, so the list and the emission must move together |
| `AC-9` name swap | rename a helper at its `declare` site | ✅ | 🔴 **DEFEATED — the row here previously claimed this reddened, and it did not.** The pin only read the list, never the module. Repaired to compare the module's declared `ken_boundary_*` symbols against the permitted set |
| `AC-1` unreachable tag | keep a tag no disposition produces | ✅ | 🔴 **found by inspection, not by a pin** — `ImmediateCapability` / `ImmediateResource` were in the closed set and unreachable. Fixed by removing them (`f934d233`); ⚠ **recorded as a residual: no mechanical check enforces tag reachability**, so a future unreachable tag is review-caught, not CI-caught |

## Mutation provenance

⚠ Provenance was established by **content delta** (`sha256` + `git diff
--numstat`), never by re-counting an anchor. That mattered: the single-line
anchor `let child = b.ins().load(types::I64, MemFlags::trusted(), address, 0);`
occurs **twice** in `boundary_value_clif.rs` — in `define_field` and in
`define_host_payload`. A naive anchor count would have been ambiguous about
which site was mutated; each mutation was therefore applied through a
uniqueness-asserted multi-line block.

All three restored byte-identically, each verified with `git diff --quiet`
(exit 0). ⚠ `git diff --stat` always exits 0 and is not an emptiness test.

## `AC-8` — the census, predicted before measuring

| pin | predicted | measured |
|---|---|---|
| `correspondence_adds_no_emitted_unit_to_the_production_census` | unchanged | ✅ unchanged |
| `the_backend_production_surface_inventory_is_closed` | unchanged, 12 modules / list 13 | ✅ unchanged |
| `px8i_…identical_local_helper_clif` (`LOCAL_HELPER_COUNT = 6`) | unchanged | ✅ unchanged |
| `lowering/mod.rs` census row | stays `0`/`0`/`0` | ✅ stays `0`/`0`/`0` |

⇒ The frame's original *"the pin you will trip"* prediction was **false for
this placement**, reported before building rather than discovered as a green.
Every landed census is scoped to `cranelift_backend/**`; `native_int_clif.rs`
already declares 8 functions and appears in none of them. **A pin's silence is
scoped to the question it asks**, so their silence about a sibling file is not
evidence — which is why `AC-9`'s boundary-helper census exists at all.

---

# ⛔ ARCHITECT BLOCK of `78a57d90` — two defects in the PRODUCTION mechanism

Neither was repairable at the pin layer, and that is the point worth recording:
**every pin above was green, sound, and asking the wrong question.** The
controls verified that the representation behaved as specified. They could not
observe that the specification itself made a persistent word name storage that
dies, nor that the interface had no way to build one.

## 1. A persistent tag on an ephemeral locator

Every handle payload was an **invocation-arena node index**, while
`check_escape` permitted `Persistent*` words to leave the invocation from the
tag alone. After the arena died, a word cleared to escape named freed storage;
the real `SlotId` was reachable only by first resolving the ephemeral node.

★ **This is the failure mode of a pin that measures conformance to a design.**
Every control asked *"does the code do what the representation says?"* — yes.
None asked *"does the representation's permission agree with its lifetime?"*
That question has no natural home in a per-mechanism control, which is why it
took a reader with the whole design in view.

## 2. A construct interface with one constructor

`make_immediate` was the only producer. Every live handle was minted Rust-side
before publication, so the `D5` controls demonstrated a separately compiled
consumer walking a **Rust-built fixture** — the half `#10` already had. A
producer with dynamic children had no executable way to mint the word.

⚠ **`AC-4` says "construct, discriminate, and project."** The candidate
discharged two of three, and the AC read as satisfied because the two it did
discharge were the two the controls were built around. *A control set assembled
around the mechanism you built cannot notice the clause you did not build.*

## The mutations on the new mechanism

| # | mutation (compile-preserving) | reddens | collateral |
|---|---|---|---|
| **M4b** | `resolve` jumps with `arena` instead of the loaded persistent region — thresholds untouched | survival, both constructor controls, record, nested aggregate, escape, frozen, malformed | 9 tests |
| **M5** | `store_field` drops the persistent-parent escape guard (`dangling := 0`) | `b2v_a_persistent_node_refuses_an_invocation_owned_child` | **none — 1 test** |
| **M6** | the frozen-prefix comparison becomes `index >= 0` | `b2v_the_frozen_prefix_refuses_emitted_mutation` | **none — 1 test** |
| **M7** | the capacity ceiling returns `ERR_BOUNDS` instead of `ERR_CAPACITY` | `b2v_construction_fails_closed_at_each_ceiling` | **none — 1 test** |
| **M8** | `make_immediate` bakes a constant payload | the non-constant constructor control, and survival | 2 tests |

⭐ **M5/M6/M7 each redden exactly one control and nothing else.** That is the
property worth reporting, not the count: a mutation that reddens nine tests
proves the suite noticed *something*, but not which detector fired. A
single-test redden names its detector.

⚠ **M4b is deliberately the surgical form.** The blunt version — setting
`LAST_PERSISTENT_TAG` to `FIRST_HANDLE_TAG - 1` — reddens 11 tests including
the threshold pin itself, which fires by construction and therefore tells you
nothing about the dispatch. M4b leaves the thresholds intact and breaks only
the region lookup, so the 9 reddened tests are all reacting to the *dispatch*.

All five restored byte-identically, each verified with `git diff --quiet`
(exit 0).

⚠ **One anchor missed, and the green it produced was not evidence.** M6's first
form did not match — `rustfmt` had wrapped the expression across three lines
after I wrote the anchor. The run reported **22 passed**, which reads exactly
like "the mutation failed to redden" and is in fact *the unmutated build*. The
uniqueness assertion caught it; without it, a formatting-induced no-op would
have been recorded as a defeated pin. **The harness now asserts the mutation
text is present before restoring, so a silent no-op cannot pass as a result.**

## Evasions attempted against the new pins

| pin | evasion attempted | compiles? | verdict |
|---|---|---|---|
| `AC-6` persistent survival | resolve persistent words against the arena | ✅ | 🟢 reddens — the control **drops** the first arena and resolves through a second invocation that shares only the store |
| `AC-6` survival vacuity | make both regions the same table, so "survival" is trivially true | ✅ | 🟢 guarded by the orphan-arena positive control: an invocation bound to **no** persistent region must return `ERR_BOUNDS`. If resolution silently used the arena, that assertion fails |
| `AC-6` frozen prefix | let emitted code rewrite a materialized node's `SlotId` | ✅ | 🟢 **M6**, one test. Positive control: a node emitted code *allocated* must remain writable, so "refuses everything" cannot pass |
| `AC-7` one layer down | persistent parent embeds an invocation-owned child | ✅ | 🟢 **M5**, one test. **Two** positive controls: a persistent child is admitted, and an invocation parent may hold an invocation child — so the refusal is about the child's owner, not about `store_field` refusing |
| `AC-4` construction is real | producer returns a word for a node Rust built | ❌ **does not compile in the control's shape** | 🟢 the producer is a **separate JIT module** with its own helper graph, compiled once and called three times; it receives the head at run time and holds no image of it |
| `AC-4` constant payload | `make_immediate` ignores its argument | ✅ | 🟢 **M8** — two of the three heads redden. The single-head form would have passed |
| `AC-4` capacity | admit one allocation past the reservation | ✅ | 🟢 the ceiling controls assert the **exact** status, so an off-by-one that still refuses is not enough; **M7** shows an inexact status reddens |
| `AC-4` overflow | caller-supplied `field_count = u64::MAX` wraps the sum into "fits" | ✅ | 🟢 refused — the addend is bounded **before** the sum, and the control passes `u64::MAX` explicitly |
| `AC-1` region bands | reorder `BoundaryTag` so the owner bands are no longer contiguous | ✅ | 🟢 `b2v_the_region_thresholds_agree_with_referent_owner` reddens; it also asserts both bands are **non-empty**, so agreement cannot hold vacuously |

### ⚠ Residuals — stated, not detected

| residual | why no mechanism | who catches it |
|---|---|---|
| tag **reachability** — a tag no disposition produces | needs a whole-program reachability argument over the disposition; the closed-set pin asks a different question | review (unchanged from the prior candidate) |
| emitted-constructed persistent nodes are **not content-addressed** (`NULL_SLOT`) | interning is a content-addressing pass over a whole value; it is not Θ(1) at a construction site | pinned as an *assertion of the limit* in the survival control, and documented at `BoundaryPersistentImage`. Closing it is a `B2F` lifecycle decision |
| the escape check does not **walk** a structure | O(size) at every crossing, re-answering a question settled at construction | the construction-time invariant is enforced at both paths (`materialize` debug-asserts; `store_field` returns `ERR_ESCAPE`) and **M5** pins the emitted half |

⛔ **The middle row is the one to read twice.** "Survives the invocation" and
"is content-addressed" are different properties, and a persistent region could
easily be described as giving both. It gives the first. The control asserts
`NULL_SLOT` explicitly so the boundary is inherited rather than assumed.

---

# AC → discharging control

Required by the frame's second amendment (`origin/main` = `fdda953f`): one row
per AC, with evidence. ⛔ **An AC with no control reads exactly
`NO CONTROL — open residual`** — there is no cell here for "covered by review",
because a taxonomy with nowhere to put the honest answer records it as covered.

| AC | discharging control | evidence |
|---|---|---|
| **AC-1** closed + type-enforced | `b2v_the_tag_set_is_closed_in_both_directions`; `b2v_emitted_code_admits_exactly_the_closed_tag_set` | Rust sweep + **256-byte emitted sweep** through `class` and `escape_check`; admitted count `== ALL.len()`, rejected `== 256 - ALL.len()`. A new tag is a compile error at `from_bits`'s closed `match` and at `ALL` |
| **AC-1** region bands | `b2v_the_region_thresholds_agree_with_referent_owner` | the CLIF's numeric bands classify every tag exactly as `referent_owner()`; both bands asserted non-empty |
| **AC-2** no value-specialization | **the compiler** + `b2v_the_immediate_handle_choice_tracks_magnitude_only` | `boundary_value` imports no `NativeSeedEnvironment` and no environment vector — passing one **does not compile**. Magnitude cases test `MAX`, `MAX±1`, `MIN`, `MIN±1` |
| **AC-3** exhaustive, no wildcard | `b2v_ac3_the_lowered_boundary_disposition_has_no_wildcard_arm` | arm-*head* enumeration (every `=>` line starts `Lowered::` or `\|`), 21 variants named, `Constructor`/`HostResult` checked **positionally** outside the fail-closed block, single-dispatch assertion. ⚠ first form was **defeated** by a binding catch-all |
| **AC-4** **construct** | `b2v_emitted_code_constructs_a_nonconstant_constructor_and_a_consumer_reads_it`; `…constructs_both_host_result_arms`; `…constructs_a_record_readable_by_name`; `b2v_construction_fails_closed_at_each_ceiling` | a separately compiled **producer** mints each live class from `alloc` + `store_*`; one compiled body, three runtime heads; every ceiling and closed-set refusal asserted at its **exact** status |
| **AC-4** **discriminate** | `b2v_emitted_code_selects_the_host_result_arm_at_runtime`; `…constructs_both_host_result_arms` | both arms, runtime discriminant, same compiled body |
| **AC-4** **project** | `b2v_emitted_code_projects_a_non_constant_constructor_field`; `…projects_a_nested_aggregate` | separately compiled consumer; no step runs in Rust |
| **AC-5** M1/M2/M3 | recorded in the mutation table above and in the prior candidate's rows | each names **which** detector fired and its `left`/`right` |
| **AC-5** M4b/M5/M6/M7/M8 | the new mutation table | **M5, M6, M7 redden exactly one control each** — the detector is named by the redden itself |
| **AC-6** owner distinguishable | `b2v_referent_owner_distinguishes_persistent_from_borrowed` | `BoundaryReferentOwner` is a **distinct type** from `AbiStorageOwner`; the pair is non-degenerate (`left: 1, right: 2`) |
| **AC-6** persistent identity | `b2v_a_constructed_persistent_word_survives_the_invocation_arena`; `b2v_the_frozen_prefix_refuses_emitted_mutation`; `b2v_equal_values_share_one_persistent_referent` | the arena is **dropped** and a second invocation resolves the same word; orphan-arena positive control returns `ERR_BOUNDS`; one slot ⇒ one word, byte-identical across invocations |
| **AC-7** escape, exact error | `b2v_borrowed_ingress_fails_closed_on_escape_with_an_exact_error`; `b2v_a_persistent_node_refuses_an_invocation_owned_child` | exact `ERR_ESCAPE`; malformed ⇒ `ERR_TAG`, not `ERR_ESCAPE`; the construction-time invariant the Θ(1) check rests on is itself pinned, with both mirrors as positive controls |
| **AC-8** INERT | `correspondence_adds_no_emitted_unit_to_the_production_census`; `the_backend_production_surface_inventory_is_closed`; `px8i_…identical_local_helper_clif` | all three **unchanged and re-run**; `lowering/mod.rs` census stays `0`/`0`/`0`. ⚠ **Helper delta re-baselined: 13 → 19** — `alloc`, `store_slot`, `store_tag_id`, `store_scalar`, `store_field`, `store_name`. Stated here before the verdict, not discovered in it |
| **AC-9** Θ(1) per module | `b2v_helper_population_does_not_grow_with_the_value_population`; `b2v_the_helper_inventory_is_closed_and_named` | two module emissions compared, and the value population varied ×1024 with the helper count fixed; the inventory pin reads the **module's actually declared** `ken_boundary_*` symbols against the permitted set. ⚠ first form was **defeated** — it only read the list |
| **AC-1** tag *reachability* | **`NO CONTROL — open residual`** | a tag no disposition can produce is caught by review, not by CI. Found by inspection at `f934d233`; nothing mechanical enforces it |
| **AC-6** persistent *content-addressing* | **`NO CONTROL — open residual`** | an emitted-constructed node carries `NULL_SLOT`. The **limit** is pinned (the survival control asserts it); the property is not delivered. Closing it is a `B2F` lifecycle decision |
