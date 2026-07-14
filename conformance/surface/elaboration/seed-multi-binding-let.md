# LET-4 multi-binding `let` conformance seed

These cases pin the sequential local-binding group specified by
`docs/program/wp/let4-multi-binding-let.md` S1–S6. They extend the existing
single-binding roots in `seed-elaboration.md` and the existing effect-order root
in `../effects/seed-effects.md`; they do not duplicate those mechanisms.

**Status and reachability.** The one-binding and explicitly nested controls are
LIVE on the grounding base. Every source containing a grouped binding separator
is **RED-UNTIL (LET-4 surface)**: before the grouped grammar lands it cannot
produce the lossless/typed source consumed by resolution or formatting. Every
grouped formatter-byte assertion is therefore
**RED-UNTIL (LET-4 surface + formatter)**. The build must rerun
`parse_lossless` over every noncanonical input and every canonical expected
block below before flipping either label.

The mandatory mechanical sweep on exact base `6d6504d6` produced:

| Layout case | Noncanonical input | Canonical expected block |
|---|---|---|
| D1 one binding | reaches | reaches |
| D2 short group | does not reach | does not reach |
| D3 long group | does not reach | does not reach |
| D4 compound match RHS | does not reach | does not reach |
| D5 group in match arm | does not reach | does not reach |
| D6 comments | does not reach | does not reach |
| D7 nested-to-group | reaches | does not reach |
| D8 literate body | does not reach | does not reach |

The sweep called the real `parse_lossless` on all sixteen strings. This is why
D1 is live, why D7's input is a live compatibility control, and why every
grouped expected block names the surface precondition as well as the formatter.

Expected text in section D is derived from S6 and the general `31 §1d` physical
text, spacing, comment, match, and declaration productions. It is not sampled
from a formatter implementation. If the amended normative chapters admit a
second output for any fixture, this seed must stop for a spec correction rather
than choose between them.

---

## A. Grammar and separator boundaries

### surface/elaboration/let4-one-binding-compatibility-control (control)

- spec: S1; `32 §3`; `39 §5.2`–`§5.4`
- given: `const one : Nat = let value : Nat = Zero in value`
- expect: **LIVE — accepted** and lowered through the existing single local
  binding.
- why: the new list grammar has cardinality one as a compatibility case. A
  parser that requires a separator or two bindings regresses the existing
  surface.

### surface/elaboration/let4-two-binding-group-accepted

- spec: S1; `32 §3`; LET-4 AC1
- given: `const two : Nat = let first = Zero; second = Suc first in second`
- expect: **RED-UNTIL (LET-4 surface) — accepted** as exactly two bindings and
  one final body.
- why: this is the positive arm for the separator cases. The second occurrence
  of `first` also makes a parser that truncates the list at `;` observable at
  resolution.

### surface/elaboration/let4-explicit-nested-compatibility-control (control)

- spec: S1/S3; `32 §3`; `39 §5.3`–`§5.5`; LET-4 AC1
- given: `const two_nested : Nat = let first = Zero in let second = Suc first
  in second`
- expect: **LIVE — accepted** and lowered to the existing ordered two-node
  `Term::Let` nest.
- why: S6 changes this spelling's canonical output, not its acceptance or
  meaning. This live arm catches a parser or lowering change that accidentally
  removes the explicitly nested compatibility form.

### surface/elaboration/let4-separator-is-between-bindings

- spec: S1; `32 §3`; LET-4 AC1
- given: the accepted source above, compared with
  `let first = Zero; in first` and
  `let first = Zero, second = first in second`
- expect: **RED-UNTIL (LET-4 surface)** — the first malformed source rejects at
  `in` with a focused missing-binding-after-separator error; the second rejects
  at `,` as an invalid separator. Neither may recover by silently dropping a
  separator or inventing a binding.
- why: correct `;`-between/no-trailing accepts while the two one-token boundary
  changes reject. A permissive optional-trailing or comma-list grammar flips a
  negative arm.

### surface/elaboration/let4-match-rhs-semicolon-partition

- spec: S1; `32 §3` and the `match` arm grammar; LET-4 D0.3/AC1
- given:

  ```ken ignore
  fn chosen (input : Nat) : Nat =
    let first = match input { Zero ↦ Zero; Suc n ↦ n };
        second = Suc first
    in second
  ```

