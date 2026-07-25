# `RT-FNSPLIT-B2O` `D6` — the 59-call disposition, BY OWNER

**Status: derived report. Evidence, never authority.** The authority is the
ownership mapping in the semantic plane (`SemanticOwner`, validated by
`SemanticPlane::validate_function_units`). This document reports the `lower_expr`
call population against that mapping; if the two disagree, the mapping is right
and this document is stale.

- WP: `RT-FNSPLIT-B2O` · branch base `origin/main = 3baa80f4`
- Re-derived on `3baa80f4` — **every line number below was measured on my own
  base**, not inherited. `git diff --quiet 9d515c9d 3baa80f4 -- crates/` exits 0,
  so the frame's anchors hold, but the retained-body consumer set was last stated
  against an older tree and is re-grepped here.
- Date: 2026-07-25

## The population: 59, and why it is not 58

Measured with the tokenizer (`identifier_occurrences`,
`lowering/core/tests/control.rs`), pinned by
`the_lower_expr_call_population_is_dispositioned_by_owner_not_by_site`:

| quantity | value | how |
|---|---|---|
| `lower_expr` tokens in `lowering/core.rs` | **60** | tokenized, comments stripped |
| definitions (`fn lower_expr(`) | **1** | declaration-line match |
| **production calls** | **59** | `tokens − definitions`, derived |
| what `grep -c 'self\.lower_expr('` returns | **58** | ⛔ wrong |

⛔ **The missing call is the entry point.** The root call is spelled
`compiler.lower_expr(` (`lowering/core.rs:188`) and takes `root_static_origin`,
so it *seeds* the descent rather than traversing. A receiver-spelled scan is a
census of the **receiver**, and at the entry the object is a local being built,
so it is spelled by its variable name. The pin carries a discriminator — a
two-call input where the tokenizer answers 2 and the `self.`-scan answers 1 — so
"use the tokenizer" is a checked property rather than advice.

⇒ The count is asserted as a **relation** (`calls = tokens − definitions`), so a
call added or removed reddens with an arithmetic explanation rather than a bare
mismatch.

⚠ **Correction (Architect, `evt_5984e30gv9f0k`).** This line previously said the
count was asserted as a relation **"never as a frozen `59`"**. That was false:
`control.rs` literally asserts `calls == 59`. The pin carries **both** the
relation *and* the absolute baseline. Both are intended — the relation explains a
failure, the baseline detects one — but the report must not describe the pin as
weaker than it is in order to sound more principled than it is.

## ⛔ Why there is no 59-row table, and why that is the point

`AC-5` of the superseded `B2F` frame required one disposition row per source
site. **That was withdrawn as unsound, not as incomplete.** Disposition is a
function of the **occurrence and its reaching path**, not of the site: for the 14
caller-dependent sites the same parameter carries both a retained body and
ordinary sub-expressions, so one row cannot answer for both. A per-site table
could have been filled in *completely* and still been wrong — a taxonomy with no
cell for the honest answer reads as complete.

**The repair is not a better table, and — corrected under the ruling — it is not
a closed inventory of Rust functions either.** An earlier revision said exactly
that, and it is the claim this respin withdraws. ⇒ **The repair is to disposition
by the plan graph**: an occurrence's `SemanticOwner` and the planned edge kind,
which is what `validate_function_units` already enforces and what no Rust
refactor can move.

## The disposition, derived from the mapping

`B2O` makes an `EdgeKind::StaticBody` edge the **one and only** owner boundary
(`D3`, checked in `validate_function_units`). Therefore:

> A call into `lower_expr` crosses an owner boundary **iff the occurrence it
> lowers is a `StaticBody` target** — that is, iff it lowers a **retained body**.

Retained bodies reach the lowering through one `origin → expression` route.
Re-grepped on `3baa80f4`:

```text
StaticTransitionPlan::source_occurrence        1 production call  core.rs:4183
  └── retained_body_occurrence  (def core.rs:4176)
        ├── 7 direct consumers   core.rs:327, 605, 620, 764, 4817, 4829, 4954
        └── machine_body_occurrence  (def core.rs:4203, calls at :4208)
              └── 3 consumers     core.rs:3869, 3890, 4023
```

