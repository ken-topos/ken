# SURF-def-refinement — replace the `type` keyword with `def`

**Owner:** Team Language (leader → implementer → QA)
**Kind:** surface keyword swap (parser · lexer · elaborator · formatter ·
spec · conformance). **No kernel change.**
**Size:** S–M. **Risk:** low (semantics unchanged), with one grammar-hazard pin.
**Base:** cut `wp/SURF-def-refinement` from `origin/main`
(`git rebase origin/main` before working). Frame authored by the Steward
(operator-directed); **does not require Spec-enclave elaboration** — it goes
straight to Team Language. The Spec enclave enters only at the **merge gate**
(diff touches `spec/`+`conformance/` → CV Spec vote + Architect soundness vote).

## 1. Objective

Rename the type-level **definition** keyword from `type` to **`def`**, with the
identical grammar, semantics, and elaboration. The construct is unchanged:

```
def ConId tyvar* "=" type      -- was: type ConId tyvar* "=" type
```

The right-hand side is any type expression — a **refinement** `{ x : A | φ }`, a
Σ/Π abbreviation, or a plain alias. Nothing about what the RHS may be, how it
elaborates, or the kernel changes. Only the surface keyword changes.

### Why (fixed rationale — do not reopen)

A mathematical *definition* is inherently a **refinement**: "an X is a Y such
that P" — a *continuous* function, a *prime* number, a *sorted* list — a base
narrowed by conditions. That is exactly what this construct does; a plain alias
is the **zero-condition** case ("define, narrowing by nothing"). `def` is the
math-faithful spelling of that slot. This is an **operator decision**
(2026-07-09).

