# WP — kenfmt: break after `↦` when a match arm wraps

**Owner:** Language ring (owns the formatter — B1–B4 + capstone C).
**Reviewer/Gate:** Language QA (locked workspace + golden layout) **+ Architect
semantics-preservation gate** (a formatter that changes parsed meaning is
catastrophic — the C-capstone soundness AC applies).
**Size:** M (one-line engine change; the weight is the atomic catalog reflow +
strict-gate re-arm).
**Branch:** `wp/kenfmt-match-arm-break-after-arrow`, cut fresh from
`origin/main` **after LET-3 P2 merges** (see Sequencing).
**CI:** ⛔ FULL CI — touches `catalog/` (catalog-wide reflow).
**Source:** operator feedback `local/kenfmt-feedback-3.md` (2026-07-15); lambda
disposition ruled by operator 2026-07-15 (uniform — see §Fixed inputs).

## Objective

In a `match` arm `pattern ↦ body`, when the arm does **not** fit on one line,
the line break must fall **immediately after the map-to `↦`** — the entire body
moves to a new line indented one level — instead of leaving the body's first
token on the `↦` line and wrapping the body's arguments below it.

Dispreferred (current):
```
Cons h t ↦ cong
  (List a)
  (List a)
  ...
```
Required:
```
Cons h t ↦
  cong
    (List a)
    (List a)
    ...
```

An arm that fits on one line stays on one line (`Zero ↦ Refl`, `Right ↦
fallback`) — the rule fires **only** when the arm wraps.

## Fixed inputs (settled — do not reopen)

1. **The break is group-gated, never unconditional.** A one-line-fitting arm
   must stay flat; only an arm that overflows the width budget breaks after `↦`.
   (This is the whole point of the pretty-printer `Group`: flatten-or-break as a
   unit.)
2. **The rule applies to ALL wrapping bodies, including lambdas.** A wrapping
   `pattern ↦ λx. …` arm breaks after `↦`, moving the `λ`-binder to its own
   indented line (operator ruling, 2026-07-15):
   ```
   Cons h t ↦
     λhxs.
       <body>
   ```
   Applications (`cong …`, `list_append …`) and lambdas (`λhxs. …`) are the two
   dominant wrapping shapes; both break after `↦`. Do **not** special-case
   lambdas to stay on the `↦` line.
3. **The `↦` glyph and token spelling are unchanged.** This WP is layout only;
   `crates/ken-elaborator/src/format.rs` (token-spelling canonicalization) is
   **not** touched.
4. **Semantics preservation is the soundness invariant.** The reformatted
   catalog must parse to the identical AST as before, wholesale. No proof,
   declaration, or token content changes — only line breaks and indentation.
5. **Width budget unchanged** (96 columns) and **indent unchanged**
   (`INDENT_WIDTH`, the same nest every other layout rule uses).

## The mechanism (grounded; re-derive line numbers from current source)

The layout engine is **`crates/ken-elaborator/src/layout.rs`** (NOT
`format.rs`). The decision point is **`print_match_arm`** (~`layout.rs:1037`):

- A `compound` classifier (`is_compound_expr` ~`:1959`, `is_let_chain` ~`:1975`,
  plus `EMatch`) already routes nested-`match` and `let`-chain bodies through a
  branch that emits `hard_line()` after `↦` and nests the body — **this is
  already the required layout.**
- The defect is the **`else` branch**: it glues `head` (`pattern … ↦`) to the
  body with a **hard, non-breaking `Doc::text(" ")`** (~`layout.rs:1060`), so
  `EApp` and non-compound `ELam` bodies stay on the `↦` line and only their own
  internal groups break.

**The change:** in the `else` branch, replace the flat `text(" ")` glue with a
**group-gated breakable line + nest**, structurally matching the `compound`
branch but conditional on fit — e.g.
`concat([head, concat([Doc::line(), print_expr(body)]).nest(INDENT_WIDTH)]).group()`.
Because the group flattens-or-breaks as a unit, a fitting arm stays flat and a
wrapping arm breaks after `↦`. (Confirm the exact combinator names against
current `layout.rs`; `Doc::line()` is the soft line at ~`:37-97`.)

You may instead fold the `else` and `compound` branches if that is cleaner —
the end state is that **every** arm uses one group-gated break-after-`↦`
shape, with compound bodies (which never fit) always breaking and simple bodies
breaking only on overflow. Either factoring is acceptable; the observable
output is what the ACs pin.

