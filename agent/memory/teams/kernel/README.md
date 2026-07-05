# teams/kernel — kernel-team lessons

Loaded by the kernel team (leader / implementer / QA), in addition to `fleet`,
`build`, and the role scopes. Kernel-specific soundness and mechanism lessons —
the trusted base, conversion, reduction, termination, and the elaborator seams
that feed the kernel.

| Lesson | One-line |
|---|---|
| [cast-direction-test-at-nondegenerate-endpoints](cast-direction-test-at-nondegenerate-endpoints.md) | Test a directional cast at non-degenerate endpoints |
| [conv-reduction-arm-gate-needs-termination-stress](conv-reduction-arm-gate-needs-termination-stress.md) | A conv/reduction-arm gate needs a termination stress test |
| [dependent-match-construction-fails-closed-via-infer-elim](dependent-match-construction-fails-closed-via-infer-elim.md) | Dependent-match construction is not in the TCB; it fails closed via infer_elim |
| [enriching-opaque-former-kind-is-kernel-clean](enriching-opaque-former-kind-is-kernel-clean.md) | Enriching an opaque former's kind is kernel-clean |
| [exhaustive-term-traversals](exhaustive-term-traversals.md) | A new `Term` variant needs every shared exhaustive walker extended |
| [gate-widening-exposes-latent-bugs-in-newly-reachable-code](gate-widening-exposes-latent-bugs-in-newly-reachable-code.md) | Gate-widening exposes latent bugs in newly-reachable code |
| [hand-built-elim-motive-and-method-gotchas](hand-built-elim-motive-and-method-gotchas.md) | Hand-built Term::Elim motive/method gotchas |
| [kernel-completeness-gap-shapes](kernel-completeness-gap-shapes.md) | Three recurring shapes of kernel completeness gap (intro/elim, congruence, whnf-asymmetry) |
| [sct-completeness-fix-conservative-construction](sct-completeness-fix-conservative-construction.md) | An SCT completeness fix must use conservative construction |
| [sct-unapplied-self-reference-over-accepts](sct-unapplied-self-reference-over-accepts.md) | An SCT gate keyed on applied occurrences over-accepts unapplied self-reference |
| [trust-root-reduction-change-needs-full-workspace-gate](trust-root-reduction-change-needs-full-workspace-gate.md) | Any trust-root reduction/whnf change needs a full-workspace gate, not `-p ken-kernel` |
| [trust-root-test-coverage-discipline](trust-root-test-coverage-discipline.md) | How to write a trust-root type-checker test suite that actually catches soundness bugs |
