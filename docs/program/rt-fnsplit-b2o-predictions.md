# `RT-FNSPLIT-B2O` — the D5 predictions, recorded BEFORE measuring

**Status: prediction record. Written before any code change and before any test
run, so that the "before" is unforgeable rather than asserted.**

- WP: `RT-FNSPLIT-B2O` (frame `docs/program/wp/RT-FNSPLIT-B2O-body-ownership.md`)
- Branch base: `origin/main = 3baa80f4232f090de75ad0e3f386c329ca9bb7d6`
- Date: 2026-07-25
- Author: `runtime-implementer`

`D5` requires each new value to be **predicted from the design**, with its
reason, before it is measured — *"a count that differs from the prediction is a
finding to route, not a number to update."* This file is that record. Nothing
here was read off a test run; every number is derived from the construction.

## Grounding — the frame's anchors re-derived on `3baa80f4`

The frame's anchor table was measured on `9d515c9d`. `9d515c9d..3baa80f4` is
three commits, all under `docs/program/`; `git diff --quiet 9d515c9d 3baa80f4 --
crates/` exits 0. **Every source anchor in the frame therefore holds verbatim on
my base**, and I re-read each one rather than inheriting it:

| frame anchor | verified on `3baa80f4` |
|---|---|
| `ClosureBody` control node made **first**, exits to the shared terminal | `static_transition.rs:833-835` (`Closure`), `:850-852` (`LexicalClosure`) |
| `StaticBody` edge targets `body.entry` | `:845`, `:871` |
| scheduling entries | `:216` (field), pushed `:1728` (root) and `:1734` (transparent declarations) |
| `entries` uniqueness already checked | `:1158` |
| `entries.first()` caveat | `:1047` |
| whole-graph reachability walk seeded from `entries` | `:1275` (crosses **all** edges, including `StaticBody`) |
| exactly one `Terminal` / one `TrapTerminal`, no outgoing edges | `:1251`, `:1258` |
| id aliasing | `semantic_ir.rs:534-536` |
| `functions.len() == nodes.len()` enforced | `semantic_ir.rs:752` |
| positional `function` checks | `semantic_ir.rs:850`, `:853-856` |
| `assert_eq!(functions.len(), nodes.len())` | `static_transition.rs:2239` |
| `helper_definitions` | `static_transition.rs:1687` |

## The seed derivation, stated as the reason for prediction 1

Seeds are `plan.entries` ∪ `{edge.to | edge.kind == StaticBody}`, per the ruling.
`entries` is **not** derivable from the graph — it is planner state pushed at
`:1728`/`:1734` — so `build_semantic_plane` must receive it. `StaticBody`
targets **are** derivable from `edges`.

**Totality argument** (why the partition covers every non-sentinel node): the
plan already enforces that every node is reachable from `entries` when crossing
*all* edges (`:1275`, `reachable != closed_nodes` is a planner error). Take any
node `v` and any such path from an entry. Decompose it at its **last**
`StaticBody` edge. If there is none, the whole path starts at an entry and
crosses no `StaticBody`. If there is one, the suffix after it begins at a
`StaticBody` **target** — a seed — and crosses no `StaticBody`. Either way `v` is
reached from some seed without crossing `StaticBody`. ⇒ totality follows from an
invariant the planner already enforces, and is not a new assumption.

**Exclusivity does NOT follow** from that argument and is the thing `D3`/`D4`
must actually check.

## The five `D5` predictions

### 1. `functions.len() = entries.len() + count(StaticBody edges)`

**Concrete, for the census fixture `nested_resource_bracket(n)`** (the fixture
`semantic_census` uses, `static_transition.rs:1806`, called with
`&BTreeMap::new()` so there are **no transparent declarations**):

