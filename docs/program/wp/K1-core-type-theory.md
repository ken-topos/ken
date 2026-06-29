# K1 — Core dependent type theory (kernel trust root) — Steward frame

> **Owner:** Team Kernel · **Size:** L · **Risk:** ★★★ (the trust root — its
> soundness is the soundness of every Ken program) · **Branch:**
> `wp/K1-core-type-theory` · **Feeds:** K2 (observational layer), K3 (value
> model), V0 (elaborator), X1 (interpreter) → **G1 vertical slice**.
>
> **Pipeline status: Steward frame → _awaiting spec-leader elaboration_ → Team
> Kernel.** This is the Steward's frame (scope, settled-decision pinning,
> deliverable outline, acceptance, guardrails). Per the WP release process
> (steward playbook §2c) the **spec-leader elaborates it** — bringing
> `spec/10-kernel/11–14` + `conformance/kernel/` to full team-ready rigor on this
> branch — **before** Team Kernel is released. Kernel must **not** start until the
> elaborated package is on `main` and the WP is kicked off.

## 0. Scope — K1 is the CORE calculus only (spec 11–14)

K1 is **`spec/10-kernel/11-syntax`, `12-universes`, `13-pi-sigma`,
`14-inductive`** — the well-understood MLTT-style core. It is the foundation the
observational layer sits on, **not** the observational layer itself. Deliver:

1. **Core syntax + representation (`11`).** The core-term type (fully-explicit,
   no implicits/sugar); **de Bruijn indices** as the reference representation
   (impl may differ if the observable judgments are identical); universe-level
   syntax (`0 | suc | max | lvar`); contexts as a term-variable telescope; the
   **global environment** Σ (transparent def, opaque constant/postulate,
   inductive decl, primitive decl); **capture-avoiding substitution + weakening**;
   raw well-formedness (every index resolves) as the precondition to typing.
2. **Universes (`12`).** The **predicative, non-cumulative, _checked_** `Type ℓ`
   hierarchy with level formation/comparison; the universe typing/formation
   rules; **no `Type : Type`** (the paradox is ill-typed). *(The strict-prop
   universe **Ω** and its definitional proof-irrelevance + Heyting logic are
   **K2** — K1 may reserve the `Ω` former in the grammar but does not implement
   its proof-irrelevance/logic.)*
3. **Π and Σ (`13`).** Dependent functions `(x:A)→B` with intro `λ`, application,
   **β and η**; **genuinely dependent** pairs `(x:A)×B` (`B` may mention `x`) with
   `(t,u)`, `.1`/`.2`, and **η**. Formation/intro/elim/computation typing rules.
4. **Inductive families (`14`).** Inductive declarations, constructors, the
   **dependent eliminator**, **strict-positivity** admission check, and
   **ι-reduction** (eliminator on a constructor). Worked set: `Nat`, `Bool`,
   `List`, `Vec` (a dependent family), plus user inductives.
5. **Just enough typing + conversion to make 1–4 checkable + testable.**
   Syntax-directed `check`/`infer` for the formers above, and the **basic
   structural conversion** (α via de Bruijn, β/η/ι, and δ-unfolding of
   transparent defs) needed to type them. The **full decidable conversion**
   (lazy-WHNF NbE + Ω proof-irrelevance + the `Eq`/`cast` equations + **SCT**
   termination gating δ) is **K2c (`17`)** — K1 builds only the conversion its
   own rules require, structured so K2c can extend it.

Deliverable: the **`kernel` crate core** (`spec/10-kernel/11–14`) + property
tests. Primitives/literals (`Int`, `Bytes`, …) enter as **global declarations**,
not core term formers (`11 §1`) — keep the grammar closed and small.

## 1. Settled decisions — FIXED inputs (do not reopen)

Cite these; the spec-leader elaborates *within* them, the implementer builds *to*
them. None is open.

| Decision | Resolution (source) |
|---|---|
| Equality discipline | **Observational (OTT), NOT cubical** — no interval, `Glue`, univalence, or HITs. K1 must introduce **no cubical/dimension machinery** (the context is term-variables only). (ADR 0005; `11 §2–3`, README §3) |
| Universes | **Predicative, non-cumulative, checked**; `Type 0 : Type 1 : …`; **no `Type:Type`**. Ergonomics (cumulativity feel) live in the elaborator, not the kernel. (`OQ-2` decided; `12`) |
| Σ | **Genuinely dependent** (`B` may mention `x`); projections + η. (README §6 — a by-construction property that MUST NOT be compromised) |
| Level | **Set-level** (UIP/funext/propext are definitional — but those are the **K2** observational story; K1 is the set-level core they extend). (README §3) |
| Representation | **de Bruijn indices** are the reference; α-equivalence is syntactic identity; substitution is capture-avoiding. (`11 §2`) |
| Environment | **Append-only, acyclic**; δ-unfolding well-founded; transparent vs opaque (postulate) constants. (`11 §4`) |
| Trust boundary | The kernel is **small, permanent, Rust**, and only ever sees **fully-explicit core terms** (the **de Bruijn criterion**) — elaboration, proof search, diagnostics live **outside** it. (README §2) |

