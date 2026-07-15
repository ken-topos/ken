# WP case-eq-dependent-match-sugar — dependent case-analysis on a stuck comparison

**Owner:** Language team (elaborator). **Steward-framed** (2026-07-11). Base:
`origin/main` (re-verify `file:line` cites at pickup — the elaborator moves).
**Inner-ring**, `crates/ken-elaborator/src`. Soundness-**adjacent** but
**fail-closed**: the construct is **pure elaborator sugar that desugars to
existing primitives** (`J`/`subst` + the equality type) — **no new kernel
primitive, zero TCB growth**; the kernel re-checks the elaborated transport, so a
motive-synthesis bug **rejects an ill-typed proof, never admits one**. →
**@architect soundness-adjacent gate**. Surface syntax is user-visible →
**Spec ratifies** the grammar clause (CV in loop).

**Status: SHAPE FIXED (Architect `evt_65bw0yqbrqsf8`), SHOVEL-READY, BUILD HELD.**
The design is resolved (see "Design — RESOLVED" below). The build is held on the
single remaining gate: **the operator's fleet implementer→terra migration.** The
Steward releases the kickoff. Spec surface-clause routing may run in parallel.

## Context — why this is pressing

Ken currently has **no way to dependently `match` a stuck/neutral application**
(e.g. `da.eq x y`, `compare x y`) in a **proof-relevant** position. The
dependent-motive machinery has nothing to bind — the scrutinee is an
*application, not a bound variable* — so `match (da.eq x y) { … }` fails to form
a motive (`KernelRejected TypeMismatch { expected: Type 0, found: @N }`).
Architect ruling `evt_3zgjdyzyhgrsk` documents this as a known limitation.

The workaround is a **verbose hand-rolled idiom**, re-paid in full at every site:
`bool_dichotomy e` → `Or (Equal Bool e True) (Equal Bool e False)` → a **named
dispatch helper** whose declared return type pins the goal → an **explicit `J`/
`subst` transport** with a **hand-written motive binding a fresh `Bool` var**
(`λb. IsTrue (match b { True ⇒ … ; False ⇒ … })`) → two arms. It is correct but
**expensive in tokens and proof complexity**, and it is now the *default tax* on
any abstract-element-decidable proof.

**Three occurrences and counting — the subsume-don't-proliferate trigger**
(`docs/PRINCIPLES.md`):
1. `catalog/packages/Core/Logic/EmptyDec.ken.md` (`dec_eq_decides`, ~:130–137);
2. `catalog/packages/Data/Collections/Map.ken.md` (`dispatch_on_q1`/`q2`,
   `lookup_found_dispatch_*`, ~:275–279, :1001/:1018/:1215/:1232, :697/:709/:732);
3. `catalog/packages/Core/Classes/LawfulClasses.ken.md` (`list_deceq_sound_cons_dispatch`,
   just landed with `deceq-structural-liftings`).

Every future decidable-element lemma — `lawful compare`, `Ord (List a)`,
`Ord (Pair a b)` on the operator's near path — re-pays it. Subsuming the idiom
into one construct removes a recurring, compounding cost.

## Goal

A `match` **modifier** — `match e eqn: h { … }`, Architect-ruled shape
(`evt_65bw0yqbrqsf8`) — that lets an author **dependently eliminate a
stuck/neutral scrutinee (a finite nullary-ctor enum) in proof-relevant
position**, with the
elaborator automatically: (a) **generalizing** the scrutinee to a fresh bound
variable, (b) **synthesizing the dependent motive** over it, (c) **inserting the
`J`/`subst` transport**, and (d) **binding, in each branch, the equation**
`scrutinee = <that constructor>` as a named hypothesis the branch body can use.

It must **desugar to the exact primitives the hand idiom uses today** (the
`J`-transport + equality type), so it is **behaviorally identical to a correct
hand-rolled proof** and adds **nothing to the TCB**. The win is purely
authoring-ergonomic: what today is ~15–30 lines of dispatch-helper + explicit
motive collapses to a direct dependent match.

## Design — RESOLVED (Architect ruling `evt_65bw0yqbrqsf8`, 2026-07-11)

