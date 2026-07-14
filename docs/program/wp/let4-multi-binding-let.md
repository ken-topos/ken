# LET-4 — `multi-binding-let`: sequential local binding groups

**Status:** SCHEDULED (Steward, 2026-07-14). Operator-originated.
**Owner:** Language team.
**Spec companion:** Spec enclave pins S1–S6 FIRST; Language builds from the
landed pin. CV derives the scoping and exact-layout oracles.
**Review:** Language QA → Architect terminal → Spec/CV ratification → CI.
**Size:** M. **Risk:** low-to-moderate surface/elaboration risk; **zero TCB
change**.
**Candidate branch:** `wp/let4-multi-binding-let`.
**Grounding base:** `origin/main @ f8096072`.

> **Treat every anchor in this frame as PERISHABLE.** If a fixed input is false
> against the landed code, **say so with exact tree anchors and ESCALATE** — do
> not quietly build around it. *This frame has already been corrected once for
> exactly that; see "Steward corrections" below.*

## ⛔ LET-4 SUPERSEDES LET-1b (`docs/program/wp/let1b-flat-let-chains.md`)

**LET-1b is HELD and will not be built.** It framed the nested-chain staircase as
a *formatting* defect and would have amended §31 to render a nest flat. **LET-4's
S6 canonicalizes a maximal nested chain INTO the grouped spelling** — so after
LET-4 a nested chain **never survives formatting**, and LET-1b's production would
be dead the day LET-4 merged.

**The staircase was never a formatting bug. It was a SURFACE bug wearing a
formatting bug's clothes:** Ken had no way to *say* "these bindings are one
group," so a group had to be spelled as a nest, and we then asked the printer to
render a nest as though it weren't one. **The fix is the language, not the
printer.** LET-1b's two real findings are folded in below (D1 → AC5.1/AC5.4;
the reader gate → **AC-READER**).

## ★ Steward corrections to the source draft (it was written against `c5f73b9c`)

| # | The draft says | The tree says |
|---|---|---|
| **1** | The **shredding** defect (`let left_chars : List` / `Char = ...`) and the `is_compound_expr` chain defect are **live**, and motivate this WP. | **Both are FIXED.** That was **LET-1**, merged `ec980d76`. `layout.rs:1068` already reads `matches!(body, Expr::ELet(..))`. **The live defect is the STAIRCASE, not the shred** (see LET-1b for the exact emitted text). |
| **2** | Follow-on §1–4 (style guide, checked guides, `write-ken.md`, `agent/teams/foundation/` overlays) is **future work**. | **ALREADY DONE.** That was **LET-2**, merged `8853b475`. Only follow-on §5 (the catalog pilot) remains — that is **LET-3**, and it is **BLOCKED ON LET-4**. |
| **3** | The canonical width budget is **96 columns**. | **CORRECT** — `layout.rs:12`, `pub const CANONICAL_WIDTH: usize = 96`. Verified, not assumed. |

**Consequence booked:** LET-2's guidance and guide examples teach the
**single-binding** spelling. When LET-4 lands, the canonical spelling becomes the
**group**, so `write-ken.md`, both `catalog/guide/` strands, and the Foundation
overlays need a refresh. **That is LET-2b, and it rides LET-4's landing** — it is
a scheduled certainty, not a risk.

## Objective

Extend Ken's local `let ... in ...` expression with a semicolon-separated
binding group:

```ken ignore
let
  left_chars : List Char = string_to_list_char left;
  right_chars : List Char = string_to_list_char right;
  left_round_trip : String = list_char_to_string left_chars;
  right_round_trip : String = list_char_to_string right_chars
in
  body
```

The group has **sequential, nonrecursive** semantics. It is canonical surface
sugar for the nested local lets Ken already supports:

```ken ignore
let left_chars : List Char = string_to_list_char left in
  let right_chars : List Char = string_to_list_char right in
    let left_round_trip : String = list_char_to_string left_chars in
      let right_round_trip : String = list_char_to_string right_chars in
        body
```

This is Lisp `let*` behavior under Ken's ordinary `let` spelling. Do not add a
second `let*` keyword. The grouped syntax improves authored code without adding
a new core term, recursion mechanism, or evaluation rule.

## Motivation

