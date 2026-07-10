# ADR 0013 — Int decidable-equality kernel posture

**Status:** Accepted (operator-ruled 2026-07-10; Architect-designed
`evt_36tfyha8nrrd0`; grounded vs `origin/main @ b2997f37`). Supersedes the
per-catalog `Axiom` posture for `DecEq Int` / `Eq Int`. Sibling of
[ADR 0010](0010-lawful-deceq-requires-canonical-carrier.md).

## Context

`DecEq Int` and `Eq Int` currently ship their `sound`/`complete` law fields as
**visible catalog `Axiom`s** (`LawfulClasses.ken.md`), riding the already-trusted
`eq_int` primitive reduction. `Char` is a refinement whose carrier lowers to `Int`
(`{c:Int | isScalar c}` → `Int` as-built), so `Eq Char` bottoms out at `Eq Int`.
The operator directed us to "do the right thing" — make these lawful without a
scattered postulate.

**The load-bearing constraint (verified):** `Int` is an **opaque primitive with no
induction**. The kernel's `conv`/`whnf` does not execute `PrimReduction::Op`, so
`eq_int 5 5` is **neutral at kernel type-checking time** and `Eq Int` is neutral
(`obs.rs:84`, the named gap). Consequently the **universal** laws —
`∀ (x y : Int). IsTrue (eq_int x y) → Equal Int x y` and the converse, quantified
over *abstract* `x, y` — are **irreducibly trusted**: nothing reduces under an
abstract `Int`, so no computation and no reflection can prove them closed. **No
kernel move can make the universal `DecEq Int` laws `Axiom`-free.** This is the
honest boundary; the decision below is the most honest posture reachable, not an
elimination of trust.

## Decision

Two separable layers. Ship both (operator concurred).

### Layer 1 — named kernel certificate (WP DS-6a, required)

Replace the per-instance catalog `Axiom`s with **one named kernel
decidable-equality certificate** for the `Int` primitive, admitted as trusted
kernel vocabulary and counted honestly in `trusted_base()`. `instance DecEq Int` /
`Eq Int` law fields point at the certificate instead of `Axiom`.

- **General, opt-in mechanism** (not `Int`-specific): an opaque primitive type
  *may* register a decidable-equality certificate (its `eq` op + the
  `sound`/`complete` cert pair). `Int` registers with `eq_int`. **Unregistered
  primitives stay neutral = today** (fail-safe). Matches subsume-don't-proliferate
  (#7) and reflect-don't-extend (#6).
- **TCB delta — the whole new trusted line item:** *"the kernel trusts `eq_int`
  decides propositional `Int` equality, both directions"* (split sound + complete).
  It adds **no new primitive** (`eq_int` is already trusted) and admits **nothing
  more** (only `Int`, only equality). The trust becomes one kernel-audited line
  shared by all consumers — like `Top`/`Bottom` are fixed kernel vocabulary —
  instead of a postulate copied into each catalog package.

### Layer 2 — native literal reduction (WP DS-6b, recommended, separable)

Give the kernel literal values via a native `Term::IntLit(value)` variant plus an
`eq_reduce` arm at the `Int` type:

```
Eq Int (IntLit m) (IntLit n)  ⇝  Top     if m == n
                              ⇝  Bottom  if m != n
                              ⇝  neutral if either operand is not a literal
```

Then `Equal Int 5 5` is proved by `refl` and `Equal Int 5 6` is **refutable** —
**genuinely `Axiom`-free and computing for the concrete goals catalog code
exercises**, exactly the way `DecEq Bool` already works via `eq_at_inductive`.

- **Layer 2 adds ZERO trusted lines.** Deciding `5 == 6` is a kernel *decision*
  (like constructor discrimination), not an assumption. It grows audited kernel
  *code* (the `IntLit` variant threaded through subst / whnf / conv / hash /
  typecheck / serialize) and, as a bonus, **retires the current
  fresh-opaque-`Decl::Primitive`-per-occurrence literal hack + `num_values`
  side-table** (reflect-don't-extend win).
- The **universal** laws (Layer 1) stay trusted; **concrete** equality no longer
  is.

## Consequences

- **`Char` rides free.** Refinement carrier-lowering means `Char` *is* `Int`
  definitionally, so `Eq Char` bottoms out at `Eq Int` with **zero Char-specific
  kernel work** — someone writes the trivial `instance DecEq Char` (none exists
  today; only `Ord Char` by transport).
- **`String` does NOT derive** from the `Int` certificate (separate opaque
  primitive, no `eq_string` reduction). `Eq`/`Ord String` is a **separate
  follow-on WP** with its own trust decision — preferred route: decide
  `Equal String` through `string_to_list_char : String → List Char` and
  `Eq (List Char)` (inductive → kernel-decidable once `Char`/`Int` is), reusing
  inductive decidability rather than adding trusted string primitives. Not bundled
  here.
- **Spec divergence flagged (separate reconcile, not this ADR's to fix):**
  `spec/13-pi-sigma.md:133` defines refinement types as a genuine Σ, which diverges
  from the as-built **carrier-erasure** (`{x:A|φ} → A`, φ dropped). Routed to the
  Spec enclave.
- **Conformance (discriminating, fail-closed):** assert the *specific*
  `KernelRejected`/`TypeMismatch`, not bare `is_err`; each arm must FLIP on the
  same mechanism —
  - **over-equate (soundness):** a proof of `Equal Int 5 6` is kernel-**REJECTED**
    (Layer 2: `⇝ Bottom`);
  - **under-equate (completeness):** `refl : Equal Int 5 5` is **ACCEPTED**
    (Layer 2: `⇝ Top`) — *this arm only passes with Layer 2; it is the test that
    proves computation, not relocated trust*;
  - **neutral:** `Eq Int x y` for abstract `x, y` stays **neutral** — neither
    accepted nor refuted (guards over-eager reduction);
  - **zero-Axiom-delta:** elaborating the new instances adds exactly the
    certificate line(s) to `trusted_base()` and **no** catalog `Axiom` (DS-2-style
    before/after set-diff); a closed `dec_eq_decides Int 5 5` needs zero `Axiom`.

## Regime

Per the run's boundary rules this kernel/TCB change may LAND this run through the
full Kernel-team ring + the Architect soundness gate + conformance. The soundness
bar does not relax; the Architect gates the certificate's exact rule and the
`eq_reduce` arm at pseudocode level.
