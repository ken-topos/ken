# WP · ADR-0015 build fast-follow — retire the `use` keyword

**Owner:** Language (build) · **Consumers:** none (contained; no downstream
WP depends on this) · **Reviewer:** QA · **Architect:** informational (surface
lexer/parser retirement, no kernel semantics) · **Size:** S · **Base:**
`origin/main @ 5efa317b`. **Status: READY** — Steward-framed as the
ADR-0015 build fast-follow; SPEC side (§32/§33) already merged.

## Objective

Remove the retired `use M` open-import surface form from the Rust host so the
`use` token no longer lexes/parses as an open-import declaration keyword, and
delete the now-unreachable open-import AST/elaborator path — matching ADR-0015,
which already landed on the spec side.

## Fixed inputs (settled — do not reopen)

- **ADR 0015** (`docs/adr/0015-remove-open-import-use.md`, *Accepted*) removed
  the `use M` open-import form. The three provenance-preserving forms remain:
  `import M`, `import M as N`, `import M (…)`. The ADR explicitly names this
  build fast-follow: *"retire the `use` production and, if `use` is a reserved
  keyword, free it. Confirm the keyword's current status when the build item is
  framed"* (`§ Grammar / lexer / parser`).
- **Spec is CURRENT, not stale.** `spec/30-surface/32-grammar.md` and
  `spec/30-surface/33-declarations.md` carry **no** `use`/open-import production
  on `origin/main` (verified: every `use` hit in both files is English prose).
  This WP does **not** re-derive the spec.
- **Scope-map anchors** (file:line — *verify against landed code at pickup, not
  these lines*):
  - Lexer keyword variant: `crates/ken-elaborator/src/lexer.rs:49`
    (`KwUse` enum variant) and `:447` (`"use" => Token::KwUse` lexing arm).
  - Parser decl dispatch: `crates/ken-elaborator/src/parser.rs:205`
    (`Token::KwUse => self.parse_use_decl(start)`).
  - Parser production: `crates/ken-elaborator/src/parser.rs:956-966`
    (`fn parse_use_decl`, emits `Decl::ImportDecl { kind: ImportKind::Open }`).
  - Parser prose/error strings naming `use`: doc comment
    `crates/ken-elaborator/src/parser.rs:142`; decl-keyword error list
    `crates/ken-elaborator/src/parser.rs:213`.
  - AST: `ImportKind::Open` variant `crates/ken-elaborator/src/ast.rs:355-356`;
    doc comments naming `use M` at `ast.rs:328` and `:355`.
  - Elaborator: the `ImportKind::Open` scope-binding arm
    `crates/ken-elaborator/src/modules.rs:286-290`.
  - Error doc: `crates/ken-elaborator/src/error.rs:89` — the
    `AmbiguousReference` variant's doc frames it as a *"`use`-open ambiguity"*.
    **The variant itself STAYS** — it is still reached by the N3 local×import
    clash (`spec 33 §3.3`); only its `use`-open framing is stale.
- **`ImportKind::Open` has exactly one producer** — `parse_use_decl`. Once that
  goes, the `Open` variant and its `modules.rs` arm are unreachable dead code
  and MUST be removed (the QA no-dead-code gate would otherwise fail).
- **Catalog is CLEAN** — no `.ken`/`.ken.md` source under `catalog/` spells a
  `use M` open import (verified; all `catalog/` `use` hits are prose). No
  catalog migration is in scope.

## Pinned decision (settled — do not leave open)

**`use` becomes a RESERVED word that fails closed with a migration diagnostic —
NOT a freed identifier.** Mirror the existing `type` → `KwTypeReserved`
precedent (`lexer.rs:36`/`:438`, `parser.rs:192-197`): lex `use` to a reserved
token and, at the decl-dispatch site, emit a pointed `ParseError` such as

> `` `use` is retired (ADR-0015); use `import M`, `import M as N`, or `import M
> (…)` for a provenance-preserving import. ``

**Rationale.** (1) **Codebase precedent** — `KwTypeReserved` is the established
pattern for a recently-live keyword whose surface was retired; this keeps the
lexer's keyword-table shape uniform. (2) **Migration friendliness** — the
removal is fresh, so stale `use M` sources are likely; a reserved-with-message
token turns them into a one-line pointer to `import`, whereas freeing `use`
would lex `use M` as `Ident("use") ConId("M")` and surface a confusing
"expected declaration keyword, found Ident" error. (3) **Zero future cost** —
if the operator later wants `use` available as an ordinary identifier, freeing
it is a trivial follow-up; reserving now is the strictly friendlier default
during the migration window.

*ADR-0015's "free it" language is read as non-binding on this sub-choice (it
says "**if** `use` is a reserved keyword" and defers status confirmation to the
build framing). This decision is the Steward's to flip; if the operator prefers
a fully-freed identifier, swap the reserved arm for removal from the keyword
table and adjust the parser-error AC accordingly.*

## Mandated deliverable outline

1. **Lexer** (`lexer.rs`). Repurpose the `KwUse` variant (`:49`) into a reserved
   token (e.g. `KwUseReserved`, comment `"use" — reserved (ADR-0015); retired
   open-import, not a decl keyword`), keeping the `"use" => …` lexing arm
   (`:447`) so `use` still tokenizes as a keyword rather than an identifier.
