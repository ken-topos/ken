# WP · kenfmt batch-3 — vertical balance for over-width groups (split high, not low)

**Owner:** Language (kenfmt owner) · **Reviewer:** QA + Steward (AC5
semantics spot-check) · **Architect:** informational (layout = tool/style, not
kernel semantics) · **Size:** M · **Base:** `origin/main @ f811a93a` (batch-2a
+ #37 landed). **Status: READY (QUEUED behind #37 — now MERGED).** — operator-directed 2026-07-13
(Pat), from the `local/kenfmt-feedback-2.md` before/after review of the
batch-2a-merged catalog.

## Why (operator finding on the batch-2a-merged catalog `5efa317b`)

Batch-2a fixed the **fits-on-one-line** case: a short nested application now
collapses horizontally. But the **over-width** case is still wrong. When a
construct exceeds the width budget, the formatter **fills horizontally until the
width runs out, then breaks at the lowest point** — producing an unbalanced
splay: a long first line, then a tail splayed one atom per line, plus mid-atom
breaks like `(g : b`⏎`→ c)`. This is pervasive — roughly 1,200 splayed regions
across the merged catalog (Map ~855, `EffectfulClasses` ~262, `LawfulClasses`
~40, `LawfulFunctors`, `Validation`, `Collections`, `Parsing`, …).

The two splay flavors have one root cause (see the change-site map below): the
over-width path breaks at the **deepest** breakpoint that overflows instead of
at the **highest** top-level boundary, because top-level children are joined by
non-breaking hard spaces (fill flavor) or share a single flat group across all
nesting depths (mid-atom flavor). Batch-3 replaces fill-then-break-low with a
**consistent break at the top level, recursing into each child**.

## Objective

Switch over-width application / arrow-chain / return-type groups from FILL to
CONSISTENT-BREAK ("Wadler group") mode — a group fits flat on one line, or ALL
its top-level children break to their own lines at +2 indentation and each child
is recursed into — so the catalog renders in a balanced vertical arrangement
with no mid-atom breaks. Then re-sweep the corpus, regenerate the fixture, and
add an automated balance gate.

## The rule to implement (derived from `local/kenfmt-feedback-2.md`)

**The governing principle (operator, quoted):** "split higher in the AST rather
than lower … move up from the point where a split is required to a higher level
to provide a more balanced vertical arrangement." And: "arguments are indented
**+2 relative to the function's indentation**."

Concretely, a group is one of an application spine, an arrow chain, or a return
type. For each:

1. **Fit-flat first.** If the whole group fits the width budget at its current
   column, render it on one line (batch-2a behavior, unchanged).
2. **Else consistent-break at the TOP level only.** Break at *every* top-level
   child boundary at once — never fill some then break the rest, and never push
   the break down into a deeper node. Each top-level child goes on its own line,
   indented **+2 from the parent group's indentation**.
3. **Recurse into each child.** A child that fits flat at its new column stays
   horizontal; a child that does not gets the same consistent-break treatment.
   Because each child is its own group, an inner arrow like `(g : b → c)` stays
   flat when it fits — no mid-atom `b`⏎`→ c`.

**Top-level boundaries per group kind:**

- **Application** `f x y z` → break before each argument:

  ```
  Equal
    X
    Y
    Z
  ```

- **Return type** `: Equal X Y Z` (over-width) → the head stays with the colon,
  arguments splay +2:

  ```
  : Equal
      X
      Y
      Z
  ```

  (matching the `assoc for list_append` target: `: Equal`⏎`    (List a)`⏎
  `    (list_append …)`⏎`    (list_append …)`).

- **Arrow chain** `A → B → C → …` → break at **each top-level `→`**, with the
  arrow leading each continuation line, +2:

  ```
  fusion_law :
    (a : Type)
    → (b : Type)
    → (c : Type)
    → (g : b → c)
    → (h : a → b)
    → (x : f a)
    → Equal (f c) (map a c (comp a b c g h) x) (map b c g (map a b h x))
  ```

  Note `(g : b → c)` stays flat (its inner `→` does **not** break with the
  outer chain), and the final `Equal …` return stays flat because it fits at
  +2. This is the operator's preferred `fusion_law` target.

**Alternative (NICE-TO-HAVE, not required).** The operator noted a second,
"perhaps harder to specify" form: fill the arrow-args that fit on one line and
consistent-break only the overflowing tail:

```
fusion_law :
  (a : Type) → (b : Type) → (c : Type) → (g : b → c) → (h : a → b) → (x : f a)
  → Equal
      (f c)
      (map a c (comp a b c g h) x)
      (map b c g (map a b h x))
```

**The PRIMARY required behavior is the full consistent-break form above.** The
fill-the-fitting-prefix alternative is explicitly out of scope for AC purposes —
implement it only if it falls out cleanly; do not block on it.

## Fixed inputs (settled — do not reopen)

- **Operator feedback `local/kenfmt-feedback-2.md`** is the spec of record for
  the target rendering. Every before/after pair (the `assoc for list_append`
  proof; the `Functor.fusion_law` class field; the `fusion for option_map`
  proof) is a golden target. The two load-bearing rules, quoted above: **split
  high, not low** and **arguments indent +2 relative to the function's
  indentation**.
- **Width budget** = the layout `CANONICAL_WIDTH` constant (fit/no-fit
  threshold), **= 96** as landed on `origin/main`
  (`crates/ken-elaborator/src/layout.rs:12`; operator raised it from 88 to 96
  on 2026-07-13). Batch-3 is a *break-shape* change, not a *budget* change —
  **do not touch the constant**; regenerate the fixture against the landed
  `CANONICAL_WIDTH = 96`.
- **Batch-1 R1/R1a signature ladder + R2 atomic proof selectors are frozen.**
  `Core/OrdNat.ken.md` must stay **byte-identical** to its `431e36ea`-approved
  rendering (R1 horizontal-first ladder + R1a inline arrow-returns), and R2
  proof selectors stay atomic. Batch-3 only changes what happens **after** a
  group is decided over-width; it must not alter any rendering that already fits
  or any batch-1 signature ladder.
- **Layout-only, semantics-preserving** (the hard invariant): whitespace /
  line-breaks only — never reorder, rename, insert, or drop a token; token-kind
  stream + AST unchanged per source, mechanically gated (`kenfmt_b1_lossless`,
  `kenfmt_b2_token_kind`).
- **`layout.rs` change-site anchors** (tag: **verify at pickup** — line numbers
  drift; re-locate by function name):
  - `grouped_token_slice` — `crates/ken-elaborator/src/layout.rs:597`. Splits a
    token slice into top-level paren segments, but joins them with
    `needs_space`-derived **literal spaces** (`:632`–`:640`, a `Doc::text(" ")` /
    `Doc::Nil`, **not** a `Doc::line()`) inside a single `.group()` with no
    `+2 nest`. This is the **fill flavor**: with no top-level break opportunity
    the group can only break *inside* the deepest paren segment that overflows →
    the `: Equal (List a) (list_append …) (list_append`⏎`a`⏎`xs …` splay. Fix:
    join top-level segments with breakable `Doc::line()`, `.nest(INDENT_WIDTH)`
    the continuation, wrap as one consistent group.
  - `doc_token_slice` (`TokenLayout::Soft`) —
    `crates/ken-elaborator/src/layout.rs:936`, with the arrow/atom soft-line
    insertion at `:959`–`:967` driven by `soft_break_between`
    (`:1746`, fires when `right == Token::Arrow`). All arrow soft-lines across
    every paren depth live in **one flat group**, so the binary group rule
    breaks them all at once — the **mid-atom flavor** (`(g : b`⏎`→ c)`). Fix:
    give each top-level arrow arm / paren sub-slice its own group so interior
    arrows stay flat while only top-level `→` boundaries break. Class fields
    reach this path via `print_block_inner` (`:1029`) → `doc_token_slice`
    directly (they do **not** go through `grouped_token_slice`), so the
    paren-aware sub-grouping must apply here too.
  - `print_decl_signature` — `crates/ken-elaborator/src/layout.rs:493`. Already
    breaks between *binders* with `Doc::line()` + `.nest(INDENT_WIDTH)` +
    `.group()`; the return-type clause after `:` is a single
    `grouped_token_slice`, so it inherits the fill flavor above. The return-type
    `: Equal …` consistent-break is delivered by fixing `grouped_token_slice`.
  - `print_application` — `crates/ken-elaborator/src/layout.rs:837`. This is the
    **model of correct behavior**: `Doc::line()` before each arg, single
    `.group()`, `.nest(INDENT_WIDTH)`, each arg recursively `print_expr`'d.
    Over-width breaks each arg to its own +2 line and recurses — exactly the
    target. The token-path fixes above should make the signature/type path match
    this shape. Confirm no regression here.
  - `soft_break_between` — `crates/ken-elaborator/src/layout.rs:1746` — supplies
    the top-level arrow break points; keep, but ensure they land at the right
    group granularity (top-level chain, not shared with inner parens).
  - `render` / `fits` / binary group rule —
    `crates/ken-elaborator/src/layout.rs:97`, `:168`. Pure binary-group
    renderer, **no fill pass** — do not add one. The fix is purely in how the
    printers *build* the group tree, not in the renderer.

## Mandated deliverable outline

1. **`layout.rs` consistent-break fix.** Over-width application / arrow-chain /
   return-type groups break at the top level (+2 nest, recurse into each child),
   never fill-then-break-low, never break mid-atom. Delivered by the change-site
   fixes above. No change to R1/R1a/R2, to the fit-flat (batch-2a) path, or to
   `CANONICAL_WIDTH`.
2. **Automated balance/splay GATE (new, complementary — added to the strict-gate
   set).** A test over the frozen corpus that **rejects the splay signature** —
   runs of lone-atom continuation lines and mid-arrow-fragment lines (a
   continuation line that is a bare atom like `a` / `xs`, or begins/ends a broken
   `→` inside what should be one flat arm). This is distinct from the existing
   `canonical_reformat_has_no_pathological_line_expansion`, which counts **lines**
   only and cannot see *balance*: a fill-splay and a balanced break can have the
   same line count. The new gate must (a) be green on the balanced output and
   (b) **provably fire** on a pre-fix / hand-crafted splayed sample (assert it
   rejects, so the gate is not vacuous). Home it with the layout goldens
   (`crates/ken-elaborator/tests/kenfmt_b3_layout.rs`) and/or the capstone
   (`crates/ken-elaborator/tests/kenfmt_c_capstone.rs`).
3. **Catalog-wide re-sweep + regenerate the frozen-corpus fixture.** Run the
   fixed `ken fmt` over the whole frozen corpus (all catalog `.ken.md` +
   `examples/rosetta/*.ken` + `…/Verify/ProofErasureBoundaryChecker.ken` — the
   31-file set the gates cover) and regenerate the canonical fixture from the
   new balanced output. Update `FRAME_LINE_COUNTS` / the `frame_total` oracle in
   `kenfmt_c_capstone.rs` to the new counts.
4. **Keep the three strict gates green on the regenerated corpus.** ⚠️ These
   gates are **ALREADY ARMED on `origin/main`** — batch-2a removed their
   `#[ignore]` markers when it merged (`git grep 'gate paused' origin/main --
   crates/` is EMPTY; do **not** trust a stale local working tree). So batch-3
   does **not** un-pause anything; it must **regenerate the frozen fixture from
   the balanced output** (deliverable 3) so these already-armed gates stay green
   against the new balanced corpus:
   - `crates/ken-cli/tests/ken_fmt.rs::strict_frozen_corpus_gate_is_green`,
   - `…/kenfmt_c_capstone.rs::canonical_frozen_corpus_is_a_31_file_fixed_point`,
   - `…/kenfmt_c_capstone.rs::canonical_reformat_has_no_pathological_line_expansion`.
   Because they are live, a formatter change that regenerates the corpus WILL
   flip them red until the fixture is regenerated in the same commit — regenerate
   fixture + formatter change land together. (Re-locate line numbers at pickup.)

## Two-stage operator review (same discipline as batch-2a, which caught a defect)

- **STAGE 1 — fix + one representative worst-case file.** Land the `layout.rs`
  consistent-break fix and apply the fixed formatter to **one** representative
  deep-proof file — recommend **`Core/LawfulFunctors.ken.md`** or
  **`Core/EffectfulClasses.ken.md`** (they carry the arrow-chain `fusion_law`
  case *and* the `Equal`-splay return case). Hand the diff to the **Steward**,
  who presents the balanced rendering to the operator. **Do not** re-sweep the
  catalog or touch the fixture/gates until the operator approves.
- **STAGE 2 — sweep + gate + re-arm (after approval).** Catalog-wide re-sweep,
  regenerate the fixture, add the balance gate, re-arm the three strict gates →
  QA → Steward AC5 semantics spot-check → publish.

## Acceptance criteria

- **AC1 — over-width groups render balanced.** Over-width application /
  arrow-chain / return-type groups render per `local/kenfmt-feedback-2.md`:
  concretely the `Functor.fusion_law` class field renders as its preferred
  target (each top-level `→` on its own +2 line, `(g : b → c)` flat, final
  `Equal …` return flat), and the `assoc for list_append` proof's return type
  renders `: Equal`⏎`    (List a)`⏎`    (list_append …)`⏎`    (list_append …)`.
- **AC2 — no mid-atom breaks.** No continuation line is a broken fragment of an
  atom or of an inner arrow (`(g : b`⏎`→ c)` must not occur); the new balance
  gate (AC7) enforces this corpus-wide.
- **AC3 — +2 indentation.** Every broken top-level child is indented exactly +2
  from its parent group (`INDENT_WIDTH`), per the operator's rule.
- **AC4 — batch-1 no regression.** `Core/OrdNat.ken.md` is **byte-identical** to
  its `431e36ea`-approved rendering (R1 ladder + R1a inline returns); R2 proof
  selectors stay atomic. Batch-1/2a goldens in
  `crates/ken-elaborator/tests/kenfmt_b3_layout.rs` /`b1_acceptance.rs` /
  `b2_acceptance.rs` stay green.
- **AC5 — semantics-preserving, whole corpus.** Token-kind stream + AST
  unchanged per file (`kenfmt_b1_lossless`, `kenfmt_b2_token_kind`); Steward AC5
  spot-check confirms no token reorder/insert/drop.
- **AC6 — whole-corpus idempotent fixed point.** `ken fmt --check` green on
  every frozen-corpus source; format-of-format = format; the three
  already-armed strict gates stay GREEN against the regenerated fixture (the
  fixture is regenerated in the same commit as the formatter change).
- **AC7 — balance gate green AND non-vacuous.** The new splay/balance gate
  passes on the balanced corpus **and** is shown to reject a pre-fix / crafted
  splayed sample (asserted in-test), so the class cannot regress silently.
- **AC8 — validate LOCALLY TARGETED only** (operator hard rule, COORDINATION
  §12: NO local `cargo test --workspace` — the box OOMs). Layout goldens + the
  re-armed gate tests + affected crates via `scripts/ken-cargo -p <crate>` /
  `--test <name>`; CI runs the full `--locked` gate + conformance suite at merge.

## Do-not-reopen guardrails

- **Do not touch the batch-1 R1/R1a horizontal-first signature ladder or R2
  atomic proof selectors** — no change to any rendering that already fits, or to
  the batch-1 signature/return ladder. Batch-3 only changes the *over-width
  break shape*.
- **Do not change `CANONICAL_WIDTH`** or add a fill/packing pass to the renderer.
  The fix lives in how the printers build the group tree.
- **Layout-only, token/AST preserved** — never reorder/rename/insert/drop a
  token; the mechanical invariant gates stay green.
- **Do not touch** kernel, `Cargo.*`, `.github/`, `spec/`, or `conformance/`.
  This is a formatter/layout WP scoped to `crates/ken-elaborator/src/layout.rs`,
  its tests, and the regenerated catalog fixture.

## Sequencing (Steward)

QUEUED behind #37. Two-stage, gated on operator review between stages (the
batch-2a pattern that caught a defect): **(1)** `layout.rs` fix + one
representative deep-proof file (`LawfulFunctors` or `EffectfulClasses`) →
**Steward presents to operator** → approve → **(2)** catalog re-sweep + regen
fixture + add balance gate + re-arm the three strict gates → QA → Steward AC5
spot-check → publish. Closes the over-width vertical-balance gap on the kenfmt
line.
