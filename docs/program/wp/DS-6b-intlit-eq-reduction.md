# WP DS-6b — kernel-native IntLit + Eq Int value-reduction (the "right thing")

**Owner:** Kernel team. **Operator-ruled** (2026-07-10, "do the right thing");
**Architect-designed** ([ADR 0013](../adr/0013-int-decidable-equality-kernel-posture.md),
`evt_36tfyha8nrrd0`). **Separable from and builds on [DS-6a](DS-6a-int-deceq-certificate.md)**
— bigger term-language change; its own review. Read ADR 0013 first.

## Goal

Make **concrete** `Eq Int` **compute**, so closed `Int`-equality is discharged
`Axiom`-free by the kernel — `Equal Int 5 5` proved by `refl`, `Equal Int 5 6`
refutable — exactly the way `DecEq Bool` works via `eq_at_inductive`. The universal
laws stay trusted (DS-6a's certificate); concrete equality no longer is.

## Scope

1. **Kernel-native literal.** Add a `Term::IntLit(value)` variant (BigInt value).
   Thread it through **every** term traversal: subst / whnf / conv / hash /
   typecheck / serialize (the `zonk_term`-style exhaustiveness discipline — a
   missed arm is a latent bug).
2. **`eq_reduce` arm at `Int`:**
   ```
   Eq Int (IntLit m) (IntLit n)  ⇝  Top     if m == n
                                 ⇝  Bottom  if m != n
                                 ⇝  neutral if either operand is not a literal
   ```
   Wire it to consult the DS-6a per-primitive registration (the registered
   literal-decider), so the mechanism stays general, not `Int`-hardcoded.
3. **Retire the literal hack** — replace the fresh-opaque-`Decl::Primitive`
   -per-occurrence minting (`check.rs:1106`, unconditional `fresh_id()`) + the
   elaborator `num_values` side-table with the native `IntLit` (reflect-don't-extend
   — this is the cleanup the variant enables, not scope creep).

## Boundary / constraints

- **ZERO new trusted lines.** Deciding `m == n` on two BigInt literals is a kernel
  **decision** (like constructor discrimination / no-confusion), not an assumption.
  The `trusted_base()` delta for this WP is **empty**. It grows audited kernel
  *code*, not trusted surface — the Architect confirms this at the gate.
- **Reduction only refines neutrality:** reduce `Eq Int` **only** when *both*
  operands are literals; otherwise neutral. No new stuck states, no new loops;
  termination unaffected (primitive types are leaves).
- Zero change to the universal-law trust (that is DS-6a's certificate, unchanged).

## Acceptance bar (the full discrimination — each arm FLIPs on this mechanism)

Assert the *specific* `KernelRejected`/`TypeMismatch`, never bare `is_err`:

- **over-equate (soundness):** `Equal Int 5 6` kernel-**REJECTED** (`⇝ Bottom`).
- **under-equate (completeness):** `refl : Equal Int 5 5` **ACCEPTED** (`⇝ Top`) —
  **this arm passes only with DS-6b**; it is the test that proves *computation*,
  not relocated trust. (Green-vs-green guards nothing here — the verdict must flip
  vs a DS-6a-only base.)
- **neutral:** `Eq Int x y` for abstract `x, y` stays **neutral**.
- **termination / exhaustiveness:** the new `IntLit` arm is handled in every term
  traversal (grep the traversal set); deep/large literals do not diverge.

## Gate

Kernel ring → **@architect soundness gate** (gates the `eq_reduce` arm + the
`IntLit` threading at pseudocode level; confirms zero trusted-line delta and
neutrality-only-refined) → **conformance** (the four arms above, verdict-flip vs a
DS-6a base) → `git_request` to Steward → CI-gated merge. Prominent kernel land —
logged for the operator. Own retro; flag every judgment call. No WP-token
identifiers in production source; `Term::IntLit` and its arms are permanent
mechanism, named for what they do.