The catalog authoring review in
`local/ken-let-authoring-style-report.md` found zero local `let` expressions in
all 27,282 tangled Ken lines under `catalog/packages/`. Proof-heavy code instead
repeats large endpoints and intermediate states inside nested applications.

Local names are the right authoring mechanism, but the current single-binding
syntax becomes a deeply nested `let ... in let ... in ...` ladder when several
stages belong together. The existing formatter also handles a long chain of
simple nested lets poorly: a checked, idempotent probe was canonicalized into
token-by-token soft wraps such as `let left_chars : List` followed by
`Char = ...` on the next line.

A binding group addresses both problems:

- authors can present one coherent local pipeline as one construct;
- semicolons make binding boundaries explicit without repeated `in` tokens;
- sequential scope supports dependent later annotations and values; and
- kenfmt gains a stable structured layout for a binding list.

The change follows the project's reader-first principle. It spends a small
amount of surface machinery to let a human read algorithms and proofs in terms
of named stages rather than reconstructing a term tree.

## Settled semantics — do not reopen

### S1. Grammar

The accepted local expression grammar is:

```text
let_expr    ::= "let" let_binding (";" let_binding)* "in" expr
let_binding ::= ident (":" type)? "=" expr
```

- `;` occurs **between** bindings and is omitted after the last binding.
- A trailing `;` before `in` is rejected.
- `,` is not an alternate binding separator.
- A one-binding expression remains valid with its existing spelling.
- Newlines carry no syntax; the same group may be written horizontally when it
  fits.

### S2. Sequential scope

Bindings enter scope from left to right. For binding `b_i`:

- every earlier binding `b_1 ... b_(i-1)` is in scope in its annotation and
  RHS;
- `b_i` itself is not in scope in its annotation or RHS; and
- no later binding is in scope.

All group bindings are in scope in the final body.

This permits dependent staging:

```ken ignore
let
  a : Type = choose_type tag;
  x : a = choose_value tag
in
  use a x
```

The example above is schematic until the spec companion selects a self-contained
checked fixture. Its scoping behavior is normative.

### S3. Desugaring

For bindings `b1; b2; ...; bn`:

```text
let b1; b2; ...; bn in body
  == let b1 in (let b2 in ... (let bn in body))
```

The equality is by surface desugaring to the same nested resolved/core lets, not
by a new kernel conversion rule. Grouped and explicitly nested spellings must
produce structurally identical resolved/core terms modulo source spans.

### S4. Evaluation and effects

Evaluation is left to right. Each RHS is evaluated once before evaluation
continues to the next binding, and the final body is evaluated last. This is the
existing strict `Term::Let` behavior.

For effectful expressions, the group preserves that same order:

```text
let x = e1; y = e2 in body
  == bind e1 (lambda x. bind e2 (lambda y. body))
```

The grouping must never hoist an RHS across a match branch, lambda, handler, or
another binding.

### S5. Duplicate names

A grouped spelling may not bind the same identifier twice. Reject duplicates
with a focused diagnostic naming the duplicate. Although explicit nested lets
can express lexical shadowing, repeated names inside one visually sibling group
obscure the sequence and defeat its documentary purpose.

An inner let in the final body may still shadow a group binding under the
ordinary lexical rule.

### S6. Canonical authored and formatted form

A maximal directly nested, sequential local-let chain of at least two bindings
has the grouped form as its canonical kenfmt output. Thus both accepted inputs:

```ken ignore
let x = first; y = second x in finish y
```

and:

```ken ignore
let x = first in let y = second x in finish y
```

format to the first spelling when it fits the 96-column budget.

When the group does not fit, kenfmt emits:

```ken ignore
let
  x = first;
  y = second x
in
  finish y
```

The bindings are indented one level from `let`; `in` aligns with `let`; the body
is indented one level from `in`. Compound RHS expressions nest relative to their
binding. A binding type such as `List Char` remains an atomic fitting subgroup
and is never broken one token per line.

Comments retain their binding attachment and never cross a syntactic boundary.
The formatter may coalesce directly nested lets but never introduces, removes,
reorders, duplicates, or renames a binding.

## Current implementation grounding

Re-verify these locations on pickup:

- `spec/30-surface/32-grammar.md §3` admits one local binding:
  `"let" ident (":" type)? "=" expr "in" expr`.
- `crates/ken-elaborator/src/parser.rs::parse_let_expr` parses one name,
  optional annotation, RHS, `in`, and body.
