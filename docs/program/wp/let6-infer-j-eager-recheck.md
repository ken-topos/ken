# LET-6 · Lower the eager `J` recheck in elaborator `infer_j`

**Owner:** Team Language · **Size:** S · **Branch:**
`wp/let6-infer-j-eager-recheck` · **Base:** current `origin/main` (`cf741d3b`)
**Gate:** Language QA + **Architect** · **CI:** **FULL** (elaborator + the LET-3
P2 `map_build_acceptance` harness + the conformance `J` negatives)

**This unblocks LET-3 Phase 2**, which is re-held on exactly this defect.
Foundation's full 34-family candidate and the minimized one-declaration red
reproducer are both preserved; nothing in the frozen naming table changes.

## 1. The defect (Architect's ruling `evt_29s79hphxdvhd` — this is the fixed input)

**It is a real elaborator COMPLETENESS defect — not a dependent-`J` metatheory
restriction, and not a kernel/conversion bug.** The earned `entry_key`
vocabulary is valid and stays.

**Why the `let`-alias is already definitionally equal (no transport needed):**

- `conv::whnf(Term::Let)` **zeta-reduces** with `subst0(body, val)`.
- Kernel `check(Term::Let)` checks the **substituted** body against the outer
  expected type.
- `subst0` descends through `Term::J`.

So once the complete `Term::Let` reaches the **universal whole-result kernel
check**, every `entry_key` inside the `J` is replaced by `pair_fst k v e`; the
bridge endpoint and the aliased endpoint become identical. **No propositional
transport is semantically required.**

**Where the false rejection is:** `infer_j` checks the let body under an
**assumption-only `Context`** — it pushes only `entry_key : k`, *not* the
definition `entry_key := pair_fst k v e`. Inside that body `infer_j` constructs
`term_j` and **immediately** runs a construction-time `kernel_infer` on it. At
that premature boundary the enclosing `Term::Let` **does not yet exist** and the
`Context` carries types only, so the kernel sees a neutral `Var(0)` on one
endpoint and the raw projection on the other — exactly the reported
`TypeMismatch` (`g160` vs `g162`).

> **⇒ That construction-time `kernel_infer` is NOT the soundness net.** The
> load-bearing net is the **whole-result** kernel check: `declare_def` for
> declarations and the final `kernel_check` in standalone `elaborate_rexpr`.
> Those see the enclosing `let` and zeta-substitute it. The prior
> surface-transport differential already disabled this local eager check and
> showed all six positive/negative cases still gated correctly.

## 2. The repair — remove the eager recheck from `infer_j` ONLY

In `infer_j` (elaborator; kernel target is `check.rs::infer_j`, unaffected),
**delete the construction-time `kernel_infer` block and its `zonked_ctx` /
`zonked_term_j` setup**, retaining exactly:

```rust
let term_j = Term::J(
    Box::new(motive_core),
    Box::new(base_core),
    Box::new(eq_core),
);

Ok((term_j, result_ty))
```

**Update the comment** at that site to state precisely that **whole-result
admission is the sole soundness net**, and that **eager checking in an
assumption-only local `Context` is incomplete for definitional `let` aliases**
(a neutral binder variable cannot zeta-reduce to its definition until the
enclosing `Term::Let` is admitted as a whole).

**Anchors (re-derive by NAME/STRUCTURE — line numbers are from `cf741d3b` and
may move):** `fn infer_j` (`elab.rs`, ~2382); the `let term_j = Term::J(…)`
construction (~2449); the `let zonked_term_j = cx.metas.zonk_term(&term_j);` +
`kernel_infer(cx.env, &zonked_ctx, &zonked_term_j)…` block to remove (~2462–2463)
and the `zonked_ctx` built just for it. Treat every claim in §1 as perishable: if
a fixed input is false against the landed code, **say so and escalate** — do not
build around it.

## 3. Acceptance criteria (Architect's discriminator net — all required)

- **AC1 — the retained reproducer flips.** The one-declaration `entry_key`/`J`
  reproducer (`delete_from_list_acc_lookup_none_dispatch`, minimized against
  byte-original `cf741d3b`) is **RED on `cf741d3b`, GREEN after the repair.**
- **AC2 — positives + negatives still gated.** Existing surface-transport `J`
  positives and malformed-`J` negatives remain green; **the negatives must still
  be REJECTED — by the universal kernel boundary**, not by the removed eager
  check. (`green-vs-green-does-not-confirm-a-fix`: show a malformed `J` is still
  rejected after removal.)
- **AC3 — prove the whole-result net still fires for standalone exprs.** Add a
  standalone-expression malformed-`J` negative (or an equivalent compile pin)
  proving `elaborate_rexpr` still performs its final `kernel_check`. This guards
  against accidentally removing the *real* net along with the eager one.
- **AC4 — INTEGRATION WITNESS: re-run the exact Map harness**
  (`map_build_acceptance`), including **`delete_from_list_acc_lookup_none_dispatch`**
  and **`union_from_list_acc_lookup_assoc_inner`**. Both must elaborate.
- **AC5 — ZERO delta** in `crates/ken-kernel/**` (incl. `Context`, conversion,
  `Term::J`, `Term::Let`, `trusted_base`) and in `catalog/**`. The repair is
  confined to elaborator `infer_j` plus its tests. **No catalog workaround.**
- **AC6 — no-regression is GREEN IN CI**, never a local `--workspace` run
  (operator hard rule, `COORDINATION.md §12`). Targeted `scripts/ken-cargo -p`
  locally.

## 4. ⛔ Guardrails (Architect's do-nots — binding)

- **⛔ Do NOT change kernel `Context`, conversion, `Term::J`, or `Term::Let`.**
  The kernel is correct; it already zeta-reduces the whole-result `Let`.
- **⛔ Do NOT add a transport / `J`-bridge** to the Map proof, **do NOT leave the
  rigid endpoint raw**, and **do NOT exempt the member from the frozen
  vocabulary.** The naming table is unchanged and correct.
- **⛔ Do NOT weaken the universal `declare_def` / standalone `kernel_check`
  boundary.** You are removing a *redundant premature* check, not the net. If you
  find yourself touching the whole-result admission path, **STOP AND ESCALATE.**
- **⛔ Do NOT special-case `let` or `J`.** The fix is deletion of an
  over-eager check, not a new branch.

## 5. Disposition

LET-3 Phase 2 remains **held** with both preserved trees; its naming table does
not change. The frame's semantic-conservativity claim was true — its implicit
**buildability** premise ("every definitional alias survives every eager
elaborator subterm check") was false, and this WP corrects the elaborator, not
the proof. **After LET-6 merges, the Steward returns Foundation to the preserved
full 34-family candidate** (LET-3 P2 resumes unchanged).
