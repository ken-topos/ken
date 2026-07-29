# RT-DECL-CLOSURE-PORT — port transparent declaration closures to callable units

**A transparent declaration whose body is a closure seed is a retained residual,
and *any* retained residual routes the **whole object** to the monolithic
`RecursiveDescent` root. That root now exceeds Cranelift's per-function ceiling.
This node makes those declarations separately owned callable units so the
residual can be retired.**

**Owner:** Team Runtime (`runtime-leader` + `runtime-implementer` +
`runtime-qa`). **Branch:** `wp/RT-DECL-CLOSURE-PORT`. **Size:** L.
**Risk:** high — this is the last B2F migration seam, and its acceptance is a
*behavioural* compile, not a code-shape assertion.

**Status:** Steward frame, shovel-ready.
⛔ **SERIALIZED behind [[NATIVE-HANDLE-CARRIER]]** — both own
`lowering/core.rs`. Do not start until it merges.

⭐ **On the Linux ABI I critical path.** Sole blocker of [[PX8-ERRID-ALLOC]] →
[[PX8-ERRID-SCOPE]] → [[PX8]]; `PX8` gates 15 of that program's 19 nodes.

---

## 1. Fixed inputs

| path | blob at `origin/main = eefca112` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` |
| `crates/ken-cli/tests/rt_parity_native.rs` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` |

**Grounding:** Architect ruling `evt_3t7t27e3rv8cx`, measured in a detached
scratch worktree with diagnostic-only labels against exact `ad7298fb`
(tree `77ece013`). ⭐ No candidate or production change survived that worktree —
the diagnosis cost nothing and left nothing behind.

## 2. The mechanism, at exact anchors

**The selector** — `core.rs:181-197`, `select_body_emission_authority`:

```rust
if recursive_descent_residual(expr)
    .or_else(|| declarations.values().find_map(declaration_recursive_descent_residual))
    .is_some()
{ BodyEmissionAuthority::RecursiveDescent } else { BodyEmissionAuthority::FunctionizedUnits }
```

⇒ **Whole-program, all-or-nothing.** One residual anywhere and the entire object
takes the monolithic root.

**The residual this node retires** — `core.rs:155-172`,
`declaration_recursive_descent_residual`: a `RuntimeDeclarationKind::Transparent`
whose `body` is `RuntimeExpr::Closure { .. }` or `LexicalClosure { .. }` yields
`RecursiveDescentResidual::TransparentDeclarationClosure` (`core.rs:56`, minted
at `:163`).

**What the Architect measured on the failing fixture:**

```text
authority=RecursiveDescent
residual=TransparentDeclarationClosure
declaration=...::buffer_nat_to_int residual=TransparentDeclarationClosure
declaration=...::main             residual=TransparentDeclarationClosure

PX8_ERRID_DIAGNOSTIC RecursiveDescent root:
Compilation error: Code for function is too large
```

⇒ The oversized function is **the `RecursiveDescent` root itself** — not a
functionized unit, not the root adapter, not a fixed helper graph.
**`FunctionizedUnits` declares and defines *zero* semantic units on this route.**

## 3. ⭐⭐ THE TRAP — retiring this residual may not change the authority

⛔ **`TransparentDeclarationClosure` is ONE OF FIVE** residual variants
(`core.rs:41-57`): `ProducerMatchCall`, `MatchScrutineeRecursor`,
`LexicalCallArgumentRecursor`, `SeedClosureCall`, `TransparentDeclarationClosure`.
The selector takes the **first** residual it finds, and the expression walk
(`recursive_descent_residual(expr)`) is consulted **before** the declaration walk.

⇒ ⭐ **Retiring this one residual does not entail that the fixture reaches
`FunctionizedUnits`.** Another variant may be present and simply unreported,
because the selector short-circuits at the first hit.

**This is why `AC-1` is a compile, not a code-shape assertion.** A deliverable
that removes the residual and reports success while the fixture still fails is
the exact failure mode this section exists to prevent.

⚠ **Before building anything, measure the full residual set** on the fixture —
enumerate *every* variant that fires, not the first. If others fire, ⛔ **stop
and report**: the node's scope is then wrong and it is mine to re-cut, not yours
to widen.

## 4. Deliverables

- **`D1`** — **Full-residual diagnostic first.** A temporary or permanent
  enumeration reporting every residual variant present on a given program, not
  the short-circuited first. Run it on the fixture and record the complete set.
  ⛔ Do not proceed to `D2` until `D1`'s result is posted.
- **`D2`** — **Planner-owned callable declaration units.** Transparent
  closure-seed declarations become separately owned callable units rather than
  bodies recursively lowered into the generated root.
- **`D3`** — **Typed capture / parameter / result / trap transport** across that
  unit boundary.
- **`D4`** — **`DeclarationRef` calls** to those units
  (`core.rs:148`, `:7238` are the existing reference sites).
- **`D5`** — **Complete owner/phase validation**, in place **before**
  `TransparentDeclarationClosure` is removed from the retained residual.
- **`D6`** — Remove the residual variant, and only then re-run `AC-1`.

## 5. Acceptance criteria

- **`AC-1` (the only one that decides the node).** `scripts/ken-cargo test -p
  ken-cli --test rt_parity_native
  fs_write_at_malformed_offset_narrows_to_invalid_offset` **compiles and passes**
  on a tree carrying `ad7298fb`'s semantic delta. ⛔ Not "the residual is gone" —
  **the object builds.**
- **`AC-2`.** `D1`'s complete residual enumeration is recorded in the tree, with
  the fixture's full set named. If the set is larger than
  `{TransparentDeclarationClosure}`, that is a reportable finding, not a silent
  scope widening.
- **`AC-3`.** `D5`'s owner/phase validation is present and fails closed **before**
  `D6` lands. A commit ordering that removes the residual first fails this AC.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved: a new `RuntimeExpr` form must still be unable to compile until the
  classifier assigns it. ⛔ Do not replace the exhaustive match with a wildcard.

## 6. ⛔ Banned scope

- ⛔ **Deleting the selector residual** without the port. Named and banned by the
  ruling as an unproved shortcut.
- ⛔ **Selectively inlining fewer declarations.** Same — banned by name.
- ⛔ **A second [[PX8-ERRID-ALLOC]] size reduction.** The feature delta is
  **exonerated**; shrinking its identity mapping trades semantics for bytes.
- ⛔ **Retiring the other four residual variants.** If `D1` shows they fire,
  report it — do not absorb them. Each is its own migration seam.
- ⛔ **Reopening [[RT-NATIVE-FNSPLIT]] or [[RT-SCALE-B]].** Both closed on gates
  that were met. `RT-SCALE-B` explicitly excluded the `RecursiveDescent` root,
  which is why its verdict is untouched by this.

## 7. Hard stop

Report and stop if `D1` shows residuals beyond `TransparentDeclarationClosure` on
the fixture, or if the port lands and `AC-1` still fails. ⛔ Do not attempt a
size reduction in either case. This is hard-stop territory and **#21 fires the
§5a research pull** — the count of record is **20**, carried on
[[NATIVE-HANDLE-CARRIER]].

## 8. What landing this closes

[[PX8-ERRID-ALLOC]] is released the moment this merges — its candidate
`ad7298fb` is rebased and preserved, and Foundation owes no rebuild, only a
re-run. That in turn releases [[PX8-ERRID-SCOPE]] and clears the last of `PX8`'s
three blockers on this path.

⭐ **The selector's own doc comment calls it "the one *temporary* B2F migration
selector" (`core.rs:174`).** This node is that migration finishing, not a new
mechanism.