- `crates/ken-elaborator/src/ast.rs::Expr::ELet` stores one name, annotation,
  RHS, and body.
- `crates/ken-elaborator/src/resolve.rs` resolves the annotation and RHS before
  pushing the binder, then resolves the body with the binder in scope. This is
  already the required nonrecursive scoping rule for one binding.
- `crates/ken-elaborator/src/elab.rs` lowers `RExpr::RLet` to `Term::Let` and
  composes effect rows from RHS and body.
- `crates/ken-kernel/src/term.rs` and `check.rs` already own the single core
  `Let`; `conv.rs` zeta-reduces it.
- `crates/ken-interp/src/eval.rs` evaluates `Term::Let` strictly and shares the
  evaluated result.
- `crates/ken-elaborator/src/layout.rs::print_let` and `is_compound_expr` own
  current formatting. `is_compound_expr` recurses through a nested `ELet`
  without treating that nested let as structurally compound when every RHS is
  simple, causing the observed soft-token layout failure.

The implementation should retain a surface-level binding-list representation
long enough for formatting, comments, and per-binding diagnostics, then lower
to the existing nested `RLet`/`Term::Let` representation. Do not add a grouped
kernel term.

## Required D0 audit

Before implementation, Language posts a short grounding audit in the WP thread:

1. Reconfirm the cited parser, AST, resolver, elaborator, lossless-span,
   formatter, effect-row, and evaluator paths on the actual base.
2. Inventory local-let uses in `.ken`, `.ken.md`, conformance fixtures, and Rust
   source strings, distinguishing authored syntax from Rust implementation
   `let`.
3. Confirm that a semicolon terminating a group RHS is unambiguous beside:
   - semicolons inside `match` braces;
   - a grouped let used as a match-arm body;
   - an RHS that is itself a local let; and
   - an RHS containing an arrow expression.
4. Pin the surface AST representation for `LetBinding`, including an individual
   source span for its name, annotation, and RHS sufficient for focused
   diagnostics and comment attachment.
5. Confirm the canonicalization comparison used by parse-preservation treats
   grouped and nested spellings as the same lowered AST.
6. Confirm no kernel, runtime IR, prelude, `Cargo.lock`, or trusted-base movement
   is required. If one appears necessary, stop and route a scope fork.

## Deliverables

### A. Spec pin — Spec enclave

Update the normative surface chapters before or with the Language build:

1. `spec/30-surface/32-grammar.md` — grouped grammar and sequential scoping.
2. `spec/30-surface/31-lexical.md` — horizontal and broken canonical layouts,
   semicolon-between/no-trailing, comment behavior, and nested-chain
   canonicalization.
3. `spec/30-surface/39-elaboration.md` — left-to-right scope extension and
   lowering to nested `Let` terms.
4. `spec/30-surface/36-effects.md` — grouped form as left-associated monadic
   sequencing in source order.
5. Any surface overview table that currently presents only the one-binding
   grammar.

CV independently derives the scoping and exact formatter-output oracles from
these rules. Expected formatter text must not be copied from the implementation
under test.

### B. Surface parser and AST — Language

1. Add a surface `LetBinding` representation with per-binding spans.
2. Parse one or more semicolon-separated bindings followed by one `in` and one
   body.
3. Preserve existing one-binding and explicitly nested spellings.
4. Reject a missing binding after `;`, trailing `;`, comma separator, duplicate
   group name, missing `in`, and malformed annotation with focused diagnostics.
5. Preserve lossless token/trivia ownership for every binding and the body.

### C. Resolution and elaboration — Language

1. Resolve bindings sequentially: resolve annotation/RHS, then push that name
   before the next binding.
2. Resolve the body with the entire group in scope and restore the prior scope
   afterward, including every error path.
3. Lower the group to nested existing `RLet`/`Term::Let` nodes in source order.
4. Preserve dependent result substitution and bidirectional checking.
5. Preserve effect-row union and strict left-to-right effect-tree lowering.
6. Make no kernel declaration, conversion, SCT, or trusted-base change.

### D. Canonical formatting — Language

1. Print the maximal direct sequential-let chain as one binding group.
2. Keep the whole group horizontal only when it fits the canonical width.
3. Otherwise emit the exact structured block from S6.
4. Format compound RHSs structurally without splaying fitting type or
   application subgroups.
