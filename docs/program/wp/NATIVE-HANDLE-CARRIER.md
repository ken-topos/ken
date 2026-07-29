# NATIVE-HANDLE-CARRIER — carry the sealed `BufferHandle` to native GREEN

**The elaborator half is done and preserved. The residual is one `ken-runtime`
primitive arm plus a real rebase, and landing it closes
[[PX8-F-CAP-41]] Phase 2 in the same merge.**

**Owner:** Team Runtime (`runtime-leader` + `runtime-implementer` +
`runtime-qa`). **Branch:** `wp/NATIVE-HANDLE-CARRIER`. **Size:** M.
**Risk:** medium — the code slice is S; the rebase is where this gets lost.

**Status:** Steward frame, shovel-ready.
⛔ **SERIALIZED behind [[RT-NATIVE-FNSPLIT]]** — do not start until it merges
(Steward ruling `evt_1v37rgez26kmf`). Both own `lowering/core.rs`.

⭐ **On the Linux ABI I critical path.** `PX8` gates 15 of that program's 19
nodes; this is one of `PX8`'s three blockers.

---

## 1. ⭐ Premise correction — there is ONE input ref, not two

Both node files say to "fold `c07e63c2` with `f0eb65ce`." **Measured: there is
nothing to fold.** `f0eb65ce` is `c07e63c2`'s parent:

```
c07e63c2  NATIVE-HANDLE-CARRIER: preserve arbitrary-precision body literals
f0eb65ce  WIP PX8-F-CAP-41: seal capacity-carrying buffer handle
8ebe370a  PX8-F-CAP-41 Phase 1 (§38 fold)   <- merge-base with main
```

⇒ **Take `c07e63c2` alone.** It already carries the handle/admission impl *and*
the elaborator slice. ⛔ Do not attempt a merge of the two refs; you would be
merging a commit with its own ancestor.

| ref | sha | what |
|---|---|---|
| `origin/preserved/native-handle-carrier-c07e63c2` | `c07e63c2` | **the input** |
| `origin/preserved/px8-f-cap-41-p2-buffer-handle-f0eb65ce` | `f0eb65ce` | its parent, informational |

⚠ Both are `preserved/*` refs, **not** live WP branches. Cut
`wp/NATIVE-HANDLE-CARRIER` from `c07e63c2`; leave the preserved refs untouched.

---

## 2. What is already GREEN, and what remains

**Done in `c07e63c2` (Foundation, `ken-elaborator` only — no `ken-runtime`):**
the driver's `MissingClosureMetadata` collapse was de-erased and the true root
cause fixed — checked-core `BigInt` literals were being narrowed to `i64`, and
the CAP-41 fixture reaches `u64::MAX` through the checked `intToUInt64` bound.
Body-view, computational-IH census, and erasure are **GREEN**. Interp half is
GREEN on all four CAP-41 rows.

**The residual, measured at `origin/main = 5404108a`:** the fixture now fails
only at object emission —

```
int_to_uint64_raw is not in the supported native set
```

`grep -rn 'int_to_uint64_raw' crates/ken-runtime/src/` returns **nothing**. The
primitive is absent from the native lowering entirely.

