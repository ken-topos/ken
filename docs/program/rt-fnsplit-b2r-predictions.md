# `RT-FNSPLIT-B2R` — predictions, recorded before measurement

**Base:** `wp/RT-FNSPLIT-B2R-representation-abi` at `c5edea8b` (`origin/main`).
**Written:** 2026-07-25, **before** building `abi.rs` and before running any
`ken-cargo` measurement.

`AC-10` requires the predicted values to exist before the measured ones. This
file is committed on its own so the commit graph, not my word, orders them. A
number re-fit after measurement measures nothing; a miss recorded here stays
legible as a miss.

## Grounding already done (measured before predicting — these are inputs)

These are facts I read off the tree at `c5edea8b`, not predictions:

- `crates/ken-runtime/` is **byte-identical** between `e470ab65` (where the
  frame's anchors were re-derived) and `c5edea8b`, proved with
  `git diff --quiet`. Every frame anchor carries unchanged.
- A `StaticBody` edge runs `closure_occurrence.entry -> body.entry`
  (`static_transition.rs:858`, `:884`). So a unit seeded by a `StaticBody`
  target has **exactly one** defining closure occurrence: that edge's `from`.
- `SemanticSourceSeed.capture_slots` is `captures.len()` for
  `RuntimeExpr::Closure` and `RuntimeExpr::LexicalClosure`, and `0` for every
  other shape (`semantic_ir.rs:240-263`).
- Provenance is already carried as data: `RuntimeExprShape::Closure` (seed) vs
  `RuntimeExprShape::LexicalClosure` (lexical), on the record's `source`.
- `RuntimeGroundValue` is closed at six variants: `Bool`, `Int`, `Bytes`,
  `String`, `Constructor`, `Record` (`ir.rs:514`).
- `ImportedDeclarationRef` **is** planned as an ordinary `Evaluate` occurrence
  (`static_transition.rs:683`), so an exclusion check over it is reachable
  rather than vacuous.

## P1 — function-unit and descriptor population

`units == entries.len() + count(StaticBody edges)` is already validated by
`B2O`, so the ABI prediction is that descriptors are **exactly** total over it.

| fixture | predicted units | predicted ABI descriptors |
|---|---|---|
| `leaf` | 1 | 1 |
| `let-if` | 1 | 1 |
| `match` | 1 | 1 |
| `lexical-closure-call` | 2 | 2 |
| `computational` | 1 | 1 |
| `computational-nested` | 1 | 1 |
| `computational-under-let` | 1 | 1 |
| **total over the seven** | **8** | **8** |

On `b2o_two_closure_fixture` + the transparent declaration (2 entries, 2
`StaticBody` edges): **4 units, 4 descriptors**, composed as

- 2 units defined by a **scheduling entry** — 0 declared parameters, 0 captures;
- 2 units defined by a **`LexicalClosure`** occurrence — 1 parameter (`x`), 0
  captures.

## P2 — `AC-2`, the irrelevant caller binding

Wrapping the fixture in one more `Let { value: unit(), body: <fixture> }` adds
planned nodes but adds **no** scheduling entry and **no** `StaticBody` edge.

- **Predicted: unit count stays 4.**
- **Predicted: the two closure descriptors' SHAPE fields are identical** — slot
  count, slot kinds, carriers, ownership modes, provenance.
- **Predicted: the IDENTITY fields shift** — `planned_node` and `origin` are
  positional over the node table, and the extra `Let` renumbers it.

⇒ The comparison must be on **shape**, not on the whole descriptor. I am
recording that before measuring so a shape-only comparison cannot later look
like it was narrowed to make a red test green.

## P3 — `AC-3`/`AC-4`, seed-value independence

**Predicted outcome: ⭐ CANNOT VARY BY CONSTRUCTION, not "a test stays green".**

`build_abi_plane` will take `&SemanticPlane` plus the plan graph. Neither type
holds a `RuntimeGroundValue` or a `Lowered`. So "the descriptor does not vary
with the particular runtime value" is enforced by the **constructor's
signature** — there is no value in scope to inspect.

- **MEASURED:** the ABI builder's inputs contain no runtime value.
- **CLAIMED:** a seed capture's layout is not chosen by inspecting its value.
- **THE GAP:** this pins the **descriptor**. It does **not** pin that `B2F`'s
  emission path will stay value-independent — that is `B2F`'s obligation, and
  the residual is recorded rather than covered.

Because the compiler is doing the work, `AC-3`'s test is a **positive control
that the mechanism is reachable**, not the enforcement. I predict the seed
carrier is one **fixed closed carrier** covering all six `RuntimeGroundValue`
variants, selected by **provenance alone**.

## P4 — the layout language, stated honestly before I write it

**Predicted: no per-slot static TYPE is derivable from this plane.** The atoms
carry `ParamName` and `CaptureSymbol` — names, not types. So I predict the
layout language must be a **closed carrier language keyed on slot role and
provenance**, not a derived type lattice, and that per-origin variation is
carried by **arity and provenance mix** rather than by per-slot type.

If measurement contradicts this — if a static type is in fact derivable — that
is a miss and I will record it as one rather than restating the design.

## P5 — `AC-5`, imported exclusion

- **Predicted:** an `ImportedDeclarationRef` occurrence gets **no** callable
  descriptor and the ABI validator returns a planner error naming dependency
  linking, **before** emission.
- **Predicted:** the check is **non-vacuous** — the occurrence is reachable in
  a plan, per the grounding above.
- **Predicted:** the positive control (intra-module recursion / bundle)
  validates **green**, so the exclusion is distinguishable from a gap.

## P6 — `AC-6`/`AC-7`, inert and dependency-free

- `FunctionBuilder::new(` in production backend sources: **1**, unchanged.
- `.define_function(` in production backend sources: **1**, unchanged.
- New callable target units, call edges, dispatch edges, callbacks, flags,
  alternate entries: **0**.
- `Cargo.toml` / `Cargo.lock` deltas: **0 bytes**.
- `syn` / `proc-macro2` / `quote` edges added to any `ken-*` crate: **0**.

## P7 — `AC-8`, the inverted control

- **Predicted GREEN** under: a Rust wrapper around the builder, a method
  rename, a visibility change, a `fn` moved between files.
- **Predicted RED** under: a repointed `StaticBody` edge, a corrupted recorded
  owner, a dropped capture slot, an added implicit caller-env tail.

A pin that reddens on any of the first four is a **defect in the pin** and gets
reported as a finding, not repaired into greenness.

## P8 — `AC-9`, the source-text-reading pin population

**Predicted files touched by this WP:**

1. `crates/ken-runtime/src/cranelift_backend/planning/static_transition/abi.rs`
   (new)
2. `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`
   (module declaration + tests)
3. `crates/ken-runtime/src/cranelift_backend/planning/static_transition/semantic_ir.rs`
   (visibility widening only, if needed)
4. `docs/program/rt-fnsplit-b2r-abi-report.md` (new)
5. this file

**Predicted source-text-reading pins in that set: 0.** The `include_str!`-based
pins all live in `lowering/core/tests/control.rs`, which I predict this WP does
**not** touch.

⚠ If the touched set grows to include `control.rs`, this prediction is wrong and
the population is whatever the closed enumeration over the *actual* touched
files returns. The enumeration is closed over the files, never over a list of
pin names I remember.

## P9 — size

- New production lines in `abi.rs`: **600–800**.
- New test lines: **400–600**.
- Production lines changed in existing files: **< 20** (a module declaration and
  any visibility widening).

## What would make me stop before writing code

Recorded now so a later stop is not reconstructed:

- if a `PredeclaredFunction` turned out to have **no** derivable defining
  occurrence (it does — proved above);
- if provenance were **not** recoverable without reading source text (it is —
  `RuntimeExprShape`);
- if `ImportedDeclarationRef` were **unreachable** in a plan, which would make
  `AC-5` vacuous and force an escalation rather than a green test (it is
  reachable).

All three cleared, so construction proceeds.
