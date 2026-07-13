# WP kenfmt C — capstone: `ken fmt` tool + atomic whole-catalog reformat + strict gate

- **Series:** kenfmt (Ken's gofmt/black). **Capstone** — P0 / S / B1 / B2 / B3 /
  B4 are all landed; C applies the engine and turns on the gate.
- **Owner:** Language ring (owns the formatter). **Reviewer:** **Architect gate**
  (AST/semantics-preservation is the soundness AC — a formatter that changes
  parsed meaning is catastrophic).
- **Size:** M. **Risk:** medium-mechanical — the *engine* is proven by the
  B1–B4 gates; C's risk is (a) the CLI wrapper and (b) that the one-time reformat
  produces a large diff that must be AST-preservation-verified wholesale.
- **Branch:** `wp/kenfmt-c-capstone` (off `origin/main @ 96f5c45b`).
- **Deps:** B1–B4 (all merged). **Freeze:** merges ONLY in the Steward-scheduled
  **catalog-quiet window** (open on kickoff; see "Freeze" below).

## Objective

Land, in **ONE atomic merge** inside a catalog-quiet freeze window: (1) the
**`ken fmt` CLI tool** (in-place reformat + `--check` mode), (2) the **whole-
catalog + rosetta canonical reformat** it produces, and (3) the **strict
`ken fmt --check` CI gate** that blocks any non-canonical file — green on arrival,
no grandfathered violations.

## Fixed inputs (settled — do NOT re-decide)

Cite the kenfmt work program (`docs/program/wp/kenfmt-work-program.md §C` +
guardrails) and the canonical-form spec (`spec/30-surface/31-lexical.md §1a–d` +
`§Literate`). The B-series already settled the canonical form; C **applies** it.

- **The formatter is whitespace + canonical-token only** — it must **never alter
  the parsed AST** (the soundness AC). This is the whole point of the B1–B4
  parse-/elaboration-preservation gates; C inherits them at whole-corpus scale.
- **One canonical form. No escape hatch, no config, no `--width` knob** exposed
  (88-col is fixed internally). Verbatim regions (string/comment/foreign/temporal
  payloads) are **semantic, not style escapes** — untouched.
- **No literal normalization; no import/field/row/instance sorting** — source
  order is resolution-relevant and part of the author's explanatory order.
- **`.ken` gets B3 layout; `.ken.md` gets B4 role-gated splicing** (parseable
  bodies → full B3; only non-parseable `ken ignore`/`ken reject` → B2 token-canon;
  non-parseable `ken`/`ken example` → hard error). The tool dispatches by
  extension onto the **existing** `layout::format_ken` / `literate::format_ken_md`
  entry points — **do not** build a third formatting path.

## The corpus C reformats (exact)

**Catalog (15):** the 14 `catalog/**/*.ken.md` + the 1
`catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken`.
**Rosetta (16):** `examples/rosetta/**/*.ken`.
(Enumerated in the kickoff; the `--check` gate covers exactly this set. If the
gate globs, the glob must enumerate every Ken-source root — catalog + rosetta —
and nothing outside them.)

## Mandated deliverable outline (each item ends in a concrete choice)

1. **`ken fmt` CLI subcommand** in `ken-cli`: `ken fmt <paths…>` rewrites each
   file in place to canonical form; `ken fmt --check <paths…>` writes nothing,
   exits **non-zero** if any file is non-canonical, and **prints each offending
   path** (and ideally a diff or first divergent line). Dispatch `.ken` →
   `format_ken`, `.ken.md` → `format_ken_md`; a parse-error in a must-parse body
   is a **named, loud** failure, never a silent skip. No new formatting logic —
   this is a thin CLI over the landed engine.
2. **The whole-catalog + rosetta reformat**: run `ken fmt` over the exact corpus;
   commit the canonical result. This is a **large but mechanical** diff — its
   correctness is the AST-preservation gate below, not visual inspection.
3. **Strict CI gate**: wire `ken fmt --check` over the corpus into the workspace
   CI so any future non-canonical file fails the build. **Green day-one** — no
   silently grandfathered violations; any deliberate exemption is **`log`ged**
   explicitly (there should be none).
4. **Fold the two tracked carries** (subsume-don't-proliferate):
   - **task #44** — fold `layout.rs::token_text`'s 11-entry glyph map onto B2's
     `canonical_token_spelling` (one home for the §1b table), at this pipeline-
     integration point.
   - **Architect B4 non-blocking notes** — add the **empty-fence first-pass
     discriminator** test, and add the one-line `format.rs` doc note that the
     "source is never re-lexed" invariant now has the sanctioned
     `canonicalize_lexed_tokens` exception (the B4 non-parseable fallback).

## Acceptance criteria

- **AC1 — the gate is real and green day-one.** `ken fmt --check` over the exact
  corpus passes on the reformatted tree, and **fails** on a deliberately
  de-canonicalized fixture (assert the non-zero exit + the offending path is
  named). No grandfathered violation; no exemption unless `log`ged.
- **AC2 — whole-corpus AST/elaboration preservation (the soundness AC).** For
  **every** reformatted file, the parsed AST (and elaboration outcome) is
  identical pre- and post-reformat — the B3/B4 parse-preservation property run
  over the *actual reformat diff*, both `.ken` and `.ken.md`. A single meaning-
  changing reparse fails the WP.
- **AC3 — idempotence.** `ken fmt` on already-canonical output is a byte no-op
  over the whole corpus.
- **AC4 — `.ken.md` prose/marker byte-identity** (B4's property) holds over the
  reformat: only recognized fence bodies change; prose, markers, roles, order
  untouched.
- **AC5 — representative review.** The reformat diff is reviewed across the
  representative constructs the work program names: dependent telescopes, class
  laws, deeply nested proofs, and **all four fence roles** — plus the Architect's
  8 semantic-preservation gates from the B-series review.
- **AC6 — literal locked-workspace CI green** on the exact final SHA:
  `cargo build --workspace --locked && cargo test --workspace --locked`, with the
  new `ken fmt --check` gate active.

## Do-not-reopen guardrails

- **No language/semantic change** — whitespace + canonical-token only; never
  alters the parsed AST.
- **No escape hatch, no config, no width knob.** One canonical form.
- **No literal normalization; no import/field/row/instance sorting.**
- **No LSP/editor integration, no doc generation** — later tooling, out of scope.
- **`canonical_unicode` is a seed, not a foundation** — do not extend the raw-byte
  path into the real formatter.
- **Do not** reformat anything outside catalog + rosetta (no `spec/` fences, no
  `docs/`, no `local/`).

## Freeze (Steward-managed)

The Steward **opens the catalog-freeze window at kickoff** and announces it fleet-
wide: **no catalog change may merge until C lands.** Concurrent lanes are
partitioned by artifact — `crates/`-only and `spec/`-only lanes (e.g. CLI I-1 in
Runtime) run alongside; **catalog-touching lanes are held** (e.g. §5.2 Bytes-sig
reconcile, Program II package floor). Any parked catalog change (e.g. a
Librarian doc commit) rebases + `ken fmt`s onto the reformatted `main` **after** C
merges. The Steward closes the window the moment C is on `origin/main`.

## Flow

Language builds one lane (CLI tool + reformat + gate) → **Architect gate**
(AST-preservation is the soundness AC) → `git_request` to the Steward →
honesty-gate + CI-poll publish **in the freeze window** → **C CLOSED** (retros
in). This closes the entire kenfmt series.
