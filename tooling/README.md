# `tooling/` — ecosystem integrations for Ken

This directory holds **integrations that teach external tools about Ken** —
syntax highlighters, editor modes, language servers, and parser grammars. They
are *not* part of the compiler (`crates/`), the spec (`spec/`), or the catalog
(`catalog/`); they are the surface where Ken meets the wider developer
ecosystem. Each lives in its own subdirectory.

## Present

| Integration | Path | What it feeds |
|---|---|---|
| **highlight.js** | [`highlight-js/`](highlight-js/) | Client-side syntax highlighting for `.ken` / `.ken.md` in the browser (Markdown readers, docs sites, blogs). |

## Anticipated (demand-pull — added when a real consumer needs one)

Named here so the category's shape is legible, not because they are owed. Per
the demand-pull discipline (`docs/program/06-catalog-campaign.md`), each lands
when something concrete pulls it in, not speculatively.

- **`textmate/`** — a TextMate (`.tmLanguage`) grammar. This is what **GitHub
  Linguist** consumes to colour `.ken` blocks in repos and gists, and what
  VS Code's default highlighting uses.
- **`tree-sitter/`** — a tree-sitter grammar: incremental parsing for editor
  code-navigation, Neovim, and GitHub's newer code-nav path.
- **`lsp/`** — a Ken language server (diagnostics, hover, go-to-def) speaking
  LSP, reusing the real elaborator (`crates/ken-elaborator`) rather than
  re-implementing it.
- **`emacs/`** — a `ken-mode.el` major mode (font-lock + indentation, ideally an
  `lsp-mode` client over the server above).

## Grounding discipline (why these don't rot)

Every syntax definition here is a **projection of the real language**, not an
independent guess. Each is derived from the authoritative sources and **carries
a header comment naming its grounding source** so it can be re-synced when the
language moves:

- **Tokens** (keywords, operators, comment syntax, literal forms, the
  `ConId`/`Ident` uppercase split, Unicode operator aliases like `Ω`→`Omega`):
  the lexer, `crates/ken-elaborator/src/lexer.rs`.
- **Constructs** (declaration keywords, `.ken.md` literate fences, effect-row
  syntax): the surface spec, `spec/30-surface/`.

A highlighter or grammar is allowed to be **coarser** than the compiler (it
categorises for the eye; it does not type-check), but it must not assert syntax
the lexer does not accept. When the lexer gains or retires a token, the matching
integration is updated in the same spirit as the extension-gated fence-check
keeps guide examples honest.