| class | count | crosses an owner boundary? |
|---|---|---|
| retained-body occurrences, direct | 7 sites | **yes** — the occurrence is a `StaticBody` target |
| retained-body occurrences, via the machine wrapper | 3 sites | **yes** — same route, one delegation deeper |
| everything else | the remaining calls | **no** — `child_occurrence` traversal, intra-owner by `D3` |

### ⛔⛔ WITHDRAWN — the route oracle is out, and the claim it defended
### was never required

**Architect ruling 2026-07-25 (`evt_5yxjd1zqnyvcq`, durable at `architect/work`
`8bff7b7a`).** Three revisions of this section carried a mechanized closure claim
over the *Rust route set*. The mechanism is withdrawn and the claim is narrowed.

⛔ **The authority is the plan graph** — an occurrence's `StaticOriginId`, its
validated `SemanticOwner`, and the planned edge kind. **Rust syntax cannot prove
reachability**: it has no name resolution, no macro expansion, and no
indirect-call semantics. More fundamentally, **a Rust wrapper or a same-named
method in another `impl` creates no Ken function-unit boundary** — so a pin that
reddened when one was added was measuring implementation topology and reporting
success.

### The reachability claims removed, and the sentence that replaced each

| removed claim | replaced by |
|---|---|
| *"the inventory **cannot grow silently**"* | *"the routing-function inventory is **not mechanized**; a new Rust route is a review boundary."* |
| *"one route with a pinned consumer count"* / *"a tenth consumer cannot appear without reddening the lookup-count pin"* | *"the `source_occurrence` pin constrains that identifier only, and says nothing about who calls `retained_body_occurrence`."* (This correction was itself correct and is retained as history.) |
| *"the inventory closes **DRIFT**, not reachability"* | *"nothing here closes drift; drift in the Rust route set is unobserved by any pin in this WP."* |
| *"`retained_body_helper_is_private` ⇒ `mod.rs` cannot reach the helper, so the inventory is still correct"* | *"the helper is declared with no visibility qualifier"* — the declaration survives; the entailment does not. |
| the pin name *`..._has_no_reach_into_any_emission_path`* | renamed to *`..._is_named_in_production_only_by_the_module_that_defines_it`*, because a type is reachable without being named. |

### ⛔ Rust-route closure is UNMECHANIZED

**No pin in this work package establishes that the set of Rust functions able to
reach a retained body cannot grow.** That is guarded by **review, not by CI**,
and this sentence is the honest cell rather than a gap.

The residuals, named rather than hidden:

1. a call relocated into a **nested `fn`**;
2. **equal-named methods across `impl` blocks** (legal across distinct types);
3. **macro-generated routes** — invisible to any source scan, and to `syn`,
   which parses invocations rather than expansions;
4. **same-named nested `fn`s in sibling blocks** of one method.

⚠ There is **no test that greps this report for a forbidden phrase**, and there
must not be: such an oracle fires on the prose *denying* the claim, so it would
redden on the very sentences above. The discharge is this table plus review.

### Frozen review evidence

Everything below is **evidence for a reader, never authority for a gate**, and
nothing downstream may key on it:

| frozen observation | value on this SHA |
|---|---|
| tokenized production calls into `lower_expr` | **59** |
| `source_occurrence` identifier occurrences | definition + 1 call inside `retained_body_occurrence` |
| `retained_body_occurrence` consumer mentions | 8 |
| `machine_body_occurrence` consumer mentions | 3 |
| files declaring the retained-body helper | `lowering/core.rs` only |
| helper's declared visibility | no qualifier |

⭐ **And the part that cost four candidate SHAs.** The phrase *"cannot grow
silently"* appears **zero** times in the WP frame and **zero** times in this
report at `97db6f0b`, the first QA-approved tree. It was introduced **by a fold**,
and then four folds were spent defending it. A claim that outruns its evidence
has two repairs — strengthen the evidence, or **narrow the claim** — and nobody
asked whether the claim was required. It was not. ⇒ **Before hardening a
mechanism to support a claim, check whether the claim is required at all.**
## What this does NOT say

- **It does not partition the 59 calls into two disjoint sets of source sites.**
  It cannot, and that is the finding: for the caller-dependent sites the answer
  is per `(site × reaching path)`. ⛔ **Nothing here is closed.** An earlier
  revision ended this bullet *"what is closed is the routing-function
  inventory"*; that inventory is withdrawn and no closure claim replaces it.
  Disposition derives from `StaticOriginId` + validated `SemanticOwner` + the
  planned edge kind, and from nothing in this document.
