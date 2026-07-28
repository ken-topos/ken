# roles/conformance-validator — conformance-validator-specific lessons

Loaded by conformance-validator, in addition to `fleet` and `enclave`. Lessons
specific to building and guarding the black-box conformance corpus and casting
the independent Spec review vote.

| Lesson | One-line |
|---|---|
| [builtins-tcb-audit-disciplines](builtins-tcb-audit-disciplines.md) | Auditing a native-vs-derived primitive-op registry |
| [conformance-oracle-grounding-fallback](conformance-oracle-grounding-fallback.md) | How to ground /conformance expected results when the prototype oracle isn't runnable |
| [conformance-validator-casts-spec-review-vote](conformance-validator-casts-spec-review-vote.md) | The conformance-validator casts the Spec review vote on every merge Decision touching spec/conformance |
| [delivery-contract-op-list-can-overscope](delivery-contract-op-list-can-overscope.md) | Casting the Spec vote on a delivery contract: re-derive scope from source |
| [spec-crossref-must-resolve-to-content-not-a-number](spec-crossref-must-resolve-to-content-not-a-number.md) | A spec cross-reference must resolve to its claimed CONTENT, not just a `§`-number — six instances, one discipline |
