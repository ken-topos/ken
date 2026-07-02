# WP conversions — the `IntN↔Int` floor + checked/saturating DEMOTE

**Phase-2 tranche #4 (BUILTINS PROVIDE). Team: Runtime. Base: `origin/main`.**
Steward frame. Cite ADR 0009 (capability-supply: provide the missing native
reductions; bracket-the-untrusted). Posture is F1/F2/F3/Decimal-Char: native
reductions live in the **tested-not-trusted `ken-interp` ring**, netted by an
independent oracle — **not** a kernel change and **not** a K3 trusted-base
promotion. **Kernel is untouched.**

## Objective

Build the last numeric BUILTINS unit: the **`IntN↔Int` conversion floor** (the
final GAP row of `spec/10-kernel/18a-primitive-registry.md` §5 / §5.7), and use
it to **DEMOTE** the derived overflow-total ops out of the native path. Two
gains in one WP: the floor is the surface Rosetta explicit-conversion examples
need, and it *shrinks* the conceptual native op set (`checked_*`/`saturating_*`
leave it).

The floor has three native ops and unlocks two derived families:

- **`IntN.toInt : IntN → Int`** — **widening, TOTAL.** Every fixed-width value
  fits the arbitrary-precision `Int` (F1). No partiality, no obligation.
- **`Int.toIntN : Int → Option IntN`** — **narrowing, PARTIAL, face (c).** The
  partiality lives in the **return type** (`None` out of `[T_MIN, T_MAX]`), with
  **zero reliance on a trusted backstop** (`18a §2` face-(c), the
  default-preferred posture — no obligation to discharge, no degrade path). A
  narrowing that returns a **silently truncated value** is **blocked**.
- **`neg_intN : IntN → IntN`** — **stays NATIVE, checked-not-wrap.** Does **not**
  demote: `neg(MIN_intN)` overflows the fixed range (`18a §5`, ~L256), so it
  degrades like an F2 bare op (stuck `Neutral`/panic on `MIN`), never wraps.

Unlocked derived (leave the TCB — `N/A` oracle, they are checked Ken not trusted
reductions):

- **`checked_add/sub/mul_intN : T → T → Option T`** ≙
  `Int.toIntN (op_int (IntN.toInt a) (IntN.toInt b))` — the narrowing `None`
  **IS** the overflow semantics (`18a §5`, ~L439). One `add` + two conversions,
  constant-factor, no cliff.
- **`saturating_add/sub/mul_intN : T → T → T`** ≙ widen → clamp-compare
  (`leq_int` against `T_MIN`/`T_MAX`) → narrow (`18a §5`, ~L440). Total by clamp.

## Code sites (verify on `origin/main` — grep to confirm, lines drift)

