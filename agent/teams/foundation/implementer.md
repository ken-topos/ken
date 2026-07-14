# Foundation implementer overlay

For any edit to `catalog/**/*.ken` or `catalog/**/*.ken.md`, load
`agent/playbooks/tools/write-ken.md` before authoring. After the proof or
behavior closes, plan a final exposition pass: name a repeated non-atomic term,
proof endpoint, invariant, or domain stage with a local `let` when the name lets
the reader describe the remaining body at a higher level than the right-hand
side. Bind at the narrowest scope, preserve branch and effect order, and use a
top-level helper for recursion or genuine reuse.

Expression length is evidence, never the decision. Keep a familiar one-step
expression, small exhaustive match, direct recursion, constructor assembly, or
obvious one-step proof inline when a binding would only hide its syntax. Name
roles such as `sorted_tail` or `left_round_trip`, never `tmp` or `value2`; split
a helper or lemma when a long chain creates a local namespace. Do not prescribe
or target a binding count.

Run `ken fmt` and the exact affected package checks, inspect the formatted
source itself, and re-run `ken check`. A formatter fixed point is a syntax
property, not a readability verdict.
