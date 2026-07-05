---
scope: fleet
audience: (see scope README)
source: private memory `verify-field-order-arity-against-declaration-not-prose`
---

# Verify field order/arity against the declaration, not prose

A build's prose citing a payload type by name (`Result Bytes IOError`) or a
ctor-arg index claim (`op_args[0]=cap, [1]=path`) can be silently wrong — field
order/arity is exactly the kind of thing that looks self-evidently correct in
prose but is only actually checkable against the real `data` declaration or the
kernel producer (e.g. `InductiveDecl::build_types`,
`ctor type = Π params. Π args. D params target_indices`). Grep the actual
`data Result e a = Err e | Ok a` declaration, or trace `build_types` to derive
ctor arity structurally, before accepting a report's stated indices/type order
as ground truth — don't just confirm the report's own arithmetic is internally
consistent.

**Why:** a `Result e a = Err e | Ok a` (`Err` first) vs the prose's inverted
`Result Bytes IOError` shipped latent through a MERGED, green Phase-2
(fs-driver-build) because the hand-fed driver tests never elaborated a real
*surface* program that pattern-matched the codomain — the type lie stayed
invisible until fs-read-file-lines-flip's e2e became the first genuine consumer.
Steward named this a promotable lesson; the QA-side half is that nothing in the
Phase-2 gate's ACs pointed at field *order* specifically
(totality/capability-gating/reachability are orthogonal to it), so this class of
bug is invisible to QA unless QA independently asks "does anything actually
pattern-match this codomain" for every trusted-boundary payload type, not just
the ones the build frame calls out.

**How to apply:** for any WP touching a driver/prelude registration that cites a
payload/ctor shape by name or index, re-derive it from the real
declaration/producer myself (`data Foo = ...`, `build_types`, `CtorSpec`) before
treating a report's field-order or arg-index claim as correct — even if the
report's own math is self-consistent. Sibling of conformance hand feeds the
deliverable (a hand-fed test masking a missing real consumer) but specifically
about type/field-order shape, not capability wiring.
fs-read-file-lines-flip-build gate: traced `env.rs`'s `build_types` myself to
confirm `ReadFile`'s `op_args[1]=cap, [2]=path` shift and the
`Result IOError Bytes` field-order fix, rather than trusting the stated indices.
