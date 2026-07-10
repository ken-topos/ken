# WP deceq-structural-liftings — lawful `DecEq (Pair a b)` + `DecEq (List a)`

**Owner:** Foundation team. **Steward-framed** (2026-07-10). Base:
`origin/main` (grep `file:line` cites at pickup — the catalog moves).
**Outer-ring** catalog `.ken.md` + acceptance test; no `crates/**/src`,
`ken-kernel`, or `Cargo.lock`. Proof-carrying lawful instances → **Architect
fidelity/soundness gate**.

## Context — where this sits

First open brick of the operator's compare-based lexicographic-order path
(`lawful DecEq → lawful compare → Collections rework → Ord (Pair)/Ord (List)`).
The **base** `DecEq` instances are already lawful on main —
`DecEq {Int, Bool, Char}` in `Core/LawfulClasses.ken.md` (`Char`
certificate-backed via transport off `DecEq Int`, ADR 0013 Layer 1). What is
**not** landed is the **structural lifting** of `DecEq` over the compound and
recursive carriers — `Pair` and `List`. Those liftings are this WP, and they are
exactly what a compare-based order needs (compare's `Eq` pivot is a
decidable-equality decision on the structure).

Also the **first hard-level implementer trial on the Codex-native +
gpt-5.6-terra seat** (foundation-implementer). Reviewers apply normal rigor.

## Goal

Deliver two proof-carrying instances of the existing `Core` `DecEq` class
(`class DecEq a { eq : a→a→Bool ; sound : (x)(y)→IsTrue (eq x y)→Equal a x y ;
complete : (x)(y)→Equal a x y→IsTrue (eq x y) }`):

1. **`DecEq (Pair a b)`** given `DecEq a`, `DecEq b` — the bounded warm-up
   (single constructor, two components). `eq` conjoins the two field decisions;
   `sound`/`complete` are a bounded case over `mk_pair`, using each component's
   `sound`/`complete` + the `mk_pair` congruence.
2. **`DecEq (List a)`** given `DecEq a` — the real challenge. `eq = list_eq a
   (DecEq a).eq` (`list_eq` is `Collections`, `~:741`). `sound`/`complete` are a
   **structural `List` induction**: `Nil/Nil` → refl; `Cons/Cons` → head via
   `(DecEq a).sound`/`.complete` + tail via the IH + the `Cons` congruence;
   `Nil/Cons` and `Cons/Nil` → `list_eq` reduces to `False`, so `sound` is
   **vacuous via `absurd` on `IsTrue False`** and `complete` is vacuous via
   `Nil`/`Cons` discrimination — closed honestly, **never papered**.

## Scope

- The two instances + their real `sound`/`complete` proofs, built on the
  existing `list_eq`/`Pair` helpers (`Collections`) and the base `DecEq`
  dictionaries. Reuse, don't re-mint.
- **Fold the one-line Collections stale-doc fix:** `Collections.ken.md ~:826`
  says lawful `DecEq Char` is "not yet landed" — it *is* landed; correct that
  line (it blocks nothing but is now false).

### Out of scope

- lawful `compare`/`Ord (List)`/`Ord (Pair)` — later bricks on the path;
- any `Decimal`/non-canonical-carrier `DecEq` (soundness-gated, `90-open-
  decisions.md`);
- kernel/`Cargo`/spec/`trusted_base` changes of any kind.

## Design seam to settle at pickup (flag, don't guess)

**Instance homing (orphan rule, `33 §5`).** An instance must home with the
class (`DecEq`, `Core/LawfulClasses`), or the carrier's package. `Pair` is
defined in `Collections`; `List` is prelude. Decide each instance's home so
**neither is an orphan** — likely `Collections` (which owns `Pair`/`list_eq`
and imports the `Core` `DecEq`), but confirm against `33 §5` and the
`lawful-classes-lane` precedent (Ord/DecEq Char homed next to their `Int`
twins). If the rule forces a split home or is ambiguous, **surface it to
Steward/enclave — do not force it.**

## Acceptance criteria

- **AC1 — real lawful instances.** Both instances elaborate + type-check; `eq`
  computes; `sound`/`complete` are **real proofs** — grep-clean of `Axiom`,
  `postulate`, `sorry`, or empty stubs. The vacuous cross-constructor `List`
  cases are discharged via `absurd`/discrimination, **not** a wildcard that
  dodges the structure.
- **AC2 — discriminating checks flip.** A positive case (`[1,2] eq [1,2]` →
  `sound` yields `Equal`) and a negative case (`[1,2]` vs `[1,3]` → `eq` is
  `False`, and the mismatch is handled), asserted with specific outcomes, not
  bare `is_err()`. Same for `Pair`.
- **AC3 — zero trust drift.** `trusted_base()` delta empty; no new
  `Decl::Opaque`; `crates/ken-kernel`/`Cargo.lock` untouched. Grep-confirmed.
- **AC4 — build.** Workspace-green in CI at merge (QA re-runs the suite
  independently). Local: **targeted builds only** (`-p <crate> <test>`), never a
  full local `cargo build`.

## Gate

Foundation ring (foundation-leader → foundation-implementer → foundation-qa) →
**@architect fidelity/soundness gate** (proof-carrying instances: greps the
tangled code for `Axiom`/`Refl`-paper, confirms the vacuous cases are honest and
the homing is non-orphan) → `git_request` to Steward → **CI-gated** merge.
Outer-ring, no soundness urgency beyond the honesty of the law proofs. Own the
retro (record the harness readout — this is the hard-implementer terra trial).
**No WP-token identifiers in the tangled source** (self-grep the whole diff).
Re-verify `file:line` cites against the catalog at pickup.
