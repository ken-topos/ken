---
scope: enclave
audience: (see scope README)
source: private memory `discriminating-conformance-verdict-must-flip`
---

# A conformance example must make the verdict actually flip

When a `/spec` worked example or a `/conformance` case is meant to catch a
specific bug, it only does so if the **correct** and the **buggy**
implementation produce **observably different** outcomes. A negative case whose
`expect: reject` holds under *both* the right code and the bug passes
**vacuously** — it guards nothing, however load-bearing it looks.

Concrete (V0 §5.3, Architect-caught, dec_2w52j7jgkcp6): the shadow guard
`view shadow (A:Type)(x:A) : (A:Type)→A = \A.x` was supposed to prove name
resolution doesn't capture. But the correct resolution (`x→Var 1`) AND a capture
bug (`x→Var 0` = the shadowing `\A`) **both kernel-reject** (codomain mismatch
either way) — same verdict, vacuous. (The muddle that hid it: "the captured `x`
would still type-check as `x:Type` under the inner `A`" — false; capturing `x`
under `\A` makes it *be* the `A:Type` binder, which rejects.)

**Two fixes, both good, prefer the first or both:**
1. **Assert the structural output directly** (verdict-independent): pin the
   emitted core — e.g. the resolved de Bruijn index is `Var 1`, not `Var 0`. A
   structural assertion catches the bug regardless of the downstream verdict and
   tests exactly the property in question.
2. **Choose a genuinely discriminating program** where the verdict **flips**:
   `view f (A:Type)(x:A) : Type → A = \B.x` — correct (`x→Var 1`, body `x:A`
   meets codomain `A`) **accepts**; capture bug (`x→Var 0` = `B:Type`,
   `Type ≢ A`) **rejects**. Construction recipe: pair a shadowing binder of a
   **different type** than the intended one with a codomain the **intended**
   binder satisfies — then right vs. wrong land on opposite verdicts.

**Why:** "exercise every guard" (COORDINATION §7) is not enough — a guard that
*runs* but yields the same observable on correct and broken code is no guard.
Ask: "if the implementation had the exact bug this case targets, would THIS case
go green-vs-red, or green-vs-green?" Same as the over-equating shortcuts — spec
conv omega shortcut trap — but for *test discriminating power*, not the
algorithm: there the guard fired on the adjacent case; here the *test* fires the
same on the buggy case. Extends trust root test coverage discipline: a green
corpus is soundness/correctness evidence only if each case's verdict actually
depends on the property it claims to pin.

**How to apply:** for every negative (`reject`) conformance case and every "this
would be silently wrong" worked example, state the *correct* and the *buggy*
outcome explicitly and confirm they differ — verdict flip, or an asserted
structural output (emitted term, resolved index, normal form) that differs. If
they don't differ, the case is decorative; redesign it (recipe above) or add the
structural assertion. Especially load-bearing for **untrusted producers** (V0
elaborator): where the kernel backstops most bugs, the cases that matter are
precisely the few it *cannot* backstop (a mis-resolution that stays well-typed),
and those are the easiest to write vacuously.

**Complement — flip on a *hidden* axis (collapsed-dimension, K-api §5.1 erratum,
2026-06-30).** A case can flip **correctly on one axis** yet be **green↔green on
a *different* axis** — the one the bug lives on — when a **parameter fixed "for
simplicity" collapses that axis**. My K2c-s2 quotient-respect cases used a
**constant motive**, so the respect schema's `cast (M[y]) (M[x]) … (f y)`
collapsed by regularity *regardless of direction*: they flipped on
respect-validity but were **vacuous on the cast direction**, which is how a
reversed-direction spec bug survived authoring *and* review (I'd reconciled them
against the then-buggy spec and inherited the reverse — conformance reconcile
inherits spec metatheory bugs). Rule: **enumerate every axis the
schema/reduction depends on and ensure some case exercises each *without*
collapsing it** — a flip on axis A is no evidence about axis B. Tell: a fixed
parameter (constant motive, same level, equal indices, `refl` proof) is often
the one collapsing the bug's axis; vary it in ≥1 case. The cast-direction
instance (+ the `whnf`-on-inductive-Cast escape when `infer_cast` blocks
opaque-motive tests) is its own record — cast direction test at nondegenerate
endpoints; this is the general principle it specializes.

**Complement — illustrative *witnesses* in spec prose flip too (L1 §7 AC1,
2026-06-30).** The rule reaches past worked *traces* to any concrete **witness
value** placed in spec prose to illustrate a guard — a build team transcribes it
into a test, so a degenerate witness ships a green-vs-green case. My
`35-numbers` §7 AC1 ("integer arithmetic above 2⁵³ is exact, not f64-rounded")
used the witness `10²⁰` (`100000000000000000000`), which is **itself f64-exact**
(10²⁰ = 5²⁰·2²⁰, 5²⁰ < 2⁵³) — so a bare-value assertion is green-vs-green under
the very f64-carrier bug it illustrates. The *claim* was true; the *witness* was
non-discriminating. CV independently re-derived this and hardened the
conformance to **off-grid `10²⁰+1`** (nearest f64 rounds back to 10²⁰, ULP 2¹⁴),
which flips. **Tell I missed:** I checked the AC *claim* for truth but never ran
the *witness* to a flip — "is this number, given the exact bug, exactly
representable in the wrong rep too?" For a `> 2^k`-exactness claim, pick a
witness in the gap (`2^k + 1`-shaped, off the coarse grid), never a round power
whose factorization stays under the boundary. Spec-side mirror of the
conformance verdict-flip: same "run the example to a verdict before committing
it" check, applied to numeric illustrative values, not just `reject` cases.
(Promoted as the L1 carry.)

**Complement — non-observable properties (X1, 2026-06-29).** Some ACs target a
property that, by the language's semantics, is **not observable in the result
value at all** — so *no* value-verdict can flip, and a "result is correct" case
is automatically vacuous. Branch laziness / short-circuit in a **pure, total,
effects-out** core is the canonical case: `if true then x else Y` returns `x`
whether or not the evaluator wrongly forces `Y` (totality ⇒ `Y` can't diverge;
purity ⇒ `Y` has no effect to skip), so forcing the untaken arm wastes work but
changes nothing. Same shape for sharing/dedup (equal *value* either way; only
the **slot id** differs) and evaluation order. The rule: **assert a
STRUCTURAL/TRACE output** the bug perturbs — the untaken arm is *not interned* /
not in the eval trace; the dedup'd result is the *same slot id*, not merely `==`
— and **honestly state** why it isn't a value-flip *and* the exact condition
that would make it one (an effect, an opaque-non-total divergent branch). Don't
dress a non-observable property as a value assertion. This is the dual of the
flip rule: there, force the verdict to differ; here, the verdict *can't* differ,
so move the assertion to the structural layer where the bug *is* visible.
(Validated by spec-leader on X1 AC3; grounded in `42 §3.4`/`§3.6`/`§3.7`.)