> ✅ **RESIDUAL RE-MEASURED AND STILL TRUE at `origin/main = 06cb2964`**
> (Steward, 2026-07-29 — supersedes the `dca1b793` and `5404108a` measurements).
> `int_to_uint64_raw` is **still absent** from `crates/ken-runtime/src/` — zero
> occurrences — and on `main` it appears only in `crates/ken-interp/src/eval.rs`,
> the interp half this frame already records as **GREEN**.
>
> ⭐⭐ **This re-measurement is worth more than the two before it.** Between them
> the **entire `RT-NATIVE-FNSPLIT` arc landed and closed** — `RT-FNSPLIT-RECUR-PORT`
> plus the Scale nodes rewrote `crates/ken-runtime/src/cranelift_backend/`
> wholesale (`lowering/core.rs` **+3899/−1022**, `lowering/mod.rs` **+3654/−156**).
> ⇒ **A rewrite of exactly the subsystem that owes this primitive did not build
> it.** The node still has its subject; no re-framing pass is owed.
>
> ✅ **Both input refs still resolve on `origin`** (checked, not assumed):
> `preserved/native-handle-carrier-c07e63c2` → `c07e63c2`, and
> `preserved/px8-f-cap-41-p2-buffer-handle-f0eb65ce` → `f0eb65ce`.
>
> ⚠ **The one number that rotted:** `§3` says `main` is **215** commits ahead of
> `8ebe370a`; it is now **303**. ⛔ Do not re-pin that figure — the **derivation**
> is the pin (`git rev-list --count 8ebe370a..origin/main`) and it grows with
> every merge. It moved in the direction that makes `§3`'s argument *stronger*,
> not weaker: the rebase is more of a deliverable now, not less.
>
> ⛔ **`§3`'s churn table is now an UNDERSTATEMENT, and that matters.** It was
> measured when the collision was elaborator-only. The FNSPLIT arc has since
> rewritten the native lowering you must add the primitive to, so `§4`'s native
> arm lands in a **different mechanism** than the one it was written against.
> ⇒ Re-derive `§4` against current `main` at pickup. The *ruling* stands; the
> code it points at does not.

---

## 3. ⛔ The rebase is a deliverable, not a preliminary

`c07e63c2` is based at `8ebe370a`. **`origin/main` is 215 commits ahead of
that**, and the collision is not incidental:

| file | main's churn since `8ebe370a` | the branch's own churn |
|---|---|---|
| `crates/ken-elaborator/src/prelude.rs` | +100 | +115 |
| `crates/ken-elaborator/src/erasure.rs` | +99 | +43 |
| `crates/ken-elaborator/src/compiler_driver.rs` | +25 | +30 |
| `crates/ken-cli/tests/px8ta_oriented_subcontinuation.rs` | +37 | +2 |

⇒ **All three production files of the elaborator slice were also edited on
`main`.** This is a genuine three-way merge over the exact lines the slice
changes, not a fast-forward.

⭐ **The failure mode this AC exists to catch:** a rebase that resolves a
`prelude.rs` conflict by taking the branch side wholesale **silently reverts
215 commits' worth of landed work in that file**, and every targeted test still
passes because the reverted work has its own tests elsewhere. ⛔ Do not resolve
conflicts by side-preference. Re-derive each hunk.

**Control (`AC-1`):** after the rebase, `git merge-tree origin/main <your-sha>`
and confirm every blob that `main` advanced in those four files survives with an
OID that reflects **both** changes — ⛔ not the branch's pre-rebase OID.

---

## 4. The native arm — Architect-ruled, `evt_7xrcjp0apb4f1`

⛔ **Settled inputs. Do not re-litigate.**

`int_to_uint64_raw` is **value identity**, ⛔ **NOT** a machine `i64 -> u64`
conversion. Ken's fixed-width carriers share the exact `Int` runtime
representation. The native arm must:

- require exactly one `Lowered::Int` argument;
- return **that same `Lowered::Int` unchanged** — including the native-Int tag
  sidecar and payload/arena slot;
- preserve `18446744073709551615` as the existing **Big signed-magnitude**
  value;
- leave range admission to the derived checked `intToUInt64` wrapper, which
  proves `0 <= n <= u64::MAX` before calling the raw cast.

**Extend the existing identity arm**, currently at
`crates/ken-runtime/src/cranelift_backend/lowering/core.rs:6827`:

```rust
"uint8_to_int" | "int_to_uint8_raw" => {
    let [value]: [Lowered; 1] = lowered_args.try_into().map_err(...)?;
    let Lowered::Int { .. } = value else { return Err(unsupported(...)) };
    Ok(value)
}
```

