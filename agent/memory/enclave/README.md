# enclave — T1 clean-room design/soundness lessons

Loaded by the T1 enclave: steward, architect, spec-author, spec-leader,
conformance-validator. Design-reasoning, soundness-review, and spec-authoring
discipline — the recurring shapes of a fork ruling, a soundness audit, or a
conformance-reconcile that generalize across WPs. A lesson tied to one specific
role's own mechanics (not shared reasoning) belongs in `roles/<role>` instead.

| Lesson | One-line |
|---|---|
| [abstraction-visibility-feature-soundness-gate](abstraction-visibility-feature-soundness-gate.md) | Soundness-gating a namespacing/visibility/abstraction build |
| [attribute-a-suite-arm-reject-before-calling-it-a-gap](attribute-a-suite-arm-reject-before-calling-it-a-gap.md) | Attribute a suite-arm reject before calling it a gap |
| [buildability-ruling-must-ground-every-axis](buildability-ruling-must-ground-every-axis.md) | A "buildable now" ruling must ground every capability axis the proof touches |
| [capability-gate-three-state-lifecycle](capability-gate-three-state-lifecycle.md) | A capability-gate has three prose states; the middle one goes stale in both directions |
| [carrier-canonicity-axis-for-lawful-class-laws](carrier-canonicity-axis-for-lawful-class-laws.md) | Carrier canonicity is a distinct soundness axis for lawful-class laws |
| [cbv-eliminator-method-laziness](cbv-eliminator-method-laziness.md) | CBV eliminator methods must be held unevaluated |
| [class-dict-explicit-vs-implicit-abstract-tyvar](class-dict-explicit-vs-implicit-abstract-tyvar.md) | A class-dict blocker over an abstract tyvar is usually only the implicit path |
| [coexist-over-subsume-when-trust-levels-differ](coexist-over-subsume-when-trust-levels-differ.md) | Coexist over subsume when trust levels differ |
| [conformance-assert-at-locked-granularity](conformance-assert-at-locked-granularity.md) | A conformance case must assert at the spec's locked granularity |
| [conformance-reconcile-inherits-spec-metatheory-bugs](conformance-reconcile-inherits-spec-metatheory-bugs.md) | Content-reconciling a conformance case inherits the spec's metatheory bugs |
| [contract-spec-defer-spelling-not-concept](contract-spec-defer-spelling-not-concept.md) | Defer the spelling, not the concept, in a wire/serialization spec |
| [deceq-on-noncanonical-carrier-inhabits-bottom](deceq-on-noncanonical-carrier-inhabits-bottom.md) | A DecEq instance over a non-canonical carrier is genuinely unsound |
| [differential-verify-which-mechanism-is-the-net](differential-verify-which-mechanism-is-the-net.md) | Differential-verify which mechanism is the actual soundness net |
| [disclaimed-framing-still-binds-your-own-companion-artifact](disclaimed-framing-still-binds-your-own-companion-artifact.md) | Disclaiming a framing doesn't protect your own companion artifact |
| [discriminating-axis-vacuous-until-capability-lands](discriminating-axis-vacuous-until-capability-lands.md) | A discriminating axis can be design-real yet build-vacuous |
| [discriminating-conformance-verdict-must-flip](discriminating-conformance-verdict-must-flip.md) | A conformance example must make the verdict actually flip |
| [dont-preempt-technical-fork-with-sequencing](dont-preempt-technical-fork-with-sequencing.md) | Don't pre-empt a technical fork with a sequencing ruling |
| [eliminator-termination-finiteness-not-stuckness](eliminator-termination-finiteness-not-stuckness.md) | An eliminator's termination argument is finiteness, not stuckness |
| [enclave-elaborates-autonomously-no-build-team-pulls](enclave-elaborates-autonomously-no-build-team-pulls.md) | The spec enclave elaborates autonomously — never pull in a build-team lead |
| [enclave-ruling-in-thread-is-not-a-durable-deliverable](enclave-ruling-in-thread-is-not-a-durable-deliverable.md) | An enclave ruling articulated in a thread is not yet a deliverable |
| [grounding-a-fabricated-citation-two-failure-modes](grounding-a-fabricated-citation-two-failure-modes.md) | Fixing a laundered citation has two failure modes past stripping the token |
| [higher-kinded-class-param-and-funext-definitional](higher-kinded-class-param-and-funext-definitional.md) | Higher-kinded class params are hard-coded to Type0; funext is definitional |
| [kernel-backed-claim-grep-the-emission-not-the-name](kernel-backed-claim-grep-the-emission-not-the-name.md) | Verify a kernel-backed claim by grepping the emission, not the name |
| [kernel-backed-obligation-certificate-vs-discrimination](kernel-backed-obligation-certificate-vs-discrimination.md) | A kernel-backed obligation can notarize a certificate without re-deriving the discrimination |
| [kernel-rejects-is-completeness-fix-is-where-soundness-converts](kernel-rejects-is-completeness-fix-is-where-soundness-converts.md) | The kernel rejecting a valid program is completeness; the soundness risk is in the fix |
| [laundered-citation-authority](laundered-citation-authority.md) | A stale citation gains false authority by propagating |
| [layer-dependent-pin-at-unconditional-layer](layer-dependent-pin-at-unconditional-layer.md) | Author a layer-dependent conformance pin at the unconditional layer |
| [obligation-must-descend-into-structure](obligation-must-descend-into-structure.md) | A proof obligation over a structured term must descend per-branch |
| [package-ecosystem-comprehensive-standard-small-contrib](package-ecosystem-comprehensive-standard-small-contrib.md) | Package-ecosystem strategy: comprehensive standard library, small contrib surface |
| [perf-primitive-vs-fix-the-evaluator-fork](perf-primitive-vs-fix-the-evaluator-fork.md) | Perf-primitive vs fix-the-evaluator: prefer fixing the evaluator |
| [proof-relevant-inductive-cannot-be-declared-at-omega](proof-relevant-inductive-cannot-be-declared-at-omega.md) | A proof-relevant multi-constructor inductive cannot be declared directly at Omega |
| [reconcile-binds-a-co-reviewers-plausible-reading-too](reconcile-binds-a-co-reviewers-plausible-reading-too.md) | The reconcile-re-derive duty binds a co-reviewer's plausible reading too |
| [reconcile-own-over-claim-then-grep-coupled](reconcile-own-over-claim-then-grep-coupled.md) | Fixing your own over-claim requires grepping the same claim in coupled artifacts |
| [reconcile-proof-rides-elaboration-merge-not-build-phase](reconcile-proof-rides-elaboration-merge-not-build-phase.md) | A reconcile fix can land in the elaboration merge, not the build phase |
| [resource-blowup-on-small-code-is-a-checker-bug](resource-blowup-on-small-code-is-a-checker-bug.md) | A resource blowup on small source code is a bug to fix at the root |
| [scope-review-vote-to-my-lane](scope-review-vote-to-my-lane.md) | Scope your review vote to your own lane |
| [sigma-sort-pi-vs-sigma-over-equating](sigma-sort-pi-vs-sigma-over-equating.md) | Pi lands in Omega by codomain; Sigma must be keyed on both components |
| [sizing-a-subsume-fix-enumerate-every-piece](sizing-a-subsume-fix-enumerate-every-piece.md) | Sizing a subsume-fix: enumerate every piece the generalization needs |
| [sole-net-prefer-structural-self-evidence-over-positional-scalar](sole-net-prefer-structural-self-evidence-over-positional-scalar.md) | When a check is the sole runtime soundness net, prefer a self-evidencing representation |
| [soundness-AC-static-vs-runtime-face](soundness-AC-static-vs-runtime-face.md) | A soundness AC often has a static face and a separately-deferrable runtime face |
| [spec-claim-kernel-admittance-vs-staging](spec-claim-kernel-admittance-vs-staging.md) | Don't claim kernel admittance by trusting a sibling chapter's prose |
| [spec-conv-omega-shortcut-trap](spec-conv-omega-shortcut-trap.md) | A proof-irrelevance shortcut that fires on the universe as well as its elements is unsound |
| [spec-enclave-always-compact-before-new-work](spec-enclave-always-compact-before-new-work.md) | The spec enclave compacts unconditionally before every new work unit |
| [spec-example-must-satisfy-its-own-rules](spec-example-must-satisfy-its-own-rules.md) | An illustrative spec example must satisfy the rules it sits under |
| [spelling-currency-sweep-separate-from-vacuity](spelling-currency-sweep-separate-from-vacuity.md) | Run a spelling-currency sweep separate from the vacuity check |
| [systems-os-kernel-interface-first-party](systems-os-kernel-interface-first-party.md) | Systems / OS-kernel interface is first-party standard-package content |
| [tested-not-trusted-posture-needs-reachability-precondition](tested-not-trusted-posture-needs-reachability-precondition.md) | A tested-not-trusted posture is sound only with a reachability precondition |
| [transcription-moves-contract-requires-three-part-reconcile](transcription-moves-contract-requires-three-part-reconcile.md) | A transcription that moves the WP contract needs a three-part reconcile |
| [transport-schema-degenerate-endpoint-trap](transport-schema-degenerate-endpoint-trap.md) | Review a transport/cast schema at non-degenerate endpoints |
| [trust-level-claim-grep-per-check-both-directions](trust-level-claim-grep-per-check-both-directions.md) | A trust-level claim must be grepped per check and in both directions |
| [trust-level-prose-vs-locked-adr-crosscheck](trust-level-prose-vs-locked-adr-crosscheck.md) | Cross-check a trust-level characterization against the locked ADR |
| [trusted-by-typing-guarantee-is-not-kernel-proved-Q](trusted-by-typing-guarantee-is-not-kernel-proved-Q.md) | A by-typing trusted guarantee is not kernel-proved; it projects to P, never Q |
| [trusted-primitive-refinement-codomain-witness](trusted-primitive-refinement-codomain-witness.md) | A trusted primitive with a refinement-typed codomain must establish the refinement |
| [tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases](tt-vs-refl-endpoint-rule-for-inductive-equal-law-bases.md) | The tt-vs-Refl endpoint rule for inductive Equal-law bases |
| [untrusted-layer-backstop-hole-for-omissions](untrusted-layer-backstop-hole-for-omissions.md) | An untrusted layer's bug is safe only for what it supplies, not what it omits |
| [verdict-mapping-silence-is-a-latent-conformance-bug](verdict-mapping-silence-is-a-latent-conformance-bug.md) | A verdict-mapping silence is a latent conformance bug |
| [verified-showcase-predicate-must-be-defined-not-postulated](verified-showcase-predicate-must-be-defined-not-postulated.md) | A verified-X showcase whose predicate is postulated is vacuous |
| [verify-proposed-fix-excludes-the-counterexample](verify-proposed-fix-excludes-the-counterexample.md) | Verify a proposed fix actually excludes the counterexample |
| [verify-symbol-exposure-not-just-call-site-safety](verify-symbol-exposure-not-just-call-site-safety.md) | Verify symbol exposure structurally, not by checking call-site safety |
| [wp-frame-stale-vs-landed-kernel](wp-frame-stale-vs-landed-kernel.md) | A WP frame's description of the kernel can be stale vs the landed kernel |
