# RT-DECL-CLOSURE-PORT D3 — typed transport across the callable-unit boundary

Owner, phase and transport evidence for the declaration-owned callable unit
`D2` created.

- **Base:** `3e19087891e333638e74b84fcda09a5459d54f15` (D2 accepted) on
  `wp/RT-DECL-CLOSURE-PORT-typed-units`.
- **Boundary under test:** the `StaticBody` edge into an
  `AbiUnitDefinition::CallableDeclaration` unit.

## 1. The measurement, taken before any edit

Probed on the D2 fixture (one transparent closure-seed declaration and one
anonymous closure), reading the planned units directly:

```text
unit[1] SchedulingEntry { ingress: Empty }                      (params 0, captures 0)
unit[3] CallableDeclaration { declaration_origin: 11, Lexical } (params 1, captures 2)
        Parameter ValueWord   ord 0   OwnedByFrame          / ActivationFrame
        Capture   ValueWord   ord 0   OwnedByFrame          / ActivationFrame
        Capture   ValueWord   ord 1   OwnedByFrame          / ActivationFrame
        Result    ResultWord  ord 0   TransferredToCaller   / ActivationFrame
        Control   ControlWord ord 0   OwnedByFrame          / ActivationFrame
        Trap      TrapWord    ord 0   OwnedByFrame          / ActivationFrame
        Store     StoreHandle ord 0   BorrowedForActivation / PersistentStore
call edge  caller unit[1] -> callee unit[3]  kind StaticBody
```

**All four transport classes named by `D3` — capture, parameter, result and
trap — are present and typed at the boundary, and no transport fact is
absent.** They are there by construction from `D2`: routing the new arm through
`declared_arity`, `push_slots` and the fixed `CONVENTION_SLOTS` gives the
callable unit the complete ordered run, and `validate_boundary_layouts` already
compares that run against the caller-derived signature slot by slot.

⚠ **Stated plainly rather than dressed up:** `D3` did not need new transport
machinery, and this document does not claim any. Its content is the evidence
that the transport is real across **both** producers, and the one genuine hole
it closed, below.

## 2. The genuine hole, and it was silent

`reject_imported_capture_edges` (`C4`) is the one site the leader flagged, and
the flag was right. `D2` routed it through `closure_shaped_captures`; `D3`
proves that routing was load-bearing rather than tidy.

**Measured causally.** A transparent declaration whose closure body captures an
`ImportedDeclarationRef`:

| `C4` recognises | outcome for the same program |
|---|---|
| `ClosureBody` **and** `CallableDeclaration` (landed) | **refused** before the unit receives a callable descriptor |
| `ClosureBody` alone (the pre-`D2` shape, restored by mutation) | **accepted** |

⇒ Had `C4` kept matching `ClosureBody` alone, an imported value would have
crossed into a declaration-owned frame **with every test green**. Nothing would
have gone red, because an exclusion that stops seeing a population keeps
returning the same answer it returned before: "no violation".

This is the opposite failure mode from the one `D2` hit at
`validate_boundary_layouts`, which *rejected* the new arm and turned three tests
red immediately. **Same reclassification, two site classes, opposite symptoms —
and only the loud one is discoverable by running the suite.**

## 3. Both `StaticBody` producers, not one

`Closure` and `LexicalClosure` differ in exactly the axis under test, so a
fixture with one leaves the other's transport unmeasured:

| producer | provenance | capture carrier |
|---|---|---|
| `LexicalClosure` | `Lexical` | `ValueWord` |
| `Closure` | `Seed` | `GroundValueCarrier` |

Both are asserted. A declaration whose body is a `Closure` owns a
`Seed`-provenance callable unit whose captures cross as `GroundValueCarrier` —
the carrier is a function of provenance, and the seed case is not a
near-duplicate of the lexical one.

## 4. Controls

| control | catches |
|---|---|
| `d3_the_callable_declaration_boundary_carries_typed_transport` | a missing or mistyped transport class at the boundary |
| `d3_a_seed_provenance_declaration_transports_with_its_own_carrier` | transport proved for the lexical producer only |
| `d3_an_imported_capture_on_a_declaration_owned_unit_is_refused` | `C4` silently not enforced for declaration-owned units |

The transport run is asserted as an **exact ordered slot run**, not as counts —
a count agrees with a run holding the right number of wrong slots — and
ownership plus storage owner are asserted per slot, since a capture that
transferred to the caller or lived in the persistent store would be a different
transport with identical slot kinds.

The `C4` control carries its own **positive control on the mutation**: it
asserts the program is *accepted* under the narrowed predicate. Without that,
the refusal could be some other rejection firing anyway, and the test would
prove nothing about the population shrink it exists to detect.

## 5. What D3 does NOT do

- **No call edge.** `D4` (`DeclarationRef` calls) remains held. The
  `DeclarationCall` edge still targets the declaration's zero-arity
  `SchedulingEntry`; the callable unit is reached only by its `StaticBody`
  edge, from the declaration's own entry.
- **Residual and selector untouched** — `lowering/core.rs` is byte-identical to
  the base for the second deliverable running, so
  `TransparentDeclarationClosure` still fires and these programs still select
  `RecursiveDescent`. Retiring it is `D6`.
- No other residual absorbed, no `PX8` size reduction, no preserved-prototype
  code as candidate input.

## 6. Validation

`scripts/ken-cargo test -p ken-runtime` — **624 passed, 0 failed, 1 ignored**,
plus `26` and `14`. The `D2` base measured `621 + 26 + 14`; the delta is exactly
the three tests in §4. `D1`'s non-short-circuit enumerator and its compound
control, and `D2`'s owner-split controls, re-run green and are unmodified.