5. Preserve leading, interstitial, and trailing comments at their binding.
6. Support `.ken` and exact tangled fences in `.ken.md`; non-Ken prose and fence
   markers remain byte-identical.
7. Assert exact positive layout, not only parse preservation, idempotence, and
   width.

### E. Tests and conformance — Language QA + CV

Add focused parser, resolver, elaborator, effect, lossless, formatter, CLI, and
conformance cases. The tests must exercise the mechanisms enumerated in the
acceptance criteria, not only verify that a file elaborates.

## Acceptance criteria

### AC1 — Grammar and compatibility

- Existing one-binding and explicitly nested local lets still parse and
  elaborate.
- A group with two or more `;`-separated bindings parses.
- Semicolon is between bindings with no trailing separator; comma rejects.
- Match-arm, nested-let-RHS, compound-match-RHS, and arrow-RHS discriminator
  cases parse to the intended boundary.

### AC2 — Sequential dependent scope

- A later annotation depends on an earlier bound type and elaborates.
- A later RHS consumes an earlier value and elaborates.
- The final body consumes every group binding and elaborates.
- A binding cannot reference itself or a later binding; each rejects with the
  specific unbound-name diagnostic.
- A duplicate group name rejects with its specific duplicate-binding
  diagnostic.
- Intentional shadowing in a separately nested body remains valid.

### AC3 — Identical lowering, zero TCB delta

- Grouped and explicitly nested spellings produce structurally identical
  resolved and core terms modulo spans.
- The structural oracle shows the same ordered nest of `Term::Let` nodes; an
  elaborate-only assertion is insufficient.
- `git diff -- crates/ken-kernel crates/ken-runtime Cargo.lock` is empty.
- No new `Axiom`, opaque declaration, primitive, postulate, fixpoint, SCT path,
  or trusted-base entry appears.

### AC4 — Strict evaluation and effects

- A pure evaluator case demonstrates that each staged RHS feeds the next and
  the final value is unchanged from the nested spelling.
- An effectful structural oracle proves source-order sequencing of at least two
  distinguishable effects or operations.
- A binding inside one match arm is not hoisted into the scrutinee or another
  arm.
- Grouped and nested forms emit the same ordered effect tree/core structure.

### AC5 — Canonical formatter output

Golden tests independently pin:

1. short one-binding let stays horizontal;
2. short multi-binding group stays horizontal;
3. long group uses the S6 block layout;
4. a typed `List Char` binding remains a fitting subgroup;
5. a compound `match` RHS nests under its binding;
6. a grouped let in a match arm is indented relative to that arm;
7. comments before, between, and after bindings retain their attachment;
8. explicitly nested input canonicalizes to the grouped spelling; and
9. canonical input is byte-identical after a second format.

Every golden has both directions: noncanonical input → canonical output and
canonical output → itself. Parse/lowered-AST preservation, elaboration
preservation, idempotence, and the 96-column bound all remain green, but none
substitutes for the exact positive layout assertions.

### ★ AC-DERIVE — the expected layout comes from the SPEC, never from the code

**CV derives every expected formatter text from the amended S1–S6 productions.**
**Never paste the formatter's output into an expected string.** An exact-text
test whose expected value you copied from the implementation is **a rubber stamp
wearing an oracle's clothes** — it can only ever agree with the code.

**And if two texts are equally admissible under S6: STOP and escalate.** That is
a **spec gap**, not a coin flip. *An implementer's aesthetic preference is not a
canonicalization rule* (LET-1's implementer stated this stop condition in advance
and it is now fleet doctrine).

### ★★ AC-READER — a reader must LOOK, and no test may discharge this

**Render each of the following, paste them VERBATIM into the handoff, and state
the property in a reader's terms:**

1. the broken multi-binding group (≥ 4 bindings);
2. a grouped `let` inside a `match` arm;
3. the guide's `let_staged_color` example.

**State, in words: every binding and the final body sit at the same indentation;
the body is not indented deeper than the first binding.**

> **⛔ This AC CANNOT be discharged by a passing test. If your evidence is a green
> oracle, you have not discharged it.**

**Why this AC exists — read it before you argue with it.** **LET-1 was titled
"*readable* let-chain layout."** It shipped six acceptance gates:

```
exact emitted text · AST preservation · token preservation
idempotence · ≤ 96 columns · zero trusted_base() delta
```