- **It does not close DRIFT either.** An earlier revision said the inventory
  guaranteed that a new function able to reach a retained body could not be added
  silently. ⛔ **That guarantee is withdrawn and nothing replaces it in this WP.**
  A new Rust route can be added and no pin here will observe it; see the
  UNMECHANIZED section above for the four named residuals.
- **The five provenance classes** (32 `child_occurrence` + 9
  `case_body_occurrence` + 14 caller-dependent + 2 synthesized + 1 direct) remain
  **evidence inputs**, never the authority partition. They are not restated here
  as a disposition, because doing so would reintroduce the site keying.
- **Nothing here is a scaling claim.** No emitted unit changed; `B2O` is inert.

## MEASURED / CLAIMED / THE GAP

- **MEASURED:** a **validated ownership partition** of the plan graph — every
  planned node carries exactly one `SemanticOwner`, and `StaticBody` is the only
  cross-owner edge kind, both enforced as planner errors before emission.
  Separately and as **frozen evidence only**: 59 tokenized production calls into
  `lower_expr`, one `origin → expression` route, and 8 + 3 consumer mentions.
- **CLAIMED:** that the plan graph carries a total, exclusive, validated
  occurrence → function-unit ownership mapping, and that **semantic disposition
  is a function of that graph alone** — mutation-verified in both directions:
  a Rust wrapper relocation and an equal-named method in a second `impl` each
  leave it **unchanged** (`AC-10a`/`10b`), while repointing a `StaticBody` edge
  is **refused by planning** (`AC-10c`).
- ⚠ **NOT CLAIMED — and this is the claim that was withdrawn:** that the set of
  Rust functions able to reach a retained body is closed, bounded, or observable.
  **It is none of those here.** No count, no inventory, and no source scan in
  this WP constrains it; that is a review boundary, stated in its own section
  above. The frozen 59-call census is evidence a reader may consult and **not**
  something any gate may key on.
- **THE GAP:** this says **where** the boundaries are and **which calls** cross
  one. It says nothing about **what may cross** — no signature, no slot layout,
  no calling convention, no lifetime or ownership rule. ⛔ Hard-stop #9's missing
  native value representation is **not** discharged by anything in this node or
  this report; that is `RT-FNSPLIT-B2R`. Reading "the boundary population is
  closed" as evidence that functionization is now buildable is the same inference
  #5 and #8 were defeated on.

---

# `AC-12` — every source-text-reading pin, classified

**The population was closed by a sweep I ran, not by the frame's table** — the
frame gives the *discriminator*, not the *population*, and a hand list covers
only the cases someone thought of.

**Discriminator, fixed before counting:** a pin is *source-text-reading* iff its
assertion consumes the **text of a `.rs` file** (`include_str!` / a path read)
rather than a value the compiler or planner produced. A pin that reads a plan,
a descriptor, an owner, or an edge is **not** in this population.

## ⚠ The count, and the two misses

**PREDICTED 11 (committed at `4160c70a` before any sweep). MEASURED 14.**

⛔ **And the first sweep said 16, which was wrong.** Its brace counter did not
skip string literals, so `{}` inside `assert!` format strings ran each function's
extent past its real end and swallowed downstream `include_str!` calls —
producing bodies of 900–1400 lines, which is impossible for these tests. **The
implausible magnitude is what exposed it**, not a failing assertion. Rebuilt with
a literal-aware lexer and cross-checked against an independent needle count over
independently-derived extents; the two agree at 14.

Two distinct prediction errors, recorded rather than smoothed:

1. **I named two pins that are not in the population** —
   `the_routing_function_enumerator_sees_a_relocated_call` and
   `the_field_inventory_extractor_sees_an_added_term_field` run on **synthetic
   strings**, not on real source files.
2. **I missed five that are**, including
   `the_semantic_seed_api_accepts_only_occurrence_origins` in
   `static_transition.rs` — a file where I predicted **zero**.

⇒ Net −3, but the composition error is larger than the net, and that is the
point of closing the enumeration by sweep.

## The rows

