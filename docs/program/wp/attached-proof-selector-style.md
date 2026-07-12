# WP #28: attached-proof reference style — `::` → `(proof name for subject)`

**Owner:** Foundation (catalog authoring). **Size:** S/M — judgment pass over
catalog source. **Risk:** low — pure surface; all spellings elaborate to the
identical proof term (spec §8.2 / §32-§33). **Base:** `origin/main @ 69570b26`
(post-#29; fetch + re-verify at pickup). **Process (operator ruling): LIGHT — no
spec enclave.** Review = Architect-terminal (surface + fidelity + the
recursive-position judgment call). Steward honesty-gates + merges.

> ### ↻ DISPOSITION (2026-07-12): BARE FORM — the #29 parser extension has landed
>
> The operator ruled **Option B**: extend the parser to admit `proof name for
> subject` as a **bare expression atom**, then do this rewrite in that clean bare
> form (**not** the mandatory-parens `(proof refl for leq_nat)` the first pass
> produced). **WP #29 landed that atom** (spec `proof_ref` @ `f64c788b`; parser
> @ `69570b26`) — so the bare form now parses and elaborates. This WP re-runs on
> the bare form.
> - **The parked candidate `29094048` (168 sites, all parenthesized) is
>   SUPERSEDED but its inventory is the starting point.** Re-run = **strip the
>   grouping parens** off the simple/bare occurrences → `proof name for subject`;
>   **keep** parens only where they aid readability of an **applied** selector in
>   a dense expression (now *optional grouping*, not required — the atom makes
>   `proof p for s a b` parse as `((proof p for s) a) b`).
> - **The operator's example instance below is the north star**: bare for the
>   simple fields (`refl = proof refl for leq_nat`), grouped only for the dense
>   applied `total`. Match that texture.

## Objective

Operator style ruling (2026-07-12): in catalog **code**, attached-proof
references should use the **readable bare selector form `proof name for
subject`** (grouping parens optional, kept only for dense applied readability),
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

- **All three spellings resolve to the identical term.** After #29, `proof name
  for subject` is a **bare expression atom** (spec §32-grammar `proof_ref`,
  §33-declarations), equal-AST to the grouped `(proof name for subject)` and to
  the `subject::name` path — all desugar to the same `Expr::EAttachedProofRef` /
  `subject::name` global. **No elaborator/parser/spec change — this is a catalog
  source rewrite only** (the parser/spec work was #29, now merged).
- **Direction:** `subject::name` → **bare** `proof name for subject` in authored
  catalog **code** (grouping parens optional — keep only for dense applied
  readability). Do not touch declaration heads (already `proof … for …`),
  `fn`/`const`, glyphs, or the Ω-partition.

## Scope — CODE references only; the `::`-documenting prose STAYS

The catalog has ~222 `::` occurrences; a blind sweep would corrupt the docs.
Classify and convert **only code**:

- **IN scope (convert):**
  - **Instance field values** — `refl = leq_nat::refl` →
    `refl = proof refl for leq_nat` (bare). (The operator's explicit locus.)
  - **Bare references** (whole RHS / argument position) — bare, no parens:
    `= leq_nat::refl` → `= proof refl for leq_nat`.
  - **Applied selectors in term position** — the atom binds tightest, so parens
    are **optional grouping**: `bool_or::eq_true_of_or (leq_nat x y) …` may be
    written bare `proof eq_true_of_or for bool_or (leq_nat x y) …` **or** grouped
    `(proof eq_true_of_or for bool_or) (leq_nat x y) …`. **Keep the grouping
    parens where they aid readability in a dense applied expression** (the
    operator's `total` field is the model); drop them for simple cases.
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

## Application-boundary rule (post-#29 — the grammar is now the atom)

`proof p for s` is a **primary atom that binds tightest**; the subject is a
single `path`, so **application binds OUTSIDE the selector**: `proof p for s a b`
parses as `((proof p for s) a) b` (NOT the retired "greedy absorb" reading
`proof p for (s a b)`). So an applied selector is **correct bare** — the parens
are optional grouping, kept only for readability. Green elaboration + the #29
crate tests (which pin exactly this nesting) are the proof. The implementer
confirms the parse per site.

## Acceptance criteria (testable)

1. Every in-scope code `subject::name` attached-proof reference is the **bare**
   selector form `proof name for subject` (grouping parens only where they aid
   dense-applied readability, per the `total` model); `::`-documenting prose
   unchanged.
2. **Semantics-preserving:** each converted reference elaborates to the same
   proof term (same instance/proof checks); no proposition/proof-term/`fn`/
   `const`/glyph change. All touched packages elaborate green.
3. **Structural completeness proof:** full `scripts/ken-cargo test --workspace`
   green + catalog acceptance nets green (a mis-resolved selector fails to
   elaborate).
4. **Whole-harness check:** if any exact-source assertion pins a converted
   reference's `::` spelling (as CAT5 pinned a glyph in #26), migrate the coupled
   assertion in the same branch (Steward-authorized, assertion-literal only) and
   grep the whole test tree; workspace green.
5. **Scope clean:** no `spec/`, `conformance/`, `crates/**` source (beyond an
   authorized coupled-assertion sync), Cargo, or lockfile change; `git diff
   --check` clean.

## Do-not-reopen guardrails

- All spellings are already valid (spec §32-§33, #29 merged) — this is style,
  not a feature. Don't touch the parser/spec/elaborator.
- Don't convert `::`-documenting prose; don't touch declaration heads, glyphs,
  `fn`/`const`, or the Ω-partition.
- Report (don't force) any proof-body self-reference that won't take the selector
  form.
