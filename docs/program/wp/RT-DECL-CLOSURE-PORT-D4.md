# RT-DECL-CLOSURE-PORT D4 — the complete call to the declaration-owned unit

Routing evidence for `DeclarationRef` calls after the `D2`/`D3` unit and its
typed boundary.

- **Base:** `c98f1ab202171a939802689293152d9ceddb5e34` (D3 accepted) on
  `wp/RT-DECL-CLOSURE-PORT-typed-units`.
- **Contract:** Architect ruling `evt_ks8gdpgbahf` — D4 installs the *complete*
  call; D6 is activation only and may not add inputs, capture transport, target
  projection, or validation.

## 1. What moved

`connect_declaration_calls` mapped every `DeclarationRef` occurrence to
`declaration_entries[symbol]` — the declaration's scheduling entry. For a
closure-seed declaration that entry is a zero-input thunk, so the call named a
unit declaring none of the declaration's parameters or captures.

```text
before:  DeclarationRef ──DeclarationCall──> scheduling entry (0 params, 0 captures)
after:   DeclarationRef ──DeclarationCall──> CallableDeclaration unit, for a
                                             Closure/LexicalClosure seed only
```

The target is derived **forward**, from the one `StaticBody` edge leaving that
declaration's own entry. There is no reverse body search, no second recorded
table, and no blanket retarget.

## 2. The partition, and why it is a partition

A non-closure transparent declaration keeps its scheduling entry, because that
entry *is* its unit and it is legitimately a zero-input thunk. A blanket
retarget would break every declaration call the corpus already makes — the existing
`computational_match_declaration_ref_emits_and_runs_the_declaration_owned_unit`
compiles and runs exactly that shape under `FunctionizedUnits`.

Two derivations of the same fact are cross-checked, so a plan where they
disagree fails in planning rather than picking one:

| the declaration's planned occurrence | forward `StaticBody` edge | outcome |
|---|---|---|
| `Closure` / `LexicalClosure` seed | exactly one | `CallableDeclaration`, target is the body |
| anything else | none | `SchedulingEntry`, target is the entry |
| seed | none | refused — no callable unit to retarget to |
| non-seed | present | refused — a thunk cannot own a body |
| seed | two or more | refused — ambiguous callable unit |

The class is **recorded**, not re-inferred: `plan.declaration_call_targets`
keyed by the reference occurrence. Both classes are reached by the same edge
kind and both resolve to a `FuncId`, so an empty input slice type-checks against
either; erasing the distinction is what would let a callable target take the
`&[]` call.

## 3. The two edge laws this widened

Both were correct for the pre-`D4` graph and both had to move exactly one step.

**A declaration call's target is a function-unit head.** The ruled head set is
`plan.entries` ∪ every `EdgeKind::StaticBody` target. The old law admitted only
the first half, because before `D4` no declaration call could reach the second.
It now admits the full head set and nothing wider — the target must still be its
unit's *seed*, and that check now runs on both classes rather than on neither.

⚠ **Correction to this document's first version, and to the comment it
described.** The semantic-plane predicate is "is a static-body unit head" — the
whole such population, anonymous `ClosureBody` included. It is **not** the
callable-declaration discriminator, and the original wording presented it as
one. The exact class is decided by `AbiPlane::validate_declaration_call_targets`
reading the `AbiUnitDefinition`; the plan is fail-closed across the two layers
together, never by the semantic predicate alone. The predicate is now named
`static_body_head` for what it measures.

**A declaration call may target the caller's own unit — for one class only.** A
closure-seed declaration that refers to itself does so from inside its own body,
which after the retarget *is* the unit being called. That edge is direct
recursion. The distinctness ban is kept intact for the scheduling-entry class,
where it still means what it always meant. The self-loop admitted here is a
*structural* permission; that the target is the exact declaration-owned
`CallableDeclaration` of the referenced symbol is the ABI plane's to certify.

⚠ This second change was **not anticipated in the D4 scope** and was found by
the suite: `recursive_declaration_shape_change_hits_typed_boundary` went red
with `declaration call edge does not cross a function unit boundary`. It is a
consequence the ruling's unconditional retarget implies, not a scope expansion,
and it is flagged rather than absorbed.