- expect: **RED-UNTIL (LET-4 surface) — accepted**. The semicolon inside
  `{ ... }` separates match arms; after the closing `}` the next semicolon
  separates `first` from `second`. The parser produces two let bindings, and
  `first`'s RHS is one complete two-arm match.
- why: braces put the parser in the arm-list state, while `let ... in` puts it
  in the binding-list state. A parser that treats both semicolons as members of
  one undifferentiated list changes the binding count or match-arm count.

### surface/elaboration/let4-grouped-let-is-one-match-arm-body

- spec: S1/S4; `32` match grammar; LET-4 D0.3/AC1/AC4
- given:

  ```ken ignore
  fn branch_value (flag : Bool) : Nat =
    match flag {
      True ↦ let first = Zero; second = Suc first in second;
      False ↦ Zero
    }
  ```

- expect: **RED-UNTIL (LET-4 surface) — accepted** with two match arms. The
  `;` between `first` and `second` is consumed by the let-binding list; the
  `;` after `second` is consumed only after the mandatory `in` has closed that
  list, so it separates the `True` and `False` arms.
- why: this is the inverse orientation of the preceding boundary. It would
  fail if a grouped let were hoisted out of its arm or if the binding separator
  prematurely ended the arm body.

### surface/elaboration/let4-nested-let-and-arrow-rhs-boundaries

- spec: S1/S3; `32 §3`; LET-4 D0.3/AC1
- given:

  ```ken ignore
  const nested_rhs : Nat =
    let first = let inner = Zero in inner;
        step : Nat → Nat = λn. Suc n;
        result = step first
    in result
  ```

- expect: **RED-UNTIL (LET-4 surface) — accepted** as three outer bindings.
  The inner `in inner` closes only the nested RHS; `Nat → Nat` remains one type
  annotation; the two outer semicolons remain binding separators.
- why: pins the other two delimiter-adjacent D0 forms without changing the
  surrounding group. A greedy RHS or arrow parser changes the outer binding
  count and fails the structural result.

---

## B. Sequential, nonrecursive scope

### surface/elaboration/let4-dependent-later-annotation-accepted

- spec: S2; `39 §5.3`–`§5.5`; LET-4 AC2
- given: `const dependent : Nat = let a : Type = Nat; x : a = Zero in x`
- expect: **RED-UNTIL (LET-4 surface) — accepted**. `a` resolves in the second
  annotation, and `x` has the elaborated type selected by the first binding.
- why: a simultaneous-scope or annotation-before-prior-bindings
  implementation rejects this valid dependent stage.

### surface/elaboration/let4-earlier-value-feeds-later-rhs-and-body

- spec: S2/S4; `39 §5.3`–`§5.5`; LET-4 AC2
- given: `const staged : Pair Nat Nat = let first = Zero; second = Suc first in
  mk_pair Nat Nat first second`
- expect: **RED-UNTIL (LET-4 surface) — accepted**; the later RHS resolves
  `first`, and the body resolves and consumes both `first` and `second`.
- why: together with the two negative cases below, this is a controlled
  left-to-right scope experiment rather than an elaborate-only green case.

### surface/elaboration/let4-self-reference-is-out-of-scope

- spec: S2; `39 §5.3`; LET-4 AC2
- given: `const self_bad : Nat = let self_group_probe : Nat =
  self_group_probe; ok = Zero in ok`
- expect: **RED-UNTIL (LET-4 surface) — rejected** with
  `UnresolvedCon { name = "self_group_probe" }` at the occurrence in the first
  RHS. A bare `is_err`, a later type error, or resolving the occurrence to its
  own binder does not satisfy this case.
- why: the binder is pushed only after its annotation and RHS resolve. The
  unique name prevents a global from making the negative vacuous.

### surface/elaboration/let4-self-reference-in-annotation-is-out-of-scope

- spec: S2; `39 §5.3`; LET-4 AC2
- given: `const self_type_bad : Nat = let self_type_probe : self_type_probe =
  Zero; ok = Zero in ok`
- expect: **RED-UNTIL (LET-4 surface) — rejected** with
  `UnresolvedCon { name = "self_type_probe" }` at the annotation occurrence.
  The harness must prove resolution stopped there; a later type error is not
  the required result.
- why: S2 applies the nonrecursive boundary independently to annotations and
  right-hand sides. The preceding case alone would leave the annotation path
  free to push the binder early.

### surface/elaboration/let4-later-reference-is-out-of-scope

