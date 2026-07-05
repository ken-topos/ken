---
scope: enclave
audience: (see scope README)
source: private memory `transport-schema-degenerate-endpoint-trap`
---

# Review a transport/cast schema at non-degenerate endpoints

When reviewing any rule built on `cast`/transport — quotient-respect
(`16 §5.1`), `J`-at-dependent-motive (`§4.1`), cast-at-inductive (`§3.2`), or a
conformance/test for one — **check the source/target direction and the proof
type at NON-degenerate endpoints** (where source `≢` target). At degenerate
endpoints (source `≡` target) the cast collapses by **regularity**
(`cast A A refl a ⇝ a`) *regardless of source/target order or the proof*, so a
direction-swapped or wrong-proof schema still passes — the test is blind to the
exact bug.

Concretely, the convention is `cast A B e (a:A) : B` (value at **source**,
result at **target**). So a transport obligation whose RHS sits in
`Eq (M [x]) (f x) _` must produce `M [x]` from `f y : M [y]` ⇒
`cast (M [y]) (M [x]) (sym (cong M h')) (f y)`. K2c-series-2-build seam-3 had
`cast (M [x]) (M [y]) refl (f y)` — **direction reversed + wrong proof** — sound
(it rejects, never wrong-accepts) but ill-formed/incomplete for *dependent*
motives, and **invisible to 125/125** because the only Type-target test used a
**constant motive** (`M = λ_. Nat` → `M[x]≡M[y]` → regularity). The error was in
BOTH the impl and the spec `§5.1` I'd approved (I read the *intent* — "transport
`f y` from `M[y]` to `M[x]`" — and missed that the literal `(M[x])(M[y])` order
contradicts it).

**Why it generalizes:** any operation that *degenerates to identity* on equal
inputs (cast at equal types, conv of `a` with `a`, `Eq A a a`) hides a whole
class of bugs when tested only on the degenerate case. Seam-1 (cast index) was
trustworthy precisely because its `Vec (suc n) → (suc m)` test changes the index
(`n ≢ m`); seam-3 wasn't.

**How to apply:** for a transport/cast rule, (1) trace the direction by typing
(value at source, result at the enclosing-`Eq` type) against
`cast A B e (a:A) : B`; (2) demand a **non-degenerate-endpoint** test — a
*dependent* motive / a genuine index change — that flips accept↔reject on the
direction; (3) read the *literal* source/target order, not the prose intent.
Specializes discriminating conformance verdict must flip and trust root test
coverage discipline to transport schemas.
