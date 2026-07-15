# L4-export — the `export` re-export declaration

**Owner:** Team Language (language-leader → implementer → QA).
**Source:** spec `30-surface/33-declarations.md §3.2/§4` (normative, ES3) +
`30-surface/32-grammar.md §…` (grammar) + the L4 module system in
`crates/ken-elaborator/src/modules.rs` (landed).
**Size:** M · **Risk:** low · **Deps:** none (import / `pub` / loader landed).
**Gate:** Language QA + Architect §14 (soundness) + CV Spec-vote (conformance
path). **FULL CI** (touches `crates/`). Publisher on a RESOLVED Decision.

> ### ✅ RELEASE STATUS — RELEASED (operator greenlight, 2026-07-15)
> **Decision 2 is RESOLVED.** Pat concurred with the Steward's toolchain-ordering
> recommendation and said *proceed*: this `export` WP is the **next** Language-lane
> build (before CC9; package-manager later; PX3 stays behind CC9). This is a
> **settled fixed input** — do not re-ask (`[[settled-operator-approval-is-a-fixed-
> input-never-re-ask]]`). Released to the Language ring via the Handoff Gate +
> kickoff. Anchors below re-grounded against `origin/main @ 21e089ae` at release.

## Objective

Wire the dedicated **`export` re-export declaration** into the surface pipeline,
completing the L4 module system's public-interface surface. The `pub` export
*marker* is already landed (`parser.rs:1035`, `parse_pub_decl`; dispatch
`parser.rs:223`); this WP adds the
two `export_decl` forms the spec makes normative but the parser does not yet
accept (no `export` token, no `ExportDecl` AST node, no dispatch arm):

- **`export M (foo, Bar as baz)`** — a **facade** republish: takes selected
  names directly from `M`'s exports, requires no prior `import M`, and **does
  not bind them in the current module's body scope**. It is itself a loader
  dependency edge to `M`.
- **`export foo, Bar as baz`** — an **in-scope** republish of names already
  resolved in the current module (imported or locally defined).

Both permit per-name `as` renaming, which changes only the **published surface
name**, never the canonical identity.

## Fixed inputs — settled; DO NOT reopen

1. **Semantics are normative** (`33 §3.2`, ES3): the two forms; facade =
   loader-edge + publish-**without**-body-binding; in-scope = republish
   already-resolved names (an unresolved listed name is an ordinary
   unresolved-name surface error); per-name `as` renames the published name
   only, canonical identity unchanged; a facade **requires** a selection list.
   `§4` = visibility / public interface (private-by-default, settled).
2. **Grammar is pinned** (`32`): `export_decl ::= "export" ( ModPath
   selection_list | export_item_list )`; `export_item ::= name ("as" name)?`.
3. **The keyword spelling is SETTLED.** `export` / `as` / the selection-list
   syntax mirror the **already-landed `import` vocabulary**; OQ-syntax module
   keywords are marked **"Not iterating"** (`90-open-decisions.md:222-223`). Do
   **not** propose an alternate spelling or relitigate OQ-syntax.
4. **Zero `trusted_base()` delta.** The module system **elaborates away** to the
   kernel's flat append-only `Σ` (`33` header; `30-taxonomy.md §1.1`, ES1
   minimality). **No kernel change.** If any change appears to touch the kernel
   or the trusted base, STOP — it is a misimplementation.
5. **Facade `export M (…)` is a loader dependency edge** — role-blind dotted-path
   identity, identical to `import`; it participates in lazy discovery and
   `ImportCycle` detection (`33 §3.2`).
6. **Reuse the landed machinery — do NOT build a parallel resolver or a second
   public-interface representation.** The pieces already exist:
   - `ModuleState.exports` pubmap (`modules.rs:52`) — *"the export table IS the
     interface"*; today built from `pub` decls. This WP extends its
     construction.
   - `resolve_ref` (`modules.rs:183`), `apply_import` (`263`), `load_unit`
     (`417`), the dependency-edge collector `imported_module_paths` (`311`).
   - `parse_dotted_module_path` (`parser.rs:981`) and the `LParen` selective-list
     loop with `ImportItem { name, rename }` + `as` (`parser.rs:988-1007`).
7. **Scope:** `crates/ken-elaborator` + its tests + `conformance/` corpus only.
   **No new dependency. No package-manager work** — L4's package-manager half
   stays deferred (operator).

## Mandated deliverable outline

Each item ends in a concrete implementable choice, not a survey.

1. **Lexer** (`lexer.rs`) — add `Token::KwExport` for `"export"`, mirroring
   `KwPub` (`:50`) / `KwImport`. One token variant + one keyword-table entry.
2. **AST** (`ast.rs`) — add `Decl::ExportDecl { form, span }` with
   `enum ExportForm { Facade { module: String, items: Vec<ImportItem> },
   InScope { items: Vec<ImportItem> } }`. **Reuse `ImportItem { name, rename }`**
   — the `export_item` shape is identical. Extend `Decl::name()` / `span()` /
   the `Pub`-style helpers following the `ImportDecl` arm (`ast.rs:415` name(),
   `:455` span(); `ImportItem` struct `:374`): an `ExportDecl` has no declared
   name of its own.
