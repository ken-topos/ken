# WP — FOSSIL: migrate `lemma … = Axiom` fossils to the `axiom` keyword

**Owner:** Foundation ring (catalog authorship).
**Reviewer/Gate:** Foundation QA. **No Architect terminal gate** — this is a
spec-guaranteed mechanical sugar swap, and QA's trusted-base-invariance check
(AC3) is the semantic gate. *Escalation:* if QA finds **any** trusted-base /
postulate-ledger delta, STOP and route to the Architect before proceeding — a
non-empty delta would mean the sugar is not actually equivalent (a spec-vs-impl
gap), not a catalog edit.
**Size:** S (3 source sites in 2 files + 2 coupled test-oracle files + this
frame sync + a formatter re-run; the weight is the trust-invariance
verification, not the edit).
**Branch:** `wp/axiom-fossil-sweep`, cut fresh from `origin/main` at kickoff.
**CI:** ⛔ FULL CI — touches `catalog/` (never `--doc-only`).
**Source:** operator decision 2026-07-14 (the `lemma = Axiom` → `axiom`
migration; the 4 `LawfulClasses` `= Axiom` are instance-method fields and are
**explicitly out of scope** — the sugar does not cover them).

## Objective

Ken now has a first-class `axiom name : T` declaration keyword. The pre-sugar
spelling `lemma name : T = Axiom` — using the `Axiom` builtin as a lemma body —
is a **workaround fossil**: it is exactly the contortion the `axiom` keyword was
added to retire. Migrate the three remaining fossil sites to the keyword. Per
`spec/30-surface/32-grammar.md §…`: **`axiom N : T` ⇒ `lemma N : T = Axiom`** is
a mechanical expansion introducing no new kernel/semantics — so this migration
**changes surface tokens at three sites and changes nothing else**: identical
elaborated term, identical trusted base, zero trust delta.

## Fixed inputs (settled — do not reopen)

1. **Exactly three sites, two files** (grounded on `origin/main @ 8dabdeca`;
   re-derive the line numbers from current source before editing — they may
   drift):
   - `catalog/guide/proof-techniques.ken.md:159` (single-line):
     `lemma prim_eq_axiom : Equal Bool (eq_int five five) True = Axiom`
   - `catalog/guide/proof-techniques.ken.md:~349` (wraps; `Axiom` on `:351`):
     `lemma string_to_list_char_retraction : (text : String) → Equal String
     (list_char_to_string (string_to_list_char text)) text = / Axiom`
   - `catalog/packages/Data/Text/StringBijection.ken.md:~13` (wraps;
     `Axiom` on `:15`): the **same** `string_to_list_char_retraction` lemma
     (this is the canonical `\`\`\`ken` source; the proof-techniques copy is in a
     `\`\`\`ken example` fence).
2. **The 4 `Core/Classes/LawfulClasses.ken.md` `= Axiom` sites are OUT** (`refl`/
   `antisym`/`trans`/`total` on `Ord Int`). They are **instance-method fields**,
   not top-level `lemma` declarations; the `axiom` keyword is a declaration form
   and does not cover instance fields. Do not touch them. Do not touch any
   backtick-`\`Axiom\`` prose reference or any `Axiom` used as a match-arm /
   lambda body.
3. **Zero trust delta is the invariant.** The migrated `axiom N : T` must
   elaborate to the identical postulate as `lemma N : T = Axiom` — same trusted
   base, same audit label, same `Axiom`/postulate count. This is what AC3 pins.
4. **Layout is the formatter's job, not yours.** After the token edits, the
   catalog must stay formatter-canonical: run `ken fmt`, do not hand-lay-out the
   `axiom` decls. The formatter already supports `AxiomDecl` (a distinct AST
   node; `layout.rs::print_decl_signature`, the token-preserving signature
   printer shared with `data`), so an `axiom` decl round-trips to a fixed point.

## The mechanism (grounded)

At each of the three sites, rewrite the declaration head and drop the body:

```
lemma NAME : T = Axiom        ⇒        axiom NAME : T
lemma NAME                              axiom NAME
    : T =                                   : T
  Axiom
```

i.e. replace the `lemma` keyword with `axiom`, and delete the trailing
`= Axiom` (single-line) or the `=` at the end of the type line **and** the
`Axiom` continuation line (wrapped form). The type `T` is unchanged verbatim.

Grounding already done for the frame (do not re-litigate, but re-verify against
current source):
- `axiom` is a real keyword: `lexer.rs` `KwAxiom` (`"axiom"`), `parser.rs`
  `parse_axiom_decl` builds `Decl::AxiomDecl { name, theorem, span }` — a
  distinct AST node, **not** desugared to `LemmaDecl` at parse time.