**Shape fixed; shovel-ready. Spec ratifies only the exact token spelling.**

- **1 Surface — a `match` MODIFIER, not a keyword.** `match e eqn: h { <ctor> ⇒
  … }` (Coq `destruct…eqn:`): plain `match` + a modifier that per branch
  generalizes `e`'s occurrences in the *goal* to a fresh var, synthesizes the
  motive, inserts the `J`/`subst` transport, binds `h : Equal T e <ctor>`.
  Uniform whether `e` is a stuck application or a bound var. **Rejected:** a
  dedicated `case_eq` keyword (grammar proliferation) and the `if`-decidable form.
  Spec pins the token (`eqn:`/`as h`/`with h`); Architect pinned the *shape*.
- **2 Generality — finite NULLARY-constructor enums.** Covers **Bool** (the 3
  sites) **and `OrdResult = Lt|Eq|Gt`** (the compare/`Ord` near-path) — one
  deliberate step past Bool-only, because a Bool-only MVP would force a re-frame
  within a brick or two (proliferation-by-under-generalization). **Rejected:**
  general match over inductives with fields (existentials + index unification, a
  large project, unneeded). Machinery is identical for 2-vs-N nullary ctors;
  all-nullary-ctor is the natural boundary, Bool is not. See
  [[subsume-idiom-generality-grounds-on-near-path-types]].
- **3 Desugar — existing primitives only, TCB delta empty (confirmed).** Motive
  `M := λ(v:T). Goal[e:=v]`; per-branch `J`/`subst` over `Equal T e <ctor>`. The
  generalized **enum-dichotomy** `(v:T) → Or_N (Equal T v Ci)` is a **DERIVED
  checked term** synthesized via `T`'s own eliminator returning `Refl` per branch
  (as `bool_dichotomy` returns Refl-at-canonical) — **never a postulate**. Only
  `T`-elim + `J` + `Equal` + `Refl`: no new `Decl::Opaque`, empty `trusted_base()`
  delta, `ken-kernel` untouched.
- **4 Motive synthesis + fail-closed (backstop holds).** Find `e`'s goal
  occurrences → abstract → kernel-infer `M : T → Type` → check each branch against
  `M <ctor>` → assemble → **submit the whole term to `kernel_infer`** (same
  discipline as `resolve_instance_dictionary`). Output type is pinned to the
  author goal and rechecked → a wrong/under-general motive is `KernelRejected`,
  never wrong-but-accepted. Discipline: if a well-typed motive can't be formed,
  **error explicitly (fail-closed)** — never a silent partial abstraction.
- **5 Adoption — this WP is elaborator-only; 3-site migration is a fast-follow.**
  This WP touches only `crates/ken-elaborator/src` + tests, subsumption shown
  **in-test**. A **separate outer-ring adoption WP** mechanically migrates
  EmptyDec + Map + deceq (behavior-preserving). Rationale: tightest blast radius,
  capability bakes before touching landed catalog, clean ring separation.
  **Operator may override to all-3-now.**

**Bake the "revert the hypothesis" pattern into the normative examples.** The
sound-direction sites transport a *hypothesis* (`h : IsTrue (list_eq (Cons)(Cons))`)
while the goal doesn't mention `e`. The author π-abstracts the hypothesis into the
goal (helper *returns a function* `IsTrue(…) → Equal …`), so `e`'s occurrences
live in the goal's domain and transport automatically — each branch receives an
already-reduced domain (`IsTrue(list_eq xs ys) → …` in the `True` arm; `Bottom →
…` = `λh. absurd h` in a false arm). This collapses the ~15–30-line explicit-J
dispatch to a direct match — Spec's normative section shows the
reverted-hypothesis example (largest token payoff). Always bind `h` even if
unused; generalize **all** occurrences (no partial-occurrence control this WP).

_Original open-question phrasing (each now answered by the rulings above):_

1. **Surface syntax (Spec-ratified, user-visible).** e.g. `match e eqn: h { … }`
   (Coq `destruct … eqn:`), a dedicated `case_eq` keyword, or an `if`-style
   decidable form. Whatever lands is a `33`/`32` grammar clause + a normative
   section, with CV verifying code↔spec (the constrained-instance-naming pattern).