3. **Parser** (`parser.rs`) — add the dispatch arm `Token::KwExport =>
   self.parse_export_decl(start)` beside the `KwImport` / `KwPub` arms
   (`~:215-222`). `parse_export_decl`: consume `export`; **disambiguate by the
   grammar's own rule** — it is a **facade IFF** a dotted `ModPath` is
   *immediately followed by `(`* (a selection list); otherwise it is an in-scope
   `export_item_list`. (`export M` with no `(` is **not** a facade — it is an
   in-scope item list of the single name `M`.) Reuse `parse_dotted_module_path`
   for the ModPath and the `:988-1007` selective-list loop (with `ImportItem` /
   `as`) for both forms' item lists.
4. **`modules.rs` export-table construction** — when a unit's decls are expanded
   into its `exports` pubmap, handle `Decl::ExportDecl`:
   - **InScope:** for each item, `resolve_ref(scope, exports, item.name, span)`;
     on `Ok(canonical)` insert `item.rename.unwrap_or(item.name) → canonical`
     into **this** unit's pubmap; on `Err`, propagate the ordinary
     unresolved-name surface error. Does **not** re-bind (already in scope).
   - **Facade:** ensure the loader edge to `module` is followed (existing
     `load_unit` / discovery path); then look up `exports.get(module)?.get(
     item.name)` → canonical, and insert `item.rename.unwrap_or(item.name) →
     canonical` into **this** unit's pubmap; a name absent from `M`'s exports →
     surface error. **Does NOT bind item names in the current unit's body
     scope** — the load-bearing facade-doesn't-bind rule (AC2).
5. **Loader edge** (`modules.rs`) — extend the dependency-edge collector
   (`imported_module_paths`, `:311`, or its equivalent) so facade `export M (…)`
   module paths are collected: lazy discovery follows them, and a cycle closed
   through a facade export raises `ImportCycle` with the correct edge-order
   payload.
6. **Formatter / lossless** (`lossless.rs`, `layout.rs`, kenfmt) — round-trip
   `Decl::ExportDecl` (both forms + `as`) to a fixed point; the lossless CST
   preserves it. **Update the kenfmt test oracle** `TOP_LEVEL_PREFIXES`
   (`crates/ken-elaborator/tests/kenfmt_c_capstone.rs:207-211`): **add
   `"export"`**, and — folding the parked oracle-hygiene item (foundation-leader
   `evt_6c82z4w6z1e92`) — **add accepted `"let"` and remove retired/reserved
   `"use"`** from the same set.
7. **Tests + conformance** — the AC cases below, each independent; add
   `conformance/` fixtures for both `export` forms (this is what pulls the CV
   Spec-vote).

## Acceptance criteria (testable)

- **AC1 — parse.** `export M (foo, Bar as baz)` → `ExportForm::Facade`;
  `export foo, Bar as baz` → `ExportForm::InScope`; both with the correct items
  and renames.
- **AC2 — publishes-vs-binds discriminator pair (§7).** On the *same* importer:
  an in-scope `export foo` makes `foo` visible to an importer, **while** a facade
  `export M (foo)` in a module `E` does **not** bind `foo` in `E`'s own body (a
  body reference to `foo` in `E`, with no `import M`, is still `UnboundName`).
  The pair pins the distinction — a flipped implementation fails **both** arms.
- **AC3 — in-scope unresolved.** `export foo` where `foo` does not resolve in
  scope → `UnboundName` surface error (not a silent drop).
- **AC4 — facade not-exported.** `export M (foo)` where `foo ∉ M`'s exports →
  surface error.
- **AC5 — rename ≠ new identity.** `export foo as bar` publishes under `bar`; an
  importer using `bar` resolves to the **original canonical identity** of `foo`.
- **AC6 — facade edge is a loader dependency.** A facade `export M (…)` closing
  a cycle raises `ImportCycle` with the correct edge-order payload.
- **AC7 — formatter.** kenfmt round-trips both `export` forms to a fixed point;
  the lossless CST preserves them; `TOP_LEVEL_PREFIXES` includes `export` + `let`
  and excludes `use`.
- **AC8 — no trust / no spill.** **Zero `trusted_base()` delta**; no change
  outside `ken-elaborator` (+ its tests + `conformance/`); no new dependency; CI
  green (FULL workspace **in CI**, never a local `--workspace` run — §12).

## Do-not-reopen guards

The fixed inputs above are guards: the spelling is settled (do not re-spell);
the facade-doesn't-bind behavior is **normative, not a design choice**; there is
**zero trusted-base delta** (no kernel touch); reuse the landed `exports` /
`resolve_ref` / `load_unit` machinery (no parallel resolver); scope is bounded to
the elaborator crate + its tests + conformance; **no package-manager work.**

## Sizing & risk

**M**, low risk — the module system elaborates away, and every piece parallels
the already-landed `import` implementation. The three real risks each have a net:
(a) the facade publishes-without-binding distinction → **AC2** discriminator
pair; (b) missing the formatter / lossless layer (a surface-form addition trips
the static oracle) → **AC7** + the `TOP_LEVEL_PREFIXES` fold; (c) facade-edge
cycle detection → **AC6**.
