# WP #28: attached-proof reference style — `::` → `(proof name for subject)`

**Owner:** Foundation (catalog authoring). **Size:** S/M — judgment pass over
catalog source. **Risk:** low — pure surface; both spellings already elaborate
to the identical proof term (spec §8.2). **Base:** `origin/main @ 5e1ab924`
(fetch + re-verify at pickup). **Process (operator ruling): LIGHT — no spec
enclave.** Review = Architect-terminal (surface + fidelity + the
recursive-position judgment call). Steward honesty-gates + merges.

## Objective

Operator style ruling (2026-07-12): in catalog **code**, attached-proof
references should use the **readable selector form `(proof name for subject)`**,
**not** the desugared double-colon path `subject::name`. The `::` form is valid
surface but "really a way to communicate a desugared identifier to the kernel" —
it should not be the authored catalog style.

Example (operator-provided) — an `Ord Nat` instance:

```
instance Ord Nat {
  leq     = leq_nat ;
  refl    = proof refl for leq_nat ;
  antisym = proof antisym for leq_nat ;
  trans   = proof trans for leq_nat ;
  total   = λx.λy. (proof eq_true_of_or for bool_or) (leq_nat x y) (leq_nat y x) (total_leq_nat x y)
}
```

## Fixed inputs (settled — do NOT reopen)

- **Both forms are already valid surface and resolve to the same term.** Spec
  §8.2 (`spec/30-surface/33-declarations.md:541-545`): *"The canonical path is
  `subject::proof_name` … The selector syntax `(proof appends for list_append)`
  resolves to that same proof term."* Parser: `Expr::EAttachedProofRef`. **No
  elaborator/parser/spec change — this is a catalog source rewrite only.**
- **Direction:** `subject::name` → `(proof name for subject)` in authored
  catalog **code**. Do not touch declaration heads (already `proof … for …`),
  `fn`/`const`, glyphs, or the Ω-partition.

## Scope — CODE references only; the `::`-documenting prose STAYS

The catalog has ~222 `::` occurrences; a blind sweep would corrupt the docs.
Classify and convert **only code**:

- **IN scope (convert):**
  - **Instance field values** — `refl = leq_nat::refl` →
    `refl = proof refl for leq_nat`. (The operator's explicit locus.)
  - **Applied selectors in term position** — parenthesize:
    `bool_or::eq_true_of_or (leq_nat x y) …` →
    `(proof eq_true_of_or for bool_or) (leq_nat x y) …`.
  - **Bare references** (whole RHS / argument position) — no parens needed:
    `= leq_nat::refl` → `= proof refl for leq_nat`.
- **ARCHITECT JUDGMENT (verify, then convert-or-keep):**
  - **Proof-body sibling/self references** — e.g. inside `leq_nat::refl`'s body,
    `leq_nat::refl x2` (recursive) or `leq_nat::trans` (sibling). These are code
    and the principle applies, **but** the selector form in
    *recursive/self-referential attached-proof position* must be verified to
    elaborate green (the `::` path is a direct desugared name; confirm the
    selector resolves identically during the definition of that very proof).
    Convert where green + clearer; keep `::` only if a genuine
    resolution/self-reference issue is found (report it if so).
- **OUT of scope (leave `::` untouched):**
  - **Prose that documents the path convention** — e.g. "the canonical path is
    `subject::name`", "attached to `leq_nat` and uses its `leq_nat::name`
    path", the `surface-reference.ken.md` guide, `README.md` path examples. The
    docs must keep showing `::` to teach that the path exists.
  - `spec/`, `conformance/`, `crates/**` source, prelude, kernel — untouched.

## Parenthesization rule (load-bearing)

`proof p for s` used in **application position greedily absorbs the arguments
into the subject** unless parenthesized: `proof P for s a b` parses as
`proof P for (s a b)`, not `(proof P for s) a b`. So **every applied selector
must be wrapped**: `(proof P for s) a b …`. Bare (non-applied) occurrences need
no parens. The implementer verifies the exact parse; green elaboration is the
proof.

## Acceptance criteria (testable)

1. Every in-scope code `subject::name` attached-proof reference is the selector
   form; applied ones parenthesized; `::`-documenting prose unchanged.
2. **Semantics-preserving:** each converted reference elaborates to the same
   proof term (same instance/proof checks); no proposition/proof-term/`fn`/
   `const`/glyph change. All touched packages elaborate green.
3. **Structural completeness proof:** full `scripts/ken-cargo test --workspace`
   green + catalog acceptance nets green (a mis-parenthesized or mis-resolved
   selector fails to elaborate).
4. **Whole-harness check:** if any exact-source assertion pins a converted
   reference's `::` spelling (as CAT5 pinned a glyph in #26), migrate the coupled
   assertion in the same branch (Steward-authorized, assertion-literal only) and
   grep the whole test tree; workspace green.
5. **Scope clean:** no `spec/`, `conformance/`, `crates/**` source (beyond an
   authorized coupled-assertion sync), Cargo, or lockfile change; `git diff
   --check` clean.

## Do-not-reopen guardrails

- Both forms are already valid (spec §8.2) — this is style, not a feature. Don't
  touch the parser/spec/elaborator.
- Don't convert `::`-documenting prose; don't touch declaration heads, glyphs,
  `fn`/`const`, or the Ω-partition.
- Report (don't force) any proof-body self-reference that won't take the selector
  form.
