# WP `pedagogic-catalog-prototype` — Foundation

**Owner:** Foundation team (leader / implementer / QA)
**Branch:** `wp/pedagogic-catalog-prototype`
**Size:** M · **Risk:** low (additive-in-spirit; behavior-preserving)
**Depends on:** convention note §7.1 (landed `origin/main @ 9a2cf746`) — met.

## Objective

Rewrite a few mid-complexity catalog `.ken.md` sources as **pedagogic
documents**: top-down (state the result first, prove it below), using
`def`/`prop`/`lemma`/`proof` to structure, document, and motivate the code.
This is a **prototype** — its primary product is *discovery*: how to represent
Ken code with the proof-claim vocabulary so a source reads like a paper, plus a
short authoring-pattern write-up and a list of any real gaps surfaced. The
Architect's verdict (evt_261s6784m3y48) is HYBRID: the forms are real and
kernel-backed today, so nothing is built first — you prototype under the §7.1
convention and route genuine gaps back.

## Settled inputs — FIXED, do not reopen

These are decided. Treat them as ground, not as questions to relitigate.

1. **The `def`/`prop`/`lemma`/`proof` semantics and when to use each** are pinned
   in `catalog/guide/surface-reference.ken.md` **§7.1 "Choosing a form"** (the
   decision table + the Ω-vs-Type rule + `lemma`-vs-`proof … for` ownership + the
   top-down enabler). **§7.1 is your authoring contract — follow it; do not
   invent a per-file convention.**
2. **The load-bearing rule: `lemma`/`proof` require an `Omega` statement**
   (`ensure_omega_type`, elab.rs). `Equal`/`IsTrue`-typed laws and `And`-of-Ω are
   Ω → `lemma`/`proof`. **Proof-relevant** conclusions (`Or : Ω→Ω→Type`, `Σ` with
   a `Type` component, disjunction/eliminator helpers that carry a branch *as
   data*) stay `const`/`fn`. Promoting one of those to a `lemma` is **not** a bug
   to fight — it is the wrong side of the proof-irrelevance line.
3. **Top-down is supported.** Order-independent top-level resolution is decided
   (spec §33 mutual recursion) and built (the elaborator name pre-pass +
   call-graph SCC grouping in `crates/ken-elaborator/src/modules.rs`). A `lemma`
   may be *stated above* the `fn`/`const`/`lemma` it invokes. **Do not touch or
   re-investigate evaluation/declaration order** — it is done.
4. **The self-reference caveat.** A `lemma` body still cannot call *itself*. A
   proof that needs induction stays an ordinary **recursive `fn`** behind a
   **thin non-recursive `lemma` wrapper** (§7.1). Expect this for the recursive
   arithmetic proofs (see Scope).
5. **Clean-room** (CLAUDE.md / CLEAN-ROOM.md): build from `/spec` + §7.1, never
   from `local/refs/`.

## Scope

**In scope — rewrite these files, in this order:**

1. **`catalog/packages/Core/NatArith.ken.md`** (flagship, ~232 L). Canonical
   arithmetic: `fn add`/`mul` (computation — stay `fn`); the law family
   `add_zero_r`/`add_zero_l`/`add_suc_*`/`add_assoc`/`add_comm`/`mul_*`/
   `mul_add_distrib_*`/`mul_assoc` (all `Equal Nat …`-typed → **Ω → `lemma`**);
   `const add_two_three`/`mul_two_three` (values — stay `const`). This file is
   also the keyword-adoption pilot, so it doubles as that.
2. **`catalog/packages/Core/OrdNat.ken.md`** (~290 L; already uses `lemma` ×5).
   Order + its laws — finish the top-down/statement-first treatment and bring the
   remaining `Equal`/`IsTrue`-typed order laws under `lemma`/`proof` per §7.1.

**Optional third if the pattern holds cleanly:**

3. **`catalog/packages/Core/EmptyDec.ken.md`** (~299 L; already `lemma` ×2 +
   `proof` ×1). Decidability — a good `prop`/`proof … for` showcase. `Dec`/`Empty`
   have `Type`-level (proof-relevant) content, so it will also **exercise the
   Ω-vs-Type boundary in practice** — a useful stress of §7.1.