- spec: S2; `39 §5.3`; LET-4 AC2
- given: `const later_bad : Nat = let first : Nat = later_group_probe;
  later_group_probe = Zero in first`
- expect: **RED-UNTIL (LET-4 surface) — rejected** with
  `UnresolvedCon { name = "later_group_probe" }` at the first RHS occurrence.
- why: the same source accepts after only the binding order is reversed. This
  distinguishes sequential scope from simultaneous scope while holding names,
  types, and body fixed.

### surface/elaboration/let4-later-type-is-out-of-scope

- spec: S2; `39 §5.3`; LET-4 AC2
- given: `const later_type_bad : Nat = let first : later_type_probe = Zero;
  later_type_probe : Type = Nat in first`
- expect: **RED-UNTIL (LET-4 surface) — rejected** with
  `UnresolvedCon { name = "later_type_probe" }` at the first annotation.
  Reversing only the binding order accepts.
- why: this is the annotation counterpart to the later-RHS case. A parser or
  resolver that predeclares all group names would accept both and fail this
  verdict flip.

### surface/elaboration/let4-duplicate-name-is-focused-reject

- spec: S5; LET-4 AC2
- given: `const duplicate : Nat = let stage = Zero; stage = Suc stage in stage`
- expect: **RED-UNTIL (LET-4 surface) — rejected** by the specific
  duplicate-let-binding diagnostic carrying `name = "stage"` and pointing at
  the second binding name. The spec does not yet lock a Rust enum spelling, so
  this case pins the diagnostic category, payload, and span rather than
  inventing one.
- why: under ordinary nested-let shadowing the source has a meaning; the focused
  group diagnostic is therefore the only observable that distinguishes S5 from
  silently lowering first and applying the nested rule.

### surface/elaboration/let4-nested-body-shadowing-remains-valid

- spec: S5; ordinary lexical shadowing in `39 §5.3`; LET-4 AC2
- given: `const shadow : Nat = let stage = Zero; next = Suc stage in let stage =
  Suc next in stage`
- expect: **RED-UNTIL (LET-4 surface) — accepted**. The inner single-binding let
  is outside the sibling group and may shadow the group binder.
- why: positive de-selection prevents S5 from becoming a blanket ban on lexical
  shadowing.

---

## C. Lowering, evaluation, and effects

### surface/elaboration/let4-grouped-and-nested-core-are-identical

- spec: S3; `39` local-let lowering; LET-4 AC3
- given: the pair
  `let first = Zero; second = Suc first in Suc second` and
  `let first = Zero in let second = Suc first in Suc second`
- expect: **RED-UNTIL (LET-4 surface)** — after erasing source spans, both
  resolved trees and stable core serializations are byte-identical. Each is the
  same ordered two-node `Term::Let` nest: the outer value is `Zero`; the inner
  value refers to the outer binder; the final body refers to the inner binder.
  The `trusted_base()` sets are equal.
- why: two successful elaborations are green-vs-green under a grouped AST that
  never lowers correctly. Ordered structural identity makes a swapped or
  simultaneous lowering observable.

### surface/elaboration/let4-strict-pure-evaluation-matches-nested

- spec: S4; `42 §3.2`; LET-4 AC4
- given: the grouped and nested pair from the preceding case
- expect: **RED-UNTIL (LET-4 surface)** — each evaluates to `Suc (Suc Zero)`;
  each RHS is evaluated once before the next stage.
- why: the concrete value verifies feed-forward evaluation. Core identity is
  still the stronger order/sharing oracle; this value case does not substitute
  for it.

### surface/elaboration/let4-sugar-adds-no-trust-or-core-form

- spec: S3; `39 §1`; LET-4 AC3
- given: elaborate the grouped source from
  `let4-grouped-and-nested-core-are-identical` in a fresh environment, while
  recording the core constructors and `trusted_base()` before and after
- expect: **RED-UNTIL (LET-4 surface)** — the output uses only the existing
  `Term::Let` constructor, adds no opaque declaration or trusted-base entry,
  and requires no kernel/runtime-IR/prelude/Cargo change.
- why: the grouped surface is sugar. Core equality with the nested control plus
  exact trusted-base set equality catches both an accidental new core form and
  a hidden postulate used to justify the lowering.

### surface/elaboration/let4-effect-tree-is-source-ordered

- spec: S4; `36 §2.2`/`§2.4`; LET-4 AC4; existing interaction-tree order home
  `../effects/seed-effects.md` EFF2
