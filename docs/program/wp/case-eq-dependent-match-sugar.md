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

**Status: FRAMED, BUILD HELD.** Do not kick to a build seat yet — held pending
(a) the Architect pre-shaping the surface/mechanism (routed in parallel so this
is shovel-ready) and (b) the operator's fleet implementer→terra migration. The
Steward releases the kickoff.

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
1. `catalog/packages/Core/EmptyDec.ken.md` (`dec_eq_decides`, ~:130–137);
2. `catalog/packages/Data/Collections/Map.ken.md` (`dispatch_on_q1`/`q2`,
   `lookup_found_dispatch_*`, ~:275–279, :1001/:1018/:1215/:1232, :697/:709/:732);
3. `catalog/packages/Core/LawfulClasses.ken.md` (`list_deceq_sound_cons_dispatch`,
   just landed with `deceq-structural-liftings`).

Every future decidable-element lemma — `lawful compare`, `Ord (List a)`,
`Ord (Pair a b)` on the operator's near path — re-pays it. Subsuming the idiom
into one construct removes a recurring, compounding cost.

## Goal

A surface construct — provisionally `case_eq` — that lets an author **dependently
eliminate a stuck/neutral scrutinee in proof-relevant position**, with the
elaborator automatically: (a) **generalizing** the scrutinee to a fresh bound
variable, (b) **synthesizing the dependent motive** over it, (c) **inserting the
`J`/`subst` transport**, and (d) **binding, in each branch, the equation**
`scrutinee = <that constructor>` as a named hypothesis the branch body can use.

It must **desugar to the exact primitives the hand idiom uses today** (the
`J`-transport + equality type), so it is **behaviorally identical to a correct
hand-rolled proof** and adds **nothing to the TCB**. The win is purely
authoring-ergonomic: what today is ~15–30 lines of dispatch-helper + explicit
motive collapses to a direct dependent match.

## Design seams — Architect/Spec to settle (flag, don't guess)

These are **not** for a build seat to choose. The Architect pre-shapes the
concept; Spec ratifies the surface. Grounded routing below.

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
