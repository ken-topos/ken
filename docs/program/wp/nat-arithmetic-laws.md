# WP nat-arithmetic-laws — canonical `Nat` `add`/`mul` + algebraic laws (free lemmas)

**Owner:** Ergo team (catalog). **Steward-framed** (2026-07-11). Base:
`origin/main` (re-verify `file:line` at pickup). **Outer-ring catalog proof:** no
`crates/**/src`, no `ken-kernel`, no `Cargo.lock`, no `/spec`, no `/conformance`.
**@architect fidelity gate** (real structural-induction proofs — no `Axiom`/
`Refl`-paper on a general statement). **Queues behind `case-eq-adoption`** on the
Ergo serial lane (Track B: case_eq cleanup → **this** → DS-8c). Independent of
`case_eq` (pure Nat induction, no stuck-comparison dispatch).

## Settled inputs — pin, do NOT reopen

- **NOT a `class Num` / `Semiring`.** The numeric-class question is **OPEN**
  (`spec/90-open-decisions.md`, the `class Num` gate). The arithmetic laws ship as
  **free `fn` lemmas** returning `Equal Nat …` — the *same shape* as the landed
  `List`/lawful-class law lemmas (`Collections`/`LawfulClasses`) — **never** a
  `Num`/`Semiring` class or instance. Framing them as a class would trip the open
  decision; laws-as-lemmas sidesteps it entirely and is honest about the trust
  level.
- **`Nat = Zero | Suc` and `sub` (saturating monus) already exist** (OrdNat /
  Collections). `Ord Nat` + `min`/`max`/`compare` landed (`OrdNat.ken.md`). This
  WP adds the **additive/multiplicative** layer they don't cover.
- **`nat_add` exists only locally** in `Capability/Parsing/Parsing.ken.md:379`
  (`match n { Zero ⇒ m ; Suc n2 ⇒ Suc (nat_add m n2) }` — recursion on the
  **second** argument). The canonical `add` here reflects that definition; see
  seam 1 on unifying Parsing's local copy.

## Goal — canonical ops + the core algebraic laws

1. **Canonical `add` and `mul`** (`fn : Nat → Nat → Nat`), structurally
   recursive (reflecting `nat_add`'s second-argument recursion; `mul` built on
   `add`). `sub` is the existing saturating monus (reuse, don't redefine).
2. **The core algebraic laws, as free lemmas proved by structural induction on
   `Nat`, zero-`Axiom`:**
   - **Additive:** `add_zero_r` (`add n Zero = n`, definitional), `add_zero_l`
     (`add Zero n = n`, by induction), `add_suc_r`/`add_suc_l` helpers,
     `add_assoc`, `add_comm`.
   - **Multiplicative:** `mul_zero_r`/`mul_zero_l`, `mul_one_r`/`mul_one_l`,
     left+right **distributivity** of `mul` over `add`, `mul_assoc`, `mul_comm`.
   The `comm`/`distrib`/`assoc` proofs need the `add_suc`/`add_zero` helper
   lemmas threaded through `cong`/`trans` — the classic Peano-induction chain, the
   same proof idiom the landed `List` law lemmas use.

## Design seams — Architect fidelity gate (flag at pickup, don't guess)

1. **Placement + Parsing's local `nat_add`.** New Core file (e.g.
   `Core/NatArith.ken.md`) vs extending `Core/OrdNat.ken.md`. And Parsing's local
   `nat_add` (`Parsing.ken.md:379`) is a duplicate of the canonical `add` — decide
   whether to **unify Parsing onto the canonical `add`** in this WP (behavior-
   identical, one definition) or leave it and flag a follow-on. Recommend: define
   canonical `add`/`mul` here; unify Parsing only if it's a clean, green,
   behavior-preserving swap — else flag it. Architect/leader call.
2. **Recursion convention (proof-shape-determining).** `add` recurses on the
   **second** arg, so `add n Zero = n` is **definitional** but `add Zero n = n`
   needs induction — the law proofs' structure follows from this. Confirm the
   convention is fixed and consistent with the existing `nat_add`/`OrdNat` ops so
   downstream consumers see one arithmetic.
3. **Law scope.** The core set above is the target; if a specific law needs an
   unexpectedly deep helper chain, prove the core (`assoc`/`comm`/`distrib`/
   identities) and **size-defer + flag** any extra, never `Axiom`/`Refl`-paper it.

## Scope

- A new (or extended) Core catalog file: canonical `add`/`mul` + the law lemmas.
- Acceptance tests beside the landed Nat/Ord suites: each law lemma
  **elaborates + kernel-checks** at its general statement; a few concrete
  evaluations (`add 2 3 = 5`, `mul 2 3 = 6`) as sanity. Targeted builds
  (`-p <crate> <test>`); full-suite green in CI at merge.

### Out of scope

- Any `class Num`/`Semiring`/numeric-class machinery (OPEN decision — see pinned
  inputs).
- `Int`/`Decimal` arithmetic (opaque-carrier; separate, and `Num Int`/`Num
  Decimal` hit the open-decision wall — `90-open-decisions.md`).
- Any kernel/spec/conformance change; `trusted_base()` delta empty.

## Acceptance criteria

- **AC1 — canonical ops.** `add`/`mul` defined, structurally recursive,
  termination-checked (SCT sound zone); concrete evaluations green.
- **AC2 — laws proved.** `add_assoc`/`add_comm`/identities and
  `mul_assoc`/`mul_comm`/`mul`-over-`add` **distributivity**/identities proved as
  **real induction terms** (the Architect greps the tangled code for `Axiom`/
  `Refl`-paper on general statements). Any deferred law is flagged, not papered.
- **AC3 — not a class.** No `class`/`instance` for a numeric structure; laws are
  free `fn` lemmas (`Equal Nat …`). Grep-confirmed.
- **AC4 — zero trust/build delta.** No `Axiom`/`postulate`/`Decl::Opaque`/`sorry`;
  `trusted_base()` delta empty; no `crates/**/src`/`ken-kernel`/`Cargo.lock`/
  `/spec`/`/conformance` touch. Grep-confirmed. Workspace-green in CI.

## Gate

Ergo ring (ergo-leader → ergo-implementer → ergo-qa) → **@architect fidelity
gate** (real induction proofs, no `Axiom`/`Refl`-paper; laws-as-lemmas not a
class; SCT-sound recursion) → `git_request` to Steward → **CI-gated** merge (real
catalog code + tests, not doc-only). Outer-ring, no soundness urgency. Own the
retro (terra harness readout). **No WP-token identifiers in production/tangled
source.** Re-verify `file:line` cites at pickup.