- given: grouped and explicitly nested forms of
  `let first = perform Console (Write "1"); second = perform Clock now in second`
- expect: **RED-UNTIL (LET-4 surface)** — both lower to the same ordered tree
  `Vis (Write "1") (λ _. Vis now (λ second. Ret second))`; the observable mock
  trace is `[Console.Write "1", Clock.now]`.
- why: two distinguishable effects make source order observable. Reversing or
  hoisting a binding changes the `Vis` spine and trace; merely comparing final
  values would be vacuous.

---

## D. Exact canonical layout — both orientations

For every D case the harness performs both assertions:

1. the stated noncanonical/alias input formats to the exact canonical bytes;
2. those canonical bytes format to themselves.

It also compares lowered AST/core identity before and after formatting. Raw
surface-token equality is not required when S6 coalesces repeated `let`/`in`
tokens into the grouped spelling.

### surface/formatting/let4-short-one-binding-stays-horizontal

- spec: S6; `31 §1d`; LET-4 AC5.1
- given: `const tiny : Nat = let x : Nat = Zero in x`
- expect: **LIVE canonical bytes**:

  ```ken ignore
  const tiny : Nat = let x : Nat = Zero in x
  ```

- why: the grouped canonicalization rule does not pessimize a fitting
  cardinality-one let.

### surface/formatting/let4-short-group-is-horizontal

- spec: S6; `31 §1d`; LET-4 AC5.2
- given: `const tiny : Nat = let x=Zero ; y = Suc x in y`
- expect: **RED-UNTIL (LET-4 surface + formatter)** — canonical bytes:

  ```ken ignore
  const tiny : Nat = let x = Zero; y = Suc x in y
  ```

- why: the complete expression fits within 96 display columns, so S6 selects
  the unique horizontal production; spacing follows `31 §1d`.

### surface/formatting/let4-long-group-uses-one-flat-level

- spec: S6; `31 §1d`; LET-4 AC5.3/AC5.4
- given:

  ```ken ignore
  fn chars (left : String) (right : String) : List Char = let left_chars:List Char=string_to_list_char left ; right_chars : List Char = string_to_list_char right; joined_chars: List Char=append Char left_chars right_chars ; final_chars : List Char = joined_chars in final_chars
  ```

- expect: **RED-UNTIL (LET-4 surface + formatter)** — canonical bytes:

  ```ken ignore
  fn chars (left : String) (right : String) : List Char =
    let
      left_chars : List Char = string_to_list_char left;
      right_chars : List Char = string_to_list_char right;
      joined_chars : List Char = append Char left_chars right_chars;
      final_chars : List Char = joined_chars
    in
      final_chars
  ```

- why: the whole expression does not fit, so S6 selects the block production.
  All four bindings and the final body have the same indentation; `List Char`
  stays an atomic fitting subgroup, and there is no trailing `;`.

### surface/formatting/let4-compound-match-rhs-nests-under-binding

- spec: S6; `31 §1d` match production; LET-4 AC5.5
- given: the guide declaration with its group written horizontally and an ASCII
  match arrow:

  ```ken ignore
  fn let_staged_color (c : Color) : Bool = let selected_red=is_red c; confirmed_red = match selected_red { True |-> True ; False |-> False } in confirmed_red
  ```

- expect: **RED-UNTIL (LET-4 surface + formatter)** — canonical bytes:

  ```ken ignore
  fn let_staged_color (c : Color) : Bool =
    let
      selected_red = is_red c;
      confirmed_red =
        match selected_red {
          True ↦ True;
          False ↦ False
        }
    in
      confirmed_red
  ```

- why: the first fitting RHS remains beside its binding; the compound match
  nests one level under the second binding. The group body returns to the
  bindings' indentation rather than continuing a nested-let staircase.

### surface/formatting/let4-group-in-match-arm-has-disjoint-semicolons

- spec: S1/S6; `31 §1d` match production; LET-4 AC5.6
- given:

  ```ken ignore
  fn branch_value (flag : Bool) : Nat = match flag { True |-> let first=Zero ; second=Suc first in second ; False |-> Zero }
  ```

- expect: **RED-UNTIL (LET-4 surface + formatter)** — canonical bytes:

  ```ken ignore
  fn branch_value (flag : Bool) : Nat =
    match flag {
      True ↦
        let
          first = Zero;
          second = Suc first
        in
          second;
      False ↦ Zero
    }
  ```