2. **Parser** (`parser.rs`). Replace the dispatch arm at `:205` with a
   fail-closed reserved arm modeled on `KwTypeReserved` (`:192-197`) emitting the
   pinned migration `ParseError`. **Delete `parse_use_decl`** (`:956-966`).
   Remove `'use'` from the decl-keyword error-list string (`:213`) and drop
   `use` from the shared doc comment (`:142`).
3. **AST** (`ast.rs`). Remove the `ImportKind::Open` variant (`:355-356`) and
   drop `use M` from the `ImportDecl` / `ImportKind` doc comments (`:328`,
   `:355`) — leaving the three provenance forms.
4. **Elaborator** (`modules.rs`, `error.rs`). Delete the `ImportKind::Open`
   match arm (`modules.rs:286-290`). Reword the `AmbiguousReference` doc
   (`error.rs:89`) to describe the surviving local×import / selective-collision
   clash (`33 §3.3`) — **keep the variant** (still reached by N3).
5. **Tests** (`es3_modules_acceptance.rs`, `dotted_module_path_parser.rs`).
   Migrate the four `use`-dependent tests off the retired form:
   - `dotted_module_path_parser.rs:59-67` (`use_accepts_a_dotted_module_path`)
     — retarget onto a surviving import form, or replace with a test asserting
     the retirement diagnostic.
   - `es3_modules_acceptance.rs` `four_import_forms_resolve_to_one_binding`
     (`:223-…`, uses `use M` at `:248`) — becomes a **three**-import-forms
     single-binding test (qualified / aliased / selective).
   - `es3_modules_acceptance.rs` `top_level_local_import_clash_is_rejected_
     latently` (`:205-221`, uses `use M` at `:210`) — re-express the still-live
     local×import clash via `import M (foo)` + a local `foo` (the rule survives;
     only its `use` spelling is gone).
   - `es3_modules_acceptance.rs` `use_open_ambiguity_rejected_naming_both`
     (`:175-200`, two colliding `use`s) — the two-open collision is no longer
     expressible; replace with two colliding **selective** imports or delete.
   - **Add** one positive test: a source spelling `use …` yields the pinned
     retirement `ParseError` (not a generic keyword error).

## Acceptance criteria

- **AC1 — `use M` no longer parses as an open import.** No parser path
  constructs `ImportKind::Open`; `parse_use_decl` is gone. `git grep
  'parse_use_decl\|ImportKind::Open'` returns nothing.
- **AC2 — retired-`use` diagnostic.** A `.ken` source containing a `use`
  declaration fails elaboration with the pinned migration message pointing at
  `import` (assert the specific `ParseError`, not just `is_err`).
- **AC3 — no dead code.** The `ImportKind::Open` variant and its `modules.rs`
  arm are removed; the crate compiles with no unreachable-pattern / dead-variant
  warning, no `gate paused` marker introduced.
- **AC4 — surviving forms + rules intact.** The three provenance imports
  (`import M`, `import M as N`, `import M (…)`) still parse and resolve; the N3
  local×import / selective-collision clash still yields `AmbiguousReference`
  (re-expressed test green).
- **AC5 — catalog layout/AST unaffected.** No `catalog/` source changes; the
  formatter/AST fixtures for existing catalog packages are untouched (this WP
  edits no `.ken.md`).
- **AC6 — validate LOCALLY TARGETED only** (operator hard rule, COORDINATION
  §12: **NO** `cargo test --workspace` — the box OOMs). Run the affected suites
  via `scripts/ken-cargo -p ken-elaborator --test es3_modules_acceptance` /
  `--test dotted_module_path_parser` (and the crate build `-p ken-elaborator`);
  CI runs the full `--locked` gate + conformance at merge. "No regression" means
  **green in CI**, never a local `--workspace` run.

## Do-not-reopen guardrails

- **Do NOT touch the namespace-clash / re-export design** — the general
  MRES-6 clash rule (#39) and the re-export surface form (#36/N5) are a separate
  Architect-design lane; this WP does not add, rename, or re-spell any import
  form, and does not resurrect `Open` under a new name for re-export.
- **Do NOT touch kernel, `Cargo.*`/lockfile, `spec/`, `docs/adr/`, or
  `.github/`.** Surface lexer/parser/AST/elaborator only.
- **Do NOT edit `catalog/` sources** — it is already clean of `use M`.

## Notes / stragglers for the Steward

- **Spec-lane straggler (NOT this build WP):** the lexical chapter's keyword
  list `spec/30-surface/31-lexical.md:352` still lists `use` (and the paragraph
  at `:358-364` records `view` as retired / `type` as reserved but omits
  `use`). §31 is the Spec enclave's, not Language's — flag it as a small
  spec-side follow-up, out of scope here.
- **Slightly more than mechanical:** the test edits are a genuine **migration**,
  not deletions — three `es3_modules` tests exercise still-live rules
  (single-binding identity, N3 latent clash, ambiguity naming) that must be
  re-expressed against the surviving import forms, plus one new diagnostic test.
  The source edits proper are fully mechanical.
