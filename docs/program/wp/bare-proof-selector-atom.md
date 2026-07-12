# WP #29: bare `proof name for subject` selector atom

**Owners (two lanes):** Lane A = **Spec enclave** (spec §8.2/§33 production +
conformance fixtures); Lane B = **Language build** (`parser.rs` atom).
**Size:** S — one parser atom arm, one spec production, a small fixture set.
**Risk:** low — pure surface-grammar addition; **zero elaborator-semantics,
prelude, kernel, or TCB delta** (same AST node, same `subject::name` desugar).
**Base:** `origin/main @ 4435216c` (fetch + re-verify at pickup). **Design:
LOCKED by the Architect** (`evt_6ta5b68mjjy6x`, thread thr_3mnt59c84z9fa) — do
not reopen; transcribe and implement it. **Review = Architect-terminal**
(surface grammar + application-boundary + decl-vs-expr separation).

## Why (the #28 prerequisite)

The operator ruled (2026-07-12) that catalog code should reference attached
proofs via the readable selector form the style ruling wrote —
`refl = proof refl for leq_nat` (**bare, no parens**) — not the desugared
`subject::name` path. But `proof … for …` is **not currently an expression
atom**: today the parser recognizes it only after `LParen`, so the sole valid
form is the heavier `refl = (proof refl for leq_nat)` (parens on bare *and*
applied). The operator chose **Option B**: extend the parser to admit the bare
atom, then re-run the #28 catalog rewrite in the clean bare form. This WP is
that parser extension. **#28's catalog rewrite is gated on this landing.**

## Fixed inputs (settled by the Architect — do NOT reopen)

**Grammar (spec §8.2 canonical-path + §33 selector syntax):**

```
atom      ::= … | ident | literal | "(" expr ")" | proof_ref
proof_ref ::= "proof" ident "for" path    -- desugars to  path::ident
                                           --   (Expr::EAttachedProofRef)
```

- `proof_ref` is a **primary/atom — it binds tightest.** Application and every
  infix operator wrap *outside* it:
  - `proof p for s a b` = `((proof p for s) a) b` — **application binds outside
    the selector** (the opposite of the retired "greedy absorb" misdiagnosis).
  - `f proof p for s` = `f (proof p for s)`.
- The **subject is a single `path`** (the existing `parse_path` — `leq_nat`,
  `Mod.sub`, `::`-segmented paths all valid as subject). The selector ends at
  the end of that path; `parse_path` never absorbs following args (the existing
  parenthesized applied cases already prove this).
- **Bare and parenthesized forms produce the IDENTICAL
  `EAttachedProofRef { subject, proof_name }` AST** and desugar to the same
  `subject::name` global. After this WP, parenthesization is **optional
  grouping** (still useful for explicitness in dense expressions), never
  required.
- **Recursive/self-ref: nothing special.** The bare atom yields the identical
  node, so `proof refl for leq_nat x2` = `(proof refl for leq_nat) x2` — already
  green across the 168 converted sites in the parked `29094048` inventory.

## Deliverable outline (each ends in a concrete implementable choice)

### Lane A — Spec enclave (spec-author + CV) — **doc-only, lands FIRST**

1. **Spec §8.2 / §33 grammar production.** Add `proof_ref` as a surface **atom**
   equivalent to `subject::name`; state the **application-boundary rule**
   (subject is a `path`; application binds outside the selector) and that the
   bare and parenthesized forms are the identical AST / same desugar. This is a
   pure documentation of the locked design — spec-author transcribes it,
   spec-leader coordinates, **CV casts the spec vote**. Merge doc-only (fast).
2. **Conformance fixtures** (CV authors; land with Lane B where they go green —
   they cannot be green until the parser atom exists). Cover:
   - **bare** — `field = proof p for s` elaborates to the same result as
     `field = s::p`.
   - **applied** — `proof p for s a b` = `(proof p for s) a b` (application
     outside).
   - **recursive/self-ref** — a bare selector inside the proof's own body.
   - **decl-vs-expr disambiguation** — a proof *declaration* whose body is
     itself a bare selector (`proof foo for bar … = proof baz for qux`) parses
     head-then-body with **no cross-talk** (decl-position `proof` = decl head;
     expr-position `proof` = this atom; already separate parse phases — assert
     it).

### Lane B — Language build (leader + implementer + QA)

3. **`parser.rs` atom arm.** Add one `Token::KwProof =>` arm to
   `parse_atom_expr` (the primary/atom match, alongside the `KwOld` / `LParen`
   arms), with the **same body as the existing `LParen`-gated block at ~1965**
   (`expect_ident` → `expect_contextual_ident("for")` → `parse_path`), **minus**
   the surrounding `LParen`/`RParen`. **Leave the `LParen`-gated block
   untouched** (it stays valid as the identical-AST equivalent). This is the
   **only code change** — no other parser rule, no elaborator, no prelude, no
   kernel.

## Acceptance criteria (testable)

1. **Grammar:** `proof name for subject` parses as a bare expression atom in
   every expression position (instance field RHS, argument, proof-body); the
   subject is a single path; application binds outside
   (`proof p for s a b` ⟶ `((proof p for s) a) b`).
2. **AST/desugar identity:** bare and parenthesized forms produce byte-identical
   `EAttachedProofRef` and desugar to the same `subject::name`. The existing
   parenthesized surface still parses (unchanged).
3. **Conformance:** the fixture set (bare / applied / recursive / decl-vs-expr)
   elaborates green with results identical to the `::` path.
4. **Zero non-surface delta:** no elaborator-semantics, prelude, kernel, TCB,
   Cargo, or lockfile change beyond the one `parser.rs` atom arm + spec +
   conformance fixtures. `git diff --check` clean.
5. **Full `scripts/ken-cargo test --workspace` green.**

## Sequencing

**Lane A spec production (doc-only) merges first** → **Lane B parser atom +
conformance fixtures** (Architect-terminal surface review) merges second →
**then #28 re-kicks**: Foundation re-runs the 168-site catalog rewrite in the
bare form (drop the parens from the parked `29094048` inventory; the
reverse-substitution proof carries straight over as the target).

## Do-not-reopen guardrails

- The grammar is **locked** (Architect `evt_6ta5b68mjjy6x`). Do not redesign the
  application boundary, re-introduce "greedy absorb", or change the subject from
  a `path`.
- **No parser change beyond the one atom arm**; the `LParen`-gated block stays.
- **No kernel/prelude/elaborator-semantics touch** — same AST, same desugar.
- Don't touch declaration-position `proof` parsing (`parse_attached_proof_decl`)
  — the decl head and the expr atom stay cleanly separate.
