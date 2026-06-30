# WP L2 — sum types, `match`, exhaustiveness, refinements

**Owner:** Team Language. **Branch:** `wp/L2-data-match` (cut from
`origin/main`).
**Stream / gate:** L-stream → **G6**; **unblocks B2** (`Temporal` datatype) and
**T3** (test framework). **Depends on:** L1 (numbers) — **merged**; the kernel
inductive machinery (`14`, K1/K1.5) — **merged**. **Spec source:**
`spec/30-surface/34-data-match.md` (+ `14` inductives, `32 §4` patterns, `39`
elaboration, `22 §3/§4` match-as-hypothesis).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `34` to team-ready rigor +
> conformance before Team Language builds. **Perishable:** `data`/`match` lower
> to
> the **landed** kernel inductive + eliminator machinery (K1, and K1.5's W-style
> Π-bound admission — `check_no_pi_bound_recursive` is *retired*); pin against
> the
> landed `14`/kernel, not this line.

## 1. Objective (one line)

Deliver Ken's algebraic-data surface: `data` declarations (real inductive types
with real eliminators, including indexed/GADT-like families), `match` (compiled
to the dependent eliminator with per-branch type refinement), **required
exhaustiveness + reachability checking**, and refinement types `{x:A | φ}` — all
honest (no stubs, no sentinels), no new kernel rule.

## 2. Settled inputs — FIXED, do not reopen

Per `34`:

1. **`data` = a REAL inductive type (`14`, K1) — not stubbed.** It declares the
   type former, its **constructors** (real introduction forms), and the
   **generated dependent eliminator `elim_D`** (real elimination with
   computation:
   `elim_D … (Some x) ≡ …`). Constructors may be recursive **subject to strict
   positivity** (`14 §2`). **No new kernel rule** — this is surface syntax over
   the
   landed inductive machinery.
