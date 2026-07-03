# Tree traversal

In-order traversal of a binary tree.

Reference: <https://rosettacode.org/wiki/Tree_traversal>

## Status

**Blocked — genuine, high-severity capability gap, see `KNOWN-GAP.md`.**
A `data` type with a constructor holding 2+ fields of its own recursive
type (the ordinary shape of any binary tree) cannot be `match`ed. This
isn't a workaround-able ordering or encoding problem — the dependent-match
motive computation itself produces a type mismatch, isolated via a minimal
repro fully independent of this example's own traversal logic. Blocks every
ordinary binary tree, not just this example.

Routed to language-leader / Steward, with a root-cause investigation
in progress separately. The fix (an elaborator defect in
`crates/ken-elaborator/src/elab.rs`'s match-motive construction) is its own
properly-gated fix, not patched here.