**This supersedes the `SURF-def-named-definitions` outcome ("no new `def`
keyword") and does NOT reopen it.** That WP rejected `def` as a *general/value*
definition duplicating `const`/`fn`/`type`. This WP is different in kind: `def`
takes the **refinement/type slot** (replacing `type`), never the value
slot. No contradiction — the earlier objection (duplication) does not apply,
because `def` here has exactly one meaning and it is not `const`/`fn`'s.

## 2. Fixed inputs (pinned — settled, not for the team to relitigate)

- **`def` = the type-level definition/refinement construct**, semantically
  identical to today's `type`. Same production shape, same RHS grammar, same
  elaboration.
- **`type` becomes a reserved word, not a free identifier.** Remove it as the
  declaration keyword; keep it **reserved** (rejected as an identifier) to avoid
  confusion and preserve future optionality. (Steward scope call.)
- **Untouched — explicit guardrails (§5).** The `const`/`fn`/`proc` purity split
  (SURF-1), `data`/`record` constructions, `prop`/`lemma`/attached-`proof`
  (SURF-named-proof), the refinement type former `{ x : A | φ }` and its
  elaboration, and the **`Type` universe**. None of these change.
- **No kernel delta.** No new `Term`/`Decl` variant, no `trusted_base` change,
  no elaboration-target change. `def` elaborates to precisely what `type`
  elaborated to. State this as a hard AC (§4).

## 3. ★ The grammar hazard — pin before touching anything

`type` occurs in the spec/grammar in **three distinct roles**; this WP renames
**only the first**:

| Role | Example | Action |
|---|---|---|
| **keyword terminal** (the declaration) | `"type" ConId tyvar* "=" type` | **RENAME → `"def"`** |
| **grammar nonterminal** (type-expression category) | `type ::= …`, `binder ":" type`, `(":" type)?` | **DO NOT CHANGE** |
| **`Type` universe** (the sort) | `"Type" level?`, `{A : Type}` | **DO NOT CHANGE** |

A naive text `s/type/def/` **will break the grammar and every signature**
(`(x : type)` metavariable, `Type` universe). The change is surgical: only the
keyword terminal `"type"` in a *declaration* position becomes `"def"`. Every
appearance of `type` as the type-expression nonterminal and every `Type`
universe stays. Verify each edited site against this table.

## 4. Mandated deliverables + acceptance criteria

Each deliverable ends in a concrete, checkable outcome.

1. **Lexer (`crates/…` + `spec/30-surface/31-lexical.md`).** `def` is a keyword;
   `type` is removed as a keyword and **reserved** (not a valid identifier).
   `Type` (universe) untouched. **AC:** `def` lexes as keyword; a program using
   `type` as an identifier is rejected; `Type` still lexes as the universe.
2. **Parser / grammar (`crates/…` + `spec/30-surface/32-grammar.md`).** The
   declaration production `"type" ConId tyvar* "=" type` becomes
   `"def" ConId tyvar* "=" type`. `type` **nonterminal** and all `(":" type)`
   uses are unchanged (§3). **AC:** `def Foo a = { x : List a | … }` parses;
   `def Bar = SomeAlias` parses; `type Foo = …` **no longer parses** (the
   discriminating negative).
3. **Elaboration (`crates/…` + `spec/30-surface/39-elaboration.md`).** `def`
   elaborates to the **same** as `type` did. **AC:** a `def` refinement and a
   `def` alias produce byte-identical elaborated terms to the pre-rename `type`
   forms; **zero `trusted_base` delta**; no new `Term`/`Decl` variant (grep the
   diff — kernel crates untouched).
4. **Declarations chapter (`spec/30-surface/33-declarations.md`).** Rewrite the
   `type T … = …` bullet as `def T … = …`, framed as "a **definition**: a base
   type narrowed by conditions (a refinement / Σ / Π); a plain alias is the
   zero-condition case." Keep the inline refined-field examples
   (`{ n : Int | n ≥ 0 }`) as-is. **AC:** the chapter no longer references a
   `type` keyword; the lattice reads `def` / `data` / `record` / `const` `fn`
   `proc` / `prop` `lemma` `proof` / `class` `instance`.
5. **Formatter (L-fmt).** The mandated formatter canonicalizes any surviving
   `type` declaration to `def`; ASCII↔Unicode canonical form and the
   confusable-resistant (TR39) lexer both recognize `def`. **AC:** formatting a
   file with a legacy `type` decl emits `def`; round-trip is idempotent.
6. **Diagnostic (should-have).** When `def` is used where a *value*
   is meant (e.g. `def double x = x * 2` — lowercase head / value RHS), emit a
   steering error: *"`def` defines a type (refinement or alias); use `fn` for a
   function or `const` for a value."* **AC:** that program yields the steering
   diagnostic, not a bare parse error. If cheap, land it; if it balloons scope,
   split to a follow-up and note it.
7. **Conformance seeds (`conformance/surface/declarations/`).** A seed pair:
   `def` refinement + `def` alias **parse and elaborate**; `type Foo = …` is
   **rejected** (discriminating negative). Optionally cover the §6 diagnostic.
   **AC:** seeds pass against the built implementation.
8. **Migration (land-together).** Grep `spec/` and `packages/` for the `type`
   **keyword** (declaration position only) and rewrite to `def`. The catalog has
   **zero** `type`-keyword declarations today (verify), so this is spec-example
   text plus stray. **AC:** `cargo test --workspace` green; no `type`-keyword
   declaration remains in `spec/` or `packages/`; the `type` nonterminal and
   `Type` universe are untouched everywhere.
9. **Catalog style-guide note (`docs/program/07-catalog-style-guide.md`,
   optional).** One short "definitions are refinements" note: when to reach for
   `def` vs `data` vs `prop` vs `class`. Non-blocking; fold if cheap.

**Global no-regression AC:** `scripts/ken-cargo test --workspace` green (a
grammar/lexer change's blast radius is workspace-wide — validate the workspace,
not just the touched crate).

## 5. Do-not-reopen guardrails

- Do **not** touch the `const`/`fn`/`proc` purity split, `data`/`record`,
  `prop`/`lemma`/`proof`, the refinement type former, or the `Type` universe.
- Do **not** add a new kernel `Term`/`Decl` variant or change `trusted_base`.
  If any change appears to require a kernel edit, **stop and escalate to the
  Steward** — that means the rename isn't semantics-preserving, which it is.
- Do **not** re-litigate whether `def` should be a general/value definition —
  it is the refinement/type slot only (const/fn own values).
- Do **not** rename the `type` **nonterminal** or `Type` **universe** (§3).

## 6. Cross-language note (enclave to confirm at the gate; not load-bearing)

For the record and the CV/Architect gate: both spellings are precedented across
the family — **`type`** for named refinements (Liquid Haskell, F*, Dafny) and
**`def`/`Definition`** for named subsets (Coq, Lean). This is the Steward's
recollection, **not** repo-grounded; the enclave (permissive-ref access) may
confirm at review if it wishes. It is context, not an acceptance criterion — the
decision rests on Ken's own design logic (§1), not on prior art.

## 7. Sequencing

Steward frames (this doc) → **Team Language builds directly** (leader →
implementer → QA on `wp/SURF-def-refinement`) → merge Decision pulls **Architect
(soundness) + CV (Spec, spec/+conformance/ paths)** → publisher path merges →
§10 retros → close. No Spec-enclave elaboration step.
