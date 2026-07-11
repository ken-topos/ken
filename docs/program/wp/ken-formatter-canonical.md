# WP ken-formatter-canonical — mandated canonical formatter for `.ken` / `.ken.md`

**Owner:** Language team (parser/printer). **Steward-framed** (2026-07-11). Base:
`origin/main` (re-verify cites at pickup). **Inner-ring** for the tool
(`crates/ken-*`); the whole-catalog reformat pass is mechanical/outer-ring.
**Soundness-relevant** in one specific way: a formatter that changes the parsed
meaning of a program is catastrophic → **@architect gate on
semantics-preservation** + Spec on the canonical form. **BUILD not yet kicked** —
mechanism-shape routes to Architect/Spec first (see seams); Steward releases.

## Context — promotes a standing proposal + DAG node to a build WP

This has been tracked but never framed:
- **`ds-campaign-judgment-log.md` §D1** — operator (Pat) proposal, 2026-07-10:
  a strict auto-formatter for Ken source in the `gofmt`/`rustfmt`/`black` mold
  (one canonical style, mechanically enforced). Trigger: **no line-length
  discipline on Ken code today** — `EffectfulClasses.ken.md` has many 200+ col
  lines, `catalog/guide/decomposition-abstraction.ken.md:129` ~295 cols (the
  80-col rule is prose-only; code fences are exempt).
- **`05-implementation-dag.md` — `L-fmt` milestone:** "mandated formatter
  (ASCII↔Unicode canonical) + confusable-resistant lexer (TR39)," tied to spec
  **`31 §1a/§1b`**. So a canonical formatter is already architected + spec-
  referenced.
- **`CAT-5-parsing-syntax-diagnostics.md`** already proves **formatter
  idempotence** laws for a small grammar (AC6) — the law foundation exists.

**Operator decisions (2026-07-11), binding on this frame:**
- **Scope = FULL canonical layout** (not wrapping-only, not canonicalization-core-
  only): one canonical style covering ASCII↔Unicode canonicalization + TR39
  confusable resistance + line-wrapping/column discipline + indentation + spacing
  + alignment + fence normalization, for both `.ken` and `.ken.md`.
- **Enforcement = STRICT CI GATE immediately** (`ken fmt --check` as a pre-merge
  gate): every `.ken`/`.ken.md` must be canonically formatted to merge, from the
  landing.

## Goal

A `ken fmt` tool + a strict CI gate. `ken fmt <file>` rewrites Ken source to the
one canonical form; `ken fmt --check` fails on any non-canonical file (the gate).
Covers `.ken` and the **code fences of `.ken.md`** (prose untouched — the 80-col
prose rule is separate and stays). Landed **together with a whole-catalog reformat
pass** so the strict gate is green on day one (see Sequencing — this is the
blast-radius crux).

## Design seams — Architect/Spec to shape before build (flag, don't guess)

1. **Canonical form (Spec `31 §1a/§1b`, Architect+Spec).** The exact canonical
   choices: ASCII↔Unicode **direction** (canonicalize to the read-optimized
   Unicode forms — `→`/`λ`/`Ω` — or to ASCII?), operator spacing, indent unit,
   alignment rules, fence normalization. `31 §1a/§1b` + the read-optimized-
   notation intent govern; Spec pins the normative canonical-form spec, CV verifies
   tool↔spec.
2. **Build vs adopt (Architect).** Ken's surface is bespoke → **build**, but
   *reflect* a proven pretty-printer algebra (Wadler/Prettier-style
   `Doc`+`group`+`nest` with a width target) rather than invent a layout engine —
   reflect-don't-extend on the algorithm. Confirm the approach + where it lives
   (reuse the existing printer from CAT-5's parser/printer, don't fork it).
3. **TR39 confusable-resistant lexer (soundness-relevant).** The `L-fmt` node
   pairs the formatter with a confusable-resistant lexer — canonicalizing
   confusable codepoints is security-relevant (homoglyph identifiers). Scope
   whether the lexer lands in this WP or as a paired prereq.
4. **Ownership of the whole-catalog reformat.** The tool is Language's; the
   one-time catalog reformat is mechanical (run the tool) — Librarian or the tool
   itself. Decide who drives + verifies the reformat pass.

## Scope

- `crates/ken-*` — the `ken fmt` subcommand: canonical printer (reflecting the
  existing parser/printer), `--check` mode, `.ken.md` fence-aware reformatting.
- The **whole-catalog + docs reformat pass** (mechanical, tool-run), landed
  atomically with the gate.
- The **CI gate wiring** (`ken fmt --check` in the pre-merge pipeline).
- Spec: the normative canonical-form clause (`31 §1a/§1b`), CV verifies tool↔spec
  — a paired Spec deliverable (co-lands).

### Out of scope

- Changing the prose 80-col rule (stays; separate from code formatting).
- Any semantic/language change — the formatter is **whitespace + canonical-token
  only**; it must never alter the parsed AST.
- LSP/editor integration, doc generation (later tooling — `CAT-5` out-of-scope
  list).

## Acceptance criteria

- **AC1 — semantics-preserving (the soundness AC).** For every reformatted file,
  **parse(before) ≡ parse(after)** (AST-equal modulo the canonical-token
  normalization the spec sanctions) — asserted by a round-trip test over the whole
  catalog, not a sample. A formatter that changes meaning is a hard reject.
- **AC2 — idempotence.** `fmt(fmt(x)) = fmt(x)` for every file (the CAT-5 AC6 law,
  now over the real grammar). The `--check` gate relies on it.
- **AC3 — canonical + confusable-safe.** Output matches the Spec `31 §1a/§1b`
  canonical form; confusable codepoints are normalized (or rejected) per TR39.
- **AC4 — the gate is real + green.** `ken fmt --check` is wired into CI as a
  pre-merge gate, and the **entire catalog + docs are reformatted in the landing**
  so the gate passes on day one (no pre-existing violations grandfathered
  silently — if any file is deliberately exempt, `log`/document it).
- **AC5 — `.ken.md` fences only.** Prose in `.ken.md` is byte-untouched; only code
  fences are reformatted. Verified on a literate file with long prose + long code.
- **AC6 — build.** Workspace-green in CI; targeted local builds only.

## Sequencing — the blast-radius crux (Steward-owned)

**Strict-gate + full-canonical + whole-catalog-reformat = the single
largest-blast-radius change on the roadmap.** The reformat touches *every* catalog
and doc file, so it **conflicts with any in-flight catalog/spec WP** and, once the
gate is on, **any unformatted file blocks all merges.** Therefore this WP **must
land in a catalog-quiet window** — no concurrent catalog/spec WPs mid-flight — and
the reformat + gate land **atomically** in one merge. The Steward schedules that
window (likely after the current compare/`Ord` path bricks + the case_eq adoption
follow-up land, unless the operator wants a deliberate freeze sooner). Do **not**
kick the build until the canonical form (seam 1) is Architect/Spec-fixed and the
landing window is chosen.

## Gate

Language ring → **@architect gate** (semantics-preservation AC1 + the pretty-
printer approach + canonical-form soundness) → **Spec/CV** (the `31 §1a/§1b`
canonical-form clause, non-terminal for the spec portion) → `git_request` to
Steward → **CI-gated** merge **in the scheduled quiet window**. Own the retro.
**No WP-token identifiers in production source.** Re-verify cites at pickup.