- **Reducer:** `crates/ken-interp/src/eval.rs`, `prim_reduce` — the native arms
  live alongside the existing `add_int`/`eq_int`/`leq_int`/`fixed_binop_*` arms
  (F1/F2/F5 landed here). Add the `IntN.toInt` / `Int.toIntN` / `neg_intN` arms;
  the `Option` result uses the surface `Option`/`Some`/`None` constructors the
  elaborator already knows (grep how `div`/`Option`-returning shapes are built,
  or the demote's `checked_*` will need them too).
- **Registration:** `crates/ken-elaborator/src/numbers.rs` (and/or
  `prelude.rs`) — `declare_primitive`/`reg_*` the floor ops with the exact
  registry signatures. `checked_*`/`saturating_*` become **derived defs** (Ken
  surface, not `declare_primitive`) — the demote is *removing* any native arm
  and providing the derivation.
- **Enumeration is the registry's, not this frame's:** build **exactly** the
  `IntN↔Int` set the registry §5/§5.7 defines (signed + unsigned, N∈8/16/32/64;
  `Int.toInt32 : Int → Option Int32` is the shape, `18a` ~L64). The implementer
  enumerates against the registry and states the count built.

## Hard ACs (each a gate)

1. **(soundness — the defining-law oracle)** The floor satisfies the
   **widening round-trip `narrow ∘ widen = id`**: for every in-range `x : IntN`,
   `Int.toIntN (IntN.toInt x) ≡ Some x`. This is the **preferred non-circular
   oracle** (`18a §3`: a defining law beats a reference implementation — it is
   checked against the op's output algebra, so it cannot alias the reduction it
   audits). Wire it as an executable conformance case spanning every built width
   and both signednesses.
2. **(soundness — the boundary, where a silent truncation hides)** `Int.toIntN`
   returns **`None` exactly out of range** and `Some` exactly in range, tested at
   the **exact edges**: `T_MAX → Some`, `T_MAX+1 → None`, `T_MIN → Some`,
   `T_MIN−1 → None` (`18a §2`: an oracle sampling only the interior cannot catch
   a silent out-of-domain result). A narrowing that yields a wrapped/truncated
   **value** instead of `None` at `T_MAX+1` fails this — that is the whole point.
3. **(discriminating — the existing seed oracle, verdict must FLIP)** Wire
   `seed-numbers.md` **AC5**: `explicit-conversion-is-partial-option`
   (`(x:Int).toInt64` **accepts** with type `Option Int64`) **paired against**
   `no-implicit-cross-type-coercion` (`(x:Int) + (y:Int64)` with no conversion
   **rejects** — type error, no widening coercion). The pair must flip:
   explicit-conversion accepts WHILE implicit-mixed-op rejects. Authored
   independently of this build (expected verdicts come from `35 §5`, not the
   impl) — not green-vs-green.
4. **(demote is behavior-preserving)** The derived `checked_*`/`saturating_*`
   reduce to the **same values** their native/spec semantics require, verified at
   the **overflow boundary** (not just the interior): `checked_add_intN a b`
   ≡ `None` iff `a+b` overflows `IntN` (else `Some (a+b)`); `saturating_add_intN`
   clamps to `T_MAX`/`T_MIN` at the boundary. A demote that diverges from the
   native semantics at the edge fails.
5. **(neg stays native + checked)** `neg_intN` degrades (stuck `Neutral`/panic,
   **not** a wrapped value) on `neg(MIN_intN)` — an F2-consistent case pins it;
   it is **not** demoted to a derivation.
6. **(soundness, whole-WP)** **Kernel diff empty** — `git diff --stat
   crates/ken-kernel/` returns nothing. `trusted_base()` unchanged: the floor
   ops are native interp-ring reductions the kernel leaves **stuck**
   (`PrimReduction::Op`, tested-not-trusted, netted by AC1/AC2, awaiting any
   future K3 promotion — none here); the demote *removes* native arms. **Same
   posture as F1/F2/F3.** **Workspace-green landing** (K7 discipline: QA re-runs
   `./scripts/ken-cargo test --workspace` independently, not implementer-trusted).

## Oracle discipline (`18a §3`)

The **round-trip law** (AC1) is the primary net — non-circular by construction.
The **narrowing edge** (AC2) is the boundary that a total-interior oracle misses.
The demote ops (`checked_*`/`saturating_*`) are `DEMOTE→derived`, so oracle-ref
is **`N/A`** — they are checked Ken, not trusted reductions; AC4 nets them
against the native *semantics they replace*, not against themselves.

## Out of scope / defer (verify by absence)

- **`Int.toFloat` / `Float.toDecimal` / the Float conversion arms** — Float
  arithmetic is its own tranche; this WP is the **`IntN↔Int`** floor only.
  (AC5's seed case names `Int.toFloat` as documented-lossy context; do **not**
  build the Float floor here — only `IntN↔Int` is in scope.)
- **`div_int`/`mod_int` runtime-obligation face** — separate GAP row, later.
- **`class Num` / polymorphic `fromInteger` defaulting** — gated on L-classes
  (`35 §4.2`), not this WP.
- **K3 trusted-base promotion** of any floor op — future; this lands
  tested-not-trusted like its siblings.

## Flow (thin — COORDINATION §9)

`runtime-leader → runtime-implementer → runtime-qa → Architect (soundness) + CV
(conformance) → Integrator`. One pass each. Crates-only diff (no `spec/` or
`conformance/` *spec* change — the seed cases already exist), so the review gate
is **Architect + CV-conformance + CI**. A mid-WP soundness fork → Architect; a
conformance fork → CV; a scope/lane fork → Steward. No new parties, no verbatim
relays, no cc-the-room. Thread the whole ring under the Steward's kickoff event
for **this** WP (COORDINATION §4 — do not reuse the F2/F3 thread).
