# build/qa — cross-team QA lessons

Loaded by every QA role, in addition to `fleet`, `build`, and the team/role
scopes. Independent-verification discipline: green-vs-green traps,
discriminating-case construction, and the QA gate's own failure modes.

| Lesson | One-line |
|---|---|
| [composition-wp-real-producer-may-be-deferred-engine](composition-wp-real-producer-may-be-deferred-engine.md) | Verify a composition WP's named producers are actually landed |
| [conformance-hand-feeds-the-deliverable](conformance-hand-feeds-the-deliverable.md) | A conformance test can hand-feed the very deliverable it should validate |
| [discriminating-flip-must-be-checked-per-test](discriminating-flip-must-be-checked-per-test.md) | An empirical discriminating flip must be checked test-by-test |
| [discriminator-negative-arm-must-be-expressible-and-reaching](discriminator-negative-arm-must-be-expressible-and-reaching.md) | A discriminator's negative arm can be vacuous if no upstream mechanism can express it |
| [fold-in-doc-only-conditions-while-holding-branch](fold-in-doc-only-conditions-while-holding-branch.md) | Fold in reviewer-approved doc-only fixes directly while holding a branch |
| [isolate-executed-vs-present-before-naming-perf-cause](isolate-executed-vs-present-before-naming-perf-cause.md) | Isolate executed-vs-present before naming a perf cause |
| [isolate-mechanism-from-orthogonal-fail-closed-gates](isolate-mechanism-from-orthogonal-fail-closed-gates.md) | Isolate a mechanism from orthogonal fail-closed gates when probing a boundary |
| [structural-reachability-beats-empirical-probe-for-dead-code-fix](structural-reachability-beats-empirical-probe-for-dead-code-fix.md) | Structural reachability beats an empirical probe for a dead-code fix |
| [taint-axis-orientation-needs-distinguishing-pair](taint-axis-orientation-needs-distinguishing-pair.md) | Any classification discriminator needs a non-degenerate distinguishing pair |
| [two-arm-producer-needs-a-case-per-arm](two-arm-producer-needs-a-case-per-arm.md) | A two-arm (or multi-arm) producer needs a discriminating case per arm |
