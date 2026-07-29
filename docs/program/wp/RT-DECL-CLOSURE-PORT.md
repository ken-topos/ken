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

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.** This node is
the **keystone** of that seven-node campaign, and the campaign doc carries the
three traps that bind every node in it — including **Trap 3**, which this frame's
`AC-2` now discharges explicitly. This frame predates the campaign doc and does
not repeat its content.

**Status:** Steward frame, shovel-ready.

## ⭐⭐ REORDERED 2026-07-29 — THIS NODE IS NOW **NEXT**, NOT THIRD

**Steward disposition `evt_5mtkdft1nxmwp`, on [[NATIVE-HANDLE-CARRIER]]'s hard
stop.** The order was `RT-JOIN-DISPOSITION → NATIVE-HANDLE-CARRIER → this node`.
`RT-JOIN-DISPOSITION` merged (`main = af056a78`); **`NATIVE-HANDLE-CARRIER` then
hit this node's `AC-1` row** and is preserved at
`85dcee259dc65f9e3c1d625c0ee0ed8342577492` (tree `b7cf9041`) pending this node.
⇒ **`RT-JOIN-DISPOSITION` → this node → `NATIVE-HANDLE-CARRIER` resume.**

⭐ **The premise that put `NATIVE-HANDLE-CARRIER` first was measured false.** It
was *"NHC is 5/6 green and cheap to finish."* NHC's own `AC-1` — full parity, no
partial — is **unreachable on any tree** until the ceiling below falls. It is not
blocked on a formality; it cannot complete.

⚠ **The cost that is real and is not hidden:** this node rewrites `core.rs`, so
`85dcee25` needs a **second** rebase on resume. Its `D1` already proved that
machinery on this exact branch (`git range-diff` 3/3 `=`, no side choice, four-file
provenance settled), so the cost is bounded.