⛔ **A Cranelift integer cast or an `i64` fast path truncates, wraps, or retags
the Big arm.** That is the named failure mode, and it is invisible to any test
whose operand fits in `i64`.

---

## 5. ⭐ Scope holds at UInt64 — but know what you are standing next to

Measured: the interpreter treats the **entire** representation-sharing cast
family as identity in one arm (`crates/ken-interp/src/eval.rs:1355-1369`,
`=> a.clone()`) — **22 members**. Native implements **2**.

⇒ **Every other member is a latent instance of this exact wall**, and the
diagnostic staircase will surface them one at a time.

⛔ **Do not generalize in this WP.** The Architect ruled the family
generalization **optional** and not required for CAP-41 GREEN. Adding a
wildcard over primitive names would ship 20 untested arms behind one test.
⭐ Record the 2-of-22 count in `D5` so the next WP is framed against a measured
surface rather than rediscovering it.

---

## 6. ⚠ Diagnostic-staircase contingency

`int_to_uint64_raw` is **not asserted to be the final gap.** This fixture has
revealed a new wall at every layer:

```
MissingClosureMetadata -> int_lit_outside_native_i64 -> int_to_uint64_raw -> ?
```

Acceptance is **"full two-engine oracle GREEN"**, not "the primitive was added."
Any further native gap the exact fixture hits is **surfaced and triaged**, never
worked around.

⭐ The Architect enumerated the checked closure's primitives — `leq_int`,
`and_bool`, `int_to_uint64_raw`, `sub_int`, `eq_int`, `add_int` — and native
already handles all but one. `Some`/`None`, handle construction/projection, and
result branching are constructor/control lowering, **not** primitives. ⇒ Expect
no further *primitive* gap; the retained stop condition is for a
**non-primitive constructor/effect** gap. ⛔ Do not pre-inflate scope on the
contingency.

---

## 7. Deliverables

- **`D1`** — `c07e63c2` rebased onto current `origin/main`, conflicts resolved
  hunk-by-hunk, with the stale-base check of `§3` run and reported.
- **`D2`** — the `int_to_uint64_raw` identity arm in `core.rs`, extending the
  landed `uint8_to_int | int_to_uint8_raw` arm.
- **`D3`** — the four focused discriminators of `§8` (`AC-3`), before the full
  oracle.
- **`D4`** — the CAP-41 fixture carried to **full native GREEN**, and the full
  two-engine oracle: all four CAP-41 rows absolute GREEN on **both** engines.
- **`D5`** — the Architect's six-axis matrix (a)–(f) discharged, plus the
  2-of-22 family count from `§5` and any further staircase gap encountered.

---

## 8. Acceptance criteria

- **`AC-1`** ⭐ **(the rebase, and the one most likely to be got wrong)** — no
  landed work is reverted. **Control:** `git merge-tree origin/main <sha>` shows
  the four `§3` files carrying **both** main's and the branch's changes. ⛔ A
  blob OID equal to the branch's pre-rebase OID on any of them fails this AC.

- **`AC-2`** ⭐ **(load-bearing)** — the arm is **identity, not a cast**.
  **Control:** mutate the new arm to a Cranelift `i64` cast (or an `i64`
  fast path) and show the `u64::MAX` discriminator **reddens**. ⛔ A test whose
  operand fits in `i64` cannot distinguish identity from truncation and does not
  discharge this AC — the control must be on the **Big** carrier.

- **`AC-3`** — the four focused discriminators hold:
  1. `intToUInt64 u64::MAX` reaches `Some` natively, preserving the exact Big
     value **and tag**;
  2. `intToUInt64 (u64::MAX + 1)` and `intToUInt64 (-1)` reach `None` — proving
     the checked **wrapper**, not the raw arm, owns admission;
  3. the native arm and the interpreter agree on representation identity, with
     no wrap/truncation mutation surviving;
  4. existing `UInt8` conversion behavior is unchanged.

