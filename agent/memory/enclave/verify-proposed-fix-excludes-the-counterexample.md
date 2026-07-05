---
scope: enclave
audience: (see scope README)
source: private memory `verify-proposed-fix-excludes-the-counterexample`
---

# Verify a proposed fix actually excludes the counterexample

**Ken Map law 5 (agreement), 2026-07-04 (`evt_5mjnxpchrmx03`).**
foundation-implementer HARD-STOPped: law 5
(`Ordered m → lookup key m = assoc key (toList m)`, dict laws
`trans, total; no antisym`) is FALSE, with an interp-grounded counterexample —
`T = Node (Node Leaf 0 v1 Leaf) 0 v2 Leaf` (duplicate key `0`):
`lookup 0 T = Some v2` (BST descent stops at root),
`assoc 0 (toList T) = Some v1` (in-order scan takes the first entry). Correct
hard-stop — a false proposition has no proof. Their proposed fix: **add
`antisym`** as a hypothesis ("cleanest").

**The catch: the proposed fix does NOT close the counterexample, and I only
caught it by instantiating it.** The duplicate is the *same* key `0` twice, so
`antisym` yields `Equal 0 0` (trivially true) — it excludes nothing. Instantiate
at `Int`/real `≤` (fully lawful — trans, total, **and** antisym): `T` is still a
valid `Ordered` witness (needs only `0 ≤ 0`), and `lookup ≠ assoc∘toList` still.
Adding `antisym` to the hypotheses changes neither the tree nor the
disagreement. The real missing precondition was **key-UNIQUENESS** (the two
traversal orders agree iff no order-equivalent keys); `antisym`'s genuine role
is at *build time* (insert's overwrite branch: order-equiv ⟹ overwrite ⟹
insert-reachable ⇒ distinct), not as a law-5 hypothesis. Fix = a `Distinct`
precondition, `Ordered` untouched, `antisym` back in the dict list *to discharge
Distinct*, not standalone.

**The rule for adjudicating a fix-to-a-false-statement:** a HARD-STOP arrives as
two claims — (1) the statement is false [counterexample], (2) here's the fix.
Ground BOTH. For (2), don't rubber-stamp a plausible precondition: **instantiate
the fix's added hypothesis at a concrete lawful witness and re-run the exact
counterexample.** If the counterexample *survives* the added hypothesis, the fix
is wrong however reasonable it sounds — a precondition can be the right *family*
(order laws) while missing the actual discriminator (uniqueness vs
antisymmetry). Route only the fix you verified excludes the counterexample, and
name why the plausible one doesn't (else the build chases a still-false goal).
Keep the fix's blast radius minimal (a law-5-local `Distinct` hypothesis, NOT a
global `Ordered` strengthening that reopens the already-landed law 4). Sibling
of green vs green does not confirm a fix (a fix that doesn't change the failing
case) and buildability ruling must ground every axis (clearing the axis under
debate isn't clearing all axes).
