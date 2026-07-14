# LET-1 · `kenfmt` — readable `let`-chain layout, and an oracle that can see it

**Owner:** Team Ergo · **Size:** S/M · **Branch:** `wp/let1-kenfmt-binding-layout`
**Base:** `origin/main @ 2c184550` · **Gate:** Ergo QA + **CV** (you touch
`spec/30-surface/`) · **Runs in PARALLEL with LET-2. Blocks LET-3.**

**Source:** operator-commissioned report,
`local/ken-let-authoring-style-report.md` (2026-07-14). **Its claims are verified
below at the emission — build from THIS frame, not from the report.**

## 0. Why this is first

**The catalog has ZERO local `let` bindings in 27,404 tangled Ken lines**
(I re-ran the count: 32 files, exact ```` ```ken ```` fences only — **0 uses**).
We are about to ask authors to start writing them. **If the formatter mangles a
`let` chain the moment they do, we will teach the whole fleet that `let` looks
bad** — and `ken fmt --check` will call the mangled output *canonical*, so
nothing will flag it.

**⇒ Fix the formatter BEFORE the convention ships. That ordering is the report's
and I am adopting it.**

## 1. The bug — verified, exact

`crates/ken-elaborator/src/layout.rs:1823`:

```rust
fn is_compound_expr(expr: &Expr) -> bool {
    match expr {
        Expr::EMatch { arms, .. } => { arms.len() >= 2 || arms.iter().any(|a| is_compound_expr(&a.body)) }
        Expr::ELam(_, body, _)          => is_compound_expr(body),
        Expr::ELet(_, _, value, body, _) => is_compound_expr(value) || is_compound_expr(body),
        _ => false,
    }
}
```

**A nested `ELet` is never compound *in its own right*.** Take a chain of simple
bindings — `let a = f x in let b = g y in let c = h z in body`. Every `value` is
a simple application (**not** compound), every `body` is the next `ELet`, and the
recursion bottoms out on a simple final body. **So the entire chain reports
`false`.**

The chain therefore takes the **generic soft-token layout** at
`layout.rs:1066` (`if is_compound_expr(value) || is_compound_expr(body)`), which
breaks on token boundaries rather than structure — and produces the report's
observed fixed point:

```ken ignore
let left_chars : List
Char = string_to_list_char
left in let right_chars : List
Char = string_to_list_char
right in ...
```

**Note what this is NOT:** `print_let` exists (`layout.rs:410`); the parser and
printer handle `let` correctly. **This is a LAYOUT defect, not a parse gap.** The
formatter is *sound* — it preserves the AST, it is idempotent, and it respects
the width ceiling. **It is simply unreadable**, and every gate we have is blind
to that.

## 2. ★ The oracle is the real deliverable

This is the *third* time this exact class has bitten us —
`agent/memory/fleet/formatter-soundness-gates-are-blind-to-layout-conformance.md`
records the last one. **The existing gates check that the formatter did not lie.
Nothing checks that it produced something a human can read.**

```
AST-preservation   ✅ passes on the mangled output
idempotence        ✅ passes  (it is a fixed point — just a bad one)
width ceiling      ✅ passes  (that is WHY it shreds — it breaks to fit)
ken fmt --check    ✅ reports CANONICAL
readability        ⛔ nothing looks
```

**⇒ AC6 (assert the exact emitted text) is not a nicety bolted onto this WP. It
is the point of it.** A fix without the oracle regresses the moment someone
touches `layout.rs` again.

## 3. Mandated deliverable

### 3.1 The fix

**An `ELet` whose body is itself an `ELet` is COMPOUND** — a binding *chain* is a
structural form, and when it cannot fit on one line it must take `let`/`in`
structural layout, never soft-token layout. Bindings each get their own line;
**a type annotation like `List Char` is a unit and MUST NOT be split across
lines.**

**Keep it minimal and local.** Do not restructure the printer. Do not
"improve" unrelated layout paths. **A short chain that genuinely fits on one line
must STAY on one line** (AC1) — this fix must not make simple code worse.

### 3.2 The spec touch

`spec/30-surface/` — the surface formatting contract must **explicitly classify a
nested `let` body as compound when the chain must break**, so the normative block
example and the implementation cannot drift apart again. **This is what puts CV
on your gate. Keep it to that clarification; do not re-open the formatting
contract.**

## 4. Acceptance criteria — the six layout cases, each asserting EXACT TEXT

**Every one of these asserts the emitted string, not merely "it round-trips."**

- **AC1** — a short single `let` **stays horizontal** when the whole expression
  fits.
- **AC2** — a long single binding **breaks at `=` and `in`**, per
  `spec/30-surface/31-lexical.md`.
- **AC3** — **two or more simple nested bindings stay horizontal ONLY when the
  complete chain fits**; otherwise **each binding gets structural `let`/`in`
  layout.** *(This is the bug. This is the test that currently fails.)*
- **AC4** — a chain with typed values such as `List Char` **does not split the
  type into one token per line.**
- **AC5** — nested bindings inside a `match` arm **indent relative to the arm.**
- **AC6** — **the exact emitted text is asserted** — *in addition to* AST
  preservation, idempotence, and max width. **Not instead of.**
- **AC7** — **the report's own worked example is a fixture.** Take
  `string_to_list_char_injective_with_lets` (report §"Worked example", 6
  bindings, typed proof-valued binds) verbatim, format it, and **assert the
  emitted text is the readable form.** It is the exact term that produced the
  mangled output, so it is the exact term that must come out clean.
- **AC8** — **the frozen `ken_fmt` corpus stays green.** If a corpus file's
  expected output legitimately changes, **say so explicitly and show the
  before/after** — do not silently re-baseline. *(Re-baselining a frozen oracle
  to match new behavior is how a formatter gate becomes a rubber stamp.)*
- **AC9** — **ZERO `trusted_base()` delta.** This is a layout change. Nothing
  about trust may move.

## 5. ⛔ Guardrails

- **⛔ `kenfmt` MUST NOT INVENT BINDINGS.** Introducing a `let` changes the AST
  and can change evaluation placement (Ken is call-by-value; an effectful `let`
  sequences its RHS before its body). **The formatter owns the LAYOUT of bindings
  the author wrote. It never adds, removes, reorders, or re-scopes one.** If you
  find yourself writing code that creates an `ELet` — **stop, you are building
  the wrong thing.**
- **⛔ Do not add a lint, a warning, or a binding-count heuristic.** The
  convention is a *human judgment rule* (LET-2), deliberately **not** a
  mechanical one. A `let`-count linter would breed meaningless `tmp1` bindings —
  the report is explicit about this and so am I.
- **⛔ Do not touch the catalog.** Zero `.ken.md` edits. The rewrites are **LET-3**
  and they are gated on you.
- **⛔ Do not re-baseline a frozen fixture to make a test pass.** See AC8.
- Targeted gates only — `scripts/ken-cargo -p ken-elaborator` / `--test ken_fmt`
  / `--test kenfmt_c_capstone`. **⛔ NEVER `--workspace`** (operator hard rule;
  `COORDINATION.md §12`). The full gate runs in CI.