**Three properties guaranteed _by construction_ (README §6) — MUST NOT be
compromised:** universes are checked, Σ is genuinely dependent, and the core is
built so `J`-reduces-on-non-`refl` is achievable in K2 (K1 must not foreclose
it).

## 2. Acceptance criteria (testable — the K1-scope soundness commitments)

Tie tests to `spec/10-kernel/README §5` at K1's scope; corpus lands in
`conformance/kernel/`.

1. **No `Type : Type`.** A universe-level loop is **rejected**; the classic
   paradox is ill-typed. (README §5.1)
2. **Genuinely dependent Σ type-checks** — a Σ whose second component's type
   mentions the first (e.g. `(n : Nat) × Vec A n`) checks; projections and η hold.
3. **Π β and η** hold; Σ projection-η holds.
4. **Inductive eliminator reduces (ι)** over a constructor; a **dependent**
   eliminator checks (e.g. `Vec` length-indexed elimination).
5. **Strict positivity** is enforced: a positive inductive is admitted; a
   **negative** one (e.g. `data Bad = mk (Bad → Bad)`) is **rejected** at
   admission.
6. **Subject reduction** holds across β/η/ι/δ on the K1 fragment (property test:
   if `Γ ⊢ t : A` and `t ↝ t'` then `Γ ⊢ t' : A`).
7. **Decidable checking on the K1 fragment** — `check`/`infer` terminate (the
   *full* SCT termination argument is K2c; K1's conversion must at least not loop
   on its own rules).
8. Conformance `kernel/` K1 subset passes; lint/CI green; 80-col wrap; build/test
   only via `scripts/ken-cargo -p kernel` (COORDINATION §12; the build-lock wedge
   is fixed as of `main` `4566211`).

## 3. Guardrails — out of K1 scope (do NOT build here)

- **The observational layer** — `Eq`-by-type, `cast`, derived `J`, strict-prop
  **Ω** proof-irrelevance + Heyting logic, set-quotients `A/R`, propositional
  truncation `‖A‖`. That is **K2** (`15`,`16`). K1 reserves their grammar
  (`11 §1`) but implements **none** of their typing/computation.
- **Full decidable conversion** — lazy-WHNF NbE, the `Eq`/`cast` conversion
  equations, **SCT** termination gating δ. That is **K2c** (`17`). Build only the
  basic β/η/ι/δ conversion K1's own rules need, *structured for K2c to extend*.
- **The stable kernel API surface** (`check`/`infer`/`convert`/`whnf` as the
  audited TCB boundary) — that is **K-api** (`18`). K1 may expose provisional
  internal entry points; the *stable* API is later.
- **Elaboration, proof search, diagnostics, the value model** — outside the
  kernel entirely (K3 is the value model; V0 the elaborator).

If a settled decision looks wrong-for-implementation, that is a **`question`** to
the Spec leader (behavioral) or **Architect** (the K1/K2/K2c/K-api split is a
component-design boundary) per COORDINATION §9 — never a unilateral redesign.

## 4. Logistics

- **Deps:** F2 (spec), F3 (ADRs) — both merged. **Source:** `spec/10-kernel/11–14`
  (+ README) — build from `/spec` only; **never** read `local/refs/` or the
  prototype (CLEAN-ROOM.md). **Branch:** `wp/K1-core-type-theory` off
  `origin/main`. **Size L** — the spec-leader/Kernel may split into a short
  series (e.g. 11+12 syntax+universes, then 13 Π/Σ, then 14 inductives) per
  04-git §2; each merges on its own, `main` green.
- **Critical path.** K1 gates K2 → K2c → K-api and feeds X1/V0 → **G1**. It is the
  highest-rigor WP; front-loaded deliberately (DAG §sequencing).
- **Done:** acceptance §2 met + retros in (COORDINATION §10). Merge Decision
  mentions **Architect** (always) + **Spec** (it touches `/spec`+`/conformance`).
