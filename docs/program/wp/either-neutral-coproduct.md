# Either / Coproduct · straighten the coproduct family (operator-ruled)

**Owned by the Steward** (frame); **home: Runtime** (owns the effect-composition
machinery being renamed; already spun up). **Operator-ruled** (2026-07-10, Pat),
superseding the first cut of this WP (which wrongly proposed renaming the single
coproduct to `Either` everywhere). Logged as judgment call **L5**. A **trust-root
prelude declared-type change** — full ring + Architect gate, logged prominently.

## The decision (operator — COEXIST by role, three distinct coproducts)

`Either`/`Left`/`Right` (value disjunction) and the internal effect coproduct
`Sum` (effect-signature composition) are **structurally identical in Ken** (both
Type0 coproducts) but serve **different reader-roles** — the same situation that
keeps `Result` distinct from `Either`. So name each for its role, don't unify:

| Type | Role | Constructors | Status |
|---|---|---|---|
| `Result e a` | fallible computation (error) | `Err` / `Ok` | landed (prelude) |
| `Either a b` | **user-facing value disjunction** | `Left` / `Right` | **ADD (this WP)** |
| `Coproduct a b` | effect-signature coproduct (`⊕`, internal) | `InL` / `InR` | **RENAME from `Sum` (this WP)** |
| `Sum f g` | higher-kinded **functor** coproduct (`Data.Functor.Sum`) | — | **RESERVED, named future** |

**Why `Coproduct` (operator's pick — the precise term):** the internal `Sum` is
*not* a functor coproduct (its params are `Type0 × Type0`, a value-level coproduct
of two operation-*types*), yet it squats on `Sum` — the canonical name the *real*
higher-kinded functor coproduct (`Data.Functor.Sum f g`, the natural sibling of
DS-8's `Compose`) will want if it lands. It also **leaks** into raw `ITree (Sum …)`
effect signatures. So rename it to the accurate term `Coproduct` (zero arithmetic
pun; honest CT vocabulary its effect-internals audience reads fine), freeing `Sum`.

## Scope (this WP — three coupled pieces, one owner to avoid a `prelude.rs` clash)

1. **Add `data Either a b = Left a | Right b`** to the prelude — a plain surface
   `data` next to `Option`/`Result` (`elaborate_decl`, id-recovery via
   `lookup("Either"/"Left"/"Right")`, mirroring `Result` at `prelude.rs:193/:241`).
   **Bare type only** — combinators (`either`/`mapLeft`/`mapRight`) are a named
   Foundation follow-on, NOT this WP.
2. **Rename the effect coproduct `Sum` → `Coproduct`** — **type name only; KEEP
   `InL`/`InR`** as its constructors. This is the deliberate risk reduction: the
   `eval.rs` D3.2 peel logic strips `InL`/`InR` (unchanged), so it needs at most a
   `Sum`→`Coproduct` **comment** update, no logic change. Sites: `effects::state::
   declare_sum` (→ `declare_coproduct`), `prelude.rs` global registration
   (`:271-273`, the `Sum` id → `Coproduct`), the `Sum a b` doc block (`:157-176`),
   `injectL`/`injectR` (`Term::indformer(sum_id)`), `resp_sum` + its consumers
   (`SumIds`/`run_io`/`PreludeEnv`/`declare_resp_sum`), and the effect-composition
   tests. `resp_sum` → `resp_coproduct` (Architect leaned rename for coherence —
   his call at the gate; ground any spec-D2/D3.2 `resp_sum` reference first).
3. **Spec reconcile** — the L4 subsume erratum (**landed** `main @ dcc34ed`, PR
   #446) is **superseded** and must be reversed *coherently*, not find-replaced
   (Architect catch `evt_60ahxgw3vpnqn`):
   - `spec/30-surface/34-data-match.md`: **REWRITE** the subsume note → `Either a b
     = Left a | Right b` is a **distinct declared** value coproduct (`Left`/
     `Right`); `Result` a distinct error sum (`Err`/`Ok`); no "subsumed"/"no
     first-party Either" language survives. **RESTORE `Either`** at the three
     list-sites the erratum dropped (`README.md:42`, `34-data-match.md:5`, `:633`)
     — under this WP `Either` IS declared. `:56` "ordinary prelude `data` decls":
     restore `Either` to that true set (it lands as a surface `data`).
   - `spec/30-surface/36-effects.md`: rename `Sum`→`Coproduct` (+ `resp_sum` if
     renamed) in the effect-composition text.

## Open technical sub-question (Architect / Runtime — NOT pre-decided)

`Coproduct` (the renamed effect type) is non-dependent Type0 — probe whether it
should stay **hand-built** in `effects::state` (alongside `ITree`) or migrate to a
plain surface `data Coproduct a b = InL a | InR b` in the prelude (params default
`Type0`, per the DS-8 finding — an exact match; reflect-don't-extend favors one
fewer bespoke inductive-construction site). Ground the id-consumer re-plumb before
committing; if a real constraint forces hand-built, keep it. Either way the name
lands `Coproduct`/`InL`/`InR`.

## Boundary / constraints

- **Pure rename + one trivial `data` addition; zero semantic change.** `Coproduct`
  is `Sum` renamed (structure identical); `Either` is a fresh independent type.
  **Zero `crates/ken-kernel` delta** (both stay `declare_inductive`, kernel-
  rechecked). **No back-compat alias** (no users). No `Either`/`Left`/`Right` /
  `Coproduct` name collision (grep-confirmed absent).
- **`Sum` must be fully vacated** — a residual `git grep -nw 'Sum'` over `crates/`
  + `spec/` shows no *type* `Sum` survivor (scope out unrelated hits — numeric
  addition, `resp_sum` if kept — and note what/why). `Sum` is then free for the
  future `Data.Functor.Sum`.
- **AC:** effect-composition suite green pre/post (zero regression — the peel logic
  is untouched, so this should be clean); a new value-level acceptance constructs +
  matches `Either Int String` (`Left`/`Right`) AND confirms `Coproduct`/`InL`/`InR`
  still drives effect composition; the reconciled spec note is coherent (Architect
  verifies, not a swap). Zero new `Axiom`/`postulate`.

## Named follow-ons (NOT this WP)

- **`Either` value combinators** (`either` eliminator, `mapLeft`/`mapRight`, `swap`)
  — Foundation, DS-3-adjacent (or a small DS-3b).
- **`Data.Functor.Sum f g`** — the higher-kinded functor coproduct, sibling to
  DS-8's `Compose`, owning the freed `Sum` name — a functor-combinator WP when the
  catalog grows that family.

## Gate

Ring: Runtime build → runtime-qa independent re-derivation (full effect-composition
suite + the `Either`/`Coproduct` value acceptance) → **@architect gate** (fidelity:
three roles named correctly, `Result` untouched, note coherently rewritten +
`Either` restored; soundness: zero-kernel-crate-delta, both inductives kernel-
rechecked, zero new `Axiom`; + the hand-built-vs-surface + `resp_sum` sub-questions)
→ **Spec vote** if the diff-scope check flags normative `spec/` (standing K3/V0
rule — it does) → `git_request` to Steward. **CI-gated** (`crates/` + `spec/`).
Own retro; flag every judgment call for the operator's log (L5).