| pin | claim it makes | class | disposition | evasion attempted → outcome |
|---|---|---|---|---|
| `the_retained_body_routes_are_a_closed_inventory_of_named_functions` | which Rust functions can reach a retained body | **REACHABILITY** | **REMOVED** | nested-`fn` relocation → **defeated it** (Architect, `evt_7keypnnsrr0cd`); withdrawn rather than re-patched |
| `the_method_boundary_oracle_enforces_its_impl_shape_premise` | the attribution's impl-shape premise holds | **REACHABILITY** (supports the above) | **REMOVED** | premise itself invalidated by the ancestry rewrite; removed with its mechanism |
| `a_cross_file_route_reddens_for_the_cross_file_reason` | a route added in `mod.rs` reddens | **REACHABILITY** | **REMOVED** | reddened at the *definition-count* assertion whose message recommended the evasion |
| `the_method_boundary_oracle_holds_on_the_second_impl_file` | the oracle's premise holds on `mod.rs` | **REACHABILITY** | **REMOVED** | `mod.rs` has 7 top-level `impl`s + a nested `impl Drop`; the single-impl premise was false |
| `the_routing_function_enumerator_sees_a_relocated_call` | the enumerator sees a moved call | REACHABILITY (synthetic input — **not** in the 14) | **REMOVED** | it certified the indent-8 blind spot as *intended*; the control pinned a defect as the specification |
| `retained_body_helper_is_private` (+ its new pin) | the helper carries no visibility qualifier | **DECLARATION** — entailment struck | **SPLIT / RETAINED** | widened the declaration to `pub fn` → **reddens**; the *reachability* entailment it used to carry is deleted, not defended |
| `..._is_named_in_production_only_by_the_module_that_defines_it` | which production files name `SemanticOwner` | **DECLARATION** — renamed | **SPLIT / RETAINED** | a type reachable *without* being named is the known gap; recorded, and inertness is pinned behaviorally instead |
| `the_lower_expr_call_population_is_dispositioned_by_owner_not_by_site` | 59 tokenized calls exist | **DECLARATION** | **RETAINED**, relabelled frozen evidence | line-split + path-form + `source_occurrences` conflation → all held (tokenizer) |
| `correspondence_adds_no_emitted_unit_to_the_production_census` | the emitted-unit census is unchanged | **DECLARATION** | RETAINED, unmodified | not attempted — pre-existing pin from an earlier WP, outside this respin's diff |
| `every_source_term_carrier_holds_an_occurrence_and_never_a_bare_expression` | carrier fields hold occurrences | **DECLARATION** | RETAINED, unmodified | not attempted — as above |
| `exactly_one_plan_origin_to_expression_lookup_exists` | one `origin → expression` route | **DECLARATION** | RETAINED, unmodified | already corrected once (it constrains `source_occurrence` only); no further evasion run |
| `no_collection_is_keyed_by_a_scheduling_entry` | no collection keys on an entry | **DECLARATION** | RETAINED, unmodified | not attempted — pre-existing |
| `retained_closures_carry_a_static_origin_and_no_body_term` | closures carry origins | **DECLARATION** | RETAINED, unmodified | not attempted — pre-existing |
| `the_backend_production_surface_inventory_is_closed` | the production surface inventory | **DECLARATION** | RETAINED, unmodified | not attempted — pre-existing |
| `the_entry_carrying_types_are_module_private` | entry types are module-private | **DECLARATION** | RETAINED, unmodified | not attempted — pre-existing |
| `the_semantic_seed_api_accepts_only_occurrence_origins` | the seed API's accepted origins | **DECLARATION** | RETAINED, unmodified | not attempted — pre-existing; **this is the pin I predicted did not exist** |

⚠ **The "not attempted" cells are honest, not oversights.** Those eight pins are
pre-existing declaration pins from earlier work packages; this respin is a
**subtraction** and does not touch them, so running evasions against them would
widen the diff beyond the WP without a criterion asking for it. **They are
classified, which is what `AC-12` requires; they are not re-verified.** If the
ring wants them evasion-tested, that is a separate pass and should be said so
rather than assumed from this table.

⛔ **No pin in this table is REACHABILITY and RETAINED.** That combination is
exactly what the ruling forbids, and its absence is the table's main claim.
