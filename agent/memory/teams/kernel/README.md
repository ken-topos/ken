# teams/kernel — kernel-team lessons

Loaded by the kernel team (leader / implementer / QA), in addition to `fleet`,
`build`, and the role scopes. Kernel-specific soundness and mechanism lessons —
the trusted base, conversion, reduction, termination, and the elaborator seams
that feed the kernel.

| Lesson | One-line |
|---|---|
| [exhaustive-term-traversals](exhaustive-term-traversals.md) | A new `Term` variant needs every shared exhaustive walker extended; remove catch-alls so the compiler enforces it. |

_(Proof slice — more kernel lessons land here in the full migration:
conv/reduction-arm termination stress, hand-built `Elim` motive/method gotchas,
`kernel-rejects = completeness`, trusted-primitive refinement-codomain holes.)_