- The formatter emits it as `axiom …` (token-preserving `print_decl_signature`,
  `layout.rs:585`), so `ken fmt` will not normalize it back to `lemma … =
  Axiom`. There is currently **no** `axiom` decl in the catalog, so this WP is
  also the first corpus exercise of `axiom` round-tripping — AC4 pins it.

## Mandated deliverable outline

1. **Precondition probe (do this FIRST, before editing).** In a throwaway
   fixture, write `axiom foo : Top`, run it through `ken fmt` and the checker,
   and confirm: (a) it parses/checks (elaborates to a `Top` postulate); (b) the
   formatter re-emits it as `axiom foo : Top` and is a fixed point (not
   normalized to `lemma foo : Top = Axiom`). If either fails, **STOP and surface
   to the Steward** — the fossil sweep would then require a formatter/parser
   change and is a different WP. (Expected: both pass, per the grounding above.)
2. **Edit the three sites** per §mechanism — `lemma → axiom`, drop `= Axiom`.
   Nothing else in either file changes by hand.
3. **Re-format** — run `ken fmt` over the catalog (or at least the two edited
   files); commit the formatter's output. Only `proof-techniques.ken.md` and
   `StringBijection.ken.md` may change under `catalog/`; every other catalog
   file must remain byte-identical (they are already fixed points from the
   kenfmt WP).
4. **Trust-invariance evidence** — capture the trusted-base / postulate ledger
   for the two files (or the whole catalog) **before and after** and show the
   diff is empty: same postulate set, same audit labels, same `Axiom`/
   `declare_postulate` count. This is the semantic gate (AC3).
5. **Coupled test-oracle migration** — inventory both retired-spelling
   assertions and static declaration-form enumerations across the complete
   test/fixture/golden population. Update only the coupled CC2 source-spelling
   assertions and add only `"axiom"` to kenfmt's top-level declaration-prefix
   oracle; preserve the exact-one separately-homed-assumption invariant.

## Acceptance criteria (testable)

- **AC1** — the three fossil sites are `axiom NAME : T` declarations; **zero**
  `lemma … = Axiom` top-level declarations remain in the catalog (`git grep`
  for the single-line and wrapped forms both return empty). The 4 `LawfulClasses`
  instance-field `= Axiom` sites are unchanged.
- **AC2** — both edited files still **check** green (each `\`\`\`ken` /
  `\`\`\`ken example` fence elaborates); `string_to_list_char_retraction` and
  `prim_eq_axiom` remain in scope for their downstream consumers
  (`string_to_list_char_injective`, the §3 examples).
- **AC3 (semantic gate)** — the trusted base / postulate ledger is **byte-
  identical** before and after: same postulate set, audit labels, and
  `Axiom`/`declare_postulate` count. **Zero trust delta.** (If non-empty →
  escalate to Architect, do not merge.)
- **AC4** — the reflowed catalog is a **fixed point**: `ken fmt` run again is a
  no-op, and the strict frozen-corpus gate is green on the migrated corpus (the
  `axiom` decls round-trip stably).
- **AC5** — scope: exactly five changed files: this frame; the two catalog
  sources above; and the coupled test-oracle files
  `cc2_text_codec_numeric_acceptance.rs` and `kenfmt_c_capstone.rs`. The only
  `crates/**` changes are assertions that encode the retired spelling and the
  static top-level declaration-form enumeration that must include `axiom`;
  **zero production-source `crates/`**, grammar, spec, or other catalog change.
  No `--doc-only` (touches `catalog/`).

## Do-not guards

- Do **not** touch the 4 `Core/Classes/LawfulClasses.ken.md` instance-field `= Axiom`
  sites, any prose `\`Axiom\``, or any `Axiom` used as an expression body.
- Do **not** change production `crates/**` or any test beyond the two inventoried
  coupled oracle files.
- Do **not** hand-lay-out the `axiom` decls — the formatter owns layout; you run
  it.
- Do **not** change the type `T` at any site, or any other declaration in either
  file.
- Do **not** proceed past the precondition probe if the formatter normalizes
  `axiom` away or the checker rejects it — surface to the Steward instead.
- Do **not** merge on a non-empty trusted-base delta (AC3) — that is a
  spec-vs-impl equivalence failure, an Architect matter.

## Sequencing (Steward-owned)

- **Needs the catalog-quiet window.** Touches `catalog/`; hold while any other
  `catalog/` WP is in flight. The kenfmt reflow has merged and released the
  window; PX catalog work (`Posix.ken.md`) must be sequenced around this.
- Off the critical path. Independent of the PX1/CC9 sequencing decision (this is
  Foundation/catalog; PX1 is Runtime/`crates`), so it may run in parallel with
  Runtime debt-repayment.
- A workaround fossil retired the moment its capability lands — see the fleet
  memory *the-workaround-fossil-tells-you-what-the-language-could-not-say*.