**Out of scope:** the big files (Collections, LawfulClasses, EffectfulClasses,
Map), the guide files (already pedagogic by nature), and the trivial files
(Transport, decomposition-abstraction — little to gain). Any kernel / spec /
`crates/` (non-test) / prelude / Cargo change. **No behavior change** (below).

## Mandated deliverable outline (per file)

Each rewritten source should end up as a readable top-down document:

1. **A lede** — 2–5 lines up top stating what the module establishes and why it
   matters (the motivation), before any declaration.
2. **Statement-first ordering** — headline results (the laws/props) stated near
   the top; supporting `fn`/`const` machinery below. Use the order-independent
   resolver (settled input 3) — do not reorder for the elaborator's sake.
3. **Vocabulary applied per §7.1** — every Ω-typed law becomes a `lemma` (or
   `proof … for <subject>` when it is *about* one definition and should travel
   with it); computation stays `fn`/`const`; use `prop` where a proposition
   family is worth naming as a statement to reason about.
4. **Recursive proofs** — where a law needs induction, keep the recursive `fn`
   and put a thin non-recursive `lemma` wrapper in front of it (settled input 4).
   Make the wrapper the headline; the recursive `fn` is supporting machinery.
5. **Motivating prose** — short Markdown between declarations that explains the
   *why*, not the mechanics the code already shows. (Coordinate with the
   Librarian's outsider-prose sweep style; do not re-add work-history/Findings.)

Plus **one short write-up** (a section in the WP handoff, or a `## Pedagogic
pattern` note you propose homing in the guide): what worked, what the reusable
top-down authoring pattern is, and every gap surfaced.

## Acceptance criteria (testable)

1. **Re-elaborates green.** Each rewritten file still loads and type-checks; the
   catalog-load / relevant acceptance suites pass (e.g. `scripts/ken-cargo test
   -p ken-elaborator` targeted at the catalog-load + any Nat/Ord acceptance
   binary). The **set of public declarations is preserved** (same names, same
   types) — a rename or dropped law is a regression, not a rewrite.
2. **Behavior-preserving.** No fenced `ken` declaration changes its *meaning*:
   `fn`→`lemma` is allowed only where the statement is Ω and the proof term is
   unchanged; no proof is weakened, no `Axiom`/`postulate`/`sorry`/`Opaque`
   introduced; `trusted_base_delta` unchanged (zero new TCB).
3. **§7.1-faithful.** Every Ω-typed law is a `lemma`/`proof` (or has a one-line
   note why not); no `const`/`fn` was promoted to `lemma` against
   `ensure_omega_type`; `proof … for` used only where attachment is warranted.
4. **Top-down + motivated.** Each file has a lede and statement-first ordering; a
   reader meets the headline results before the machinery.
5. **Discovery captured.** The pedagogic-pattern write-up and the surfaced-gap
   list are in the handoff.

## Do-not-reopen guardrails

- Order-independence (settled input 3) is **done** — do not re-open or re-test
  it; just rely on it.
- The Ω-vs-Type boundary (settled input 2) is **fixed** — do not attempt to make
  a `Type`-level term (`Or`, `Σ`-witness, eliminator helper) into a `lemma`.
- **No TCB / kernel / spec / prelude / Cargo change.** This is a catalog-source
  and prose rewrite. Test files may be added.
- Do not delete a public law or change its type to make it "fit" a form —
  surface the friction instead.

## Gaps to surface (route back to Steward → Architect; do NOT block on them)

- **No proof-relevant `lemma` form** — a named checked theorem whose conclusion
  is at `Type` is honestly `const : φ = proof` today. The Architect flagged this
  as the one candidate follow-up (not a blocker). If it hurts readability at
  scale in the prototype, say so with a concrete example.
- **Parser gaps already tracked** — `.field`-in-type-position and `match`-in-type
  (compare-ord surfaced both). If the top-down style bumps either, note it; the
  §7.1 style is designed to route around them (no fix required here).
- Anything new the vocabulary can't express cleanly — log it with a minimal
  reproducer.

## Notes

The prototype's value is the *pattern*, not volume: two files done well
(NatArith + OrdNat), with EmptyDec as a boundary-exercising stretch, beats three
rushed. Surface gaps as you go; the Steward routes them and, if warranted, opens
a clean follow-up WP to the Architect. `origin/main @ 9a2cf746` is the base; the
Architect ruling (evt_261s6784m3y48) and §7.1 are your standing references.
