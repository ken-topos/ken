# WP L1 — numbers & primitive scalars (Int/Decimal/fixed-width +
obligation-overflow)

**Owner:** Team Language. **Branch:** `wp/L1-numbers` (cut from `origin/main`).
**Stream / gate:** L-stream (surface) → **G6**. **Depends on:** K1 (primitive
type machinery, `14 §5`) — **merged**; V2 (obligation generation) — **merged**
(the overflow obligations ride V2's VC path). **Spec source:**
`spec/30-surface/35-numbers.md` (+ `40-runtime/43 §2` partial-primitive runtime
face, `10-kernel/14 §5` kernel primitive view).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `35` to team-ready rigor +
> conformance before Team Language builds. **Perishable "current state" lines —
> verify against landed code/spec at pickup** (K2c-series-2). First L-stream WP;
> Language has built before (ITree-lowering), so not a new team, but a new
> surface area.

## 1. Objective (one line)

Deliver Ken's numeric tower: **arbitrary-precision `Int`** (fast-path) + native
**fixed-width** ints with **obligation-generating overflow**, core
**`Decimal`**,
honestly-named **`Float`/`Float32`**, `Bool`/`Char`, **typed literal
defaulting**,
and **explicit, visibly-lossy conversions** — numbers that mean what they say so
arithmetic specs hold without overflow side-conditions.

## 2. Settled inputs — FIXED, do not reopen

Decided in `35` (`OQ-int`, `OQ-1a` — both DECIDED, see `90-open-decisions.md`):

1. **`Int` = arbitrary-precision by default** (`OQ-int`). Small-int fast path
   (`i64`) + bignum on overflow (`41 §1`). Rationale: for a *verified* language
   silent overflow is a correctness hazard; `a+b` means addition, so `a+b==b+a`
   holds with no side-condition. **Not** fixed-64.
2. **Fixed-width overflow is OBLIGATION-GENERATING** (`OQ-1a` DECIDED, the
   headline). A bare `+`/`-`/`*` on `Int8…Int64`/`UInt8…UInt64` emits a
   **no-overflow proof obligation** (`43 §2` partial-primitive discipline) —
   discharged statically like any VC: **proven in-range ⇒ total/safe; unproven ⇒
   a marked partial point that degrades to a runtime check (panic/`unknown`)**.
   "Checked" is *subsumed* as the runtime face of an undischarged obligation,
   not
   a separate mode. **Wrapping is always explicit** (`wrapping_add`/`+%` or
   `Wrapping[T]`), `checked_add : … → Option` and `saturating_add` are named ops
   — it **never silently wraps**.
3. **`Decimal` is a core type** (base-10 exact, money), literal suffix `d`.
   **Not**
   an `f64` alias.
4. **`Float`/`Float32` = IEEE-754, honestly named.** **Not** the universal value
   carrier, **not** the default for integer literals; `Float ==` is the IEEE
   minefield and Ken does not pretend otherwise (proofs over reals use
   `Decimal`/rationals/error-bounds, not `Float ==`).
5. **Typed literal defaulting** (`31 §3`): integer literal → `Int`,
   decimal-point
   → `Float`, `d`-suffix → `Decimal`. `2` and `2.0` are **different types**, not
   one `f64`. Literals are polymorphic over the numeric classes
   (`Num`/`Integral`/`Fractional`), elaborating to `fromInteger`/`fromDecimal`
   resolved by instance search (`33 §5`, `39`).
6. **No implicit coercions.** Widening/narrowing are **explicit** total/partial
   named functions; lossy ones are visible in the type (`Option` for may-fail,
   documented-lossy for `toFloat`).
7. **Kernel view (`14 §5`):** all are **primitive types** — opaque type
   constants
   with **registered, audited reductions** for the ops (so `2+3 ≡ 5 : Int`
   computes in the kernel); non-definitional laws (commutativity, ring axioms)
   are
   **propositions** in the prelude (proved against a reference model or
   axiomatized as a small visible interface), **not** new kernel rules.

## 3. Mandated deliverable outline (each item ends in an implementable choice)

Deliver in the surface/elaborator + prelude (per the spec's lowering to `41`):

1. **The type set + lowerings.** `Int` (fast-path `i64` + bignum), `Int8…64` /
   `UInt8…64` → machine widths, `Decimal` (pin the struct, e.g. `{i128 coeff,
   i32 exp}`), `Float`/`Float32`, `Bool`, `Char`. Pin each lowering to `41`.
2. **The overflow obligation (the differentiator).** Pin exactly the VC a bare
   fixed-width `+`/`-`/`*` emits (the no-overflow predicate over the operands'
   ranges), how it routes to V2's obligation machinery, and the **degrade-to-
   runtime-check** behavior when unproven. The wrapping/checked/saturating named
   ops are **separate, explicit** — never the default. **Exhaustive-by-
   construction:** the dispatch over arithmetic-op classes (obligation /
   wrapping / checked / saturating) is a sealed match, **no silent-wrap
   default**.
3. **Literal defaulting + numeric classes.** The defaulting rules (§2.5) + the
   `Num`/`Integral`/`Fractional` numeric-class hierarchy for overloading.
   **Coupling — flag, do not silently resolve:** full
   user-type-joins-by-instance
   overloading uses the typeclass instance-search of **L-classes** (`33 §5`,
   `39`);
   the **built-in** numeric defaulting (`2:Int`, `2.0:Float`, `2.0d:Decimal`)
   must
   work **without** waiting on L-classes (a fixed default table). Pin the
   boundary:
   built-ins now; user-numeric-type instancing gated on L-classes (sequence
   note).
4. **Conversions.** The explicit total/partial conversion functions
   (`Int.toInt64
   : Int → Option Int64`, `Int64.toInt : → Int` total, `*.toFloat` documented-
   lossy). No implicit coercion anywhere.
5. **Prelude laws.** The ring/commutativity/associativity laws as
   **propositions**
   (`14 §5`), with the reference-model-or-axiomatized interface kept small +
   visible (TCB discipline — these are not kernel reductions).

## 4. Testable acceptance criteria

- **AC1 (Int exactness)** Integer arithmetic **above 2⁵³ is exact**
  (`100000000000
  000000000 : Int` is not `f64`-rounded) — a structural value assertion, not a
  type check.
- **AC2 (literal types distinct)** `2 : Int` and `2.0 : Float` are **distinct
  types**; `2.0d : Decimal`. A program relying on `2 ≡ 2.0` is rejected (verdict
  flips: well-typed vs ill-typed).
- **AC3 (overflow obligation)** A bare `a + b : Int32` **emits a no-overflow
  obligation**: provably-in-range accepts as total; un-provable **degrades to a
  runtime-checked partial point** (assert the obligation is *emitted* —
  structural,
  per the untrusted-layer lesson — not just "compiles").
- **AC4 (no silent wrap)** Fixed-width overflow **never silently wraps** — the
  only
  wrapping path is the explicit `wrapping_add`/`+%`/`Wrapping[T]`, which is
  provably-modular. A test that overflows without the explicit op must NOT
  produce
  a wrapped value silently (it obligation-checks / panics).
- **AC5 (no implicit coercion)** `Int + Int64` without an explicit conversion is
  a
  type error (reject); the explicit `Int.toInt64` returns `Option`.
- **AC6 (Decimal exact)** `0.1d + 0.2d == 0.3d` (exact base-10), while the
  `Float`
  analog is honestly **not** asserted equal.
- **Conformance:** `conformance/surface/numbers/` — the `> 2^53` exactness
  regression + literal-defaulting + the overflow-obligation cases; per-case
  verdict/structural-flip + cross-case sweep (the overflow-obligation class
  agrees;
  the no-implicit-coercion reject class agrees). **QA gate (2-team build-qa
  lesson):** the overflow-obligation cases must **route through the actual VC
  emission** (observe the emitted obligation / the degrade-to-check), not
  *predicate
  about* a synthetic flag — drive a real fixed-width `+` and observe the
  obligation.

## 5. Do-not-reopen guardrails

- **`Int` is arbitrary-precision** — not fixed-64 (§2.1).
- **Overflow is obligation-generating, never silent-wrap** (§2.2) — wrapping is
  always explicit. This is the verification-language differentiator; do not
  "simplify" it to checked-by-default-as-a-mode.
- **`Float` is not the default and not the universal carrier** (§2.4).
- **No implicit numeric coercions** (§2.6).
- **No new kernel rules** — numerics are primitive types with registered
  reductions; laws are prelude propositions (§2.7). The registered reductions
  are
  **trusted kernel primitives** (audited interface) — conformance nets that the
  reductions match the reference model.

## 6. Sequencing notes

- L1 is **first in the L-stream queue**; **L2** (sum/match, needs L1) follows on
  Team Language. **Foundation** runs a **parallel** L-stream leaf (L6 Bytes/IO
  or
  L4) — coordinate shared surface conventions via the spec, not cross-branch.
- **L-classes coupling (§3.3):** built-in numeric defaulting ships in L1; user
  numeric-type instancing depends on L-classes (`33 §5`/`39`) — if L-classes is
  not yet sequenced, the enclave pins L1's built-in default table as the
  standalone deliverable and marks user-instancing as the L-classes follow-on.
- Standard §2c: frame → spec-leader elaborates `35` + conformance → merge
  (Architect + Spec) → Team Language compacted, then kicked off on this branch.