- **`AC-4`** — the six-axis matrix: (a) normalized checked declaration body
  view with the underlying error lane visible; (b) computational-IH
  census/metadata consistency; (c) erasure of handle construction, match, and
  projections; (d) runtime constructor/value lowering; (e) unchanged raw
  `Resource Buffer` host request and wire ABI; (f) constructor and both
  projections **absent from the public name map**.

- **`AC-5`** — the regression that made an honest partial inadmissible is
  repaired. **Control:** `fs_read_at_malformed_offset_narrows_to_invalid_offset`
  — a pre-existing two-bracket native read row that was GREEN before the API
  migration and then failed pre-execution — is **GREEN again**.

- **`AC-6`** — no collateral regression. **Control:** the landed `BufferSpan`
  product stays GREEN; malformed / stale / closed authority behavior is
  unchanged.

- **`AC-7`** — targeted green. **Control:** name the exact
  `scripts/ken-cargo test -p <crate>` invocations and pass counts. ⚠ A full
  `-p ken-interp` run is required if the reifier or value shape changes
  (attested `eval.rs` ⇒ OID-bump rider). ⛔ No `--workspace`
  (`COORDINATION §12`); workspace-green means **green in CI**.

---

## 9. ⛔ Banned scope

- ⛔ **No `spec/` edit.** `38-ffi-io.md` is LOCKED and the representation is
  normatively settled — a token-only handle cannot supply the raw resource
  public consumers need and would reopen the authority/ABI boundary Phase 1
  closed.
- ⛔ **No `conformance/` edit.** The four CAP-41 seed rows are the oracle, not
  the deliverable.
- ⛔ **No family generalization** over the other 20 cast members (`§5`).
- ⛔ **No honest partial.** The Architect ruled this out explicitly: the
  candidate *regresses* an already-GREEN native row (`AC-5`), so interp-only is
  not a landable state.
- ⛔ **No concurrent `lowering/core.rs` edit** while `RT-NATIVE-FNSPLIT` is
  live.

---

## 10. Contention — ⚠ read this before pinning anything

⛔ **`crates/ken-runtime/src/cranelift_backend/lowering/core.rs` WILL MOVE.**
Its blob at `origin/main = 5404108a` is
`2da09df89df2bc4c0792df999da3cae96506ec5e` and the identity arm is at `:6827` —
**both are recorded as provenance, not as pins.** `RT-NATIVE-FNSPLIT` owns an
indivisible continuation-partitioning change to that exact file and lands first.

⇒ ⭐ **Re-derive the arm's location at pickup.** Do not search for `:6827`;
search for `"uint8_to_int" | "int_to_uint8_raw"`.

⚠ Re-derive build-slot availability too. `ken-cargo` blocks silently for up to
30 minutes on lock contention; `fuser -v /tmp/ken-build-locks/build.lock` names
the holder — ⛔ don't pipe it through `head`.

---

## 11. Hard stop

⛔ Route to the Steward if:

- the rebase produces a conflict in `prelude.rs`/`erasure.rs`/
  `compiler_driver.rs` you cannot resolve without choosing a side — ⭐ say which
  hunk and what the two sides assert; **or**
- the fixture hits a **non-primitive** constructor/effect native gap (`§6`);
  **or**
- identity lowering turns out to be unsound for the Big carrier on the native
  path — that reopens the Architect's means ruling and is not yours to re-decide;
  **or**
- `AC-5`'s pre-existing row cannot be restored, which would mean the elaborator
  slice itself regressed something.

---

## 12. What landing this closes

⭐ **This merge closes BOTH [[NATIVE-HANDLE-CARRIER]] and [[PX8-F-CAP-41]]
Phase 2.** They are one deliverable — the carrier fix is meaningless without the
fixture it unblocks, and the fixture cannot land without the fix. Flip both
nodes on the same merge.

⇒ `PX8` then has two blockers left: [[PX8-WROTE-ABS]] (Verify, released) and
[[PX8-ERRID-SCOPE]] (Verify, behind [[PX8-ERRID-ALLOC]]).
