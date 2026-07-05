# WP `KL-let-check` — kernel `check` of `Term::Let` uses the wrong expected type

- **Owner:** kernel team (kernel-leader → kernel-implementer, kernel-qa).
- **Reviewer (mandatory, TCB fidelity gate on the diff):** Architect — diagnosed
  the defect (`evt_1ekm9176r61z6`) and is the TCB design authority.
- **Size:** S (a single checker arm + tests). **Risk:** low mechanism, **high
  care** — this is the trust root (`crates/ken-kernel`), so full TCB rigor
  applies regardless of diff size.
- **Branch / PR:** `wp/KL-let-check`, one PR.
- **Trigger:** surfaced by CAT-2 D1 (the wired-superclass projection
  `let d : Applicative List = … in d.functor.map …` kernel-rejected). CAT-2 D1
  landed correctly on the legitimate projection forms and does **not** depend on
  this fix; this WP closes the underlying kernel defect per the operator
  directive (2026-07-05): *fix the root cause, do not work around it* — now
  `docs/PRINCIPLES.md` §13.

> **Perishable-state caveat (verify against landed code, not this line).** Line
> numbers below are as of `origin/main @ 4bc57c8`. Re-locate the arm by its shape
> (`fn check` → `Term::Let { ty, val, body }`), not the line number, before
> editing.

## 0. Objective

The trusted kernel checker `check(env, ctx, t, ty)` — `ty` is the **expected**
type — mis-handles `Term::Let`. Fix its `Term::Let` arm to verify the
substituted body against the **enclosing expected type**, not against the
let-binder's own annotation.

## 1. Root cause (confirmed by reading the code)

`crates/ken-kernel/src/check.rs`, in `pub fn check(env, ctx, t, ty)` (~line 377),
the arm (~line 417):

```rust
Term::Let { ty, val, body } => {
    classify(env, ctx, ty)?;
    check(env, ctx, val, ty)?;
    check(env, ctx, &subst0(body, val), ty)   // <-- BUG
}
```

The pattern field **`ty` shadows the function parameter `ty`** (the expected
type). So inside the arm every `ty` is the **let-binder annotation** `A`, and the
enclosing expected type `E` is unreachable. The arm therefore checks the
substituted body against `A` and **never checks it against `E` at all**.

This is **both** a soundness hole **and** a completeness bug:

- **Soundness (accepts ill-typed terms).** `check(let x:A = v in b, E)` succeeds
  whenever `b[x:=v] : A`, *regardless of `E`*. So `(let x : Nat = 5 in x)` is
  accepted at expected type `Bool` — the kernel would certify a let-expression at
  a type it does not have. The trust root must reject this for *all* well-formed
  terms, not merely for terms a well-behaved elaborator happens to emit.
- **Completeness (rejects valid terms).** A valid `let x:A = v in b` whose body
  type `E` differs from `A` (the common case — the body rarely has the binder's
  type) is rejected with a `TypeMismatch` against `A`. This is the CAT-2 D1
  symptom: `expected: (g542 Dg72), found: (Dg72 @2)`.

The correct behavior already exists in the sibling **`infer`** arm (~line 293),
which does `infer(env, ctx, &subst0(body, val))` — infers the body's real type.
The `check` arm is the sole defect.

## 2. Mandated fix

Rename the shadowing field and check the body against the **outer** expected
type:

```rust
Term::Let { ty: let_ty, val, body } => {
    classify(env, ctx, let_ty)?;
    check(env, ctx, val, let_ty)?;
    check(env, ctx, &subst0(body, val), ty)   // outer expected `ty`
}
```

Notes:

- `subst0(body, val)` collapses the let binder (de Bruijn index 0) and yields a
  term in the **outer** context; the outer expected `ty` is in that same context,
  so no shift is required. This matches how the `infer` arm substitutes.
- **Do not** instead delete the arm and fall through to the `_` mode-switch
  (`infer` then `convert_type`). That would regress `let x:A = v in b` where the
  **body is a non-inferrable introduction form** (a bare `λ`/`pair`/`refl`):
  `infer` rejects those ("cannot infer an introduction form"). Propagating the
  expected type **into** the body preserves bidirectional check-mode and is
  strictly more complete. Keep the explicit arm; only fix the type it uses.

## 3. Acceptance criteria (all required)

- **G1 — soundness discriminator (negative test).** Add a kernel test that a
  genuinely ill-typed let is **rejected**: e.g. `check` (or an ascription that
  forces `check`) of `let x : Nat = <nat-literal> in x` against expected `Bool`
  (any `E` not convertible to `Nat`) returns a `TypeMismatch`. **First confirm on
  unmodified `origin/main` that this case is currently *accepted*** (proving the
  hole is real), then that the fix **rejects** it. Do not weaken the assertion to
  a bare `is_err()` — match the `TypeMismatch` shape (assert-specific-variant).
- **G2 — completeness discriminator (positive test).** A valid let whose body
  type differs from the binder annotation is **accepted** after the fix: a
  minimal `let x : A = v in body` with `body : E`, `E ≢ A`, checked at `E`; and
  (integration) the CAT-2 form `let d : Applicative List = Applicative_instance_List
  in d.functor.map a b g xs`. Confirm this is **rejected** on unmodified
  `origin/main` and **accepted** after the fix.
- **G3 — check-mode-into-body preserved (regression guard).** `let x : A = v in
  (λ …)` checked against a `Π` type still checks — the body is an introduction
  form that needs the expected type. Add a positive test so a future refactor to
  the mode-switch fallback (see §2) cannot silently regress it.
- **G4 — full-workspace green.** `cargo test --workspace` passes, **not** just
  `-p ken-kernel`. A kernel checker change has workspace-wide blast radius:
  downstream proof terms in `packages/` and other crates may have been riding
  either face of the bug. (K7 lesson — a kernel-soundness diff is
  `ken-kernel`-only but the *landing unit* is workspace-wide.) If any downstream
  `.ken`/test proof term must migrate because it was exploiting the accepted-
  ill-typed path, migrate it **in this same workspace-green unit** and call it out
  — but a correct elaborator should have emitted none, so expect zero downstream
  churn and treat any as a finding to report, not silently patch.
- **G5 — TCB surface unchanged.** No new `Term`/`Decl` variant; `trusted_base()`
  byte-identical; `conv.rs`/`whnf` untouched. The source diff is confined to the
  one `check` arm in `check.rs` plus tests.
- **G6 — sibling-arm audit (record the result).** Confirm and note in the PR that
  the other `Term::Let` arms are **not** affected: `raw_wf` (check.rs ~45) and
  `collect_consts_in_tb` (foreign.rs ~307) are pure structural traversals with no
  expected-type parameter (no shadowing hazard), and the `infer` arm (~293) is
  correct and left unchanged. (Steward pre-verified this; re-confirm on the head
  you build.)

## 4. Guardrails (do-not)

- The fix simultaneously **tightens** soundness and **loosens** completeness —
  both are correct; do not "restore" the old behavior in either direction.
- Do **not** loosen any other check, touch `conv.rs`/`whnf`, add an `Axiom`/
  postulate, or alter `trusted_base()`.
- Do **not** route CAT-2 or any caller around this — the fix lives in the kernel
  (PRINCIPLES §13). CAT-2 D1's landed projection forms stay as-is; a `let`-bound
  wired-projection regression may be added by the language team **after** this
  lands (optional follow-up, not part of this WP).
