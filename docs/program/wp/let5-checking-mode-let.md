# LET-5 · `check()` has no `RLet` arm — the checking-mode leak

**Owner:** Team Language · **Size:** S · **Branch:** `wp/let5-checking-mode-let`
**Base:** current `origin/main` · **Gate:** Language QA + **Architect**
**CI:** **FULL** (elaborator + the LET-3 P2 harness)

**⛔ RELEASE ORDER: this WP does NOT start until AX-2 has MERGED.** It edits
`crates/ken-elaborator/src/elab.rs`, which AX-2 is rewriting across ~24 sites
(threading `ElabCtx.owner_label` through). A concurrent WP in that file collides
head-on with the critical path.

**It unblocks LET-3 Phase 2**, which is held on exactly this defect. Foundation's
34-lemma inventory and frozen vocabulary table are unaffected and wait for it.

## 1. The defect (Architect's ruling, `evt_5dhcdfsvnvebe` — this is the fixed input)

**It is NOT a sort bug, and NOT a single-binding bug.** There is **no restriction
on binding `Type 0` data inside an `Ω` proof.** It is a **bidirectional-elaboration
mode leak**:

1. `Expr::ELet` is normalized (`resolve.rs`, `Expr::ELet` arm) into **nested
   `RExpr::RLet` nodes**. **A one-binding `let` and a binding group's outer node
   have the SAME resolved form.** There is no singleton-specific rule to repair.
2. **`check()` has no `RExpr::RLet` arm.** Its final arm is
   `_ => { let (core, inferred_ty) = infer(cx, expr)?; unify_types(…); Ok(core) }`
   — so a `let` with a **known goal** falls through to `infer()` and **the expected
   type is LOST**.
3. `infer`'s `RLet` arm is correct in itself — it checks/infers the RHS, extends
   the context, infers the body, returns `subst0(body_ty, rhs_core)`. **It does not
   assign the binder's universe to the whole `let`.**
4. The failing bodies are **checked proof `match`es**. With the goal gone they take
   `infer_match`, which builds a constant `… → Type ℓ` motive and **falls back to
   `Type 0`** when the return classifier is `Ω`. The kernel then **honestly**
   reports `expected: Type 0, found: Ω0`.

> **⇒ The kernel diagnostic is a DOWNSTREAM WITNESS, not the cause. Do not "fix"
> the kernel, and do not touch `infer_match`.**

**Why grouped bodies passed and singletons failed** — and this is the trap: those
grouped bodies happened to be **inferable applications**, while the failing
singleton wrappers put a **checked-mode `match`** immediately behind the `RLet`.
**The boundary is BODY MODE, not binding count. The correlation with binding count
is a red herring — do not build to it.**

## 2. The repair — one checking rule

```
Γ ⊢ rhs ⇐ A        Γ, x:A ⊢ body ⇐ weaken(expected, 1)
──────────────────────────────────────────────────────
Γ ⊢ let x : A = rhs in body ⇐ expected
```

- **Unannotated binding:** infer `rhs` first, **exactly as the existing
  `infer`/`RLet` arm does.** Factor the shared annotated/unannotated RHS
  preparation rather than duplicating it.
- Emit the **same `Term::Let { ty, val, body }`** the `infer` arm emits.
- **Push the binder, check the body against `weaken(expected, 1)`, and POP THE
  CONTEXT ON BOTH SUCCESS AND ERROR.**
- **⛔ Do NOT special-case `bindings.len() == 1`.** There is no such distinction in
  the resolved form, and building one would encode the red herring into the
  elaborator.

## 3. ★ The fallthrough is a CLASS, not one bug — determine the blast radius

`check()`'s `_ =>` arm hands **every unhandled variant** to `infer()` + `unify`.
Enumerated on current `origin/main`, **only 6 of 16 `RExpr` variants have a
`check()` arm** (`RApp`, `RCon`, `RLam`, `RMatch`, `RNumLit`, `RStr`).

For a genuinely **inferable** variant (`RVar`, `RUniv`, `RAsc`, `RArrow`, `RPi`,
`RProj`, `RBinOp`) infer-then-unify is **correct and loses nothing.** The leak
bites **only** a variant that **propagates the goal to a subterm**:

| variant | shape | goal-propagating? |
|---|---|---|
| **`RLet`** | binds `x`; **body inherits the goal** | **YES — this WP** |
| **`ROld(e)`** | transparent wrapper; `old e` has exactly `e`'s type | **LOOKS LIKE THE SAME LEAK — DETERMINE IT** |
| `RAttachedProofRef` | (determine) | (determine) |

**AC5 below requires you to settle `ROld` and `RAttachedProofRef`**: either they
carry the identical leak — in which case adding their arm is the *same rule* and
lands here — **or you record concretely why the goal cannot matter for them.**
**Do NOT expand beyond that determination**; if either turns out to need more than
one more arm, **STOP AND ESCALATE** rather than growing an S into an L.

*(This is the Steward's addition to the Architect's ruling, not part of it. The
ruling scoped `RLet`; I am asking only that you not leave a known-shaped sibling
unexamined while you are standing in the file.)*

## 4. Fixed inputs — ★★ EVERY LINE NUMBER BELOW WILL BE STALE

**AX-2 rewrites `elab.rs` across ~24 sites and adds a field to `ElabCtx`. Every
line number here WILL have moved by the time you start.** These anchors were true
at `origin/main @ cbaae5e7`; **re-derive by NAME and STRUCTURE, never by line**:

| anchor | how to re-find it |
|---|---|
| `fn check(cx, expr, expected, _span)` | by name |
| its `_ =>` fallthrough to `infer` | last arm of `check`'s match |
| `infer`'s `RExpr::RLet(_x, ty_opt, rhs, body, span)` arm | by pattern |
| `Expr::ELet(bindings, body, _)` normalization | `resolve.rs`, by pattern |
| `fn infer_match` | by name — **read it, do not edit it** |

**Treat every claim in §1 as perishable. If a fixed input is false against the
landed code, SAY SO AND ESCALATE — do not quietly build around it.** (Language did
exactly this on AX-2 and corrected my producer inventory before it cost anything.
Do it again here.)

## 5. Acceptance criteria

- **AC1 — pre-fix RED / post-fix GREEN** on **(a)** a singleton data `let` whose
  body is the flat `Ω`-valued `match` shape Foundation found, and **(b)** a
  singleton `let` around a **checked introduction** such as `Refl`. **Both must be
  shown red before the fix and green after** — a green-only test proves nothing
  (`green-vs-green-does-not-confirm-a-fix`).
- **AC2 — retain a WRONG-RHS REJECTION.** The new checking path must still reject
  an ill-typed RHS. A checking rule that accepts more than it should is a soundness
  regression, not a fix.
- **AC3 — the existing grouped-vs-nested and data-`let` oracles stay green**, and a
  **grouped** `let` with a **checked-mode body** must now also work. (Under the
  correct diagnosis, binding count is irrelevant — **prove that**, don't assume it.)
- **AC4 — INTEGRATION WITNESS: re-run Foundation's exact LET-3 P2 harness.** The
  lemma that failed (`union_from_list_acc_lookup_assoc_inner`) must elaborate.
  **This is the acceptance test that matters** — the unit tests can pass while the
  real corpus still fails.
- **AC5 — settle `ROld` and `RAttachedProofRef`** per §3: fix (same rule) or record
  why the goal cannot matter. **A silent omission is not an answer.**
- **AC6 — ZERO delta** in: `crates/ken-kernel/**`, `infer_match`, the surface
  grammar, the formatter, and `catalog/**`. **No catalog workaround.** If you find
  yourself editing a `.ken.md` to make this pass, **stop — that is the bug winning.**
- **AC7 — no-regression is GREEN IN CI**, never a local `cargo test --workspace`
  (operator hard rule, `COORDINATION.md §12`). Targeted `scripts/ken-cargo -p`
  locally.

## 6. ⛔ Guardrails

- **⛔ Do NOT touch the kernel.** Its `expected: Type 0, found: Ω0` is an **honest
  report of what it was handed.** Changing it would launder the bug into the TCB.
- **⛔ Do NOT touch `infer_match`.** Its `Type 0` fallback is reachable only because
  the goal was already lost. Fix the leak, not the symptom. *(If you believe
  `infer_match`'s fallback is independently wrong, that is a SEPARATE WP —
  escalate; do not fold it in.)*
- **⛔ Do NOT special-case binding count** anywhere in the fix.
- **⛔ Do NOT start before AX-2 merges.** Same file, active critical path.
