---
id: EFF-SPACE-ENSURES-PRESTATE
title: "`old` is transparent, so a space operation's `ensures` cannot express the pre/post distinction `36 §4.3` is built on"
status: closed
owner: language
resolution: "Shape B (fail closed) merged PR #1115 at origin/main=aea07d62, elab.rs blob 648df173. ⚠ PARTIAL BY DESIGN: normative pre-state semantics remain UNDELIVERED -- `old` is now unavailable and says so, instead of available and meaningless. Shape A needs the `becomes`/space cell-block surface, which this frame excluded and which has no node yet."
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: "Steward measurement 2026-07-27 at origin/main=0031dd6a, taken while scoping L5 for release to Team Language. L5 is NOT clean backlog -- crates/ken-elaborator/src/effects/ is 2614 lines across 10 files and is consumed by elab.rs, capabilities.rs, prelude.rs, modules.rs, classes.rs, compiler_driver.rs, program_admission.rs, foreign.rs and bytes.rs. Five of 36 section 6's six deliverables are built and live. This node is the measured residual of the sixth (section 4, the `space` state model), sliced to the part that is a correctness hole in already-reachable surface rather than a missing feature."
---

> ## ✅ CLOSED 2026-07-27 as **Shape B** — PR #1115, `origin/main = aea07d62`
>
> `old` no longer silently means its operand. Both `check` and `infer` reject
> `RExpr::ROld` with a span-carrying `OldPreStateUnsupported`; the resolver
> remains the sole admission gate; a pure `fn` still fails as exact
> `UnboundName(old)`; an ordinary space `ensures` without `old` still emits.
> Verified by blob: `elab.rs` = `648df173` on main and on the approved candidate.
>
> ### ⚠ THIS NODE CLOSED PARTIAL, AND THE RESIDUAL HAS NO NODE YET
>
> ⛔ **Do not read "closed" as "`36 §4.3` is delivered."** It is not. **Normative
> pre-state semantics remain unimplemented.** What changed is the honesty of the
> failure: the feature is now *unavailable and says so* rather than *available
> and wrong*.
>
> Shape A — elaborating `ensures` against the state transformer — was
> unavailable in this slice because the parser has only `space proc`: no
> `becomes`, no cell environment, no `s_pre`/`s_post` binding to elaborate
> against. ⇒ **The residual is the `becomes` / space cell-block surface**, which
> this frame deliberately excluded, and which needs a design call on whether
> space-block syntax exists at all before it can be framed. **That node is not
> yet filed.** Until it is, a reader finding this node closed and no successor
> could reasonably conclude the feature shipped. It did not.

## The measurement

**At `origin/main = 0031dd6a`.** ⛔ Re-derive at point of use.

`spec/30-surface/36-effects.md §4.3` requires that in a space operation's
`ensures`, a bare cell `cᵢ` denotes the **post-state** value and `old(cᵢ)`
denotes the **pre-state** value — well-defined, the spec says, "because the
denotation *names* the pre-state."

**`old` is parsed, resolved, correctly scope-gated — and then discarded.**

| stage | file | what happens |
|---|---|---|
| lex/parse | `parser.rs:2184` blob `e3fc6620` | `old e` → `Expr::EOld` ✓ |
| resolve | `resolve.rs:1473` blob `f05c7535` | gated to `PropCtx::SpaceOpEnsures`; outside it, `UnboundName` ✓ |
| **elaborate (check)** | **`elab.rs:614`** blob **`7029765b`** | **`RExpr::ROld(inner, span) => check(cx, inner, expected, span)`** |
| **elaborate (infer)** | **`elab.rs:2199`** | **`RExpr::ROld(e, _) => infer(cx, e)`** |

⇒ **`old(n)` elaborates to exactly the core term `n` elaborates to.** The
pre/post distinction does not exist in the emitted term. `ensures n == old(n) +
1` — the spec's own worked example at §4.3 — denotes `n == n + 1`.

⭐ **This is documented, not accidental.** `elab.rs:612` says *"`old` is
transparent in the V1 model."* So the node is not "someone forgot"; it is that
the V1 simplification was never retired and §4.3 was written against a model
that does not exist yet.

## Why it is not inert

`ObligationKind::Ensures` is real and reaches the prover: `elab.rs:5434–5452`
collects `ensures` clauses, and `extract.rs:118` lifts each to an
`ObligationTriple` with id `<def>.ensures.<n>`. So a space operation's `ensures`
**does** become a proof obligation — one built from a predicate whose `old` means
nothing.

## ⛔ The false green that hid it

`crates/ken-elaborator/tests/effects.rs` blob `373e7cb2`,
`space_old_scoped_to_ensures_type_level` — the **only** test naming `old`. Its
docstring claims *"the ensures predicate has the correct structure (pre/post)."*

**Its body never constructs an `ensures` predicate, and never constructs an
`old`.** It builds an `EffectDecl`, calls `infer_all`, asserts the row is
`[Counter]`, and calls `check_escape`. Every assertion would hold identically if
`EOld` were deleted from the AST.

⇒ The test is green, correctly named, and **structurally incapable of observing
the property its docstring asserts.**

## Scope note — what is NOT in this node

`36 §4.1`'s other half is a separate, larger gap and is deliberately excluded:
**`becomes` has no surface spelling at all** (zero hits for `becomes`/`KwBecomes`
in `lexer.rs`/`parser.rs`), and `space` parses only as `space proc`
(`parser.rs:355`) — a modifier on a view declaration, not a cell-bearing block.
That is missing *feature*, not a defect in reachable surface. File it separately;
⛔ do not let it ride in here.