2. **Generality (the key scoping call).** Narrow **MVP** — only `Bool`-valued /
   decidable-`eq` scrutinees (covers all three current sites and the whole
   compare/Ord path) — vs. a **general** dependent-match-with-generalization over
   any inductive scrutinee. Reflect-don't-extend argues for the **narrowest form
   that subsumes the three sites**; the Architect rules how far to generalize.
3. **Desugaring target.** The existing `bool_dichotomy`+`Or`+named-dispatch+`J`
   idiom, or a more direct synthesized-motive `J`/`subst`. Determines the
   TCB-neutrality argument (must desugar to *existing* checked primitives only).
4. **Motive synthesis (the hard compiler part).** How the elaborator generalizes
   the scrutinee occurrence(s) in the goal and forms the motive — the failure
   mode is a wrong/under-general motive, which **must be fail-closed** (kernel
   rejects the mistyped transport; the elaborator is untrusted).
5. **Adoption scope.** Whether this WP also **migrates the three existing sites**
   to the new construct (each a mechanical rewrite, and the token-cost payoff),
   or ships the capability + demonstration and leaves adoption as a fast-follow.
   Leader/Architect call at pickup.

## Scope

- `crates/ken-elaborator/src` — the surface parse + elaboration of the construct
  to the `J`-transport idiom (generalization + motive synthesis + branch-equation
  binding), reusing existing primitives. **No `crates/ken-kernel` change.**
- Elaborator acceptance tests: a proof-relevant match on a stuck `da.eq`-shaped
  scrutinee **elaborates and kernel-checks**; a deliberately-wrong motive/branch
  is **kernel-rejected** (specific variant, fail-closed witness); a
  representative proof (a `deceq`-List-`Cons/Cons`-shaped case) is discharged via
  the construct with **byte-fewer** lines than the hand idiom.
- If adoption is in scope (seam 5): rewrite the three sites; all existing
  acceptance suites stay green (behavior-preserving — the desugaring is the
  idiom).
- Spec: the surface grammar clause + normative section (co-authored with Spec,
  CV verifies code↔spec) — may be a paired Spec deliverable like the
  constrained-instance naming clause.

### Out of scope

- Any new kernel primitive / trust-surface growth — the construct is sugar over
  existing checked primitives; `trusted_base()` delta must be empty.
- General decidability/typeclass machinery beyond what seam 2 rules in.
- The downstream `compare`/`Ord` proofs themselves — later bricks; this WP just
  removes the tax they'd otherwise pay.

## Acceptance criteria

- **AC1 — capability.** The construct elaborates a proof-relevant dependent
  elimination of a stuck/neutral scrutinee and kernel-checks; the branch equation
  is usable; a representative `deceq`-shaped proof is discharged through it.
- **AC2 — fail-closed, verified.** A wrong motive / mismatched branch is
  **kernel-rejected** (assert the specific variant, not `is_err()`) — the
  untrusted-elaborator backstop the Architect gates.
- **AC3 — zero TCB growth.** Desugars to existing primitives only; `trusted_base()`
  delta empty; no new `Decl::Opaque`/`declare_postulate`; `crates/ken-kernel` and
  `Cargo.lock` untouched. Grep-confirmed.
- **AC4 — subsumption demonstrated.** The construct provably replaces the hand
  idiom on at least one real site with fewer lines and identical behavior (green).
  If adoption is in scope, all three sites migrate with every suite green.
- **AC5 — build.** Workspace-green in CI at merge; targeted local builds only
  (`-p ken-elaborator <test>`), never a full `cargo build`.

## Gate

Language ring (language-leader → language-implementer → language-qa) →
**@architect soundness-adjacent gate** (desugars-to-existing-primitives, empty
TCB delta, motive-synthesis fails closed) → if a normative surface clause is
added, **Spec/CV** ratify it (non-terminal for the spec portion) → `git_request`
to Steward → **CI-gated** merge. Own the retro (harness readout — a terra
implementer trial if that's the seat). **No WP-token identifiers in production
source** (self-grep the whole diff). Re-verify `file:line` cites at pickup.
