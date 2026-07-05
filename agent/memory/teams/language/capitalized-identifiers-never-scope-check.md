---
scope: teams/language
audience: (see scope README)
source: private memory `capitalized-identifiers-never-scope-check`
---

# Capitalized identifiers never scope-check

On `wp/surface-transport` (2026-07-03, Map Gap A), writing `.ken` probes and the
`transport.ken` package using spec-style capitalized generic parameter names
(`A`, `B`, `P`, mirroring `53-transport.md`'s math notation) produced repeated
`UnresolvedCon { name: "A", ... }` errors for references to a parameter that WAS
correctly bound (e.g. `view f (A:Type)(a:A): ... = ... A ...` — declaring
`(A:Type)` works, but referencing `A` later inside an expression body fails).

**Root cause, grounded in `ken-elaborator/src/resolve.rs::resolve_expr_ctx`:**
`Expr::EVar(name,_)` checks `scope.index_of(name)` first, falling back to `RCon`
only on a scope MISS. `Expr::ECon(name,_)` has NO such fallback — it
unconditionally emits `RExpr::RCon(name,...)` regardless of whether a local
binder of that name exists. Since the PARSER decides `EVar` vs `ECon` purely
from the token's lexical case (`Ident` vs `ConId`) at parse time, BEFORE any
scope information exists, a capitalized name can *never* resolve to a local
term-level binding, no matter what's in scope — only lowercase identifiers can.

This means a lambda/view binder CAN be named `A` (the binder-name parsing
accepts both `Ident`/`ConId` tokens), but referencing it anywhere in an
expression body afterward is impossible — the reference always resolves as a
(usually nonexistent) global constructor lookup instead, surfacing as
`UnresolvedCon`. Type-position (`RType`) references are a SEPARATE resolution
path (`RType::RVarTy`) that correctly consults scope regardless of case — so a
capitalized parameter referenced only in TYPE ANNOTATIONS (never in a
body/expression) can appear to work, masking the restriction until the same name
is used in a motive/body/return-value expression.

**How to apply:** every real `.ken` package already follows this convention
(`class Eq a`, never `Eq A`; `isSorted a leq xs`) — treat it as a hard grammar
rule, not a style guideline. When spelling a `.ken` source file from a spec
listing that uses capitalized math notation for generic parameters, translate
ALL such parameters to lowercase before writing the literal source, especially
any name that will be referenced inside a lambda motive, match arm, or return
expression (not just a type signature). A quick tell during debugging:
`UnresolvedCon` for a name that you're SURE is bound — check whether it's
capitalized before suspecting a real resolver/scope bug.
