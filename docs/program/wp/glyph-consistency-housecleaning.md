# WP #26: glyph-consistency housecleaning (catalog Ken sources)

**Owner:** Foundation (catalog authoring). **Size:** S/M — mechanical,
formatter-driven. **Risk:** low — pure surface; accepted-ascii → canonical
unicode that lexes to the *identical* token stream (zero AST/elaboration/trust
delta). **Base:** `origin/main @ 6234888a` (fetch + re-verify at pickup).
**Process (operator ruling): LIGHT — no retro, no spec enclave.** Review =
Architect-terminal light surface/fidelity pass (cheap insurance for a
corpus-wide surface change — the match-arm migration had a source-root scope
gap; keep it, keep it quick). Steward honesty-gates + merges.

## Objective

The catalog Ken sources mix ascii and unicode operator glyphs (operator noticed;
approved a consistency pass 2026-07-12). Normalize every **Ken code fence** to
the **canonical unicode** spelling. **Operator ruling: unicode canonical, ascii
still accepted** (the lexer/formatter already accept both — this canonicalizes
the *sources*, it does not remove ascii acceptance).

## Fixed inputs (settled — do NOT reopen)

- **Canonical direction:** unicode canonical, ascii accepted. Settled by
  operator. Do not relitigate.
- **The authoritative mapping is `crates/ken-elaborator/src/format.rs`
  `canonical_unicode` — cite it, do not restate or re-derive it.** It is the
  language's own lexical normalizer and already: (a) canonicalizes accepted
  ascii operators + the word-idents (`Omega`→`Ω`, `Sigma`→`Σ`, `Pi`→`Π`,
  `forall`→`∀`, `exists`→`∃`, `not`→`¬`, `level`/`l`→`ℓ`; `|->`→`↦`, `->`→`→`,
  `===`→`≡`, `<=`→`≤`, `>=`→`≥`, `/=`→`≠`, `<:`→`⊑`, `><`→`×`, `\/`→`∨`,
  `/\`→`∧`, `\`→`λ`), and (b) **preserves strings and `--` line comments**. Use
  it as the transform; do not hand-roll a substitution.
- **Match arms are already canonical `↦`** (WP #24) and the function arrow `→`
  is already canonical — leaving already-unicode glyphs untouched is correct,
  not incomplete.

## The load-bearing scope guard — FENCE-SCOPED ONLY

`canonical_unicode` is written for *pure `.ken` files*. Catalog packages are
**literate `.ken.md`** (prose + fences). Applying the transform to whole files
would **corrupt prose** (English "not"→¬, a stray "l"→ℓ, "->" in a sentence→→).

**So: apply the transform ONLY to the contents of ` ```ken ` code fences**
(reuse the catalog harness's existing fence extractor — the same one
`ken check` / the acceptance nets use to pull fences), never to markdown prose,
headings, or non-`ken` fences. Prose glyph usage is out of scope for this WP
(Librarian's territory if ever wanted).

## Scope — every Ken-source root, catalog side only (NOT spec/conformance)

Enumerate **every** Foundation/Librarian-owned Ken-source root (the match-arm
lesson: a corpus-wide glyph sweep is only as complete as its file inventory):

- `catalog/packages/**/*.ken.md` (all package fences)
- `catalog/guide/**/*.ken.md` (guide fences)
- `examples/rosetta/*.ken` (standalone — pure `.ken`, whole-file transform is
  correct there)
- `tooling/highlight-js/sample.ken` (if it carries ascii operator forms)

**Explicitly OUT of scope (do NOT touch):** `spec/**`, `conformance/**` (spec
enclave's; operator said no enclave), any `crates/**` Rust source, the
`prelude.rs` Ken emission, Cargo/lock, kernel/TCB. Match-arm `↦`/`|->` sites are
already migrated — do not disturb.

## Deliverables

1. Every ` ```ken ` fence (and pure `.ken` example) in scope, canonicalized to
   unicode per `canonical_unicode`. Prose/strings/`--` comments unchanged.
2. If the transform is applied via a script, include a one-shot note of the
   mechanism in the PR body (not a committed tool) so the diff is reproducible.

## Acceptance criteria (testable)

1. **Fence-scoped:** no markdown prose, heading, or non-`ken` fence changed;
   strings + `--` comments inside fences unchanged. (Spot-audit: every changed
   line is inside a ken fence / pure `.ken`.)
2. **Glyph-only, semantics-preserved:** the diff is *exactly* the
   `canonical_unicode` ascii→unicode substitutions — no identifier logic, no
   proposition/proof-term change, no `fn`/`const`/keyword change. A changed
   fence lexes to the identical token stream (that is *why* it's safe).
3. **Structural completeness proof:** full `scripts/ken-cargo test --workspace`
   green + every catalog package-acceptance net green — since a canonicalized
   fence lexes identically, any breakage (a mis-scoped prose edit, a fence
   boundary error) fails to elaborate/compile. Green = the sweep is clean and
   in-fence.
4. **Idempotence:** re-running `canonical_unicode` over the result is a no-op
   (nothing left to canonicalize in scope).
5. **Scope clean:** no `spec/`, `conformance/`, `crates/` source, `Cargo`, or
   lockfile change; `git diff --check` clean.

## Do-not-reopen guardrails

- Direction is operator-settled (unicode canonical). Don't relitigate.
- Fence-scoped only — never transform prose.
- Don't touch spec/conformance (no enclave), match arms (already `↦`), or the
  ascii acceptance in the lexer/formatter (this is a *source* normalization).
- Light process: no retro, no spec enclave.
