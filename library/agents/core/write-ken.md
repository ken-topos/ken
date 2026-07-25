# Write Ken

## 1. Use when

Use this module before writing or changing `.ken` or `.ken.md` source. Do not
use it as authority for a new language feature, a federation workflow, or a
claim that an unchecked spelling is supported.

## 2. Prerequisites

Load `read-ken.md`. Select one task module: `write-program.md`,
`author-package.md`, `prove-or-repair.md`, or
`effects-and-capabilities.md`. Read the cited spec section whenever the task
turns on an exact language rule.

## 3. Current capability

The landed authoring loop supports pure and effectful declarations, dependent
types, checked proof terms, literate Ken fences, formatting, checking, reference
execution, and native builds. The supported CLI surface is `ken check`,
`ken run`, `ken native-build`, and `ken fmt`.

## 4. Canonical forms

Probe the smallest standalone spelling before expanding it. This checked
example distinguishes a reduced equality from a stuck equality:

```ken example
fn bool_and (a : Bool) (b : Bool) : Bool =
  match a {
    True ↦ b;
    False ↦ False
  }

theorem collapsed : Equal Bool (bool_and True True) True = Proved

theorem stuck (x : Bool) : Equal Bool (bool_and x x) (bool_and x x) = Refl
```

The ordinary loop is:

```text
write the smallest probe
ken check <file>
ken fmt <file>
inspect the formatted source
ken check <file>
```

## 5. Invariants and prohibitions

- A brief's spelling is intent until the current checker accepts it.
- Required contracts, propositions, and trust boundaries belong in checked
  language constructs, not only in comments.
- At an equality goal, reduce both endpoints before choosing `Proved` or
  `Refl`.
- Name local `let` bindings for their domain or proof role, not for the
  mechanism that computes them. Keep each binding in the narrowest useful
  scope, preserve branch and effect order, and write two or more sequential
  bindings as one `;`-separated binding group before `in`.
- Keep a familiar one-step expression, small exhaustive match, direct
  recursion, or single constructor assembly inline when a local name would
  merely repeat its syntax. Expression length is evidence, never the decision;
  there is no binding quota or depth threshold.
- `(C T)` is a class applied to its head: the dictionary's type, not an
  instance value. Outside a resolved `where` call, use the synthesized global
  `C_instance_T` as the value and project its fields like an ordinary record.
  Projecting a field from `(C T)` is invalid.
- New type-like names use PascalCase; new functions and fields use snake_case.
- Do not invent imports, primitives, effects, capabilities, or proof tactics.

## 6. Decision procedure

1. Write the contract or result type before the implementation.
2. Probe any unfamiliar syntax in the smallest standalone file.
3. Implement one checked step at a time.
4. Reduce proof endpoints and choose the terminal from the resulting goal.
5. Format, inspect the formatted source, and check again.
6. Stop at the first unsupported form or unresolved trust boundary and cite it.

## 7. Failure signatures

| Signature | Likely cause | Next action |
|---|---|---|
| parse error | unsupported or misplaced syntax | reduce to a one-declaration probe |
| `Refl expects an Eq-shaped goal` | goal reduced to `Top` | inspect endpoints; try `Proved` only if both collapse |
| `Proved` type mismatch | equality remains stuck | inspect endpoints; use `Refl` only for reflexive equality |
| pure library has no `main` entrypoint | `ken run` used on a library | use `ken check` |
| effect escapes declared row | omitted or wrong `visits` row | load `effects-and-capabilities.md` |

## 8. Validation

Use `ken check` for pure libraries and literate packages, `ken run` only for a
runnable `proc main`, and `ken fmt --check` only as a formatting check. A
formatter fixed point is not a readability verdict. Re-run the exact check
after formatting.

## 9. Authority and sources

Local-binding guidance comes from
`docs/program/07-catalog-style-guide.md` §6.1. The class-record and
instance-value distinction comes from `spec/30-surface/33-declarations.md`
§5.2–5.3 and the checked example in
`catalog/guide/surface-reference.ken.md` lines 243–253. Proof-terminal guidance
comes from `catalog/guide/proof-techniques.ken.md` §1. The verified revision is
recorded in `library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

The catalog does not yet provide a standalone checked example for every landed
surface feature, and `ken check` does not prove runtime or native behavior.
Cross-package reuse may require a loader-aware test rather than a single-file
catalog check. When the guide lacks a form or the probe rejects it, stop and
report the gap; do not synthesize plausible Ken syntax.
