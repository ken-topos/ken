# RT-DECL-CLOSURE-PORT D2 — planner-owned callable declaration units

The owner/unit facts this deliverable creates, recorded against the exact tree
that creates them.

- **Base:** `38b05ac9167c0967aeb9a8bf7e3c5dc705247e9c` (D1 accepted) on
  `wp/RT-DECL-CLOSURE-PORT-typed-units`.
- **Subject:** a `RuntimeDeclarationKind::Transparent` declaration whose own
  body is `RuntimeExpr::Closure` or `RuntimeExpr::LexicalClosure`.

## 1. What existed before, measured on this tree

A transparent closure-seed declaration already produced **two** function units,
and **neither was owned by the declaration**:

| unit | seed node | arm before D2 | declared arity |
|---|---|---|---|
| the declaration's entry | the closure occurrence | `SchedulingEntry { ingress: Empty }` | `(0, 0)` |
| the closure's body | the `StaticBody` edge target | `ClosureBody { defining_origin: the anonymous closure occurrence }` | the closure's own `(params, captures)` |

The declaration's own unit therefore declared **no parameters and no captures**,
while the unit that did declare them was attributed to the anonymous closure
occurrence. That is why `DeclarationRef` could not resolve to a callable unit and
instead produced a compiler-only `Lowered::DeclarationClosure` capsule, whose
body was then recursively lowered into the generated root
(`lower_source_declaration_call`).

## 2. What D2 creates

One new closed arm on `AbiUnitDefinition`:

```rust
CallableDeclaration {
    declaration_origin: StaticOriginId,
    provenance: AbiCaptureProvenance,
}
```

- **Owner:** the **declaration's** planned occurrence
  (`plan.declaration_occurrences`), never the anonymous closure occurrence a
  `ClosureBody` records.
- **Declared arity:** the defining closure occurrence's own `ParamName` atoms
  and recorded `capture_slots` — the same computation `ClosureBody` uses, so the
  two cannot disagree about what a closure declares.
- **Capture carrier:** by provenance, exactly as `ClosureBody` — `Seed` takes
  `GroundValueCarrier`, `Lexical` takes `ValueWord`.

### The derivation is graph-derived, not a list

`unit_definitions` already classified on `(is_entry, body_edge)`. D2 splits the
`StaticBody`-target class by **who owns the defining occurrence**: a body whose
defining occurrence is in `declaration_occurrences` is that declaration's
callable unit; every other body stays an anonymous `ClosureBody`.

`declaration_occurrences` is populated only by the single loop that plans
transparent declarations, so a unit cannot be classified declaration-owned
unless the planner actually planned that declaration. This is not a
source-origin whitelist and not a syntactic test on the body — both of which
the frame's §4 prohibitions forbid.

**D2 reclassified without correcting the population, and that was wrong.**
D2 as landed left a closure-seed transparent declaration contributing **two**
functions — its `StaticBody` target, newly classified `CallableDeclaration`, and
its own now-unreachable zero-input `SchedulingEntry` at the closure occurrence.
The second has no lawful runtime meaning: it cannot call the callable unit
without the missing parameters and captures, cannot return the closure, and
cannot become a no-op without changing program meaning. D5 measured it as a
refusal at `boundary_transfer_admissibility` the first time the functionized
lane was actually entered.

The population is therefore **corrected by `D2a`**, on this same lineage: a
transparent `Closure`/`LexicalClosure` declaration contributes exactly one
declaration-owned `CallableDeclaration`, at its exact forward `StaticBody`
target, and **no separate `SchedulingEntry` function**. The relational assertion
is now

```text
functions.len() == entries.len() + StaticBody edge count - declaration-owned pairs
```

and the retained `StaticBody` relation of such a pair is a
**definition/signature relation**, not an emitted cross-unit call. See the
frame's `D2a` section (Architect ruling `evt_3twrm71vck49d`).

## 3. Invariants the new arm inherits, and where that was nearly lost

`ClosureBody` and `CallableDeclaration` sit at the **same graph position** and
differ only in ownership, so every rule keyed on "declares captures off a
closure occurrence" must reach both. These are routed through one predicate,
`AbiUnitDefinition::closure_shaped_captures`, rather than each site asking for
`ClosureBody` separately:

| site | rule |
|---|---|
| `reject_imported_capture_edges` | `C4` — an imported capture edge receives no callable descriptor |
| `validate_boundary_layouts` | `D5` — caller/callee agreement on defining occurrence, provenance, captures, parameters |
| `declared_arity`, `push_slots`, `validate_slot_run` | arity and slot layout |
| continuation capture sourcing | `LexicalCapture` / `SeedCapture` source resolution |

**Measured, not assumed:** `validate_boundary_layouts` refuses a `StaticBody`
callee that is not closure-shaped. Before it was routed through the shared
predicate, the reclassification turned **three** existing tests red with
`static body edge callee is not a closure-body unit` — including a real
end-to-end object link. That is the direct evidence the new arm reaches real
programs and that the D5 layout agreement genuinely still applies to it.

`reject_imported_capture_edges` is the one to note in review: matching only
`ClosureBody` there would have left `C4` **passing on a quietly smaller
population**, with no test red to say so.

## 4. What D2 does NOT do

Held for the deliverables that own them:

- **no call edge** to the new unit — `D4` (`DeclarationRef` calls);
- **no typed capture/parameter/result/trap transport** across the boundary — `D3`;
- **the selector residual is untouched** — `TransparentDeclarationClosure` still
  fires, `select_body_emission_authority` is unchanged, and
  `lowering/core.rs` is **byte-identical to the base**. Retiring it is `D6`;
- **no owner/phase validation** beyond the inherited rules above — `D5`;
- no other residual class absorbed, no whole-program selector change, no
  `PX8` size reduction, no preserved-prototype code used as candidate input.

Because the residual still fires, a program carrying a transparent closure-seed
declaration still selects `RecursiveDescent`. The new units are **planned** on
that route; they are not yet called. That is the intended D2 state, and it is
why `AC-1` cannot move here.

## 5. Validation

Targeted, on this crate only:

- `scripts/ken-cargo test -p ken-runtime` — **621 passed, 0 failed, 1 ignored**,
  plus `26` and `14`. The base measured `619 + 26 + 14`; the delta is exactly
  the two tests below.
- `D1`'s full-residual enumerator and its compound control
  (`d1_the_enumerator_reports_every_variant_not_the_first`) re-run green and
  are unmodified.

`AbiUnitDefinition` is `pub(in crate::cranelift_backend)`, so the reclassified
surface cannot be observed outside this crate — the targeted scope is grounded
on **item visibility**, not on which files were searched.

### The two controls, and why one is not enough

| control | catches |
|---|---|
| `d2_a_transparent_closure_seed_declaration_owns_a_callable_unit` | the declaration's body is not owned by the declaration, or carries the wrong arity |
| `d2_the_owner_split_is_causal_in_both_directions` | the split is not derived from ownership at all |

The second installs two mutations and measures that the derivation **changes**:

- `D2_IGNORE_DECLARATION_OWNERSHIP` restores the pre-port classification —
  observed `(callable, bodies) = (0, 2)`;
- `D2_CLAIM_ALL_BODIES_DECLARATION_OWNED` swallows the anonymous closure —
  observed `(2, 0)`.

Each mutation reds an assertion the other leaves green, and **both compile**.
The fixture deliberately holds a declaration **and** an anonymous closure with
**different arities** — `(1, 2)` against `(0, 1)` — because a fixture with only
the declaration cannot distinguish "classified by owner" from "classified
`CallableDeclaration` unconditionally", and equal arities would let an
assertion pass while reading the wrong unit's header.

## 6. A frame reading worth recording

The frame's row **#24 `StaticCallableElimination`** is *not* D2's mechanism, and
reading it as such would have built the wrong thing. #24 eliminates a callable
passed as a transparent-declaration **argument**, keyed on parameter ordinals;
D2's subject is a declaration whose **own body** is a closure seed. §4 states the
relation directly: #24 "is **activated by** `RT-DECL`'s unit port" — it depends
on D2, it is not D2, and it remains a `D7` matrix row.

Separately, the "four frozen surfaces, blob-identical" obligation carried in
this ring's recent history is **`RT-CONTSPEC-ACTIVATE`'s `AC-4`**, not a
constraint of this frame. This frame never freezes `abi.rs`; #24 in fact
*requires* a new explicit `AbiUnitDefinition` arm. `semantic_ir.rs`,
`boundary_value.rs` and `boundary_value_clif.rs` are nonetheless untouched here.