- `entries.len() = 1` — the root only.
- Each depth level contributes **exactly one** `LexicalClosure`
  (`:1835`, inside the `ComputationalMatch` scrutinee's `Construct`), and
  `depth == 0` bottoms out at `unit()` with none. ⇒ `count(StaticBody) = n`.

⇒ **`functions.len() = n + 1`; for `n = 3..7` that is `4, 5, 6, 7, 8`.**

### 2. The node-exact arenas are UNCHANGED

`descriptors.len() == programs.len() == records.len() == capture_layouts.len()
== nodes.len()`, exactly as today. This node moves the `functions` axis **only**;
`programs`/`records` stay one-to-one so `child_origin` (`semantic_ir.rs:664`,
which destructures `[record]`) is untouched and `B2A-C`'s correspondence keeps
working. (Frame: "do not widen this node to the programs axis.")

### 3. Every non-sentinel node has exactly ONE `Function` owner

Totality by the argument above; exclusivity by the `D3` check. The owner field is
a single field rather than a list, so "owned by two units" is unrepresentable in
the *record* — but a **wrongly assigned** owner is representable, which is why
`validate` recomputes the per-seed partition and compares, rather than trusting
the field it just wrote.

### 4. The shared-exit population is exactly `{Terminal, TrapTerminal}`

Exactly **two** descriptors carry a non-`Function` owner: one `Terminal`, one
`TrapTerminal`. Not "at least"; not "the ones with no owner". The plan already
enforces exactly one node of each kind (`:1251`), so a third shared exit is a
planner error at two independent layers.

### 5. Cross-owner edges are exactly the `StaticBody` edges

⇒ `count(cross-owner edges) = count(StaticBody edges) = n` for the fixture.
Every other edge either stays inside one `Function` owner or targets a sentinel.
⛔ `B2O` invents **no** static return edge to the caller; the body's `ClosureBody`
return node stays inside the **callee's** owner and exits through the shared
terminal (`:835`, `:852`). The dynamic return continuation is `B2R`'s.

## ⚠ Prediction on the three re-baselined sites: TWO reds, not three

The frame's READ-FIRST section says the `functions.len() == nodes.len()` equality
is *"enforced in three places"*. I re-read all three. **Two enforce it. The third
does not, and predicting it as a red would be wrong:**

| site | what it really is | predicted effect of `D1` |
|---|---|---|
| `semantic_ir.rs:752` | production planner error | **must be rewritten** — as written it would reject every plan |
| `static_transition.rs:2239` | `assert_eq!` in a test | **RED**, re-baseline to the `D5` relation |
| `static_transition.rs:1687` → `:2023` | a `#[cfg(test)]` census **field**, consumed only by an *affine-scaling* assertion | ⚠ **GREEN — silently** |

**Why the third is a silent pass, and why that is the strongest argument for
AC-6's rename.** `helper_definitions` is a field of `BoundaryB1Census`
(`#[cfg(test)]`, `:284`), produced by `semantic_census` (`#[cfg(test)]`, `:1650`).
Its only consumer (`:2023`) asserts that the **second finite difference across
`n = 3..7` is zero** — i.e. that the metric is affine in `n`. It asserts **no
absolute value**. Prediction 1 gives `n + 1`, whose second difference is `0`. So
the assertion **passes before and after**, while the quantity it names has
changed meaning from "one definition per planned node" to "one definition per
function unit".

⇒ Nothing fails, nothing warns, and the label keeps reporting. This is exactly
the frame's *"a metric keeps reporting whatever it is given"*, and it is why the
rename to a function-unit metric is an acceptance criterion rather than tidiness.

**Consumer inventory for the rename** (`grep` over the whole crate, so the
rename cannot miss a reader): `static_transition.rs:289` (field declaration),
`:1687` (production of the value), `:1981`+`:1991` (the `eprintln!` report), and
`:2023-2024` (the affine assertion). **Four sites, all `#[cfg(test)]`, no
production consumer.**

## MEASURED / CLAIMED / THE GAP

- **MEASURED:** that `functions.len()` equals `entries.len()` plus the
  `StaticBody` edge count, that every non-sentinel node resolves to one
  `Function` owner, and that the only cross-owner edges are the `StaticBody`
  edges.
- **CLAIMED:** that the plan graph carries a *total, exclusive, validated*
  occurrence → function-unit ownership mapping, against which `B2F`'s 59-call
  population can be dispositioned by owner and reaching path.
- **THE GAP:** the mapping says **where** the boundaries are. It says nothing
  about **what may cross** one — no signature, no slot layout, no calling
  convention, no lifetime or ownership rule. That is `RT-FNSPLIT-B2R`, and the
  hard-stop #9 obstruction (a native value representation) is **not** discharged
  by anything in this node. ⛔ A reader who takes "ownership is total and
  validated" as evidence that functionization is now buildable has made exactly
  the #5/#8 inference this chain has been defeated on twice.

## Falsifiers — how to catch me being wrong cheaply

1. If `count(StaticBody edges) != n` on the fixture, prediction 1 is wrong and
   the `LexicalClosure`-per-level reading of `:1835` is wrong.
2. If any non-sentinel node is reached by two seeds, exclusivity fails and the
   `ClosureBody`-return reading (`:835`, `:852`) is wrong — the return node would
   be shared rather than callee-owned.
3. If `descriptors.len()` moves off `nodes.len()`, the axes are not separable
   after all and the frame's "do not widen" boundary is not where it says it is.