## Mandated deliverable outline

1. **Engine change** in `print_match_arm` (`layout.rs`) per above — the
   `else`-branch flat glue becomes a group-gated break-after-`↦`. State in a
   comment that this is the `↦` member of the shared "break after the connective,
   nest the tail" family (siblings: signature-layout break after `:`; let-layout
   break after `=`).
2. **New oracles** (there is currently **no** test pinning a wrapping
   application or lambda arm — that shape is untested):
   - a wrapping **application** arm → breaks after `↦`, body nested (Pat's
     `cong` example is a good fixture);
   - a wrapping **lambda** arm → breaks after `↦`, `λ`-binder on its own nested
     line;
   - a **fit** arm (`Zero ↦ Refl`) in the same fixture → stays one line
     (guards against the unconditional-break regression).
   Put these in `kenfmt_b3_layout.rs` (or the closest existing match-arm suite).
3. **Preserve existing green oracles** — every current wrapping-arm fixture is a
   compound body that already breaks after `↦`; they must stay byte-identical:
   `kenfmt_b3_layout.rs` (nested-match + one-line arms), `kenfmt_let_layout.rs`
   (`ac5_nested_bindings_in_a_match_arm_…`), `kenfmt_signature_layout.rs`
   (`Zero ↦ n; Suc m ↦ sub n m`).
4. **Atomic catalog reflow** — run the formatter over the whole catalog and
   commit the reformatted `.ken.md` in the **same** WP, following the
   **C-capstone pattern**: regenerate, re-arm the strict frozen-corpus gate that
   the signature-layout WP left **paused** (`kenfmt-signature-layout.md` AC5),
   and verify AST-preservation wholesale. ~386 arms across ~23 files reflow;
   `Map.ken.md` is ~199 of them.
5. **AST-preservation proof** — the capstone semantics gate must show the
   reformatted catalog parses to the identical AST as the pre-reflow catalog
   (this is the Architect gate; it is the soundness AC).

## Acceptance criteria (testable)

- **AC1** — a wrapping application arm and a wrapping lambda arm each break
  immediately after `↦` with the body nested one `INDENT_WIDTH`; new oracles
  assert the exact emitted text (red on the current `else`-branch, green after).
- **AC2** — a one-line-fitting arm (`Zero ↦ Refl`, `Right ↦ fallback`) stays on
  one line; a new oracle pins it (guards the unconditional-break regression).
- **AC3** — every pre-existing `kenfmt_*` layout oracle stays green
  byte-for-byte (compound wrapping arms already broke after `↦`; nothing there
  moves).
- **AC4** — the reflowed catalog is a **fixed point**: running the formatter
  again is a no-op (idempotence), and the strict frozen-corpus gate is **re-armed**
  (un-paused) and green on the reflowed corpus.
- **AC5** — **AST preservation**: the reflowed catalog parses to the identical
  AST as before the reflow (Architect semantics gate). Zero token/declaration/
  proof content change — layout only.
- **AC6** — zero delta in `crates/ken-kernel/**`, the surface grammar, and
  `format.rs`; the only `crates/` change is `layout.rs` + its tests.

## Do-not guards

- Do **not** break a fitting arm — the break is group-gated on overflow only.
- Do **not** exempt lambdas from the rule (operator-ruled uniform).
- Do **not** touch the kernel, the surface grammar, `format.rs`, the width
  budget, or `INDENT_WIDTH`.
- Do **not** change any token, declaration, or proof term in the catalog — the
  reflow is whitespace/line-break only, and AC5 proves it.
- Do **not** land the engine change without the atomic catalog reflow in the
  same WP (a half-applied rule leaves the corpus non-fixed-point and re-arms a
  red gate).
- Do **not** hand-reflow — the formatter generates the catalog; you regenerate
  it.

## Sequencing (Steward-owned)

- **Hard dependency: land AFTER LET-3 P2 merges.** The reflow rewrites
  `Map.ken.md` match arms (~199 sites); LET-3 P2 is concurrently editing exactly
  those. Releasing this before P2 merges collides on the same arms.
- **Catalog-quiet window.** The atomic reflow needs exclusive catalog access —
  no other in-flight `catalog/` WP (the next Foundation catalog WP / fossil
  sweep, PX Posix work). Steward schedules the window and holds concurrent
  catalog releases during the reflow.
- Off the critical path (the LET chain / POSIX-PX is load-bearing); this rides
  the first quiet catalog window after P2.