- why: the binding `;` follows `first`; the arm `;` follows the complete body
  `second` after `in`. The final body aligns with both binding rows, relative to
  the arm, and the let is not hoisted.

### surface/formatting/let4-comments-retain-binding-attachment

- spec: S6; `31 §1d` comments; LET-4 AC5.7
- given:

  ```ken ignore
  const commented : Nat = let -- establish the seed
  first=Zero ; -- consume the prior stage
  second = Suc first; result=Suc second -- final stage
  in result
  ```

- expect: **RED-UNTIL (LET-4 surface + formatter)** — canonical bytes:

  ```ken ignore
  const commented : Nat =
    let
      -- establish the seed
      first = Zero;
      -- consume the prior stage
      second = Suc first;
      result = Suc second  -- final stage
    in
      result
  ```

- why: exact text alone would be green under comment relocation. The harness
  additionally asserts each comment's attached binding identity and token
  interval; none crosses a binding separator or `in`.

### surface/formatting/let4-nested-chain-coalesces-to-group

- spec: S3/S6; `31 §1d`; LET-4 AC5.8
- given:
  `const tiny : Nat = let x = Zero in let y = Suc x in y`
- expect: **RED-UNTIL (LET-4 surface + formatter)** — the exact horizontal
  bytes from `let4-short-group-is-horizontal`:

  ```ken ignore
  const tiny : Nat = let x = Zero; y = Suc x in y
  ```

- why: this is S6's explicit canonicalization direction. The reverse input is
  already the fixed point, and both lower to the same ordered core nest.

### surface/formatting/let4-literate-fence-changes-only-group-body

- spec: S6; `31 §1d` literate source; LET-4 AC6
- given: this document, with irregular grouped spacing:

  ````markdown
  Narrative → stays byte-identical.

  ```ken example
  const tiny : Nat = let x=Zero ; y=Suc x in y
  ```

  ```text
  let x = not Ken
  ```
  ````

- expect: **RED-UNTIL (LET-4 surface + formatter)** — exact bytes:

  ````markdown
  Narrative → stays byte-identical.

  ```ken example
  const tiny : Nat = let x = Zero; y = Suc x in y
  ```

  ```text
  let x = not Ken
  ```
  ````

- why: the recognized Ken body takes the same D2 canonical form. Markdown,
  fence markers, and the unrecognized `text` body are byte-identical, so a raw
  document-wide rewrite cannot pass.

### surface/formatting/let4-cli-covers-plain-and-literate-groups

- spec: S6; `31 §1d` physical and literate source; LET-4 AC6
- given: temporary `.ken` and `.ken.md` files containing the D2 and D8 inputs,
  respectively
- expect: **RED-UNTIL (LET-4 surface + formatter)** — `ken fmt` produces the
  exact D2 and D8 canonical bytes; `ken fmt --check` then succeeds without a
  byte change; and `ken check` accepts each formatted file through its real
  plain-source or literate pipeline.
- why: direct library formatting can be green while CLI routing omits a file
  kind or checks the pre-format bytes. Reusing the exact goldens makes this an
  integration gate, not a second layout oracle.

---

## E. AC-READER — human judgment, not a test result

The handoff must paste, verbatim, the canonical outputs from:

1. `let4-long-group-uses-one-flat-level`;
2. `let4-group-in-match-arm-has-disjoint-semicolons`; and
3. `let4-compound-match-rhs-nests-under-binding` (`let_staged_color`).

A human-equivalent reader must state that in each output every sibling binding
and the final body sit at the same indentation relative to their `let`/`in`, and
that the body is not indented deeper than the first binding. Passing the byte
goldens, fixed-point check, 96-column bound, AST/core comparison, or trust check
does **not** discharge this section.

---

## Coverage map

| LET-4 acceptance axis | Cases |
|---|---|
| AC1 grammar and compatibility | A1–A7 |
| AC2 sequential dependent scope | B1–B8 |
| AC3 identical lowering / zero trust | C1, C3 |
| AC4 strict evaluation and effects | C2, C4, A5 |
| AC5 exact canonical formatter output | D1–D8 |
| AC-DERIVE | section D derivation preamble and per-case `why` |
| AC-READER | section E; never machine-discharged |
| AC6 lossless/literate boundaries | D6, D8, D9, and existing B1/B4 corpus gates |