**All six green. NOT ONE asked whether the output was READABLE. It staircased.**

**An exact-text oracle pins WHAT the output is; it never asks whether the output
is GOOD** — and its expected value was derived from the same defective §31
production, so **it agreed with the defect by construction.** *We replaced a
stability gate with a different stability gate and called it a quality gate.*
**When a WP promises an adjective — readable, clear, fast, simple — name the gate
that measures THAT ADJECTIVE, or you will ship it unverified with every gate
green.** (Fleet memory:
`deriving-from-the-contract-cannot-detect-a-defective-contract`.)

### AC6 — Lossless and literate boundaries

- Lossless token/trivia accounting has exactly one owner for every binding
  token and comment.
- `.ken.md` formatting changes only exact tangled Ken fence bodies; prose,
  non-tangled fences, and markers are byte-identical.
- CLI `ken fmt`, `ken fmt --check`, and `ken check` cover a grouped `.ken` file
  and a grouped `.ken.md` file.

### AC7 — Spec and guide readiness

- The four normative spec chapters agree on grammar, scope, desugaring,
  evaluation order, effects, and canonical layout.
- CV's independently derived accept/reject and formatter oracles pass.
- A small checked grouped-let example lands in
  `catalog/guide/surface-reference.ken.md` only if Foundation ownership is
  available in this scheduled wave; otherwise the Foundation follow-on below is
  booked before this WP closes.

### AC8 — Verification gates

- Targeted parser/elaborator/lossless/formatter/CLI tests pass locally through
  `scripts/ken-cargo`.
- Full locked workspace tests run in CI, not as an unbounded local workspace
  invocation.
- `git diff --check` is clean.
- The final diff contains no WP identifier in production symbols, diagnostics,
  or Ken fixture identifiers.

## Out of scope — do not absorb

- No simultaneous/parallel binding semantics.
- No `let*`, `let rec`, `letrec`, or `letrec*` keyword.
- No local recursive or mutually recursive bindings.
- No fixpoint, closure-recursion, lambda-lifting, or new SCT mechanism.
- No pattern/destructuring bindings.
- No top-level multi-declaration `let` group.
- No comma separator or optional trailing semicolon.
- No automatic insertion or removal of bindings by kenfmt.
- No broad rewrite of `catalog/packages/` to introduce local names.
- No change to branch/effect placement under the guise of formatting.
- No `local/refs/` access.

If local recursion is later justified, it receives a distinct `let rec`
surface WP with mandatory signatures, captured-variable handling, and an SCT
route. It is not an extension of this parser change.

## Follow-on — Foundation catalog authoring rollout

After the syntax and formatter land, Steward should schedule a separate
Foundation-owned `catalog-let-authoring-rollout` WP grounded in
`local/ken-let-authoring-style-report.md`. It should:

1. add **Local bindings as exposition** to
   `docs/program/07-catalog-style-guide.md`;
2. expand the checked surface and proof-technique guides;
3. update `agent/playbooks/tools/write-ken.md`;
4. create the promised `agent/teams/foundation/` authoring overlays; and
5. pilot named stages in `StringBijection.ken.md` and selected small
   `Collections.ken.md` definitions before any `Map.ken.md` refinement.

Keeping that work separate prevents a language-surface WP from hiding a large
proof-source rewrite, while ensuring the new construct reaches the agents and
catalog code that motivated it.

## Gate and handoff

1. **Spec enclave** pins S1–S6 in the normative chapters; CV authors independent
   accept/reject and exact-layout oracles.
2. **Language implementer** builds parser/AST/resolution/lowering/formatter and
   targeted tests from the pin.
3. **Language QA** independently verifies the structural lowering, scoping
   negatives, effect order, comment ownership, and positive layout output.
4. **Architect terminal review** confirms zero-TCB lowering, dependent
   sequential scope, no recursion leakage, and formatter conformance.
5. **Spec/CV** ratify code-to-spec fidelity and the red-to-green conformance
   cases.
6. Steward publishes through the normal CI-gated path and books the Foundation
   follow-on before closure.

## Scheduling note

This WP touches the same parser/AST/layout seam as other Language surface and
kenfmt work. Schedule it when that ring is free rather than overlapping another
formatter or grammar WP. The spec pin and CV oracle can be prepared ahead of the
Language slot, but the implementation must build from their landed or explicitly
held candidate, not from this local draft alone.
