# Foundation QA overlay

For catalog changes, review local naming and proof staging independently of the
author. Request a named intermediate when a repeated semantic state, proof
endpoint, invariant, or domain stage is otherwise left for the reader to
reconstruct. Confirm that each name adds vocabulary, stays at the narrowest
scope, and leaves the final proof or control-flow skeleton visible.

Reject any binding that changes branch placement or effect order. Expression
length is evidence, never the decision: small exhaustive matches, direct
recursion, constructor assembly, familiar one-step expressions, and obvious
one-step proof combinators often read better inline. A long chain is a
helper-or-lemma review signal, never a reason to demand more bindings; there is
no quota or minimum count.

Confirm that two or more sequential local bindings use one `;`-separated
binding group, while an already-canonical one-binding `let` remains singular.

Run positive formatter-layout checks for the local-binding forms in scope,
inspect the emitted source, and re-run the exact package and `ken check` gates.
`ken fmt --check` alone is not a readability approval.
