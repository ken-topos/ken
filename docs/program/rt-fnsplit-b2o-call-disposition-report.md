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

**The repair is not a better table. It is a closed inventory of the functions
that can reach a retained body.**

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

### ⛔ CORRECTED — the closure claim this section originally made was FALSE

An earlier revision of this section said the population is *"characterised by ONE
route with a pinned consumer count"* and that *"a tenth consumer cannot appear
without reddening the lookup-count pin."* **Both were false on `97db6f0b`**
(Architect, `evt_5984e30gv9f0k`), and the reason is worth keeping:

`exactly_one_plan_origin_to_expression_lookup_exists` pins the identifier
**`source_occurrence`** — its definition plus its single call *inside*
`retained_body_occurrence`. It says **nothing about who calls
`retained_body_occurrence`.** So a new route could be added with that pin, and
the 59-call census, both staying green.

⭐ **And a COUNT cannot close it either.** Add a delegating wrapper and route one
existing call site through it: the `lower_expr` count is unchanged (the site
still calls it once), the `source_occurrence` count is unchanged, and even a
`retained_body_occurrence` **mention count** is unchanged — a mention *moved*, it
did not appear. **The count is invariant under adding a wrapper.**

⇒ The property is not "how many mentions" but **"which functions can reach a
retained body"**, so the mechanism is the **allowed inventory** of those
functions — reddening on a new route *whatever it is named*. A pin keyed on a
name pattern such as `*_body_occurrence` would be spelling-scoped and evaded by
naming the new helper anything else.

**The actual pins, on this SHA:**

| pin | what it closes |
|---|---|
| `the_retained_body_routes_are_a_closed_inventory_of_named_functions` | definitions counted separately from consumers (`retained_body_occurrence` 1 + 8; `machine_body_occurrence` 1 + 3), **plus** the exact permitted set of enclosing functions for each |
| `the_routing_function_enumerator_sees_a_relocated_call` | its positive control — feeds the enumerator the Architect's evasion under a non-matching name, and a non-degenerate pair where the count cannot discriminate but the inventory can |

The routing inventory, asserted exactly: `lower_recursor_residual_call`,
`lower_computational_producer_expr`, `retained_body_occurrence`,
`machine_body_occurrence`, `lower_expr` — and for the machine wrapper,
`source_call_state`, `machine_body_occurrence`.

**Mutation-verified, not argued:** adding a `resolve_tag` wrapper and relocating
one call site **compiles**, **reddens the inventory pin**, and leaves the 59-call
census and the `source_occurrence` pin **green**. That is the discrimination the
closure claim requires, and it did not exist before this pin.

## What this does NOT say

- **It does not partition the 59 calls into two disjoint sets of source sites.**
  It cannot, and that is the finding: for the caller-dependent sites the answer
  is per `(site × reaching path)`. What is closed is the **routing-function
  inventory**, not the call-site list.
- **The inventory closes DRIFT, not reachability.** It guarantees that a new
  function able to reach a retained body cannot be added silently. It does **not**
  prove that the five listed functions reach one on every path through them —
  that is per reaching path, and it is exactly what no static inventory can say.
- **The five provenance classes** (32 `child_occurrence` + 9
  `case_body_occurrence` + 14 caller-dependent + 2 synthesized + 1 direct) remain
  **evidence inputs**, never the authority partition. They are not restated here
  as a disposition, because doing so would reintroduce the site keying.
- **Nothing here is a scaling claim.** No emitted unit changed; `B2O` is inert.

## MEASURED / CLAIMED / THE GAP

- **MEASURED:** 59 tokenized production calls into `lower_expr`; one
  `origin → expression` route; 8 + 3 consumer mentions downstream of it; the
  **exact set of enclosing functions** that mention either retained-body helper;
  a validated ownership partition in which `StaticBody` is the only cross-owner
  edge.
- **CLAIMED:** the boundary-crossing subset of those calls is reached only
  through functions in that inventory, and the inventory **cannot grow
  silently** — mutation-verified, since the wrapper evasion reddens it while the
  two count pins stay green.
- ⚠ **NOT CLAIMED:** that every path through those five functions reaches a
  retained body, or that the *count* of boundary crossings is pinned. A wrapper
  keeps every count invariant, which is why the mechanism is an inventory; and
  the per-path question is the one Finding 3 established no static keying can
  answer.
- **THE GAP:** this says **where** the boundaries are and **which calls** cross
  one. It says nothing about **what may cross** — no signature, no slot layout,
  no calling convention, no lifetime or ownership rule. ⛔ Hard-stop #9's missing
  native value representation is **not** discharged by anything in this node or
  this report; that is `RT-FNSPLIT-B2R`. Reading "the boundary population is
  closed" as evidence that functionization is now buildable is the same inference
  #5 and #8 were defeated on.