⭐ **Reversing this is cheap:** run `NATIVE-HANDLE-CARRIER` first instead and it
stops again at the same row. That is what makes the reorder safe rather than a
preference.

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
  fs_write_at_malformed_offset_narrows_to_invalid_offset` **compiles and passes**.
  ⛔ Not "the residual is gone" — **the object builds.**

  ⛔⛔ **AMENDED 2026-07-29 — `AC-1` NOW REQUIRES THIS ROW GREEN ON *TWO*
  INDEPENDENT DELTAS, NOT ONE.** As originally written it read *"on a tree
  carrying `ad7298fb`'s semantic delta"* — **Foundation's delta only.**
  1. A tree carrying **`ad7298fb`**'s semantic delta ([[PX8-ERRID-ALLOC]]).
  2. A tree carrying **`85dcee25`**'s semantic delta ([[NATIVE-HANDLE-CARRIER]],
     the `Resource Buffer → BufferHandle` carrier migration).

  ⭐ **`ad7298fb` IS the measurement object for delta 1 — `e65c81b5` is NOT to be
  run, and that is settled, not a shortcut.** `e65c81b5` is the pre-rebase
  Foundation WIP; it sits on a **pre-repair** tree, and applying the `D1`
  enumerator to it conflicts in `core.rs`. ⛔ **Do not hand-resolve that conflict
  to "complete" this AC** — a resolution chosen so a row passes measures the
  resolver, which is the defect that rejected `27f9dca2`.
  ⇒ Equivalence was **measured, not asserted**: `git range-diff
  5404108a..e65c81b5 eef0cb06..ad7298fb` maps the three-commit series in order
  with final `e65c81b5 = ad7298fb` **patch-equivalent** (`D1` report,
  `evt_24dbrgg36w6by`, 2026-07-29). ⭐ Recorded here because an in-thread
  justification is not a durable deliverable, and without it a later reader
  checking `AC-1` against `e65c81b5` literally would block on work that was
  correctly done.

  ⭐ **Why the second one was added, and why omitting it would have made the
  reorder worthless.** Both deltas reach this same ceiling independently, and
  `main` alone passes the row (measured: `evt_5mtkdft1nxmwp`). With one delta in
  the AC, **this node could land fully green and `NATIVE-HANDLE-CARRIER` would
  resume and still be red** — after the queue was reordered to fix exactly that.
  ⇒ The single-delta AC does not measure that the ceiling is gone; it measures
  that *one program* got under it.

  ⚠ **A third program is known to sit near the ceiling**:
  `docs/program/issues/CI-SKIPPED-NATIVE-TESTS.md` records this row as the only
  one of seven opening **two nested resource brackets** where every sibling opens
  one, and as the 250.5s timing outlier. ⛔ Do not add it as a third required
  delta — two independent deltas is the control; ⭐ **but if either delta needs a
  size concession to pass, that is a reportable finding that this node did not
  remove the ceiling, only lowered the program under it.**
- **`AC-2`.** `D1`'s complete residual enumeration is recorded in the tree, with
  the fixture's full set named. If the set is larger than
  `{TransparentDeclarationClosure}`, that is a reportable finding, not a silent
  scope widening.

  ⛔⛔ **"Complete" is asserted here, and it owes TWO POSITIVE CONTROLS.** This
  AC as written passes on whatever set `D1` happens to emit — including a set
  that is missing a variant, which is campaign **Trap 3**
  (`docs/program/16-recursive-descent-retirement.md`).
  1. **The enumerator must be shown NOT to short-circuit.** Run it on a program
     that fires **two or more** variants and show it reports **all** of them.
     ⭐ This is the control that matters most: the enumerator's entire purpose is
     to defeat the selector's `.or_else(...)` short-circuit, and if it silently
     retained that behaviour it would report **exactly one** variant — which is
     precisely the result everyone expects to see, so nothing would look wrong.
  2. **Each variant must be shown reachable by the instrument.** For every one of
     the five variants, name a program the enumerator reports it on. A variant no
     program in the corpus reaches is a **reportable gap in the measurement**, not
     a variant that does not fire.

  ⚠⚠ **This instrument is the most leveraged object in the campaign.** The
  campaign doc directs that it be built **once here and reused** by
  [[RT-SEED-CALL-PORT]], [[RT-PRODUCER-MATCH-PORT]], [[RT-RECURSOR-TRANSPORT]]
  and [[RT-DESCENT-RETIRE]]. ⇒ A gap in it does not stay local — **every
  downstream node inherits it**, and [[RT-DESCENT-RETIRE]]'s "no residual fires
  anywhere" would then be vacuous at exactly the moment it authorizes deleting
  the lane.

  ### ✅ `AC-2` DISCHARGED 2026-07-29 at `93a6903b` (tree `c11bb8b0`)

  `D1` report `evt_24dbrgg36w6by`, accepted by `runtime-leader`
  `evt_3a595dcnam7f8`. **Both positive controls landed, and the first is
  *causal*, not asserted:**

  1. **Does not short-circuit** — the control exercises all five individual
     witnesses **plus a compound all-five population**, `1/1` green. Mutating the
     `report` visitor to short-circuit (`continue true → stops false`) turns it
     **red at the compound assertion**, observing only
     `{ProducerMatchCall, TransparentDeclarationClosure}` instead of five. Exited
     101; restored byte-identically.
  2. **All five variants reachable** — each has a named witness in the control.

  ⭐ **This is the campaign's shared instrument, now controlled.** Downstream
  nodes ([[RT-SEED-CALL-PORT]], [[RT-PRODUCER-MATCH-PORT]],
  [[RT-RECURSOR-TRANSPORT]], [[RT-DESCENT-RETIRE]]) inherit it **and this
  evidence** — ⚠ but re-prove it cheaply at each point of use, since `D2`–`D6`
  rewrite `core.rs` underneath it.

  **Scope fork result — the hard stop did NOT fire.** Both governed deltas select
  `authority = RecursiveDescent` and report **only**
  `TransparentDeclarationClosure`, on `buffer_nat_to_int` and `main`, then reach
  the known size failure. ⇒ ⭐ **This node is confirmed as the fix for both held
  candidates**, which is what the reorder assumed and did not wait for.
- **`AC-3`.** `D5`'s owner/phase validation is present and fails closed **before**
  `D6` lands. A commit ordering that removes the residual first fails this AC.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved: a new `RuntimeExpr` form must still be unable to compile until the
  classifier assigns it. ⛔ Do not replace the exhaustive match with a wildcard.
- **`AC-6` — ⭐ MEASURE THE ROOT'S COST. Added 2026-07-29 (operator: this part
  of the compiler must be both correct *and* efficient).** Record, for the
  fixture, the **emitted function count** and the **per-function code-size
  distribution** under each authority — the `RecursiveDescent` root before, and
  the `FunctionizedUnits` population after. Post the table.

  ⭐ **Why this AC exists:** [[RT-SCALE-B]] returned verdict **(a)** — linear,
  no exponent — but it was **bounded to the governed recursive resource-bracket
  populations and excluded the mutually exclusive `RecursiveDescent` root**
  (Architect, `evt_3t7t27e3rv8cx`). ⇒ **The monolithic root has never been
  scale-measured.** `"Code for function is too large"` is that unmeasured cost
  surfacing as a hard ceiling instead of as a curve. This node is the first
  point where both authorities can be measured on the same program.

  ⛔ **Report the measurement; do not tune to a threshold, and do not pin a
  number.** No target figure is set here and none may be inferred — a pinned
  size number would rot at the next merge, and the AC is discharged by the
  table existing and being routed to the Steward, not by any value in it.
  ⛔ A regression in either figure is a **reportable finding**, not a licence to
  widen this node's scope.

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
the fixture, or if the port lands and `AC-1` still fails **on either delta**.
⛔ Do not attempt a size reduction in either case.

**§5a count of record: 21**, entries **12**, next predicate check the **15th
entry**, next research pull **#24** — carried on [[NATIVE-HANDLE-CARRIER]].
⛔ The `NATIVE-HANDLE-CARRIER` stop that reordered this node is **not** #22: it
routed a red row to the node that already owned it, and no new mechanism failed.

## 8. What landing this closes

**Two held nodes, not one.**

- [[PX8-ERRID-ALLOC]] is released the moment this merges — its candidate
  `ad7298fb` is rebased and preserved, and Foundation owes no rebuild, only a
  re-run. That in turn releases [[PX8-ERRID-SCOPE]] and clears the last of `PX8`'s
  three blockers on this path.
- [[NATIVE-HANDLE-CARRIER]] resumes from preserved `85dcee25` — **11 of 12
  `rt_parity_native` rows already green**, `D1` rebase complete, the identity arm
  re-derived. It owes a second rebase over this node's `core.rs` rewrite, then
  `AC-2`'s Big-identity mutation and the two `AC-4` positive-red controls, which
  the hard stop pre-empted.

⭐ **The selector's own doc comment calls it "the one *temporary* B2F migration
selector" (`core.rs:174`).** This node is that migration finishing, not a new
mechanism.