2. **Indexed families (GADT-like) are in scope** — constructors targeting
   different **indices** (`14 §1`), e.g. `Vec a : Nat → Type`; non-emptiness
   lives
   in the type (a `head` on `Vec a (n+1)` can't be applied to `VNil`).
3. **`match` ELABORATES to `elim_D`** (`14 §3`, `39`) — nested matches → nested
   eliminators; the **dependent motive is recovered** so a branch refines the
   result type (essential for indexed families + the body-as-motive obligations,
   `22 §4`). In each branch the **scrutinee is definitionally the matched
   constructor** (`s ≡ Circle r`), which the verification layer turns into a
   **hypothesis** (`22 §3`). `match` is **not** a new kernel primitive.
4. **Exhaustiveness + reachability are REQUIRED, from day one (`34 §4`).**
   **Exhaustiveness:** the arms cover every (type-possible) constructor — a
   missing
   case is a **compile error** with the **unmatched pattern reported** (indexed
   families: only the *type-possible* constructors at the index must be covered
   —
   an impossible case need not be written). **Reachability:** a redundant arm is
   a
   warning/error. Exhaustive `match` over closed `data` needs **no default** and
   the compiler **proves totality** of the case analysis.
5. **`Result`/`Option`/`Either` are ordinary prelude `data` decls** —
   fallibility
   and absence are **honest sum types, not sentinel values**.
6. **Refinement types `{x:A | φ}`** (`12 §5`, `21 §2`) at the surface — `type
   Nat =
   {n:Int | n ≥ 0}`, `NonEmpty a = {xs : List a | xs ≠ Nil}`;
   obligation-emitting
   (the same machinery L6 cited for `at_pf`).

## 3. Mandated deliverable outline (each ends in an implementable choice)

Deliver in `ken-elaborator` (+ prelude `data` decls), lowering to the landed
kernel:

1. **`data` declaration elaboration.** Parse + elaborate `data D … = C₁ … | Cₙ
   …`
   (positional + named-record constructor args, `32 §1`) to a **landed kernel
   inductive** + its generated `elim_D`; enforce **strict positivity** (`14 §2`)
   — pin what the elaborator builds vs. what the kernel admits (K1/K1.5).
2. **Indexed families.** The explicit-index form (`data Vec a : Nat → Type
   {…}`);
   constructors at distinct indices; impossible cases pruned by the index.
3. **`match` → `elim_D` compilation.** Pattern arms (`32 §4`: constructors,
   variables, wildcards, literals, tuples, records, as-/or-patterns, guards) →
   (nested) eliminator(s) with **dependent-motive recovery** + the per-branch
   definitional refinement (`22 §3`).
4. **The exhaustiveness + reachability checker (the headline).** A surface
   algorithm (NOT a kernel rule): exhaustive-or-**compile-error-with-unmatched-
   pattern**; redundant-arm-**warning/error**; index-aware (only type-possible
   constructors). State the algorithm + the error surface.
5. **Refinement types `{x:A | φ}`** — surface syntax + obligation emission (`34
   §5`,
   `21 §2`); reconcile with the landed refinement machinery.

## 4. Testable acceptance criteria

- **AC1 (`data` is real)** A `data` decl elaborates to a kernel inductive whose
  **eliminator computes** — `elim`/`match` on `Some x` reduces to the `Some`
  branch (structural reduction, not "compiles").
- **AC2 (`match` → elim)** `match s { Circle r => … }` elaborates to `elim_D`
  and
  computes on a constructor; nested match → nested elim.
- **AC3 (exhaustiveness — the headline, discriminating)** A **non-exhaustive**
  `match` over a closed `data` is a **compile error** naming the **unmatched
  pattern**; the exhaustive version **accepts**. Verdict **flips** (exhaustive
  accept / missing-case reject) — not green-vs-green.
- **AC4 (reachability)** A **redundant** arm is flagged (warning/error);
  all-reach-
  able accepts. Verdict flips.
- **AC5 (indexed family)** `head` on `Vec a (n+1)` **cannot** be applied to
  `VNil`
  (a type error), and a `match` on a `Vec a (n+1)` **need not** write the `VNil`
  case (impossible-at-index) — both the rejection and the allowed-omission are
  asserted.
- **AC6 (branch refinement = hypothesis)** Inside a `Circle r` branch the
  scrutinee
  is **definitionally** `Circle r` — usable by the verification layer as a
  hypothesis (`22 §3`); assert the per-branch result-type refinement (a
  dependent
  motive), not just a value.
- **AC7 (refinement type)** `{n:Int | n ≥ 0}` elaborates + **emits the
  obligation**
  on construction/coercion (structural — observe the emitted VC, per the
  untrusted-layer lesson), no implicit subset coercion past `φ`.
- **Conformance:** `conformance/surface/data-match/` — AC1–AC7, per-case
  verdict/structural-flip + cross-case sweep (the exhaustiveness class agrees).
  **QA gate:** exhaustiveness/reachability cases route **real** elaboration (a
  real
  missing-case → real compile error naming the pattern), and the eliminator
  **really reduces** — never a synthetic flag.

## 5. Do-not-reopen guardrails

- **`data` is a real inductive** (`14`/K1) — no stub; **strict positivity**
  enforced; **no new kernel rule** (surface over the landed machinery).
- **`match` elaborates to `elim_D`** — not a new primitive; dependent motive
  recovered (§2.3).
- **Exhaustiveness + reachability are REQUIRED** — missing case = compile error;
  the compiler proves case-analysis totality (§2.4). This is non-negotiable
  safety.
- **Honest sums** — `Result`/`Option` are `data`, never sentinels (§2.5).
- **Indexed families:** only type-possible constructors must be covered
  (§2.2/§2.4).

## 6. Sequencing notes

- L2 **unblocks B2** (the `Temporal` datatype — `data`, not modalities — needs
  L2+B1) and **T3** (test/property framework, needs L2). Flag the B2 coupling:
  B2
  is `Temporal` as ordinary indexed `data`, so keep the indexed-family path
  clean.
- The exhaustiveness checker is a **surface** algorithm; the kernel already has
  the
  eliminator. Keep the trust boundary crisp — the kernel proves the *eliminator*
  sound; the surface proves the *match covers it* (totality of case analysis).
- Standard §2c: frame → spec-leader elaborates `34` + conformance → merge
  (Architect + conformance-validator) → Team Language compacted, then kicked
  off.