## 4. The lowering side

A naked `DeclarationRef` to either seed form now yields a compiler-only callable
binding and **does not call the unit**. Before `D4` only the `Closure` arm did;
a `LexicalClosure`-bodied declaration fell through to the `FunctionizedUnits`
branch and was called with `&[]` against a unit declaring its parameters and
captures. That call was unreachable in production but wrong-arity by
construction, and "unreachable today" is not the property this needed.

`Lowered::DeclarationClosure` gained `reference` — the planner-issued
`DeclarationRef` occurrence, which is the key the resolved call record is looked
up by. `body` remains the sole body authority, so `AC-1`'s inventory property is
unchanged; the added field is argued in the oracle rather than absorbed.

All three `Call` consumers — ordinary lowering, the computational-producer
route, and the source machine — route through **one** function, because the
input order is a property of the ABI and not of any call site:

```text
inputs = actual arguments in PARAMETER order ++ retained captures in D3 order
```

A swapped slice still type-checks, every input being a word, so a second
ordering authority would not be visible. `call_declared_declaration_unit` now
takes the real slice and delegates to the descriptor-driven
`call_declared_unit_target`, which remains the sole authority for the slot run
and rejects a mismatched slice in both directions. No identity word, runtime
lookup, `specialized_at`, copied body, or late capture discovery is introduced.

The `&[]` call survives for a genuinely zero-input non-closure declaration, and
a callable target cannot reach it: both seed forms return the binding before
that site, and the planner's independently derived class is checked there. Two
derivations must disagree for the empty call to see a callable unit, and that
fails closed.

**A control the diversion would otherwise have lost.** Each new branch returns
before `lower_recursive_declaration_call`, which consumes the pending
checked-recursion marker on entry. A branch that skipped it would leave the
marker set for whatever call came next, which the consumer would then report as
a marker transplanted to another callee — a silent mis-attribution, not a
missing feature. The shared function consumes it, and **refuses** a marker that
is present: a checked same-SCC invocation is an obligation a direct call to a
declaration-owned unit has no mechanism to honour, and discharging it by
ignoring it would be a silent accept.

## 5. Controls

| control | what it would catch |
|---|---|
| `d4_the_declaration_call_partition_follows_the_seed_class` | the partition, as an equality over the whole declaration-call population — a third class, a lost call or a duplicated one all red |
| `NeverRetarget` mutation | the retarget absent: both calls fall back to zero-input entries, the exact wrong-arity target `D4` removes |
| `AnyStaticBody` mutation | the ruled-out reverse body search: the call lands on the fixture's anonymous closure body and is refused by target class |
| `validate_declaration_call_targets` | a declaration call resolving to a `ClosureBody` or continuation unit — proven non-vacuous by the mutation above |
| `d4_a_lexical_closure_declaration_retains_a_binding_and_still_runs` | the second seed form's new binding changing what these programs compute, for a parameter and for a capture |

Two mutations and not one: a retarget can be wrong by not moving **and** by
moving to the wrong place. The fixture carries a third, anonymous closure
precisely so those two are distinguishable — with one closure in the program,
"the edge leaving this entry" and "some edge" give the same answer.

The reachability probe on the new `LexicalClosure` binding arm was run and
removed: the arm executes on that fixture, so its green run is about the code
`D4` added.

## 6. What D4 does NOT establish

⛔ **The `FunctionizedUnits` declaration call is installed but not executed.**
`TransparentDeclarationClosure` still retains the selector for every
closure-seed program, so those programs choose `RecursiveDescent` and never
reach the new branch. Altering the selector is banned in this node, so no
fixture in `D4` can drive that route.

⇒ The measured content here is planner-side routing plus the retained bindings
and the shared call construction. **A green suite is not evidence that the new
call emits.** Behavioural validation is `D5`'s and activation is `D6`'s, which
is the ordering the ruling sets.

## 7. Result

`-p ken-runtime`: **627 + 26 + 14**, zero failures, one ignored. Base at
`c98f1ab2` was 624 + 26 + 14; the three added tests are this node's.
